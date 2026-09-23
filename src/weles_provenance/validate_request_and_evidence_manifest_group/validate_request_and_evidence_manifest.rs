use super::*;

pub(crate) fn validate_request_and_evidence_manifest(
    envelope: &WelesAttemptEnvelope,
    binding: &WelesAttemptBinding,
    document: &WelesProvenanceDocument,
    product_url: &url::Url,
    manifest: &ReceiptBoundManifest,
    record_dir: &Path,
) -> Result<(), String> {
    let request = &envelope.weles_request_document;
    if request.schema != "weles.task.current"
        || request.organization_id != document.expected_claims.organization_id
        || request.origin != document.expected_claims.origin
        || request.origin != product_url.origin().ascii_serialization()
        || request.action != document.expected_claims.action
        || request.input.spis_binding != *binding
        || request.input.objective.trim().is_empty()
        || !request.credential_refs.is_empty()
        || request.evidence_policy != "full"
        || request.justification.trim().is_empty()
    {
        return Err("retained official request differs from signed attempt claims".to_string());
    }
    validate_unique_nonempty(&request.input.constraints, "request constraints")?;
    let request_value = serde_json::to_value(request)
        .map_err(|_| "retained official request could not be canonicalized".to_string())?;
    let request_digest = format!("sha256:{}", canonical_json_sha256(&request_value)?);
    if request_digest != envelope.weles_request_digest
        || request_digest != document.expected_claims.request_digest
    {
        return Err("canonical official request digest differs from the signed claim".to_string());
    }

    let requested_url = parse_http_url(&request.input.product_url, "requested product URL")?;
    let envelope_requested_url = parse_http_url(&envelope.requested_url, "attempt requested URL")?;
    // The envelope names a final URL exactly when the attempt completed, which
    // `verify_attempt_binding` already proved against the signed outcome.
    let envelope_final_origin_ok = match envelope.final_url.as_deref() {
        Some(final_url) => {
            parse_http_url(final_url, "attempt final URL")?.origin() == product_url.origin()
        }
        None => true,
    };
    if requested_url != *product_url
        || envelope_requested_url != *product_url
        || request.input.product_url != product_url.as_str()
        || request.input.product_url != envelope.requested_url
        || !envelope_final_origin_ok
    {
        return Err(
            "browser request/final URL differs from the canonical current product URL policy"
                .to_string(),
        );
    }

    validate_evidence_inventory(
        &envelope.evidence_inventory,
        &envelope.weles_task_id,
        record_dir,
        document.expected_claims.outcome == SUCCESSFUL_OUTCOME,
    )?;
    let facts = manifest.facts();
    let manifest_requested_url = parse_http_url(facts.requested_url, "manifest requested URL")?;
    // Only the successful version signs a navigation, and only it can be compared with the
    // envelope's final URL; the non-success version has neither field at all.
    let navigation_matches = match facts.navigation {
        Some((effective_url, final_url)) => {
            let manifest_effective_url = parse_http_url(effective_url, "manifest effective URL")?;
            let manifest_final_url = parse_http_url(final_url, "manifest final URL")?;
            manifest_effective_url.origin() == product_url.origin()
                && manifest_final_url.origin() == product_url.origin()
                && Some(final_url) == envelope.final_url.as_deref()
        }
        None => true,
    };
    if facts.schema != facts.expected_schema
        || facts.task_id != envelope.weles_task_id
        || facts.organization_id != document.expected_claims.organization_id
        || facts.origin != document.expected_claims.origin
        || facts.action != document.expected_claims.action
        || facts.outcome != document.expected_claims.outcome
        || facts.request_digest != document.expected_claims.request_digest
        || facts.result_digest != document.expected_claims.result_digest
        || *facts.spis_binding != *binding
        || manifest_requested_url != *product_url
        || facts.requested_url != envelope.requested_url
        || !navigation_matches
        || facts.evidence_inventory != envelope.evidence_inventory
    {
        return Err(
            "receipt-bound evidence manifest differs from the signed request/result/attempt"
                .to_string(),
        );
    }
    Ok(())
}

pub(crate) fn parse_http_url(value: &str, label: &str) -> Result<url::Url, String> {
    let parsed = url::Url::parse(value).map_err(|_| format!("{label} is invalid"))?;
    if !matches!(parsed.scheme(), "http" | "https")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(format!("{label} is not an HTTP(S) URL without credentials"));
    }
    Ok(parsed)
}

pub(crate) fn validate_unique_nonempty(values: &[String], label: &str) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    if values
        .iter()
        .any(|value| value.trim().is_empty() || !seen.insert(value.as_str()))
    {
        return Err(format!("{label} contain an empty or duplicate entry"));
    }
    Ok(())
}

pub(crate) fn validate_evidence_inventory(
    entries: &[WelesEvidenceInventoryEntry],
    task_id: &str,
    record_dir: &Path,
    require_browser_evidence: bool,
) -> Result<(), String> {
    let prefix = format!("stado://weles/recordings/{task_id}/");
    let screenshot_uri = format!("{prefix}artifacts/browser_evidence_final.png");
    let accessibility_uri =
        format!("{prefix}artifacts/browser_evidence_accessibility_tree.txt");
    let mut kinds = BTreeSet::new();
    let mut uris = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for entry in entries {
        let relative_uri = entry
            .uri
            .strip_prefix(&prefix)
            .ok_or_else(|| {
                "evidence inventory URI is not bound to the exact Weles task".to_string()
            })?;
        // Both receiver layers must judge the same document by the same rule: the bridge
        // requires every component to pass `portableAttemptComponent`, and the service
        // refuses anything outside that alphabet at retention time, so accepting a merely
        // `Component::Normal` name here would leave this layer the weaker of the two.
        if relative_uri.is_empty()
            || relative_uri.contains('\\')
            || !relative_uri.split('/').all(is_portable_attempt_component)
            || !Path::new(relative_uri)
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err("evidence inventory URI is not a canonical immutable path".to_string());
        }
        let kind_matches_uri = match entry.kind.as_str() {
            "screenshot" => entry.uri == screenshot_uri,
            "accessibility_tree" => entry.uri == accessibility_uri,
            kind => kind
                .strip_prefix("artifact:")
                .is_some_and(|path| path == relative_uri),
        };
        total_bytes = total_bytes
            .checked_add(entry.bytes)
            .filter(|total| *total <= MAX_RETAINED_EVIDENCE_BYTES)
            .ok_or_else(|| "retained evidence inventory exceeds the total byte limit".to_string())?;
        if !kind_matches_uri
            || !is_sha256(&entry.sha256)
            || entry.bytes == 0
            || !kinds.insert(entry.kind.as_str())
            || !uris.insert(entry.uri.as_str())
        {
            return Err("evidence inventory contains an invalid or duplicate entry".to_string());
        }
        let retained_path = format!("recordings/{task_id}/{relative_uri}");
        let retained_file = resolve_retained_file(record_dir, &retained_path)?;
        let retained_bytes = read_limited(&retained_file, entry.bytes)?;
        if retained_bytes.len() as u64 != entry.bytes
            || sha256_bytes(&retained_bytes) != entry.sha256
        {
            return Err("retained evidence bytes differ from the signed inventory".to_string());
        }
    }
    // Labelling above is by exact URI and outcome-independent; the DEMAND below applies
    // only to a completed attempt, exactly as the service demands them only from a
    // succeeded task and as the bridge and the worker already scope it. A cancelled task
    // that never captured anything signs an inventory that is legitimately without them.
    if require_browser_evidence
        && (!kinds.contains("screenshot") || !kinds.contains("accessibility_tree"))
    {
        return Err(
            "evidence inventory lacks the required screenshot/accessibility_tree artifacts"
                .to_string(),
        );
    }
    Ok(())
}

pub(crate) fn validate_document_shape(document: &WelesProvenanceDocument) -> Result<(), String> {
    if document.schema != PROVENANCE_DOCUMENT_SCHEMA {
        return Err("verification document schema is unsupported".to_string());
    }
    if document.client.package != OFFICIAL_CLIENT_PACKAGE
        || document.client.commit != OFFICIAL_CLIENT_COMMIT
        || document.client.key_set_version.trim().is_empty()
    {
        return Err("verification document does not name the pinned official client and key set".to_string());
    }
    let receipt = &document.receipt;
    let expected = &document.expected_claims;
    if receipt.schema != "weles.receipt.current"
        || receipt.task_id.trim().is_empty()
        || receipt.organization_id.trim().is_empty()
        || receipt.origin.trim().is_empty()
        || receipt.action.trim().is_empty()
        || !is_terminal_outcome(&receipt.outcome)
        || receipt.evidence_digest.trim().is_empty()
        || receipt.key_id.trim().is_empty()
        || receipt.signature.trim().is_empty()
        || receipt.signed_payload.trim().is_empty()
        || !is_sha256_id(&receipt.request_digest)
        || !is_sha256_id(&receipt.result_digest)
    {
        return Err("retained receipt shape is unsupported".to_string());
    }
    if document.artifact.bytes == 0
        || document.artifact.bytes > MAX_DOCUMENT_BYTES
        || !is_sha256(&document.artifact.sha256)
        || expected.evidence_digest != document.artifact.sha256
    {
        return Err("expected evidenceDigest is not bound to a bounded retained artifact".to_string());
    }
    // Every terminal outcome is verifiable as a document. What the outcome is allowed to
    // support is decided in exactly one other place, `supports_value`, which admits only
    // `SUCCESSFUL_OUTCOME`; a failure proof must be provable without being promotable.
    if !is_terminal_outcome(&expected.outcome)
        || !is_sha256_id(&expected.request_digest)
        || !is_sha256_id(&expected.result_digest)
    {
        return Err(
            "Spis provenance requires a terminal outcome with signed request/result digests"
                .to_string(),
        );
    }
    validate_spis_binding(&expected.spis_binding)?;
    validate_spis_binding(&receipt.spis_binding)?;
    if receipt.task_id != expected.task_id
        || receipt.organization_id != expected.organization_id
        || receipt.origin != expected.origin
        || receipt.action != expected.action
        || receipt.outcome != expected.outcome
        || receipt.evidence_digest != expected.evidence_digest
        || receipt.request_digest != expected.request_digest
        || receipt.result_digest != expected.result_digest
        || receipt.spis_binding != expected.spis_binding
    {
        return Err("retained receipt claim copies differ from caller expectations".to_string());
    }
    Ok(())
}

pub(crate) fn validate_document_trust(
    document: &WelesProvenanceDocument,
    trust: &WelesReceiptTrust,
) -> Result<(), String> {
    if document.expected_claims.organization_id != trust.organization_id
        || document.expected_claims.action != trust.allowed_action
        || document.client.key_set_version != trust.key_set_version
        || !trust.receipt_keys.contains_key(&document.receipt.key_id)
    {
        return Err(
            "verification document differs from the checked-in public receipt trust"
                .to_string(),
        );
    }
    Ok(())
}
