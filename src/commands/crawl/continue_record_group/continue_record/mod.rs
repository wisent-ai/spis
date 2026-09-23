use super::*;

mod cancel_race;
mod command;

pub(crate) fn continue_record(
    run_id: &str,
    catalog: &str,
    host: &str,
    host_report: &Value,
    record_name: &str,
) -> Result<()> {
    let record_guard = RecordMutationGuard::acquire(run_id, catalog, record_name)?;
    let snapshot = record_snapshot(run_id, catalog, record_name)?;
    let state = snapshot
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("unavailable")
        .to_string();
    if snapshot.get("stado_job_id").and_then(Value::as_str).is_some()
        || matches!(
            state.as_str(),
            "unavailable"
                | "completed"
                | "uploaded"
                | "imported"
                | "running"
                | "queued"
                | "cancel_pending"
                | "pending_review"
                | "cancelled"
        )
    {
        return Ok(());
    }
    if snapshot.get("cancel_intent").is_some_and(Value::is_object) {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "cancelled",
            "cancelled_before_submission",
            "durable cancel intent exists; worker submission is prohibited".into(),
        );
    }
    if host_preflight_is_retryable(host_report) {
        let diagnostic = host_report
            .get("diagnostic")
            .cloned()
            .unwrap_or_else(|| {
                json!({
                    "code": "host_probe_timed_out",
                    "retryable": true,
                    "message": "host capability probe timed out",
                })
            });
        mutate_record(run_id, catalog, record_name, |entry| {
            entry["preflight"] = host_report.clone();
            entry["state"] = json!(failed_host_preflight_record_state(host_report));
            entry["diagnostic"] = diagnostic;
            Ok(())
        })?;
        return Ok(());
    }
    let mut manifest: RuntimeManifest = match serde_json::from_value(
        snapshot.get("manifest").cloned().unwrap_or(Value::Null),
    ) {
        Ok(manifest) => manifest,
        Err(error) => {
            return mark_record_failure(
                run_id,
                catalog,
                record_name,
                "unavailable",
                "runtime_manifest_invalid",
                error.to_string(),
            );
        }
    };

    let Some(command) = command::prepared_command(
        &snapshot,
        &state,
        run_id,
        catalog,
        host,
        host_report,
        record_name,
        &mut manifest,
    )?
    else {
        return Ok(());
    };

    let before_submit = record_snapshot(run_id, catalog, record_name)?;
    if before_submit.get("cancel_intent").is_some_and(Value::is_object) {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "cancelled",
            "cancelled_before_submission",
            "durable cancel intent won the submission race".into(),
        );
    }
    if !matches!(
        before_submit.get("state").and_then(Value::as_str),
        Some("preflight_passed" | "submitting")
    ) {
        return Ok(());
    }
    if before_submit.get("state").and_then(Value::as_str) == Some("preflight_passed") {
        mutate_record(run_id, catalog, record_name, |entry| {
            if entry.get("state").and_then(Value::as_str) == Some("preflight_passed")
                && !entry.get("cancel_intent").is_some_and(Value::is_object)
            {
                entry["state"] = json!("submitting");
                entry["submission_transition"] = json!({
                    "state": "intent_persisted",
                    "attempt_id": manifest.attempt_id,
                });
            }
            Ok(())
        })?;
    }
    let armed = record_snapshot(run_id, catalog, record_name)?;
    if armed.get("state").and_then(Value::as_str) != Some("submitting") {
        return Ok(());
    }
    let recovered = load_submission_receipt(
        run_id,
        catalog,
        &manifest.record,
        &manifest.attempt_id,
    )?;
    drop(record_guard);
    let receipt = if let Some(receipt) = recovered {
        receipt
    } else {
        let output = match invoke_engine(&command, host, host_report) {
            Ok(output) => output,
            Err(error) => {
                return mark_record_failure(
                    run_id,
                    catalog,
                    record_name,
                    "submission_failed",
                    "crawler_coordinator_launch_failed",
                    format!("{error:#}"),
                );
            }
        };
        if !output.status.success() {
            let message = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
            if let Some(diagnostic) = worker_agent_program_retry_diagnostic(&message) {
                mutate_record(run_id, catalog, record_name, |entry| {
                    entry["state"] = json!("planned");
                    entry["diagnostic"] = diagnostic;
                    Ok(())
                })?;
                return Ok(());
            }
            return mark_record_failure(
                run_id,
                catalog,
                record_name,
                "submission_failed",
                "stado_submission_failed",
                message,
            );
        }
        match parse_submission(&output.stdout) {
            Ok(receipt) => receipt,
            Err(error) => {
                return mark_record_failure(
                    run_id,
                    catalog,
                    record_name,
                    "submission_failed",
                    "submission_receipt_invalid",
                    error.to_string(),
                );
            }
        }
    };
    let stado = receipt.get("stado_receipt");
    if stado
        .and_then(|value| value.get("source_revision"))
        .and_then(Value::as_str)
        != Some(manifest.source_revision.as_str())
    {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "submission_failed",
            "submission_source_mismatch",
            "Stado receipt does not bind the immutable Spis source revision".into(),
        );
    }
    // The coordinator derived both attempt URIs from the record key. The child's
    // reported values are checked against them and never adopted, so a divergent
    // final JSON line cannot relocate where this attempt's evidence is expected.
    if receipt.get("artifact_uri").and_then(Value::as_str) != Some(manifest.artifact_uri.as_str())
        || receipt.get("output_uri").and_then(Value::as_str) != Some(manifest.output_uri.as_str())
    {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "submission_failed",
            "submission_uri_mismatch",
            "crawler submission reported artifact or output URIs that are not the canonical attempt coordinates".into(),
        );
    }
    if let Err(error) = persist_submission_receipt(
        run_id,
        catalog,
        &manifest.record,
        &manifest.attempt_id,
        &receipt,
    ) {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "submission_failed",
            "submission_receipt_persistence_failed",
            error.to_string(),
        );
    }
    mutate_record(run_id, catalog, record_name, |entry| {
        entry["stado_job_id"] = receipt
            .get("stado_job_id")
            .cloned()
            .unwrap_or(Value::Null);
        entry["artifact_uri"] = json!(manifest.artifact_uri);
        entry["output_uri"] = json!(manifest.output_uri);
        entry["submission_receipt"] = receipt;
        if entry.get("cancel_intent").is_some_and(Value::is_object) {
            entry["state"] = json!("cancel_pending");
            entry["diagnostic"] = json!({
                "code": "cancel_won_submission_race",
                "message": "the durable cancel intent will be dispatched against the retained Stado job",
            });
        } else {
            entry["state"] = json!("queued");
            entry["diagnostic"] = Value::Null;
        }
        Ok(())
    })?;
    cancel_race::settle_cancel_race(run_id, catalog, record_name)
}
