use super::*;

/// Resolve the booted iOS simulator with a probe taken for THIS record.
///
/// The catalog-level host preflight is cached once per catalog, so parsing the
/// device list out of it attributed every later record to a possibly shut-down
/// simulator — and `resume` re-derived identity from the same stale text days
/// later. The desktop and terminal resolvers already probe per record; these two
/// now do the same and return the fresh check as retained evidence.
pub(crate) fn ios_booted_identity(host: &str) -> Result<(RuntimeExecutionIdentity, Vec<Value>)> {
    let check = host_probe(
        host,
        &["xcrun", "simctl", "list", "devices", "booted", "--json"],
    );
    let stdout = ready_output(&check, "fresh iOS booted-device probe")?;
    let document: Value =
        serde_json::from_str(&stdout).context("simctl booted-device report is not JSON")?;
    let mut devices = Vec::new();
    for values in document
        .get("devices")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|map| map.values())
    {
        for device in values.as_array().into_iter().flatten() {
            if device.get("state").and_then(Value::as_str) == Some("Booted")
                && device.get("isAvailable").and_then(Value::as_bool).unwrap_or(true)
            {
                devices.push(device);
            }
        }
    }
    if devices.len() != 1 {
        bail!("expected exactly one booted available iOS device, found {}", devices.len());
    }
    let identity = RuntimeExecutionIdentity {
        host: host.into(),
        observed_hostname: String::new(),
        platform: "ios".into(),
        device_id: Some(
            devices[0]
                .get("udid")
                .and_then(Value::as_str)
                .context("booted iOS device has no UDID")?
                .into(),
        ),
        device_name: devices[0].get("name").and_then(Value::as_str).map(str::to_string),
        resolved_product_identifier: String::new(),
        executable_path: None,
        product_version: None,
        executable_sha256: None,
        effective_url: None,
    };
    Ok((identity, vec![check]))
}

/// Resolve the authorized Android device with a probe taken for THIS record.
pub(crate) fn android_device_identity(host: &str) -> Result<(RuntimeExecutionIdentity, Vec<Value>)> {
    let check = host_probe(host, &["adb", "devices", "-l"]);
    let stdout = ready_output(&check, "fresh Android device probe")?;
    let devices: Vec<&str> = stdout
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let serial = fields.next()?;
            let state = fields.next()?;
            (state == "device").then_some(serial)
        })
        .collect();
    if devices.len() != 1 {
        bail!("expected exactly one authorized Android device, found {}", devices.len());
    }
    let identity = RuntimeExecutionIdentity {
        host: host.into(),
        observed_hostname: String::new(),
        platform: "android".into(),
        device_id: Some(devices[0].into()),
        resolved_product_identifier: String::new(),
        device_name: None,
        executable_path: None,
        product_version: None,
        executable_sha256: None,
        effective_url: None,
    };
    Ok((identity, vec![check]))
}

pub(crate) fn ready_output(check: &Value, context: &str) -> Result<String> {
    if check.get("ready").and_then(Value::as_bool) != Some(true) {
        bail!("{context}: {}", check.get("stderr").and_then(Value::as_str).unwrap_or("host command failed"));
    }
    let output = check
        .get("stdout")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if output.is_empty() {
        bail!("{context}: command returned no identity");
    }
    Ok(output)
}

pub(crate) fn resolve_mobile_install_identity(
    manifest: &RuntimeManifest,
    identity: &mut RuntimeExecutionIdentity,
    host: &str,
    install_check: &Value,
) -> Result<Vec<Value>> {
    let device = identity.device_id.as_deref().context("mobile identity has no device id")?;
    let product = manifest.runtime_product.identifier.as_str();
    if identity.platform == "ios" {
        let app_path = ready_output(install_check, "resolve installed iOS app bundle")?;
        if !app_path.ends_with(".app") {
            bail!("iOS application container is not an app bundle");
        }
        let info = format!("{app_path}/Info.plist");
        let executable_check = host_probe(
            host,
            &["/usr/libexec/PlistBuddy", "-c", "Print:CFBundleExecutable", &info],
        );
        let executable_name = ready_output(&executable_check, "resolve installed iOS executable")?;
        if executable_name.contains('/') || executable_name.chars().any(char::is_whitespace) {
            bail!("installed iOS executable name is invalid");
        }
        let executable_path = format!("{app_path}/{executable_name}");
        let version_check = host_probe(
            host,
            &["/usr/libexec/PlistBuddy", "-c", "Print:CFBundleShortVersionString", &info],
        );
        let version = ready_output(&version_check, "resolve installed iOS version")?;
        let digest_check = host_probe(
            host,
            &["shasum", "-a", "256", &executable_path],
        );
        let digest_output = ready_output(&digest_check, "hash installed iOS executable")?;
        let digest = digest_output.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("installed iOS executable SHA-256 is invalid");
        }
        identity.executable_path = Some(executable_path);
        identity.product_version = Some(version);
        identity.executable_sha256 = Some(digest);
        return Ok(vec![executable_check, version_check, digest_check]);
    }
    let package_path = ready_output(install_check, "resolve installed Android package")?
        .lines()
        .filter_map(|line| line.trim().strip_prefix("package:"))
        .next()
        .context("Android pm path returned no base package path")?
        .to_string();
    let version_check = host_probe(
        host,
        &["adb", "-s", device, "shell", "dumpsys", "package", product],
    );
    let version_output = ready_output(&version_check, "resolve installed Android version")?;
    let version = version_output
        .lines()
        .find_map(|line| line.trim().strip_prefix("versionName="))
        .filter(|value| !value.is_empty())
        .context("Android package has no versionName")?
        .to_string();
    let digest_check = host_probe(
        host,
        &["adb", "-s", device, "shell", "sha256sum", &package_path],
    );
    let digest_output = ready_output(&digest_check, "hash installed Android package")?;
    let digest = digest_output.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("installed Android package SHA-256 is invalid");
    }
    identity.executable_path = Some(package_path);
    identity.product_version = Some(version);
    identity.executable_sha256 = Some(digest);
    Ok(vec![version_check, digest_check])
}

pub(crate) fn resolve_terminal_identity(
    manifest: &mut RuntimeManifest,
    host: &str,
) -> Result<(RuntimeExecutionIdentity, Vec<Value>)> {
    let declared_identifier = manifest.runtime_product.identifier.clone();
    let candidates = if manifest.runtime_product.kind == "tui-slug" {
        super::crawl_tui::binary_candidates(&manifest.runtime_product.identifier)
    } else {
        vec![manifest.runtime_product.identifier.clone()]
    };
    let mut checks = Vec::new();
    let mut resolved = std::collections::BTreeSet::new();
    for candidate in candidates {
        let check = host_probe(host, &["which", &candidate]);
        if check.get("ready").and_then(Value::as_bool) == Some(true) {
            for path in check
                .get("stdout")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|path| path.starts_with('/') && !path.chars().any(char::is_whitespace))
            {
                resolved.insert(path.to_string());
            }
        }
        checks.push(check);
    }
    if resolved.len() != 1 {
        bail!(
            "expected one unique executable for {}, found {}",
            manifest.runtime_product.identifier,
            resolved.len()
        );
    }
    let path = resolved.into_iter().next().expect("one executable");
    let digest_check = host_probe(host, &["shasum", "-a", "256", &path]);
    let digest_output = ready_output(&digest_check, "hash exact executable")?;
    let digest = digest_output.split_whitespace().next().unwrap_or_default();
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("exact executable SHA-256 is invalid");
    }
    checks.push(digest_check);
    let binary = Path::new(&path)
        .file_name()
        .and_then(|value| value.to_str())
        .context("resolved executable path has no UTF-8 filename")?
        .to_string();
    manifest.runtime_product.kind = if manifest.engine == "tui" {
        "tui-binary".into()
    } else {
        "cli-binary".into()
    };
    manifest.runtime_product.identifier = binary;
    manifest.runtime_product.identity_source =
        format!("typed isolated host path resolution: {path}; sha256={digest}");
    Ok((
        RuntimeExecutionIdentity {
            host: host.into(),
            observed_hostname: String::new(),
            platform: "terminal".into(),
            device_id: None,
            resolved_product_identifier: String::new(),
            device_name: Some(declared_identifier),
            executable_path: Some(path),
            product_version: None,
            executable_sha256: Some(digest.to_ascii_lowercase()),
            effective_url: None,
        },
        checks,
    ))
}
