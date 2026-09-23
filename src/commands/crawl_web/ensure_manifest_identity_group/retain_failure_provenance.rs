use super::*;

/// Retains the signed proof that this attempt's Weles task ended in a terminal
/// NON-SUCCESS: failed, cancelled or rejected, together with the observation document and
/// the attempt envelope that let a record re-verify that proof.
///
/// The service signs and delivers a receipt for those outcomes exactly as it does for a
/// completed one, and retains their evidence manifest in the v2 shape — the same
/// contractual `evidence-manifest.json` address under the task's own recording prefix, with
/// no navigation URLs and no required evidence kind, because there was no navigation and
/// nothing is demanded of a run that failed. Without this the attempt would carry only the
/// status text, so its failure would rest on this worker's word instead of the service's
/// signature.
#[allow(clippy::too_many_arguments)]
pub(crate) fn retain_failure_provenance(
    manifest: &super::crawl::RuntimeManifest,
    attempt_root: &Path,
    private: &PrivateBridge,
    status: &weles::WelesTaskStatus,
    submission: &weles::WelesSubmission,
    binding: &weles::WelesAttemptBinding,
    identity: &weles::WelesServiceIdentity,
    base: &str,
    organization_id: &str,
    origin: &str,
    product_url: &str,
    weles_task_id: &str,
    outcome: &str,
    collected: &mut Collected,
) -> Outcome<weles::WelesProvenanceDocument> {
    let checkpoint = status.receipt_checkpoint.as_ref().ok_or_else(|| {
        WorkerFailure::new(
            "weles_receipt_checkpoint_absent",
            "a terminal Weles task must carry a freshly verified receipt checkpoint",
        )
    })?;
    let result_digest = status
        .result_digest
        .clone()
        .filter(|digest| is_sha256_id(digest))
        .ok_or_else(|| {
            WorkerFailure::new(
                "weles_status_invalid",
                "the terminal task carries no sha256: result digest",
            )
        })?;
    let claims = &checkpoint.claims;
    ensure(
        claims.task_id == weles_task_id
            && claims.organization_id == organization_id
            && claims.origin == origin
            && claims.action == weles::SPIS_WELES_ACTION
            && claims.outcome == outcome,
        "weles_receipt_claims_mismatch",
        "the verified receipt claims do not name this exact task and terminal outcome",
    )?;
    ensure(
        claims.request_digest == submission.request_digest
            && claims.result_digest == result_digest
            && claims.spis_binding == *binding,
        "weles_receipt_claims_mismatch",
        "the verified receipt does not sign this exact request, result and Spis binding",
    )?;

    let prefix = format!("stado://weles/recordings/{weles_task_id}/");
    let recordings = attempt_root.join("recordings").join(weles_task_id);
    let evidence_manifest_path = recordings.join("evidence-manifest.json");
    storage_get(&format!("{prefix}evidence-manifest.json"), &evidence_manifest_path)?;
    // These exact bytes are the receipt-bound artifact, so they are only ever copied.
    let manifest_bytes = std::fs::read(&evidence_manifest_path)?;
    let artifact_document_sha256 = crate::sha256_hex(&manifest_bytes);
    let artifact_relative = format!("weles/artifacts/{artifact_document_sha256}.json");
    write_exact(&attempt_root.join(&artifact_relative), &manifest_bytes)?;
    ensure(
        claims.evidence_digest == artifact_document_sha256,
        "weles_evidence_digest_mismatch",
        "the verified receipt evidenceDigest is not the retained evidence manifest digest",
    )?;

    let evidence: Value = serde_json::from_slice(&manifest_bytes)?;
    ensure_manifest_identity(
        &evidence,
        NON_SUCCESS_EVIDENCE_MANIFEST_SCHEMA,
        outcome,
        weles_task_id,
        claims,
        &submission.request_digest,
        &result_digest,
        binding,
        product_url,
    )?;
    // Absent, not empty: the v2 shape carries no navigation, and the service refuses to
    // write one, so a document that has either field is not the version it declares.
    ensure(
        evidence.get("effectiveUrl").is_none() && evidence.get("finalUrl").is_none(),
        "weles_evidence_manifest_invalid",
        "a non-success evidence manifest must carry no effective or final URL",
    )?;
    let inventory = signed_inventory(&evidence)?;
    let retained_paths =
        retain_signed_inventory(&recordings, weles_task_id, &prefix, &inventory, false)?;
    let observation_document_sha256 = retain_observation_document(
        attempt_root,
        manifest,
        weles_task_id,
        product_url,
        outcome,
        None,
        &inventory,
        &retained_paths,
    )?;

    let expected_claims = weles::ExpectedReceiptClaims {
        task_id: weles_task_id.to_string(),
        organization_id: organization_id.to_string(),
        request_digest: submission.request_digest.clone(),
        result_digest,
        spis_binding: binding.clone(),
        origin: origin.to_string(),
        action: weles::SPIS_WELES_ACTION.to_string(),
        outcome: outcome.to_string(),
        evidence_digest: artifact_document_sha256.clone(),
    };
    let artifact = weles::RetainedArtifact {
        path: artifact_relative,
        sha256: artifact_document_sha256,
        bytes: manifest_bytes.len() as u64,
    };
    // The same secretless verification the completed path runs: the bridge re-reads and
    // re-digests the retained bytes and validates them in the version this outcome
    // mandates.
    let verify_command = json!({
        "schema": weles::BRIDGE_COMMAND_SCHEMA,
        "operation": "verify",
        "receipt": serde_json::to_value(&checkpoint.receipt)?,
        "expectedClaims": serde_json::to_value(&expected_claims)?,
        "artifact": serde_json::to_value(&artifact)?,
    });
    let stdout = run_bridge(attempt_root, private, "verify", &verify_command, None, false)?;
    let fresh: weles::WelesProvenanceDocument = serde_json::from_slice(&stdout)?;
    ensure(
        fresh.id.strip_prefix("sha256:").is_some_and(is_sha256),
        "weles_provenance_id_invalid",
        "the bridge verification document has no framed sha256: identifier",
    )?;
    let provenance = weles::WelesProvenanceDocument {
        schema: weles::PROVENANCE_DOCUMENT_SCHEMA.to_string(),
        id: fresh.id.clone(),
        client: checkpoint.client.clone(),
        receipt: checkpoint.receipt.clone(),
        claims: claims.clone(),
        expected_claims,
        artifact,
    };
    ensure(
        fresh == provenance,
        "weles_provenance_mismatch",
        "the fresh official verification differs from the assembled provenance document",
    )?;
    retain_attempt_document(
        attempt_root,
        "weles-provenance.json",
        &serde_json::to_value(&provenance)?,
    )?;
    collected.provenance = Some(provenance.clone());

    // The attempt envelope is what binds this proof to the record at verification time:
    // `verify_attempt_binding` resolves a provenance document through the crawl run's
    // envelope, so a failure proof without one could never be re-verified inside a record.
    // It carries the signed outcome under both `state` and `outcome`, and NO final URL,
    // because the v2 manifest signs no navigation for the verifier to compare against.
    let envelope = weles::WelesAttemptEnvelope {
        schema: weles::ATTEMPT_ENVELOPE_SCHEMA.to_string(),
        run_id: manifest.run_id.clone(),
        catalog: manifest.catalog.clone(),
        record: manifest.record.clone(),
        record_key: manifest.record_key.clone(),
        attempt: manifest.attempt,
        attempt_id: manifest.attempt_id.clone(),
        stado_job_id: stado_job_id(weles_task_id)?,
        weles_task_id: weles_task_id.to_string(),
        state: outcome.to_string(),
        outcome: Some(outcome.to_string()),
        service_identity: identity.clone(),
        source_revision: manifest.source_revision.clone(),
        source_input_sha256: manifest.source_input_sha256.clone(),
        reference_sha256: manifest.reference_sha256.clone(),
        spis_binding: binding.clone(),
        weles_request_document: submission.request_document.clone(),
        weles_request_digest: submission.request_digest.clone(),
        weles_result_digest: Some(provenance.expected_claims.result_digest.clone()),
        requested_url: product_url.to_string(),
        final_url: None,
        evidence_inventory: inventory,
        weles_evidence_manifest_uri: format!("{prefix}evidence-manifest.json"),
        weles_evidence_manifest_sha256: Some(provenance.artifact.sha256.clone()),
        artifact_document_uri: format!("{base}/weles/artifacts/{}.json", provenance.artifact.sha256),
        artifact_document_sha256: Some(provenance.artifact.sha256.clone()),
        observation_document_uri: format!(
            "{base}/weles/observations/{observation_document_sha256}.json"
        ),
        observation_document_sha256,
    };
    retain_attempt_document(
        attempt_root,
        "attempt-envelope.json",
        &serde_json::to_value(&envelope)?,
    )?;
    collected.envelope = Some(envelope);
    Ok(provenance)
}

pub(crate) fn storage_get(uri: &str, destination: &Path) -> Outcome<()> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut stado = super::crawl::stado_command();
    stado.args(["storage", "get", uri]).arg(destination);
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "download retained Weles evidence",
        Duration::from_secs(300),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        return Err(WorkerFailure::new(
            "weles_evidence_download_failed",
            format!(
                "stado storage get refused {uri}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(())
}

pub(crate) fn worker_report(
    manifest: &super::crawl::RuntimeManifest,
    state: &str,
    artifact: Option<Value>,
    collected: &Collected,
    failure: Option<&WorkerFailure>,
) -> Value {
    json!({
        "schema": REPORT_SCHEMA,
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "record_key": manifest.record_key,
        "attempt": u64::from(manifest.attempt),
        "attempt_id": manifest.attempt_id,
        "engine": "web",
        "state": state,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "reference_sha256": manifest.reference_sha256,
        "bindings_file_sha256": manifest.bindings_file_sha256,
        "bindings_sha256": manifest.bindings_sha256,
        "execution_identity": serde_json::to_value(&manifest.execution_identity)
            .unwrap_or(Value::Null),
        "artifact": artifact.unwrap_or(Value::Null),
        "weles_attempt_envelope": serde_json::to_value(&collected.envelope)
            .unwrap_or(Value::Null),
        "weles_submission": serde_json::to_value(&collected.submission).unwrap_or(Value::Null),
        "weles_task_status": serde_json::to_value(&collected.status).unwrap_or(Value::Null),
        "weles_cancellation": serde_json::to_value(&collected.cancellation)
            .unwrap_or(Value::Null),
        "provenance_document": serde_json::to_value(&collected.provenance)
            .unwrap_or(Value::Null),
        "failure": failure
            .map(|failure| json!({"code": failure.code, "message": failure.message}))
            .unwrap_or(Value::Null),
    })
}
