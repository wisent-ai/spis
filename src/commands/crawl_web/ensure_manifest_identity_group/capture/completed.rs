use super::*;

/// A completed task: check its receipt, retain its signed evidence, verify it through the
/// official bridge, and retain the provenance and the attempt envelope.
#[allow(clippy::too_many_arguments)]
pub(super) fn retain_completed(
    manifest: &super::super::crawl::RuntimeManifest,
    attempt_root: &Path,
    private: &PrivateBridge,
    collected: &mut Collected,
    status: &weles::WelesTaskStatus,
    submission: &weles::WelesSubmission,
    identity: weles::WelesServiceIdentity,
    binding: weles::WelesAttemptBinding,
    base: String,
    organization_id: String,
    origin: String,
    product_url: String,
    parsed: url::Url,
    weles_task_id: String,
) -> Outcome<()> {
    let checkpoint = status.receipt_checkpoint.as_ref().ok_or_else(|| {
        WorkerFailure::new(
            "weles_receipt_checkpoint_absent",
            "a completed Weles task must carry a freshly verified receipt checkpoint",
        )
    })?;
    let result_digest = status
        .result_digest
        .clone()
        .filter(|digest| is_sha256_id(digest))
        .ok_or_else(|| {
            WorkerFailure::new(
                "weles_status_invalid",
                "the completed task carries no sha256: result digest",
            )
        })?;
    // The public task-status contract returns the task identity, the request identity,
    // the outcome, the result digest and the receipt, and never a result or artifact
    // reference; the bridge therefore normalizes both to `null`/`[]`. Retained evidence is
    // addressed by this task's own recording prefix below, so nothing here may demand a
    // reference the service never signs — but any reference that IS reported has to
    // belong to exactly this recording.
    let prefix = format!("stado://weles/recordings/{weles_task_id}/");
    ensure(
        status
            .result_ref
            .iter()
            .chain(status.artifact_refs.iter())
            .all(|reference| reference.starts_with(&prefix)),
        "weles_artifact_refs_foreign",
        "a task result or artifact reference is not bound to this exact Weles recording",
    )?;

    let claims = &checkpoint.claims;
    ensure(
        claims.task_id == weles_task_id
            && claims.organization_id == organization_id
            && claims.origin == origin
            && claims.action == weles::SPIS_WELES_ACTION
            && claims.outcome == "completed",
        "weles_receipt_claims_mismatch",
        "the verified receipt claims do not name this completed task",
    )?;
    ensure(
        claims.request_digest == submission.request_digest
            && claims.result_digest == result_digest
            && claims.spis_binding == binding,
        "weles_receipt_claims_mismatch",
        "the verified receipt does not sign this exact request, result and Spis binding",
    )?;

    let stado_job_id = stado_job_id(&weles_task_id)?;

    let recordings = attempt_root.join("recordings").join(&weles_task_id);
    let evidence_manifest_uri = format!("{prefix}evidence-manifest.json");
    let evidence_manifest_path = recordings.join("evidence-manifest.json");
    storage_get(&evidence_manifest_uri, &evidence_manifest_path)?;
    // These exact bytes are the receipt-bound artifact. Re-serializing would change the
    // digest the receipt signed, so they are only ever copied.
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
        EVIDENCE_MANIFEST_SCHEMA,
        "completed",
        &weles_task_id,
        claims,
        &submission.request_digest,
        &result_digest,
        &binding,
        &product_url,
    )?;
    // Only the successful version signs a navigation, so only here are the effective and
    // final URL required and bound to the product origin.
    let effective_url = text(&evidence, "effectiveUrl")?.to_string();
    let final_url = text(&evidence, "finalUrl")?.to_string();
    ensure(
        same_origin(&effective_url, &parsed) && same_origin(&final_url, &parsed),
        "weles_evidence_manifest_invalid",
        "the signed effective/final URLs are not same-origin with the product URL",
    )?;

    let inventory = signed_inventory(&evidence)?;
    let retained_paths =
        retain_signed_inventory(&recordings, &weles_task_id, &prefix, &inventory, true)?;

    let observation_document_sha256 = retain_observation_document(
        attempt_root,
        manifest,
        &weles_task_id,
        &product_url,
        "completed",
        Some((&effective_url, &final_url)),
        &inventory,
        &retained_paths,
    )?;

    let expected_claims = weles::ExpectedReceiptClaims {
        task_id: weles_task_id.clone(),
        organization_id: organization_id.clone(),
        request_digest: submission.request_digest.clone(),
        result_digest: result_digest.clone(),
        spis_binding: binding.clone(),
        origin: origin.clone(),
        action: weles::SPIS_WELES_ACTION.to_string(),
        outcome: "completed".to_string(),
        evidence_digest: artifact_document_sha256.clone(),
    };
    let artifact = weles::RetainedArtifact {
        path: artifact_relative,
        sha256: artifact_document_sha256.clone(),
        bytes: manifest_bytes.len() as u64,
    };
    // The bridge resolves `artifact.path` against its working directory, so the official
    // client re-reads and re-digests exactly the retained bytes.
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
        fresh
            .id
            .strip_prefix("sha256:")
            .is_some_and(is_sha256),
        "weles_provenance_id_invalid",
        "the bridge verification document has no framed sha256: identifier",
    )?;
    let provenance = weles::WelesProvenanceDocument {
        schema: weles::PROVENANCE_DOCUMENT_SCHEMA.to_string(),
        // The framed provenance id is derived by the official bridge from the receipt, the
        // trusted key-set version and the artifact; Rust only re-checks its shape above.
        id: fresh.id.clone(),
        client: checkpoint.client.clone(),
        receipt: checkpoint.receipt.clone(),
        claims: checkpoint.claims.clone(),
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
    collected.provenance = Some(provenance);

    let envelope = weles::WelesAttemptEnvelope {
        schema: weles::ATTEMPT_ENVELOPE_SCHEMA.to_string(),
        run_id: manifest.run_id.clone(),
        catalog: manifest.catalog.clone(),
        record: manifest.record.clone(),
        record_key: manifest.record_key.clone(),
        attempt: manifest.attempt,
        attempt_id: manifest.attempt_id.clone(),
        stado_job_id,
        weles_task_id,
        state: "completed".to_string(),
        outcome: Some("completed".to_string()),
        service_identity: identity,
        source_revision: manifest.source_revision.clone(),
        source_input_sha256: manifest.source_input_sha256.clone(),
        reference_sha256: manifest.reference_sha256.clone(),
        spis_binding: binding,
        weles_request_document: submission.request_document.clone(),
        weles_request_digest: submission.request_digest.clone(),
        weles_result_digest: Some(result_digest),
        requested_url: product_url,
        final_url: Some(final_url),
        evidence_inventory: inventory,
        weles_evidence_manifest_uri: evidence_manifest_uri,
        weles_evidence_manifest_sha256: Some(artifact_document_sha256.clone()),
        artifact_document_uri: format!("{base}/weles/artifacts/{artifact_document_sha256}.json"),
        artifact_document_sha256: Some(artifact_document_sha256),
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
    Ok(())
}
