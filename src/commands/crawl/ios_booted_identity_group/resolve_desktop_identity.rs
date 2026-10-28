use super::*;

pub(crate) fn resolve_desktop_identity(
    manifest: &mut RuntimeManifest,
    host: &str,
) -> Result<(RuntimeExecutionIdentity, Vec<Value>)> {
    let display_name = manifest.runtime_product.identifier.replace('\'', "\\'");
    let query = format!(
        "kMDItemDisplayName == '{display_name}' && kMDItemContentType == 'com.apple.application-bundle'"
    );
    let search = host_probe(host, &["mdfind", &query]);
    let search_output = ready_output(&search, "resolve exact desktop display name")?;
    let paths: Vec<&str> = search_output
        .lines()
        .map(str::trim)
        .filter(|path| path.ends_with(".app"))
        .collect();
    let [app_path] = paths.as_slice() else {
        bail!(
            "desktop display name {} resolved to {} application bundles",
            manifest.runtime_product.identifier,
            paths.len()
        );
    };
    let app_path = app_path.to_string();
    let metadata = host_probe(
        host,
        &[
            "mdls",
            "-raw",
            "-name",
            "kMDItemCFBundleIdentifier",
            &app_path,
        ],
    );
    let bundle = ready_output(&metadata, "resolve exact desktop bundle identifier")?;
    if bundle == "(null)" || bundle.chars().any(char::is_whitespace) {
        bail!("desktop bundle identifier is missing or invalid");
    }
    let info = format!("{app_path}/Contents/Info.plist");
    let executable_check = host_probe(
        host,
        &[
            "/usr/libexec/PlistBuddy",
            "-c",
            "Print:CFBundleExecutable",
            &info,
        ],
    );
    let executable_name = ready_output(&executable_check, "resolve desktop executable")?;
    if executable_name.contains('/') || executable_name.chars().any(char::is_whitespace) {
        bail!("desktop CFBundleExecutable is invalid");
    }
    let executable_path = format!("{app_path}/Contents/MacOS/{executable_name}");
    let version_check = host_probe(
        host,
        &[
            "/usr/libexec/PlistBuddy",
            "-c",
            "Print:CFBundleShortVersionString",
            &info,
        ],
    );
    let version = ready_output(&version_check, "resolve desktop product version")?;
    let digest_check = host_probe(host, &["shasum", "-a", "256", &executable_path]);
    let digest_output = ready_output(&digest_check, "hash desktop executable")?;
    let Some(digest) = digest_output.split_whitespace().next() else {
        bail!("shasum printed no digest for {executable_path}");
    };
    let digest = digest.to_ascii_lowercase();
    if digest.len() != SHA256_HEX_DIGITS || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("desktop executable SHA-256 is invalid");
    }
    let hardware_check = host_probe(host, &["ioreg", "-rd1", "-c", "IOPlatformExpertDevice"]);
    let hardware_output = ready_output(&hardware_check, "resolve exact desktop hardware identity")?;
    // ioreg prints the UUID as `"IOPlatformUUID" = "<uuid>"`.
    let hardware_id = hardware_output
        .lines()
        .find(|line| line.contains("\"IOPlatformUUID\""))
        .and_then(|line| line.split_once('='))
        .map(|(_, value)| value.trim().trim_matches('"'))
        .filter(|value| {
            !value.is_empty()
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
        .context("desktop hardware report has no valid IOPlatformUUID")?
        .to_string();
    manifest.runtime_product.kind = "desktop-bundle".into();
    manifest.runtime_product.identifier = bundle.clone();
    manifest.runtime_product.identity_source =
        format!("typed host display-name resolution: bundle={app_path}; executable={executable_path}; version={version}; sha256={digest}");
    Ok((
        RuntimeExecutionIdentity {
            host: host.into(),
            observed_hostname: String::new(),
            platform: "macos".into(),
            device_id: Some(hardware_id),
            resolved_product_identifier: String::new(),
            device_name: Some(display_name),
            executable_path: Some(executable_path),
            product_version: Some(version),
            executable_sha256: Some(digest),
            effective_url: None,
        },
        vec![
            search,
            metadata,
            executable_check,
            version_check,
            digest_check,
            hardware_check,
        ],
    ))
}

/// Hex digits of a SHA-256 digest: 256 bits, four per digit
/// (FIPS 180-4, https://csrc.nist.gov/pubs/fips/180-4/upd1/final).
const SHA256_HEX_DIGITS: usize = 64;

pub(crate) fn prepared_runtime_check(
    manifest: &RuntimeManifest,
    identity: &RuntimeExecutionIdentity,
    host: &str,
) -> Result<Value> {
    let proof = manifest
        .prepared_proof
        .as_ref()
        .context("no independently observed prepared-runtime proof is bound")?;
    let product = &manifest.runtime_product.identifier;
    // A proof binds one device; an identity that names none has nothing to
    // bind it to, and two absent ids are not the same device.
    let device = identity
        .device_id
        .as_deref()
        .context("the runtime identity names no device, so no prepared-runtime proof can be bound to it")?;
    if proof.device_id.as_deref() != Some(device) {
        bail!(
            "the prepared-runtime proof names device {:?}, not this runtime's {device}",
            proof.device_id
        );
    }
    if proof.schema != "wisent.runtime-preparation-proof.v1"
        || proof.product_identifier != *product
        || proof.observed_by != "stado-runtime-readiness"
        || !proof.evidence_uri.starts_with("stado://")
        || identity.product_version.as_deref() != Some(proof.product_version.as_str())
        || identity.executable_sha256.as_deref() != Some(proof.executable_sha256.as_str())
        || proof.evidence_sha256.len() != SHA256_HEX_DIGITS
        || !proof
            .evidence_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || !proof.installed
        || !proof.first_run_completed
        || proof.pending_permission_prompts != Default::default()
        || proof.pending_notification_prompts != Default::default()
        || !proof.notification_delivery_disabled
        || !proof.permission_prompt_invocation_disabled
        || !proof.notification_prompt_invocation_disabled
        || !is_rfc3339_utc(&proof.observed_at)
    {
        bail!("prepared-runtime proof does not bind the exact product with an RFC 3339 UTC observation time, first run completed, zero pending prompts, disabled permission/notification prompt invocation, and disabled notification delivery");
    }
    let check = host_probe(
        host,
        &[
            "stado-runtime-readiness",
            "verify",
            "--json",
            "--product",
            product,
            "--device",
            device,
            "--evidence-uri",
            &proof.evidence_uri,
            "--evidence-sha256",
            &proof.evidence_sha256,
        ],
    );
    let observation: Value =
        serde_json::from_str(&ready_output(&check, "verify prepared-runtime proof")?)
            .context("prepared-runtime helper output is not JSON")?;
    if observation.get("schema").and_then(Value::as_str)
        != Some("wisent.runtime-readiness-observation.v1")
        || observation.get("ready").and_then(Value::as_bool) != Some(true)
        || observation
            .get("product_identifier")
            .and_then(Value::as_str)
            != Some(product)
        || observation.get("device_id").and_then(Value::as_str) != Some(device)
        || observation.get("evidence_sha256").and_then(Value::as_str)
            != Some(proof.evidence_sha256.as_str())
        || observation
            .get("pending_permission_prompts")
            .and_then(Value::as_u64)
            != Some(Default::default())
        || observation
            .get("pending_notification_prompts")
            .and_then(Value::as_u64)
            != Some(Default::default())
        || observation
            .get("notification_delivery_disabled")
            .and_then(Value::as_bool)
            != Some(true)
        || observation
            .get("permission_prompt_invocation_disabled")
            .and_then(Value::as_bool)
            != Some(true)
        || observation
            .get("notification_prompt_invocation_disabled")
            .and_then(Value::as_bool)
            != Some(true)
        || observation.get("product_version").and_then(Value::as_str)
            != identity.product_version.as_deref()
        || observation.get("executable_sha256").and_then(Value::as_str)
            != identity.executable_sha256.as_deref()
    {
        bail!("prepared-runtime helper did not attest the exact safe state, version and executable/package digest, including disabled permission and notification prompt invocation");
    }
    Ok(check)
}

/// `YYYY-MM-DDTHH:MM:SSZ`, the exact shape `crate::now_iso_utc` emits.
///
/// The prepared-runtime proof's `observed_at` used to be a dead field that
/// implied a staleness check nobody performed. Freshness itself still comes from
/// the live `stado-runtime-readiness verify` re-check, but the timestamp is now
/// required to be a well-formed UTC instant rather than arbitrary text.
pub(crate) fn is_rfc3339_utc(value: &str) -> bool {
    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ").is_ok()
}
