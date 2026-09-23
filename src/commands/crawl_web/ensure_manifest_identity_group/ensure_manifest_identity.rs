use super::*;

/// The identity every retained evidence manifest signs, in either version: v1 for a
/// completed task, v2 for a failed, cancelled or rejected one. The version-specific part —
/// the navigation pair v1 signs and v2 does not have — is checked by the caller.
#[allow(clippy::too_many_arguments)]
pub(crate) fn ensure_manifest_identity(
    evidence: &Value,
    schema: &str,
    outcome: &str,
    weles_task_id: &str,
    claims: &weles::VerifiedReceiptClaims,
    request_digest: &str,
    result_digest: &str,
    binding: &weles::WelesAttemptBinding,
    product_url: &str,
) -> Outcome<()> {
    ensure(
        text(evidence, "schema")? == schema,
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest schema is not the version this outcome mandates",
    )?;
    ensure(
        text(evidence, "taskId")? == weles_task_id
            && text(evidence, "organizationId")? == claims.organization_id
            && text(evidence, "origin")? == claims.origin
            && text(evidence, "action")? == claims.action
            && text(evidence, "outcome")? == outcome,
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest does not name this exact task and outcome",
    )?;
    ensure(
        text(evidence, "requestDigest")? == request_digest
            && text(evidence, "resultDigest")? == result_digest,
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest request/result digests differ from the receipt",
    )?;
    let signed_binding: weles::WelesAttemptBinding =
        serde_json::from_value(evidence.get("spisBinding").cloned().unwrap_or(Value::Null))?;
    ensure(
        signed_binding == *binding,
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest carries a different Spis binding",
    )?;
    ensure(
        text(evidence, "requestedUrl")? == product_url,
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest requestedUrl is not the exact product URL",
    )
}

/// The signed inventory, typed. An empty inventory is a legitimate non-success shape: a
/// task that failed before it produced anything still signs the manifest that says so.
pub(crate) fn signed_inventory(evidence: &Value) -> Outcome<Vec<weles::WelesEvidenceInventoryEntry>> {
    let entries = evidence
        .get("evidenceInventory")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            WorkerFailure::new(
                "weles_evidence_manifest_invalid",
                "the signed evidence manifest has no evidenceInventory array",
            )
        })?;
    let mut inventory = Vec::with_capacity(entries.len());
    for entry in entries {
        inventory.push(serde_json::from_value(entry.clone())?);
    }
    Ok(inventory)
}

/// Downloads and re-proves every signed inventory entry under the attempt's own recording
/// tree, and answers the paths retained. The signed digest is never trusted on its own:
/// the retained bytes are re-hashed exactly the way `validate_evidence_inventory` re-hashes
/// them. `require_browser_evidence` is the v1-only rule — only a completed task must carry
/// the final screenshot and the accessibility tree, because only for a completed task does
/// the service demand them.
pub(crate) fn retain_signed_inventory(
    recordings: &Path,
    weles_task_id: &str,
    prefix: &str,
    inventory: &[weles::WelesEvidenceInventoryEntry],
    require_browser_evidence: bool,
) -> Outcome<Vec<String>> {
    let screenshot_uri = format!("{prefix}artifacts/browser_evidence_final.png");
    let accessibility_uri = format!("{prefix}artifacts/browser_evidence_accessibility_tree.txt");
    let mut retained_paths: Vec<String> =
        vec![format!("recordings/{weles_task_id}/evidence-manifest.json")];
    let mut kinds: BTreeSet<&str> = BTreeSet::new();
    let mut uris: BTreeSet<&str> = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for entry in inventory {
        let relative = entry.uri.strip_prefix(prefix).ok_or_else(|| {
            WorkerFailure::new(
                "weles_evidence_uri_foreign",
                "an evidence inventory URI is not bound to this exact Weles task",
            )
        })?;
        ensure(
            is_portable_relative(relative),
            "weles_evidence_uri_invalid",
            "an evidence inventory URI is not a canonical immutable path",
        )?;
        let kind_matches_uri = match entry.kind.as_str() {
            SCREENSHOT_KIND => entry.uri == screenshot_uri,
            ACCESSIBILITY_KIND => entry.uri == accessibility_uri,
            kind => kind
                .strip_prefix("artifact:")
                .is_some_and(|tail| tail == relative),
        };
        ensure(
            kind_matches_uri && is_sha256(&entry.sha256) && entry.bytes > 0,
            "weles_evidence_entry_invalid",
            "an evidence inventory entry kind, digest or length is not canonical",
        )?;
        ensure(
            kinds.insert(entry.kind.as_str()) && uris.insert(entry.uri.as_str()),
            "weles_evidence_entry_duplicate",
            "the evidence inventory repeats a kind or URI",
        )?;
        total_bytes = total_bytes
            .checked_add(entry.bytes)
            .filter(|total| *total <= MAXIMUM_EVIDENCE_BYTES)
            .ok_or_else(|| {
                WorkerFailure::new(
                    "weles_evidence_too_large",
                    "the retained evidence inventory exceeds the total byte limit",
                )
            })?;
        let destination = recordings.join(relative);
        storage_get(&entry.uri, &destination)?;
        let bytes = std::fs::read(&destination)?;
        ensure(
            bytes.len() as u64 == entry.bytes && crate::sha256_hex(&bytes) == entry.sha256,
            "weles_evidence_bytes_differ",
            "retained evidence bytes differ from the signed inventory entry",
        )?;
        if entry.kind == SCREENSHOT_KIND {
            ensure(
                bytes.starts_with(PNG_MAGIC),
                "weles_evidence_screenshot_invalid",
                "the retained final screenshot is not a PNG",
            )?;
        } else if entry.kind == ACCESSIBILITY_KIND {
            let tree = String::from_utf8(bytes).map_err(|_| {
                WorkerFailure::new(
                    "weles_evidence_accessibility_invalid",
                    "the retained accessibility tree is not valid UTF-8",
                )
            })?;
            ensure(
                !tree.trim().is_empty(),
                "weles_evidence_accessibility_invalid",
                "the retained accessibility tree is empty",
            )?;
        }
        retained_paths.push(format!("recordings/{weles_task_id}/{relative}"));
    }
    if require_browser_evidence {
        ensure(
            kinds.contains(SCREENSHOT_KIND) && kinds.contains(ACCESSIBILITY_KIND),
            "weles_evidence_incomplete",
            "the evidence inventory lacks the required screenshot/accessibility_tree artifacts",
        )?;
    }
    retained_paths.sort();
    Ok(retained_paths)
}

/// The outer Stado job this worker runs inside. `verify_attempt_binding` refuses an
/// envelope whose outer job equals the inner Weles task, so both paths read it here.
pub(crate) fn stado_job_id(weles_task_id: &str) -> Outcome<String> {
    let job = ["STADO_JOB_ID", "STADO_MACHINE_JOB_ID", "STADO_JOB"]
        .iter()
        .find_map(|name| {
            std::env::var(name)
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .ok_or_else(|| {
            WorkerFailure::new(
                "stado_job_id_unavailable",
                "the Stado job identifier was not delivered to this worker",
            )
        })?;
    ensure(
        job != weles_task_id,
        "stado_job_id_unavailable",
        "the Stado job identifier collides with the inner Weles task identifier",
    )?;
    Ok(job)
}

/// Writes this attempt's content-addressed observation document and answers its digest.
///
/// `navigation` is `Some` only for a completed attempt; a non-success observes no
/// navigation and names the terminal outcome instead, so the document states what the
/// attempt actually observed rather than carrying an empty URL pair.
#[allow(clippy::too_many_arguments)]
pub(crate) fn retain_observation_document(
    attempt_root: &Path,
    manifest: &super::crawl::RuntimeManifest,
    weles_task_id: &str,
    product_url: &str,
    outcome: &str,
    navigation: Option<(&str, &str)>,
    inventory: &[weles::WelesEvidenceInventoryEntry],
    retained_paths: &[String],
) -> Outcome<String> {
    let mut observation: BTreeMap<&str, Value> = BTreeMap::new();
    observation.insert("schema", json!(OBSERVATION_SCHEMA));
    observation.insert("run_id", json!(manifest.run_id));
    observation.insert("catalog", json!(manifest.catalog));
    observation.insert("record", json!(manifest.record));
    observation.insert("record_key", json!(manifest.record_key));
    observation.insert("attempt", json!(u64::from(manifest.attempt)));
    observation.insert("attempt_id", json!(manifest.attempt_id));
    observation.insert("weles_task_id", json!(weles_task_id));
    observation.insert("outcome", json!(outcome));
    observation.insert("requested_url", json!(product_url));
    if let Some((effective_url, final_url)) = navigation {
        observation.insert("effective_url", json!(effective_url));
        observation.insert("final_url", json!(final_url));
    }
    observation.insert("evidence_inventory", serde_json::to_value(inventory)?);
    observation.insert("retained_paths", json!(retained_paths));
    // A BTreeMap serializes in sorted key order regardless of the serde_json feature set,
    // so these bytes and their digest are stable.
    let observation_bytes = serde_json::to_vec(&observation)?;
    let observation_document_sha256 = crate::sha256_hex(&observation_bytes);
    write_exact(
        &attempt_root.join(format!(
            "weles/observations/{observation_document_sha256}.json"
        )),
        &observation_bytes,
    )?;
    Ok(observation_document_sha256)
}
