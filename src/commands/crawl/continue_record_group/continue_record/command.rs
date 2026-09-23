use super::*;

/// The command this attempt runs: the one preflight retained (when it still matches the immutable
/// attempt), or a fresh preflight's. `None` when the record was settled here instead.
#[allow(clippy::too_many_arguments)]
pub(super) fn prepared_command(
    snapshot: &Value,
    state: &str,
    run_id: &str,
    catalog: &str,
    host: &str,
    host_report: &Value,
    record_name: &str,
    manifest: &mut RuntimeManifest,
) -> Result<Option<Vec<String>>> {
    if matches!(state, "preflight_passed" | "submitting") {
    let retained = snapshot
        .get("command")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    match engine_command(&manifest, host) {
        Ok(expected) if !retained.is_empty() && expected == retained => Ok(Some(retained)),
        Ok(_) => {
            mark_record_failure(
                run_id,
                catalog,
                record_name,
                "unavailable",
                "retained_command_mismatch",
                "preflight-persisted command differs from the immutable attempt".into(),
            ).map(|()| None)
        }
        Err(error) => {
            mark_record_failure(
                run_id,
                catalog,
                record_name,
                "unavailable",
                "worker_command_unavailable",
                error.to_string(),
            ).map(|()| None)
        }
    }
} else {
    let mut preflight = record_preflight(manifest, host_report);
    let mut ready = preflight.get("ready").and_then(Value::as_bool) == Some(true);
    if ready {
        let path = reference_path(&manifest.catalog, &manifest.record);
        let display = path
            .as_ref()
            .map(|value| value.display().to_string())
            .unwrap_or_else(|error| error.to_string());
        if let Err(error) = path.and_then(|value| {
            std::fs::read(&value)
                .map_err(anyhow::Error::from)
                .and_then(|bytes| finalize_manifest_identity(manifest, &bytes))
        }) {
            ready = false;
            preflight = json!({
                "schema": "wisent.crawl-record-preflight.v2",
                "record": manifest.record,
                "ready": false,
                "diagnostic": {
                    "code": "runtime_manifest_finalization_failed",
                    "message": error.to_string(),
                    "path": display,
                },
            });
        }
    }
    let command = if ready {
        engine_command(&manifest, host)
    } else {
        Err(anyhow!("exact record preflight failed"))
    };
    let command = match command {
        Ok(command) => command,
        Err(error) => {
            let diagnostic = preflight
                .get("diagnostic")
                .cloned()
                .unwrap_or_else(|| {
                    json!({"code": "worker_command_unavailable", "message": error.to_string()})
                });
            mutate_record(run_id, catalog, record_name, |entry| {
                entry["manifest"] = serde_json::to_value(&manifest)?;
                entry["preflight"] = preflight;
                entry["state"] = json!("unavailable");
                entry["diagnostic"] = diagnostic;
                Ok(())
            })?;
            return Ok(None);
        }
    };
    mutate_record(run_id, catalog, record_name, |entry| {
        if entry.get("stado_job_id").and_then(Value::as_str).is_some()
            || entry.get("cancel_intent").is_some_and(Value::is_object)
        {
            return Ok(());
        }
        entry["manifest"] = serde_json::to_value(&manifest)?;
        entry["preflight"] = preflight;
        entry["command"] = json!(command);
        entry["state"] = json!("preflight_passed");
        entry["diagnostic"] = Value::Null;
        Ok(())
    })?;
        Ok(Some(command))
    }
}
