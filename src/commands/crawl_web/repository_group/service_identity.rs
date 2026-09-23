use super::*;

/// Mirrors `weles_provenance::validate_service_identity`.
pub(crate) fn service_identity(
    manifest: &super::crawl::RuntimeManifest,
) -> Outcome<weles::WelesServiceIdentity> {
    let service = manifest.service_identity.as_ref().ok_or_else(|| {
        WorkerFailure::new(
            "weles_service_identity_absent",
            "the runtime manifest carries no exact Weles service identity",
        )
    })?;
    let identity = weles::WelesServiceIdentity {
        name: service.name.clone(),
        generation: service.generation,
        consumer: service.consumer.clone(),
        capability: service.capability.clone(),
        active_host: service.active_host.clone(),
        endpoint: service.endpoint.clone(),
        action: service.action.clone(),
        release_id: service.release_id.clone(),
        source_revision: service.source_revision.clone(),
    };
    ensure(
        identity.name == SERVICE_NAME
            && identity.consumer == SERVICE_CONSUMER
            && identity.capability == SERVICE_CAPABILITY
            && identity.action == weles::SPIS_WELES_ACTION
            && is_service_host(&identity.active_host),
        "weles_service_identity_invalid",
        "the runtime manifest service identity is not the exact Weles browser-evidence identity",
    )?;
    ensure(
        is_service_release_id(&identity.release_id),
        "weles_service_identity_invalid",
        "the Weles service release identifier is not a weles-worker@<major>.<minor>.<patch> release",
    )?;
    ensure(
        is_git_revision(&identity.source_revision),
        "weles_service_identity_invalid",
        "the Weles service source revision is not a 40-hex git revision",
    )?;
    validate_api_endpoint(&identity.endpoint)?;
    Ok(identity)
}

/// Independently reads the live release identity from the public version endpoint, so the
/// directory copy in the runtime manifest can never stand in for the running service.
pub(crate) fn confirm_service_release(identity: &weles::WelesServiceIdentity) -> Outcome<()> {
    let url = format!("{}/version", identity.endpoint);
    let mut curl = Command::new("curl");
    curl.args([
        "--fail",
        "--silent",
        "--show-error",
        "--location-trusted",
        "--max-redirs",
        "0",
        "--max-time",
        "20",
        "-H",
        "Accept: application/json",
    ])
    .arg(&url);
    let output = super::crawl::bounded_command_output(
        &mut curl,
        "read the Weles service release identity",
        Duration::from_secs(30),
        256 * 1024,
    )?;
    if !output.status.success() {
        return Err(WorkerFailure::new(
            "weles_service_release_unavailable",
            format!(
                "the Weles version endpoint refused the release readback: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    let document: Value = serde_json::from_slice(&output.stdout).map_err(|_| {
        WorkerFailure::new(
            "weles_service_release_mismatch",
            "the Weles version endpoint did not return a JSON document",
        )
    })?;
    let release_id = alternate(&document, "release_id", "releaseId").ok_or_else(|| {
        WorkerFailure::new(
            "weles_service_release_mismatch",
            "the Weles version endpoint declares no release_id",
        )
    })?;
    let source_revision =
        alternate(&document, "source_revision", "sourceRevision").ok_or_else(|| {
            WorkerFailure::new(
                "weles_service_release_mismatch",
                "the Weles version endpoint declares no source_revision",
            )
        })?;
    ensure(
        release_id == identity.release_id && source_revision == identity.source_revision,
        "weles_service_release_mismatch",
        "the live Weles release differs from the runtime manifest service directory",
    )
}

pub(crate) fn alternate<'a>(document: &'a Value, primary: &str, secondary: &str) -> Option<&'a str> {
    document
        .get(primary)
        .or_else(|| document.get(secondary))
        .and_then(Value::as_str)
}

/// Mirrors `weles_provenance::validate_spis_binding` before the binding is ever signed.
pub(crate) fn attempt_binding(
    manifest: &super::crawl::RuntimeManifest,
    identity: &weles::WelesServiceIdentity,
) -> Outcome<weles::WelesAttemptBinding> {
    let binding = weles::WelesAttemptBinding {
        schema: weles::ATTEMPT_BINDING_SCHEMA.to_string(),
        run_id: manifest.run_id.clone(),
        catalog: manifest.catalog.clone(),
        record: manifest.record.clone(),
        record_key: manifest.record_key.clone(),
        attempt: manifest.attempt,
        attempt_id: manifest.attempt_id.clone(),
        source_revision: manifest.source_revision.clone(),
        source_input_sha256: manifest.source_input_sha256.clone(),
        reference_sha256: manifest.reference_sha256.clone(),
        artifact_uri: manifest.artifact_uri.clone(),
        output_uri: manifest.output_uri.clone(),
        service: weles::WelesAttemptBindingService {
            name: identity.name.clone(),
            consumer: identity.consumer.clone(),
            capability: identity.capability.clone(),
            directory_generation: identity.generation,
            host: identity.active_host.clone(),
            endpoint: identity.endpoint.clone(),
            action: identity.action.clone(),
            release_id: identity.release_id.clone(),
            source_revision: identity.source_revision.clone(),
        },
    };
    for component in [
        binding.run_id.as_str(),
        binding.catalog.as_str(),
        binding.record.as_str(),
        binding.attempt_id.as_str(),
    ] {
        ensure(
            is_portable_component(component),
            "web_attempt_component_invalid",
            "an attempt coordinate is not a portable attempt URI component",
        )?;
    }
    ensure(
        binding.attempt >= 1,
        "web_attempt_component_invalid",
        "the attempt number must be at least one",
    )?;
    ensure(
        is_sha256(&binding.record_key)
            && is_sha256(&binding.source_input_sha256)
            && is_sha256(&binding.reference_sha256),
        "web_attempt_component_invalid",
        "record key and source/reference digests must be lowercase 64-hex SHA-256",
    )?;
    ensure(
        is_git_revision(&binding.source_revision),
        "web_attempt_component_invalid",
        "the attempt source revision is not a 40-hex git revision",
    )?;
    let base = attempt_base(&binding);
    ensure(
        binding.artifact_uri == format!("{base}/artifacts.tar.gz")
            && binding.output_uri == format!("{base}/worker-output.log"),
        "web_attempt_uri_invalid",
        "the signed Spis artifact/output URIs are not the canonical attempt coordinates",
    )?;
    Ok(binding)
}

/// The exact typed browser-evidence constraint array Weles admits.
///
/// `parseTaskRequest` in the service refuses any `input.constraints` whose canonical JSON
/// differs from `SPIS_BROWSER_EVIDENCE_POLICY.constraints`, and the browser worker
/// enforces exactly that policy while it captures. The order below is the service's own
/// order and is submitted verbatim.
pub(crate) const BROWSER_EVIDENCE_CONSTRAINTS: &[&str] = &[
    "browser-permission-apis:withhold",
    "notification-apis:withhold",
    "permission-notification-controls:withhold",
    "system-ui-downloads:withhold",
    "authentication-signup-recovery:withhold",
    "mfa-trusted-device:withhold",
    "message-submission:withhold",
    "commerce-payment:withhold",
    "destructive-confirmation:withhold",
    "network:exact-public-origin-pinned",
    "interactive-controls:default-deny",
];

/// The submitted constraint list.
///
/// Weles enforces one immutable withholding policy for every browser-evidence task and
/// admits only that exact array, so this worker never negotiates its own weaker
/// vocabulary: it refuses to submit unless the immutable runtime manifest asks for
/// exactly the withholding the policy performs, and then submits the policy verbatim.
/// The origin is not restated here; it is pinned by `network:exact-public-origin-pinned`
/// against the request `origin`, which the service requires to equal the exact product
/// URL origin and which is signed into the receipt as a core claim.
/// `validate_request_and_evidence_manifest` runs `validate_unique_nonempty` over exactly
/// this vector.
pub(crate) fn task_constraints(constraints: &super::crawl::RuntimeConstraints) -> Outcome<Vec<String>> {
    ensure(
        constraints.no_first_run_consent
            && constraints.no_system_permission_prompts
            && constraints.no_notifications
            && constraints.no_purchase
            && constraints.no_final_destructive_action
            && constraints.headless,
        "web_constraints_invalid",
        "the immutable runtime constraints do not request the exact Weles browser-evidence withholding policy",
    )?;
    Ok(BROWSER_EVIDENCE_CONSTRAINTS
        .iter()
        .map(|value| (*value).to_string())
        .collect())
}

pub(crate) fn text<'a>(document: &'a Value, key: &str) -> Outcome<&'a str> {
    document
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            WorkerFailure::new(
                "weles_evidence_manifest_invalid",
                format!("the signed evidence manifest has no string {key}"),
            )
        })
}
