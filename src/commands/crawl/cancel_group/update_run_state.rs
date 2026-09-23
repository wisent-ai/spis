use super::*;

pub(crate) fn update_run_state(run: &mut Value) {
    let states: Vec<&str> = run
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("state").and_then(Value::as_str))
        .collect();
    let state = if states.iter().any(|state| *state == "running") {
        "running"
    } else if states.iter().any(|state| *state == "cancel_pending") {
        "cancel_pending"
    } else if states.iter().any(|state| *state == "pending_review") {
        "pending_review"
    } else if states.iter().any(|state| *state == "queued") {
        "queued"
    } else if states.iter().any(|state| *state == "planned") {
        "planned"
    } else if states.iter().all(|state| *state == "imported") && !states.is_empty() {
        "imported"
    } else if states
        .iter()
        .all(|state| matches!(*state, "completed" | "uploaded" | "imported"))
        && !states.is_empty()
    {
        "completed"
    } else if states
        .iter()
        .any(|state| matches!(*state, "partial" | "completed" | "uploaded" | "imported"))
    {
        "partial"
    } else {
        "failed"
    };
    let partial = states.iter().any(|state| *state == "partial");
    run["state"] = json!(state);
    run["partial"] = json!(partial);
}

pub(crate) fn status(rest: &[String]) -> Result<()> {
    let (run_id, record) = parse_run_and_record(rest, false)?;
    let mut selected = load(run_id.as_deref())?;
    let selected_id = selected.get("run_id").and_then(Value::as_str).context("run has no id")?.to_string();
    match RunMutationGuard::acquire(&selected_id) {
        Ok(_guard) => {
            selected = load(Some(&selected_id))?;
            refresh(&mut selected);
            persist(&mut selected)?;
        }
        Err(error) => {
            selected["status_refresh"] = json!({
                "state": "read_only_snapshot",
                "diagnostic": error.to_string(),
            });
        }
    }
    print_operation("status", &selected, record.as_deref())
}

pub(crate) fn parse_run_and_record(rest: &[String], require_run: bool) -> Result<(Option<String>, Option<String>)> {
    let mut run = None;
    let mut record = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--run" => { i += 1; run = Some(rest.get(i).context("--run needs a value")?.clone()); }
            "--record" => { i += 1; record = Some(rest.get(i).context("--record needs a value")?.clone()); }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    if require_run && run.is_none() { bail!("--run is required"); }
    Ok((run, record))
}

/// Re-arm one terminal-after-acceptance record as attempt N+1.
///
/// A `failed`, `cancelled`, `lost`, `submission_failed` or `preflight_failed`
/// attempt is immutable history. Resumption never reruns the old Stado job:
/// it increments the attempt, drops the previous execution identity, and
/// recomputes every derived identity value — input digest, catalog and record
/// keys, attempt id, correlation id, Stado run id and both attempt URIs — so the
/// next submission is a genuinely distinct idempotent attempt. `queued`,
/// `running`, `submitting`, `preflight_passed`, `cancel_pending`,
/// `pending_review`, `completed`, `uploaded` and `imported` are left untouched.
pub(crate) fn rearm_record_attempt(
    run_id: &str,
    catalog: &str,
    record: &str,
) -> Result<Option<u32>> {
    let mut rearmed = None;
    mutate_record(run_id, catalog, record, |entry| {
        let state = entry
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if !matches!(
            state.as_str(),
            "failed" | "cancelled" | "lost" | "submission_failed" | "preflight_failed"
        ) {
            return Ok(());
        }
        if entry.get("cancel_intent").is_some_and(Value::is_object) {
            entry["diagnostic"] = json!({
                "code": "cancel_intent_blocks_resume",
                "message": "a durable cancel intent exists; this record will not be re-armed",
            });
            return Ok(());
        }
        let mut manifest: RuntimeManifest =
            serde_json::from_value(entry.get("manifest").cloned().unwrap_or(Value::Null))
                .context("terminal record retains no typed runtime manifest to re-arm")?;
        manifest.attempt = manifest
            .attempt
            .checked_add(1)
            .context("record has exhausted the attempt counter")?;
        manifest.execution_identity = None;
        let reference = reference_path(&manifest.catalog, &manifest.record)?;
        let bytes = std::fs::read(&reference)
            .with_context(|| format!("read committed record {}", reference.display()))?;
        finalize_manifest_identity(&mut manifest, &bytes)?;
        entry["manifest"] = serde_json::to_value(&manifest)?;
        entry["attempt"] = json!(manifest.attempt);
        entry["attempt_id"] = json!(manifest.attempt_id);
        entry["artifact_uri"] = json!(manifest.artifact_uri);
        entry["output_uri"] = json!(manifest.output_uri);
        entry["state"] = json!("planned");
        for cleared in [
            "command",
            "stado_job_id",
            "submission_receipt",
            "submission_transition",
            "preflight",
            "job",
            "lookup_error",
            "cancel",
            "cancel_result",
            "error",
            "import",
        ] {
            entry[cleared] = Value::Null;
        }
        entry["diagnostic"] = json!({
            "code": "attempt_rearmed",
            "message": format!(
                "terminal {state} attempt retained in history; attempt {} planned with fresh identity",
                manifest.attempt
            ),
        });
        rearmed = Some(manifest.attempt);
        Ok(())
    })?;
    Ok(rearmed)
}

pub(crate) fn resume(rest: &[String]) -> Result<()> {
    let (run_id, selected_record) = parse_run_and_record(rest, true)?;
    let run_id = run_id.context("--run is required")?;
    {
        let _guard = RunMutationGuard::acquire(&run_id)?;
        let mut run = load(Some(&run_id))?;
        migrate_legacy_catalog_jobs(&mut run);
        refresh(&mut run);
        persist(&mut run)?;
    }
    let snapshot = load(Some(&run_id))?;
    let original = snapshot
        .get("source_revision")
        .and_then(Value::as_str)
        .context("run has no source_revision")?
        .to_string();
    let current = build_revision()?;
    if original != current {
        bail!(
            "run {run_id} belongs to Spis revision {original}; resumption with revision {current} is refused"
        );
    }
    let mut targets = Vec::new();
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
            if selected_matches(&record_name, selected_record.as_deref()) {
                targets.push((catalog_name.clone(), record_name));
            }
        }
    }
    if targets.is_empty() {
        bail!("no crawl record matches the resume selection");
    }
    for (catalog, record) in &targets {
        match RecordMutationGuard::acquire(&run_id, catalog, record) {
            Ok(_guard) => {
                rearm_record_attempt(&run_id, catalog, record)?;
            }
            Err(error) if error.downcast_ref::<RecordLockBusy>().is_some() => continue,
            Err(error) => return Err(error),
        }
    }
    continue_start(&run_id)?;
    let mut run = import_ready(&run_id, selected_record.as_deref())?;
    print_operation("resume", &run, selected_record.as_deref())?;
    update_run_state(&mut run);
    if has_failures(&run) {
        bail!("one or more crawl records remain unresumable");
    }
    Ok(())
}

pub(crate) fn selected_matches(record: &str, selected: Option<&str>) -> bool {
    selected.is_none_or(|wanted| {
        record == wanted || record.split_once('-').map(|(_, tail)| tail) == Some(wanted)
    })
}

pub(crate) const MAX_WORKER_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;

pub(crate) const MAX_EXTRACTED_ENTRIES: usize = 20_000;

pub(crate) const MAX_EXTRACTED_BYTES: u64 = 512 * 1024 * 1024;

pub(crate) fn download_uri(uri: &str, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match std::fs::symlink_metadata(destination) {
        Ok(_) => std::fs::remove_file(destination)
            .with_context(|| format!("clear stale download {}", destination.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let mut command = crawl_storage_command();
    command.args(["storage", "get", uri]).arg(destination);
    let output = bounded_command_output(
        &mut command,
        "download retained crawl object",
        Duration::from_secs(600),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "download {uri}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}
