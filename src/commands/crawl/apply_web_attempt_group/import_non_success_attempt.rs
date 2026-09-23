use super::*;

/// One non-success attempt, or `None` when this attempt is not a published web attempt
/// that carries a signed non-success proof.
///
/// Every record mutation happens on a COPY that replaces `record` only after the whole
/// import has succeeded. `apply_web_attempt` appends the provenance reference before it
/// runs the fallible loop that re-reads and re-hashes each retained evidence file, and the
/// caller turns any error here into a diagnostic while persisting the record regardless,
/// so mutating `record` in place would leave a reference behind whose verification is
/// guaranteed to fail on the same read — a permanent record-verification failure created
/// by an attempt the summary reports as not imported. Ordering the loop earlier would fix
/// only today's arrangement; substituting the copy keeps the guarantee whatever a later
/// change does inside `apply_web_attempt`.
pub(crate) fn import_non_success_attempt(
    manifest: &RuntimeManifest,
    snapshot: &Value,
    record: &mut Value,
    record_dir: &Path,
    run_dir: &Path,
) -> Result<Option<Value>> {
    let staging = run_dir
        .join("imports")
        .join(&manifest.catalog)
        .join(&manifest.record)
        .join(format!("{}-non-success", manifest.attempt_id));
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;
    let output_log = staging.join("worker-output.log");
    download_uri(&manifest.output_uri, &output_log)?;
    let report = retained_worker_report("web", &output_log)?;
    let envelope_outcome = report
        .pointer("/weles_attempt_envelope/outcome")
        .and_then(Value::as_str);
    let (Some(outcome), true) = (
        envelope_outcome,
        report
            .get("provenance_document")
            .is_some_and(Value::is_object),
    ) else {
        // Nothing signed to verify: this attempt never reached a terminal outcome with a
        // provenance document, so there is no proof to import and nothing to diagnose.
        let _ = std::fs::remove_dir_all(&staging);
        return Ok(None);
    };
    if outcome == crate::weles_provenance::SUCCESSFUL_OUTCOME {
        bail!("a non-accepted attempt reports the successful outcome");
    }
    if !crate::weles_provenance::is_terminal_outcome(outcome) {
        bail!("failed worker report envelope outcome {outcome} is not a terminal outcome");
    }
    // The identical identity proof the accepted attempt gets, against the same immutable
    // Stado submission receipt: a failure proof is not imported on weaker evidence than a
    // success, only on a different reported state.
    let receipt = load_submission_receipt(
        &manifest.run_id,
        &manifest.catalog,
        &manifest.record,
        &manifest.attempt_id,
    )?
    .context("non-success attempt has no immutable submission receipt")?;
    let proof = verify_worker_report(&report, manifest, snapshot, &receipt, "failed")?;
    let expected_sha256 = proof
        .get("sha256")
        .and_then(Value::as_str)
        .expect("verified artifact digest")
        .to_string();
    let expected_bytes = proof
        .get("bytes")
        .and_then(Value::as_u64)
        .expect("verified artifact byte count");
    let archive = staging.join("artifacts.tar.gz");
    download_uri(&manifest.artifact_uri, &archive)?;
    let (observed_sha256, observed_bytes) =
        hash_regular_file(&archive, MAX_ATTEMPT_ARCHIVE_BYTES)?;
    if observed_sha256 != expected_sha256 || observed_bytes != expected_bytes {
        bail!("retained failed-attempt artifact differs from the worker report");
    }
    let extracted_root = staging.join("extracted");
    let members = extract_attempt_archive(&archive, &extracted_root)?;
    let attempt_relative = format!("crawl/{}", manifest.attempt_id);
    let attempt_destination = record_dir.join(&attempt_relative);
    let attempt_staged = record_dir
        .join("crawl")
        .join(format!(".{}.staging", manifest.attempt_id));
    if attempt_staged.exists() {
        std::fs::remove_dir_all(&attempt_staged)?;
    }
    std::fs::create_dir_all(attempt_staged.parent().expect("crawl parent"))?;
    std::fs::rename(&extracted_root, &attempt_staged)?;
    std::fs::write(
        attempt_staged.join("worker-report.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    install_staged_tree(&attempt_staged, &attempt_destination)?;
    let relative_report = format!("{attempt_relative}/worker-report.json");
    let mut staged_record = record.clone();
    let mut run = crawl_run_entry(manifest, snapshot, &report, &proof, &relative_report);
    run["retained_members"] = json!(members.len());
    apply_web_attempt(
        &mut staged_record,
        &mut run,
        &report,
        &attempt_destination,
        record_dir,
        false,
    )?;
    let runs = staged_record
        .as_object_mut()
        .context("reference record is not an object")?
        .entry("crawl_runs")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .context("crawl_runs is not a list")?;
    if let Some(existing) = runs.iter_mut().find(|value| {
        value.get("attempt_id").and_then(Value::as_str) == Some(manifest.attempt_id.as_str())
    }) {
        *existing = run.clone();
    } else {
        runs.push(run.clone());
    }
    runs.sort_by(|left, right| {
        left.get("attempt")
            .and_then(Value::as_u64)
            .cmp(&right.get("attempt").and_then(Value::as_u64))
    });
    // Nothing above this line has touched the caller's record.
    *record = staged_record;
    let _ = std::fs::remove_dir_all(&staging);
    Ok(Some(json!({
        "attempt": manifest.attempt,
        "attempt_id": manifest.attempt_id,
        "state": "provenance_imported",
        "outcome": outcome,
        "artifact_sha256": expected_sha256,
        "retained_members": members.len(),
        "weles_task_id": run.get("weles_task_id").cloned().unwrap_or(Value::Null),
        "supports_confirmed_material": false,
    })))
}
