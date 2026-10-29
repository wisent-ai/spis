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
    if !crate::weles_provenance::is_sha256(&digest) {
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

/// Binds the manifest's prepared-runtime proof to `identity` and has the
/// host's `stado-runtime-readiness` attest it afresh; the host's answer is
/// returned as the check's evidence.
pub(crate) fn prepared_runtime_check(
    manifest: &RuntimeManifest,
    identity: &RuntimeExecutionIdentity,
    host: &str,
) -> Result<Value> {
    let bound = crate::commands::crawl::bound_proof(
        manifest,
        identity,
        &manifest.runtime_product.identifier,
    )?;
    let mut probe = vec!["stado-runtime-readiness"];
    probe.extend(bound.verify_args());
    let check = host_probe(host, &probe);
    let observation: Value =
        serde_json::from_str(&ready_output(&check, "verify prepared-runtime proof")?)
            .context("prepared-runtime helper output is not JSON")?;
    bound.attested(&observation)?;
    Ok(check)
}
