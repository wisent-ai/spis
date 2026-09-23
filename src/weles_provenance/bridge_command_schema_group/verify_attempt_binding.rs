use super::*;

pub(crate) fn verify_attempt_binding(
    record: &Value,
    record_dir: &Path,
    document: &WelesProvenanceDocument,
    artifact_value: &Value,
    trust: &WelesReceiptTrust,
) -> Result<(), String> {
    let product_url = record
        .get("product_url")
        .and_then(Value::as_str)
        .ok_or_else(|| "current record has no product_url for receipt origin binding".to_string())?;
    let parsed_product_url = url::Url::parse(product_url)
        .map_err(|_| "current record product_url is not a valid URL".to_string())?;
    if !matches!(parsed_product_url.scheme(), "http" | "https") {
        return Err("current record product_url is not HTTP(S)".to_string());
    }
    if document.expected_claims.origin != parsed_product_url.origin().ascii_serialization() {
        return Err(
            "verified receipt origin does not match the current record product_url origin"
                .to_string(),
        );
    }
    if document.expected_claims.action != SPIS_WELES_ACTION
        || document.expected_claims.action != trust.allowed_action
    {
        return Err("verified receipt action is not the trusted Spis browser action".to_string());
    }

    let record_name = record_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "current record directory has no portable record name".to_string())?;
    let catalog_name = record_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .ok_or_else(|| "current record directory has no portable catalog name".to_string())?;
    let runs = record
        .get("crawl_runs")
        .and_then(Value::as_array)
        .ok_or_else(|| "current record has no imported crawl-run attempts".to_string())?;
    let mut matched: Option<(WelesAttemptEnvelope, &Value)> = None;
    for run in runs {
        let Some(envelope_value) = run.get("weles_attempt_envelope") else {
            continue;
        };
        let envelope: WelesAttemptEnvelope = serde_json::from_value(envelope_value.clone())
            .map_err(|_| {
                "imported Weles attempt envelope does not match the typed schema".to_string()
            })?;
        if envelope.weles_task_id != document.expected_claims.task_id {
            continue;
        }
        if matched.is_some() {
            return Err("current record repeats the receipt task across attempts".to_string());
        }
        matched = Some((envelope, run));
    }
    let Some((envelope, run)) = matched else {
        return Err("verified receipt taskId is not the imported inner Weles task".to_string());
    };
    let outcome = document.expected_claims.outcome.as_str();
    let successful = outcome == SUCCESSFUL_OUTCOME;
    let mut required = vec![
        envelope.run_id.as_str(),
        envelope.catalog.as_str(),
        envelope.record.as_str(),
        envelope.record_key.as_str(),
        envelope.attempt_id.as_str(),
        envelope.stado_job_id.as_str(),
        envelope.weles_task_id.as_str(),
        envelope.source_revision.as_str(),
        envelope.requested_url.as_str(),
        envelope.weles_evidence_manifest_uri.as_str(),
        envelope.artifact_document_uri.as_str(),
        envelope.observation_document_uri.as_str(),
    ];
    // A completed attempt signs a navigation and must name its final URL; a non-success
    // has none to name, and the v2 manifest has no field to compare it against, so the
    // envelope must leave it out rather than fill it in.
    match (successful, envelope.final_url.as_deref()) {
        (true, Some(final_url)) => required.push(final_url),
        (false, None) => {}
        _ => {
            return Err(
                "attempt envelope final URL is present exactly when the outcome is completed"
                    .to_string(),
            )
        }
    }
    if envelope.schema != ATTEMPT_ENVELOPE_SCHEMA
        || envelope.attempt == 0
        || !is_git_revision(&envelope.source_revision)
        || !is_sha256(&envelope.record_key)
        || !is_sha256(&envelope.source_input_sha256)
        || !is_sha256(&envelope.reference_sha256)
        || required.iter().any(|value| value.trim().is_empty())
        || envelope.stado_job_id == envelope.weles_task_id
        // The attempt reports exactly the outcome the receipt signed, under both names.
        || !is_terminal_outcome(&envelope.state)
        || envelope.state != outcome
        || envelope.outcome.as_deref() != Some(outcome)
        || !is_sha256_id(&envelope.weles_request_digest)
        || !envelope
            .weles_result_digest
            .as_deref()
            .is_some_and(is_sha256_id)
        || !envelope
            .weles_evidence_manifest_sha256
            .as_deref()
            .is_some_and(is_sha256)
        || !envelope
            .artifact_document_sha256
            .as_deref()
            .is_some_and(is_sha256)
        || !is_sha256(&envelope.observation_document_sha256)
    {
        return Err(
            "imported Weles attempt envelope is not a typed attempt of the signed outcome"
                .to_string(),
        );
    }
    let weles_evidence_manifest_sha256 = envelope
        .weles_evidence_manifest_sha256
        .as_deref()
        .expect("validated terminal Weles evidence manifest digest");
    let artifact_document_sha256 = envelope
        .artifact_document_sha256
        .as_deref()
        .expect("validated terminal artifact document digest");
    if envelope.catalog != catalog_name
        || envelope.record != record_name
        || run.get("run_id").and_then(Value::as_str) != Some(envelope.run_id.as_str())
        || run.get("stado_job_id").and_then(Value::as_str)
            != Some(envelope.stado_job_id.as_str())
        || run.get("record_key").and_then(Value::as_str) != Some(envelope.record_key.as_str())
        || run.get("attempt").and_then(Value::as_u64) != Some(u64::from(envelope.attempt))
        || run.get("attempt_id").and_then(Value::as_str) != Some(envelope.attempt_id.as_str())
        || run.get("state").and_then(Value::as_str) != Some(envelope.state.as_str())
        || run.get("outcome").and_then(Value::as_str) != envelope.outcome.as_deref()
        || run.get("source_revision").and_then(Value::as_str)
            != Some(envelope.source_revision.as_str())
        || run.get("source_input_sha256").and_then(Value::as_str)
            != Some(envelope.source_input_sha256.as_str())
        || run.get("reference_sha256").and_then(Value::as_str)
            != Some(envelope.reference_sha256.as_str())
        || run.get("weles_evidence_manifest_uri").and_then(Value::as_str)
            != Some(envelope.weles_evidence_manifest_uri.as_str())
        || run.get("weles_evidence_manifest_sha256").and_then(Value::as_str)
            != Some(weles_evidence_manifest_sha256)
        || run.get("artifact_document_uri").and_then(Value::as_str)
            != Some(envelope.artifact_document_uri.as_str())
        || run.get("artifact_document_sha256").and_then(Value::as_str)
            != Some(artifact_document_sha256)
        || run.get("observation_document_uri").and_then(Value::as_str)
            != Some(envelope.observation_document_uri.as_str())
        || run.get("observation_document_sha256").and_then(Value::as_str)
            != Some(envelope.observation_document_sha256.as_str())
    {
        return Err(
            "outer typed crawl run differs from the imported Weles attempt coordinates"
                .to_string(),
        );
    }
    if envelope.service_identity.action != SPIS_WELES_ACTION
        || envelope.service_identity.action != trust.allowed_action
    {
        return Err("attempt service identity action differs from public receipt trust".to_string());
    }
    validate_service_identity(&envelope.service_identity)?;
    validate_attempt_uris(&envelope)?;
    validate_spis_binding(&envelope.spis_binding)?;
    let expected_binding_service = WelesAttemptBindingService {
        name: envelope.service_identity.name.clone(),
        consumer: envelope.service_identity.consumer.clone(),
        capability: envelope.service_identity.capability.clone(),
        directory_generation: envelope.service_identity.generation,
        host: envelope.service_identity.active_host.clone(),
        endpoint: envelope.service_identity.endpoint.clone(),
        action: envelope.service_identity.action.clone(),
        release_id: envelope.service_identity.release_id.clone(),
        source_revision: envelope.service_identity.source_revision.clone(),
    };
    let binding = &envelope.spis_binding;
    if binding.run_id != envelope.run_id
        || binding.catalog != envelope.catalog
        || binding.record != envelope.record
        || binding.record_key != envelope.record_key
        || binding.attempt != envelope.attempt
        || binding.attempt_id != envelope.attempt_id
        || binding.source_revision != envelope.source_revision
        || binding.source_input_sha256 != envelope.source_input_sha256
        || binding.reference_sha256 != envelope.reference_sha256
        || binding.service != expected_binding_service
        || document.expected_claims.request_digest != envelope.weles_request_digest
        || document.expected_claims.result_digest
            != envelope
                .weles_result_digest
                .as_deref()
                .expect("validated terminal result digest")
        || document.expected_claims.spis_binding != *binding
        || document.artifact.sha256 != artifact_document_sha256
        || artifact_document_sha256 != weles_evidence_manifest_sha256
    {
        return Err(
            "verified receipt request/result/binding/artifact differs from the attempt envelope"
                .to_string(),
        );
    }
    // The signed outcome, already proved above to be the receipt's own claim, chooses the
    // manifest version. Nothing else may: a document that does not match the version its
    // outcome mandates is refused, in either direction.
    let manifest =
        ReceiptBoundManifest::parse(artifact_value, &document.expected_claims.outcome)?;
    validate_request_and_evidence_manifest(
        &envelope,
        binding,
        document,
        &parsed_product_url,
        &manifest,
        record_dir,
    )?;
    Ok(())
}

pub(crate) fn validate_attempt_uris(envelope: &WelesAttemptEnvelope) -> Result<(), String> {
    for (label, component) in [
        ("run_id", envelope.run_id.as_str()),
        ("catalog", envelope.catalog.as_str()),
        ("record", envelope.record.as_str()),
        ("attempt_id", envelope.attempt_id.as_str()),
        ("weles_task_id", envelope.weles_task_id.as_str()),
    ] {
        if !is_portable_attempt_component(component) {
            return Err(format!("{label} is not a portable attempt URI component"));
        }
    }
    let base = crate::crawl_attempt_base_uri(
        &envelope.run_id,
        &envelope.catalog,
        &envelope.record,
        &envelope.record_key,
        envelope.attempt,
        &envelope.attempt_id,
    );
    let artifact_sha256 = envelope
        .artifact_document_sha256
        .as_deref()
        .expect("validated terminal artifact document digest");
    if envelope.spis_binding.artifact_uri != format!("{base}/artifacts.tar.gz")
        || envelope.spis_binding.output_uri != format!("{base}/worker-output.log")
        || envelope.weles_evidence_manifest_uri
            != format!(
                "stado://weles/recordings/{}/evidence-manifest.json",
                envelope.weles_task_id
            )
        || envelope.artifact_document_uri
            != format!("{base}/weles/artifacts/{artifact_sha256}.json")
        || envelope.observation_document_uri
            != format!(
                "{base}/weles/observations/{}.json",
                envelope.observation_document_sha256
            )
    {
        return Err("Weles attempt URI is not the canonical coordinate reconstruction".to_string());
    }
    Ok(())
}

pub(crate) fn is_portable_attempt_component(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
        })
}
