use super::*;

/// Import every accepted attempt of one run, then run the catalog validators.
///
/// Each record is its own durable transaction under its own record lock, so a
/// busy peer, a single failing record or a crash never blocks or corrupts the
/// others. Validators and the catalog generator run once, after every record has
/// been installed, and their failure is retained as a typed run diagnostic rather
/// than left as unrecoverable dirty state.
pub(crate) fn import_ready(run_id: &str, selected_record: Option<&str>) -> Result<Value> {
    let run_dir = run_path(run_id)?
        .parent()
        .context("run path has no parent")?
        .to_path_buf();
    let snapshot = load(Some(run_id))?;
    let mut touched_catalogs: BTreeMap<String, String> = BTreeMap::new();
    for catalog in snapshot
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let catalog_name = catalog
            .get("catalog")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let engine = catalog
            .get("engine")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        for record in catalog
            .get("records")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let record_name = record
                .get("record")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if !selected_matches(&record_name, selected_record) {
                continue;
            }
            let state = record
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !matches!(state, "completed" | "uploaded") {
                continue;
            }
            let guard = match RecordMutationGuard::acquire(run_id, &catalog_name, &record_name) {
                Ok(guard) => guard,
                Err(error) if error.downcast_ref::<RecordLockBusy>().is_some() => continue,
                Err(error) => return Err(error),
            };
            let current = record_snapshot(run_id, &catalog_name, &record_name)?;
            if !matches!(
                current.get("state").and_then(Value::as_str),
                Some("completed" | "uploaded")
            ) {
                continue;
            }
            let outcome =
                import_record_attempt(run_id, &catalog_name, &engine, &current, &run_dir);
            match outcome {
                Ok(import) => {
                    mutate_record(run_id, &catalog_name, &record_name, |entry| {
                        entry["state"] = json!("imported");
                        entry["weles_task_id"] =
                            import.get("weles_task_id").cloned().unwrap_or(Value::Null);
                        entry["import"] = import;
                        entry["diagnostic"] = Value::Null;
                        Ok(())
                    })?;
                    touched_catalogs.insert(catalog_name.clone(), engine.clone());
                }
                Err(error) => {
                    mutate_record(run_id, &catalog_name, &record_name, |entry| {
                        entry["state"] = json!("partial");
                        entry["diagnostic"] = json!({
                            "code": "attempt_import_failed",
                            "message": format!("{error:#}"),
                        });
                        Ok(())
                    })?;
                }
            }
            drop(guard);
        }
    }
    let mut maintenance = Vec::new();
    for (catalog, engine) in &touched_catalogs {
        if engine == "web" {
            match run_spis_command(&["analyze-example-structures", catalog]) {
                Ok(_) => {}
                Err(error) => maintenance.push(json!({
                    "command": format!("analyze-example-structures {catalog}"),
                    "state": "failed",
                    "message": format!("{error:#}"),
                })),
            }
        }
        if let Err(error) =
            run_spis_command(&["verify-reference-evidence", "--catalog", catalog, "--apply"])
        {
            maintenance.push(json!({
                "command": format!("verify-reference-evidence --catalog {catalog} --apply"),
                "state": "failed",
                "message": format!("{error:#}"),
            }));
        }
    }
    if !touched_catalogs.is_empty() {
        if let Err(error) = run_spis_command(&["generate-example-catalogs"]) {
            maintenance.push(json!({
                "command": "generate-example-catalogs",
                "state": "failed",
                "message": format!("{error:#}"),
            }));
        }
    }
    let _guard = RunMutationGuard::acquire(run_id)?;
    let mut run = load(Some(run_id))?;
    run["import_maintenance"] = json!({
        "catalogs": touched_catalogs.keys().collect::<Vec<_>>(),
        "failures": maintenance,
    });
    if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in entries {
            aggregate_catalog_entry(entry);
        }
    }
    update_run_state(&mut run);
    persist(&mut run)?;
    Ok(run)
}

pub(crate) fn import(rest: &[String]) -> Result<()> {
    let (run_id, selected_record) = parse_run_and_record(rest, true)?;
    let run_id = run_id.context("--run is required")?;
    {
        let _guard = RunMutationGuard::acquire(&run_id)?;
        let mut run = load(Some(&run_id))?;
        migrate_legacy_catalog_jobs(&mut run);
        refresh(&mut run);
        persist(&mut run)?;
    }
    let run = import_ready(&run_id, selected_record.as_deref())?;
    print_operation("import", &run, selected_record.as_deref())?;
    let pending: Vec<String> = run
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|catalog| {
            catalog
                .get("records")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter(|record| {
            selected_matches(
                record.get("record").and_then(Value::as_str).unwrap_or_default(),
                selected_record.as_deref(),
            ) && record.get("state").and_then(Value::as_str) != Some("imported")
        })
        .filter_map(|record| {
            record
                .get("record")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect();
    if run
        .pointer("/import_maintenance/failures")
        .and_then(Value::as_array)
        .is_some_and(|failures| !failures.is_empty())
    {
        bail!("crawl evidence was imported but a catalog validator or generator failed");
    }
    if !pending.is_empty() {
        bail!("{} crawl records were not imported: {}", pending.len(), pending.join(", "));
    }
    Ok(())
}

pub(crate) fn has_failures(run: &Value) -> bool {
    run.get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|entry| {
            matches!(
                entry.get("state").and_then(Value::as_str),
                Some(
                    "unavailable"
                        | "preflight_failed"
                        | "submission_failed"
                        | "lost"
                        | "failed"
                        | "cancelled"
                        | "partial"
                )
            )
        })
}

pub(crate) fn print_operation(operation: &str, run: &Value, record_filter: Option<&str>) -> Result<()> {
    let mut catalogs = run.get("catalogs").and_then(Value::as_array).cloned().unwrap_or_default();
    if let Some(record) = record_filter {
        for catalog in &mut catalogs {
            if let Some(records) = catalog.get_mut("records").and_then(Value::as_array_mut) {
                records.retain(|item| item.get("record").and_then(Value::as_str).is_some_and(|value| value == record || value.split_once('-').map(|(_, tail)| tail) == Some(record)));
            }
        }
        catalogs.retain(|catalog| catalog.get("records").and_then(Value::as_array).is_some_and(|records| !records.is_empty()));
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for catalog in &catalogs {
        *counts.entry(catalog.get("state").and_then(Value::as_str).unwrap_or("unknown").to_string()).or_default() += 1;
        if let Some(records) = catalog.get("records").and_then(Value::as_array) {
            for record in records {
                *counts.entry(format!("record_{}", record.get("state").and_then(Value::as_str).unwrap_or("unknown"))).or_default() += 1;
            }
        }
    }
    let document = json!({
        "schema": OP_SCHEMA,
        "operation": operation,
        "run_id": run.get("run_id"),
        "state": run.get("state"),
        "source_revision": run.get("source_revision"),
        "updated_at": run.get("updated_at"),
        "counts": counts,
        "catalogs": catalogs,
    });
    println!("{}", serde_json::to_string(&document)?);
    Ok(())
}
