use super::*;

pub(crate) fn validate_service_identity(identity: &WelesServiceIdentity) -> Result<(), String> {
    if identity.name != "weles-admission"
        || identity.consumer != "spis"
        || identity.capability != "browser-evidence"
        || identity.active_host.trim().is_empty()
        || identity.action != SPIS_WELES_ACTION
        || !identity.release_id.starts_with("weles-worker@")
        || identity.release_id == "weles-worker@"
        || !is_git_revision(&identity.source_revision)
    {
        return Err("attempt service identity is invalid".to_string());
    }
    validate_api_endpoint(&identity.endpoint, "attempt service identity endpoint")
}

pub(crate) fn validate_spis_binding(binding: &WelesAttemptBinding) -> Result<(), String> {
    let required = [
        binding.run_id.as_str(),
        binding.catalog.as_str(),
        binding.record.as_str(),
        binding.record_key.as_str(),
        binding.attempt_id.as_str(),
        binding.source_revision.as_str(),
        binding.artifact_uri.as_str(),
        binding.output_uri.as_str(),
        binding.service.name.as_str(),
        binding.service.consumer.as_str(),
        binding.service.capability.as_str(),
        binding.service.host.as_str(),
        binding.service.endpoint.as_str(),
        binding.service.release_id.as_str(),
        binding.service.source_revision.as_str(),
        binding.service.action.as_str(),
    ];
    if binding.schema != ATTEMPT_BINDING_SCHEMA
        || binding.attempt == 0
        || required.iter().any(|value| value.trim().is_empty())
        || !is_git_revision(&binding.source_revision)
        || !is_sha256(&binding.record_key)
        || !is_sha256(&binding.source_input_sha256)
        || !is_sha256(&binding.reference_sha256)
        || !is_git_revision(&binding.service.source_revision)
        || !is_portable_attempt_component(&binding.run_id)
        || !is_portable_attempt_component(&binding.catalog)
        || !is_portable_attempt_component(&binding.record)
        || !is_portable_attempt_component(&binding.attempt_id)
        || binding.service.name != "weles-admission"
        || binding.service.consumer != "spis"
        || binding.service.capability != "browser-evidence"
        || binding.service.action != SPIS_WELES_ACTION
        || !binding.service.release_id.starts_with("weles-worker@")
        || binding.service.release_id == "weles-worker@"
    {
        return Err("signed Spis binding is invalid".to_string());
    }
    validate_api_endpoint(&binding.service.endpoint, "signed Spis service endpoint")?;
    validate_attempt_binding_derivation(binding)?;
    let base = crate::crawl_attempt_base_uri(
        &binding.run_id,
        &binding.catalog,
        &binding.record,
        &binding.record_key,
        binding.attempt,
        &binding.attempt_id,
    );
    if binding.artifact_uri != format!("{base}/artifacts.tar.gz")
        || binding.output_uri != format!("{base}/worker-output.log")
    {
        return Err("signed Spis artifact/output URIs are not canonical".to_string());
    }
    Ok(())
}

/// Re-derives the runtime record key and attempt identity exactly as the Weles
/// public admission service does, so the Rust layer never accepts a weaker
/// attempt binding than the runtime promises.
pub(crate) fn validate_attempt_binding_derivation(binding: &WelesAttemptBinding) -> Result<(), String> {
    let catalog_key = sha256_bytes(
        format!(
            "{}\0{}\0{}",
            binding.source_revision, binding.run_id, binding.catalog
        )
        .as_bytes(),
    );
    let record_key = sha256_bytes(
        format!(
            "{}\0{}\0{}",
            catalog_key, binding.record, binding.source_input_sha256
        )
        .as_bytes(),
    );
    if binding.record_key != record_key {
        return Err("signed Spis record key is not the runtime derivation".to_string());
    }
    let attempt_fingerprint = sha256_bytes(
        format!(
            "{}\0{}\0{}",
            binding.record_key, binding.attempt, binding.service.host
        )
        .as_bytes(),
    );
    if binding.attempt_id
        != format!(
            "attempt-{}-{}",
            binding.attempt,
            &attempt_fingerprint[..16]
        )
    {
        return Err("signed Spis attempt identity is not the runtime derivation".to_string());
    }
    Ok(())
}

pub(crate) fn validate_api_endpoint(value: &str, label: &str) -> Result<(), String> {
    let endpoint = url::Url::parse(value).map_err(|_| format!("{label} is invalid"))?;
    if !matches!(endpoint.scheme(), "http" | "https")
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.path() != "/api/v1"
        || endpoint.query().is_some()
        || endpoint.fragment().is_some()
        || endpoint.as_str() != value
    {
        return Err(format!("{label} is not the canonical exact /api/v1 base"));
    }
    Ok(())
}

pub(crate) fn load_canonical_trust() -> Result<CanonicalTrust, String> {
    let checked_in = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("weles-bridge")
        .join("weles-receipt-trust.json");
    let metadata = fs::symlink_metadata(&checked_in)
        .map_err(|_| "checked-in public trust document is absent".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("checked-in public trust document is not a regular file".to_string());
    }
    let canonical = fs::canonicalize(&checked_in)
        .map_err(|_| "checked-in public trust document could not be resolved".to_string())?;
    let bytes = read_limited(&canonical, MAX_TRUST_BYTES)?;
    let document: WelesReceiptTrust = serde_json::from_slice(&bytes)
        .map_err(|_| "public trust document does not match the typed schema".to_string())?;
    if document.schema != BRIDGE_TRUST_SCHEMA
        || document.organization_id.trim().is_empty()
        || document.allowed_action != SPIS_WELES_ACTION
        || document.key_set_version.trim().is_empty()
        || document.receipt_keys.is_empty()
        || document
            .receipt_keys
            .iter()
            .any(|(key, value)| key.trim().is_empty() || value.trim().is_empty())
    {
        return Err("public trust document is invalid".to_string());
    }
    Ok(CanonicalTrust {
        path: canonical,
        bytes,
        document,
    })
}

/// A typed bridge failure.
///
/// `code` is the bridge's own machine-readable code whenever the bridge reported for
/// itself, and a Rust-side code (`absent`, `unpinned`, `spawn-failed`, `timeout`,
/// `io-failed`) when it never got that far. `message` is the exact operator-facing text.
#[derive(Debug, Clone)]
pub struct BridgeFailure {
    pub code: String,
    pub message: String,
}

impl BridgeFailure {
    pub(crate) fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

/// One exact invocation of the checked-in bridge.
///
/// The operation is carried by `command.operation`, not by this struct: `submit`, `get`,
/// `cancel` and `verify` differ only in the command document, the output destination,
/// whether a network credential is in play, and the wall-clock budget.
pub struct BridgeInvocation<'a> {
    /// `wisent.spis-weles-bridge-command.v1` document. The bridge runs a strict per
    /// operation key allowlist, so it is passed through exactly as serialized.
    pub command: &'a Value,
    /// The public trust document this process already validated. Its bytes are handed to
    /// the child, which re-checks them against the canonical file, so the child never
    /// gets to choose its own trust.
    pub trust: &'a CanonicalTrust,
    /// Process working directory. `verify` resolves the retained artifact against the
    /// record directory; `submit` resolves a relative `--output` against it.
    pub working_dir: &'a Path,
    /// `Some(path)` persists the document to that file, which `submit` requires for its
    /// request-bound recovery; `None` returns it on bounded stdout, which `get` requires.
    pub output: Option<&'a Path>,
    /// The owner-only protected config carrying the bearer. `None` is the secretless
    /// path: without it the bridge refuses every network operation, and `verify` never
    /// reads a config at all.
    pub config: Option<&'a Path>,
    /// Wall-clock budget for the whole child process.
    pub timeout: std::time::Duration,
}
