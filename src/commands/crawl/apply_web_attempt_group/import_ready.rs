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
    for catalog in run_catalogs(&snapshot)? {
        let catalog_name = catalog.name.to_string();
        // The engine decides which maintenance an imported catalog gets; an
        // entry that names none is refused, never imported as no engine.
        let engine = catalog
            .entry
            .get("engine")
            .and_then(Value::as_str)
            .with_context(|| format!("run {run_id}: catalog {catalog_name} names no engine"))?
            .to_string();
        for record in catalog.records {
            let record_name = record_name(&catalog_name, record)?.to_string();
            if !selected_matches(&record_name, selected_record) {
                continue;
            }
            let awaiting = record
                .get("state")
                .map(State::of)
                .transpose()?
                .is_some_and(State::awaits_import);
            if !awaiting {
                continue;
            }
            let guard = match RecordMutationGuard::acquire(run_id, &catalog_name, &record_name) {
                Ok(guard) => guard,
                Err(error) if error.downcast_ref::<RecordLockBusy>().is_some() => continue,
                Err(error) => return Err(error),
            };
            let current = record_snapshot(run_id, &catalog_name, &record_name)?;
            let still_awaiting = current
                .get("state")
                .map(State::of)
                .transpose()?
                .is_some_and(State::awaits_import);
            if !still_awaiting {
                continue;
            }
            let outcome = import_record_attempt(run_id, &catalog_name, &engine, &current, &run_dir);
            match outcome {
                Ok(import) => {
                    mutate_record(run_id, &catalog_name, &record_name, |entry| {
                        entry["state"] = json!(State::Imported.name());
                        // An import that ran no Weles task records none.
                        entry["weles_task_id"] = match import.get("weles_task_id") {
                            Some(task) => task.clone(),
                            None => Value::Null,
                        };
                        entry["import"] = import;
                        entry["diagnostic"] = Value::Null;
                        Ok(())
                    })?;
                    touched_catalogs.insert(catalog_name.clone(), engine.clone());
                }
                Err(error) => {
                    mutate_record(run_id, &catalog_name, &record_name, |entry| {
                        entry["state"] = json!(State::Partial.name());
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
    let mut pending: Vec<String> = Vec::new();
    for catalog in run_catalogs(&run)? {
        for record in catalog.records {
            let name = record_name(catalog.name, record)?;
            if selected_matches(name, selected_record.as_deref())
                && State::of(&record["state"])? != State::Imported
            {
                pending.push(name.to_string());
            }
        }
    }
    if run
        .pointer("/import_maintenance/failures")
        .and_then(Value::as_array)
        .is_some_and(|failures| !failures.is_empty())
    {
        bail!("crawl evidence was imported but a catalog validator or generator failed");
    }
    if !pending.is_empty() {
        bail!(
            "{} crawl records were not imported: {}",
            pending.len(),
            pending.join(", ")
        );
    }
    Ok(())
}

/// Whether any catalog of `run` ended in a failed state. A catalog with no
/// state, or one whose state is not a declared one (the aggregate left it null
/// with a diagnostic), is a failure too; a run whose catalogs cannot be read
/// is refused rather than read as one with nothing failed.
pub(crate) fn has_failures(run: &Value) -> Result<bool> {
    Ok(run_catalogs(run)?
        .iter()
        .any(|catalog| match catalog.entry.get("state").map(State::of) {
            Some(Ok(state)) => state.counts_as_failure(),
            _ => true,
        }))
}

/// The label `entry`'s state is counted under: its state after `prefix`, or
/// `undeclared` when the aggregate left none (its `undeclared_crawl_state`
/// diagnostic says why).
fn state_label(entry: &Value, prefix: &str) -> String {
    match entry.get("state").and_then(Value::as_str) {
        Some(state) => format!("{prefix}{state}"),
        None => format!("{prefix}undeclared"),
    }
}

pub(crate) fn print_operation(
    operation: &str,
    run: &Value,
    record_filter: Option<&str>,
) -> Result<()> {
    // The catalogs as the run reader reads them, narrowed by the one record
    // selector every crawl verb uses.
    let mut catalogs: Vec<Value> = run_catalogs(run)?
        .into_iter()
        .map(|catalog| catalog.entry.clone())
        .collect();
    if record_filter.is_some() {
        for catalog in &mut catalogs {
            if let Some(records) = catalog.get_mut("records").and_then(Value::as_array_mut) {
                records.retain(|item| {
                    item.get("record")
                        .and_then(Value::as_str)
                        .is_some_and(|name| selected_matches(name, record_filter))
                });
            }
        }
        catalogs.retain(|catalog| {
            catalog
                .get("records")
                .and_then(Value::as_array)
                .is_some_and(|records| !records.is_empty())
        });
    }
    let mut labels = Vec::new();
    for catalog in &catalogs {
        labels.push(state_label(catalog, ""));
        for record in catalog
            .get("records")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            labels.push(state_label(record, "record_"));
        }
    }
    let counts: BTreeMap<&String, usize> = labels
        .iter()
        .map(|label| (label, labels.iter().filter(|other| *other == label).count()))
        .collect();
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
