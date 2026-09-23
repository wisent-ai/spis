use super::*;

/// Copy the typed Weles attempt facts into the crawl-run entry and the record.
///
/// `verify_attempt_binding` in the receipt verifier compares every one of these
/// outer fields with the inner `weles_attempt_envelope`, so they are written from
/// the envelope itself rather than restated.
pub(crate) fn apply_web_attempt(
    record: &mut Value,
    run: &mut Value,
    report: &Value,
    attempt_dir: &Path,
    record_dir: &Path,
    // True only for the accepted, completed attempt. A non-success attempt is imported so
    // that its signed failure proof is re-verified with the record, and it must never
    // write the record-level evidence inventory: see the comment at that write for what
    // this guard does and does not decide.
    confirms_record: bool,
) -> Result<()> {
    let envelope = report
        .get("weles_attempt_envelope")
        .filter(|value| value.is_object())
        .context("web worker report has no typed Weles attempt envelope")?;
    let envelope: crate::weles_provenance::WelesAttemptEnvelope =
        serde_json::from_value(envelope.clone())
            .context("web worker report envelope does not match the typed schema")?;
    let manifest_sha256 = envelope
        .weles_evidence_manifest_sha256
        .clone()
        .context("web attempt envelope has no evidence manifest digest")?;
    let artifact_sha256 = envelope
        .artifact_document_sha256
        .clone()
        .context("web attempt envelope has no artifact document digest")?;
    run["weles_task_id"] = json!(envelope.weles_task_id);
    run["weles_request_digest"] = json!(envelope.weles_request_digest);
    run["weles_result_digest"] = json!(envelope.weles_result_digest);
    run["weles_evidence_manifest_uri"] = json!(envelope.weles_evidence_manifest_uri);
    run["weles_evidence_manifest_sha256"] = json!(manifest_sha256);
    run["artifact_document_uri"] = json!(envelope.artifact_document_uri);
    run["artifact_document_sha256"] = json!(artifact_sha256);
    run["observation_document_uri"] = json!(envelope.observation_document_uri);
    run["observation_document_sha256"] = json!(envelope.observation_document_sha256);
    run["requested_url"] = json!(envelope.requested_url);
    run["final_url"] = json!(envelope.final_url);
    run["state"] = json!(envelope.state);
    run["outcome"] = json!(envelope.outcome);
    run["weles_attempt_envelope"] = serde_json::to_value(&envelope)?;
    // The verifier resolves `artifact.path` and every inventory tail relative to the
    // RECORD directory, so the attempt's content-addressed `weles/` documents and its
    // task-scoped `recordings/` tree must exist there, merged rather than replaced:
    // earlier attempts' provenance documents still point at their own digests.
    //
    // This is why every object a worker places in these two subtrees is addressed by its
    // own content or by its Weles task, never by its role: `write_immutable_file` refuses
    // a name that already holds different bytes, so one role-named document here would
    // permanently block the second import of the same record. Operational documents that
    // differ per attempt by construction stay in the attempt root instead, and reach the
    // record through the attempt-private `crawl/{attempt_id}` tree installed above.
    for subtree in ["weles", "recordings"] {
        let source = attempt_dir.join(subtree);
        if source.is_dir() {
            merge_immutable_tree(&source, &record_dir.join(subtree))?;
        }
    }
    let provenance = report
        .get("provenance_document")
        .filter(|value| value.is_object())
        .context("web worker report has no official provenance document")?;
    let provenance: crate::weles_provenance::WelesProvenanceDocument =
        serde_json::from_value(provenance.clone())
            .context("web provenance document does not match the typed schema")?;
    let provenance_id = provenance
        .id
        .strip_prefix("sha256:")
        .filter(|value| is_lower_sha256(value))
        .context("official provenance document has no framed sha256: identifier")?
        .to_string();
    let provenance_relative = format!("weles/provenance/{provenance_id}.json");
    let provenance_bytes = serde_json::to_vec_pretty(&provenance)?;
    write_immutable_file(&record_dir.join(&provenance_relative), &provenance_bytes)?;
    let reference = crate::weles_provenance::WelesProvenanceDocumentRef {
        schema: crate::weles_provenance::PROVENANCE_DOCUMENT_REF_SCHEMA.to_string(),
        path: provenance_relative,
        sha256: crate::sha256_hex(&provenance_bytes),
    };
    let references = record
        .as_object_mut()
        .context("reference record is not an object")?
        .entry("provenance_documents")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .context("provenance_documents is not a list")?;
    let reference = serde_json::to_value(&reference)?;
    if let Some(existing) = references.iter_mut().find(|value| {
        value.get("path").and_then(Value::as_str) == reference.get("path").and_then(Value::as_str)
    }) {
        *existing = reference;
    } else {
        references.push(reference);
    }
    let inventory: Vec<Value> = envelope
        .evidence_inventory
        .iter()
        .map(|item| {
            let tail = item
                .uri
                .strip_prefix(&format!(
                    "stado://weles/recordings/{}/",
                    envelope.weles_task_id
                ))
                .context("evidence inventory URI is not bound to the attempt task")?;
            let relative = format!("recordings/{}/{tail}", envelope.weles_task_id);
            let retained = record_dir.join(&relative);
            let bytes = std::fs::read(&retained).with_context(|| {
                format!("read retained Weles evidence {}", retained.display())
            })?;
            if bytes.len() as u64 != item.bytes || crate::sha256_hex(&bytes) != item.sha256 {
                bail!("retained Weles evidence {relative} differs from the signed inventory");
            }
            Ok(json!({
                "kind": item.kind,
                "uri": item.uri,
                "local_path": relative,
                "sha256": item.sha256,
                "bytes": item.bytes,
            }))
        })
        .collect::<Result<_>>()?;
    run["evidence_inventory"] = Value::Array(inventory.clone());
    // The importer side of the boundary. `weles_evidence_inventory` is the RECORD-level
    // statement that this record has confirmed browser material, so only the accepted
    // completed attempt writes it; a non-success attempt contributes its per-run
    // `evidence_inventory` and its provenance reference and nothing record-level. No
    // command in this repository reads `weles_evidence_inventory` today, so this is a
    // guard on the record's own claim, NOT what stops a failure from being counted as
    // confirmation: that is enforced by `VerifiedProvenanceSet::supports_value`, which
    // refuses any document whose signed outcome is not the successful one and through
    // which both consumers classify every item.
    if confirms_record {
        let object = record
            .as_object_mut()
            .context("reference record is not an object")?;
        object.insert("weles_evidence_inventory".into(), Value::Array(inventory));
    }
    let _ = attempt_dir;
    Ok(())
}

/// Import the signed failure proofs of this record's other published web attempts.
///
/// A non-success attempt is never the record's source. Its receipt, evidence manifest and
/// retained evidence are signed and delivered all the same, so its provenance document
/// belongs in `provenance_documents`, where `VerifiedProvenanceSet::verify_record`
/// re-verifies it on every run instead of leaving it outside every path. Nothing about the
/// record's confirmed material changes: `apply_web_attempt` is called with
/// `confirms_record: false`, so no record-level evidence inventory is written, and
/// `supports_value` refuses a document whose outcome is not the successful one, so the
/// proof cannot back a single claim.
///
/// One unreadable or unpublished failed attempt is a diagnostic, never a reason to fail
/// the accepted attempt's import: the returned values are recorded on the import summary.
pub(crate) fn import_non_success_attempts(
    run_id: &str,
    catalog: &str,
    accepted: &RuntimeManifest,
    entry: &Value,
    record: &mut Value,
    record_dir: &Path,
    run_dir: &Path,
) -> Vec<Value> {
    let mut imported = Vec::new();
    let attempts = entry
        .get("attempts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for snapshot in &attempts {
        let Some(manifest) = snapshot
            .get("manifest")
            .cloned()
            .and_then(|value| serde_json::from_value::<RuntimeManifest>(value).ok())
        else {
            continue;
        };
        if manifest.engine != "web"
            || manifest.run_id != accepted.run_id
            || manifest.catalog != accepted.catalog
            || manifest.record != accepted.record
            || manifest.record_key != accepted.record_key
            || manifest.attempt_id == accepted.attempt_id
        {
            continue;
        }
        match import_non_success_attempt(&manifest, snapshot, record, record_dir, run_dir) {
            Ok(Some(summary)) => imported.push(summary),
            Ok(None) => {}
            Err(error) => imported.push(json!({
                "attempt_id": manifest.attempt_id,
                "state": "not_imported",
                "message": format!("{error:#}"),
            })),
        }
    }
    let _ = (run_id, catalog);
    imported
}
