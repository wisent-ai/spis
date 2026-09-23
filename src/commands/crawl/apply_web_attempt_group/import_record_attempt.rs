use super::*;

/// Import exactly one accepted attempt of one record.
///
/// The whole record transaction is staged and fsynced before any rename, so an
/// interrupted import leaves the previous attempt content intact and can be
/// retried; the previous attempts and their diagnostics are preserved because
/// each attempt owns its own `crawl/{attempt_id}` subtree and its own
/// `crawl_runs` entry keyed by `attempt_id`.
pub(crate) fn import_record_attempt(
    run_id: &str,
    catalog: &str,
    engine: &str,
    entry: &Value,
    run_dir: &Path,
) -> Result<Value> {
    let manifest: RuntimeManifest =
        serde_json::from_value(entry.get("manifest").cloned().unwrap_or(Value::Null))
            .context("accepted record retains no typed runtime manifest")?;
    if manifest.engine != engine || manifest.catalog != catalog || manifest.run_id != run_id {
        bail!("retained runtime manifest does not belong to this run, catalog and engine");
    }
    let receipt = load_submission_receipt(run_id, catalog, &manifest.record, &manifest.attempt_id)?
        .context("accepted record has no immutable submission receipt")?;
    let staging = run_dir
        .join("imports")
        .join(catalog)
        .join(&manifest.record)
        .join(&manifest.attempt_id);
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;
    let output_log = staging.join("worker-output.log");
    download_uri(&manifest.output_uri, &output_log)?;
    let report = retained_worker_report(engine, &output_log)?;
    let proof = verify_worker_report(&report, &manifest, entry, &receipt, "artifact_published")?;
    let archive = staging.join("artifacts.tar.gz");
    download_uri(&manifest.artifact_uri, &archive)?;
    let expected_sha256 = proof
        .get("sha256")
        .and_then(Value::as_str)
        .expect("verified artifact digest");
    let expected_bytes = proof
        .get("bytes")
        .and_then(Value::as_u64)
        .expect("verified artifact byte count");
    let (observed_sha256, observed_bytes) =
        hash_regular_file(&archive, MAX_ATTEMPT_ARCHIVE_BYTES)?;
    if observed_sha256 != expected_sha256 || observed_bytes != expected_bytes {
        bail!(
            "retained attempt artifact differs from the worker report: expected sha256={expected_sha256} bytes={expected_bytes}, observed sha256={observed_sha256} bytes={observed_bytes}"
        );
    }
    let extracted_root = staging.join("extracted");
    let members = extract_attempt_archive(&archive, &extracted_root)?;
    let record_dir = reference_path(catalog, &manifest.record)?
        .parent()
        .context("record reference has no directory")?
        .to_path_buf();
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
    // A plain write, not `atomic_json_write`: that helper leaves a `.lock` sibling,
    // and the staged tree is published verbatim as the attempt artifact.
    std::fs::write(
        attempt_staged.join("worker-report.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    install_staged_tree(&attempt_staged, &attempt_destination)?;
    let record_path = record_dir.join("reference.json");
    let mut record: Value =
        crate::read_json(record_path.to_str().context("record path is not UTF-8")?)?;
    let source_url = record
        .get("product_url")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let relative_report = format!("{attempt_relative}/worker-report.json");
    let mut run = crawl_run_entry(&manifest, entry, &report, &proof, &relative_report);
    run["retained_members"] = json!(members.len());
    match engine {
        "web" => apply_web_attempt(
            &mut record,
            &mut run,
            &report,
            &attempt_destination,
            &record_dir,
            // The accepted attempt is the one the record's confirmed material comes from.
            true,
        )?,
        "docs" => {
            // Materialise the same verified artifact into the product corpus
            // store before recording this attempt as imported. Reusing the
            // archive already downloaded above avoids a second object request;
            // the corpus importer independently checks its digest, length,
            // extraction bounds and typed report/tree agreement.
            let readable_corpus =
                super::docs_corpus::import_worker_report_from_archive(&report, &archive)?;
            if readable_corpus
                .get("artifact_uri")
                .and_then(Value::as_str)
                != Some(manifest.artifact_uri.as_str())
                || readable_corpus
                    .get("archive_sha256")
                    .and_then(Value::as_str)
                    != Some(expected_sha256)
            {
                bail!("installed documentation corpus does not identify the immutable attempt artifact");
            }
            // Manifest equality stays here: only this side knows the immutable
            // attempt's committed content-structure digest.
            if report.get("docs_structure_sha256").and_then(Value::as_str)
                != manifest.docs_structure_sha256.as_deref()
            {
                bail!("docs worker report crawl-definition digest differs from the immutable attempt");
            }
            run["docs_structure_sha256"] = json!(manifest.docs_structure_sha256);
            run["corpus"] = report
                .get("corpus")
                .cloned()
                .context("docs worker report has no typed corpus summary")?;
            run["readable_corpus"] = readable_corpus;
        }
        _ => {}
    }
    let (motion, states) = attempt_media(engine, &attempt_destination, &record_dir, &source_url)?;
    let accessibility = accessibility_gap(engine, &attempt_destination);
    let object = record
        .as_object_mut()
        .context("reference record is not an object")?;
    // An import writes the evidence ITS OWN attempt produced and nothing else.
    // A documentation attempt retains a corpus and no browser media, and these
    // four lines used to write its empty media sections over the record: the
    // 2026-09-05 imports of `01-mdn-web-docs` and `17-swift-documentation`
    // deleted the Weles motion recording and the five local states captured on
    // 2026-08-16 - inside a transaction whose own report said `motion: 0`,
    // `states: 0`, so the record lost evidence the import never claimed to
    // have. Evidence another engine captured is not this attempt's to clear,
    // and `captured_at` names when the material that is there was captured, so
    // it only moves when the material does.
    if !motion.is_empty() || !states.is_empty() {
        object.insert("captured_at".into(), json!(crate::now_iso_utc()));
        object.insert("motion".into(), Value::Array(motion.clone()));
        object.insert("states".into(), Value::Array(states.clone()));
        object.insert("accessibility".into(), accessibility);
    }
    object.insert("evidence_status".into(), json!("partial"));
    object.insert(
        "evidence_gaps".into(),
        json!(["crawl evidence has not yet passed verify-reference-evidence"]),
    );
    let runs = object
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
    // In the same record transaction, so the failure proofs and the accepted attempt are
    // persisted by the one `atomic_json_write` below or not at all.
    let non_success = if engine == "web" {
        import_non_success_attempts(
            run_id,
            catalog,
            &manifest,
            entry,
            &mut record,
            &record_dir,
            run_dir,
        )
    } else {
        Vec::new()
    };
    atomic_json_write(&record_path, &record)?;
    let _ = std::fs::remove_dir_all(&staging);
    Ok(json!({
        "state": "imported",
        "attempt": manifest.attempt,
        "attempt_id": manifest.attempt_id,
        "artifact_uri": manifest.artifact_uri,
        "artifact_sha256": expected_sha256,
        "artifact_bytes": expected_bytes,
        "retained_members": members.len(),
        "states": states.len(),
        "motion": motion.len(),
        "worker_report": relative_report,
        "weles_task_id": run.get("weles_task_id").cloned().unwrap_or(Value::Null),
        "non_success_attempts": non_success,
        "imported_at": crate::now_iso_utc(),
    }))
}

pub(crate) fn run_spis_command(arguments: &[&str]) -> Result<String> {
    let executable = std::env::current_exe().context("resolve current Spis executable")?;
    let mut command = Command::new(executable);
    command.args(arguments);
    let output = bounded_command_output(
        &mut command,
        "Spis catalog maintenance command",
        Duration::from_secs(900),
        8 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "spis {} failed: {}{}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
