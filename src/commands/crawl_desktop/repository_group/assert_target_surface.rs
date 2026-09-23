use super::*;

pub(crate) fn assert_target_surface(
    snapshot: &Value,
    expected_pid: i64,
    expected_window: i64,
    expected_bundle: &str,
) -> Result<()> {
    // Anchored typed read: the strongest safety property in this crawler may
    // not be answered by a child element or a sibling window (finding 13).
    let observed = WindowOwnership::read(snapshot)?;
    let observed_pid = observed.pid;
    let observed_window = observed.window_id;
    if observed_pid != expected_pid || observed_window != expected_window {
        bail!(
            "Cua window ownership changed: expected pid={expected_pid} window={expected_window}, observed pid={observed_pid} window={observed_window}"
        );
    }
    let observed_bundle = observed.owner_bundle_id.as_str();
    if observed_bundle != expected_bundle {
        if system_bundle(observed_bundle) {
            bail!(
                "system-owned dialog/surface detected for bundle {observed_bundle}; no control was delivered"
            );
        }
        bail!(
            "window owner differs from exact target bundle: expected {expected_bundle}, observed {observed_bundle}"
        );
    }
    Ok(())
}

pub(crate) fn readiness_observation(
    manifest: &super::crawl::RuntimeManifest,
    product: &str,
    helper: &PinnedHelper,
) -> Result<Value> {
    let identity = manifest
        .execution_identity
        .as_ref()
        .context("desktop runtime manifest has no resolved execution identity")?;
    let proof = manifest
        .prepared_proof
        .as_ref()
        .context("desktop runtime manifest has no prepared-runtime proof")?;
    let device = identity
        .device_id
        .as_deref()
        .context("desktop execution identity has no exact device id")?;
    let proof_value = serde_json::to_value(proof)?;
    if proof.product_identifier != product
        || proof.device_id.as_deref() != Some(device)
        || proof.pending_permission_prompts != 0
        || proof.pending_notification_prompts != 0
        || !manifest.constraints.no_system_permission_prompts
        || !manifest.constraints.no_notifications
        || ["notification_delivery_disabled", "permission_prompt_invocation_disabled", "notification_prompt_invocation_disabled"]
            .iter()
            .any(|field| proof_value.get(*field).and_then(Value::as_bool) != Some(true))
    {
        bail!(
            "prepared-runtime proof does not bind the exact desktop app/device with prompt invocation and notification delivery disabled"
        );
    }
    let mut readiness = Command::new(&helper.path);
    readiness.args([
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
    ]);
    let output = super::crawl::bounded_command_output(
        &mut readiness,
        "run fresh desktop runtime-readiness verification",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "fresh desktop runtime-readiness verification failed: status={}; stdout={:?}; stderr={:?}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let observation: Value = serde_json::from_slice(&output.stdout)
        .context("fresh desktop runtime-readiness output is not JSON")?;
    if observation.get("ready").and_then(Value::as_bool) != Some(true)
        || observation.get("product_identifier").and_then(Value::as_str) != Some(product)
        || observation.get("device_id").and_then(Value::as_str) != Some(device)
        || observation.get("pending_permission_prompts").and_then(Value::as_u64) != Some(0)
        || observation.get("pending_notification_prompts").and_then(Value::as_u64) != Some(0)
        || ["notification_delivery_disabled", "permission_prompt_invocation_disabled", "notification_prompt_invocation_disabled"]
            .iter()
            .any(|field| observation.get(*field).and_then(Value::as_bool) != Some(true))
        || observation.get("evidence_sha256").and_then(Value::as_str)
            != Some(proof.evidence_sha256.as_str())
    {
        bail!("fresh desktop runtime-readiness observation did not preserve the exact prompt-disabled prepared state: {observation}");
    }
    Ok(observation)
}

pub(crate) fn verify_desktop_executable(
    expected_bundle: &str,
    manifest: &super::crawl::RuntimeManifest,
    helper: &PinnedHelper,
) -> Result<Value> {
    let identity = manifest
        .execution_identity
        .as_ref()
        .context("desktop runtime manifest has no resolved execution identity")?;
    let configured = identity
        .executable_path
        .as_deref()
        .context("desktop execution identity has no exact executable path")?;
    let path = Path::new(configured);
    if !path.is_absolute() || !path.is_file() {
        bail!("desktop execution identity path is not an absolute executable file: {configured}");
    }
    let contents = path
        .parent()
        .filter(|parent| parent.file_name().and_then(|name| name.to_str()) == Some("MacOS"))
        .and_then(Path::parent)
        .filter(|parent| parent.file_name().and_then(|name| name.to_str()) == Some("Contents"))
        .context("desktop execution identity is not an exact Contents/MacOS executable")?;
    let info = contents.join("Info.plist");
    let metadata = |key: &str| -> Result<String> {
        let mut command = Command::new("/usr/bin/plutil");
        command.args(["-extract", key, "raw", "-o", "-"]).arg(&info);
        let output = super::crawl::bounded_command_output(
            &mut command,
            "read desktop bundle metadata",
            Duration::from_secs(30),
            1024 * 1024,
        )
        .with_context(|| format!("read {key} from {}", info.display()))?;
        if !output.status.success() {
            bail!(
                "read {key} from {} failed: status={}; stdout={:?}; stderr={:?}",
                info.display(),
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    };
    let bundle = metadata("CFBundleIdentifier")?;
    if bundle != expected_bundle {
        bail!(
            "desktop executable bundle changed immediately before launch: expected {expected_bundle}, observed {bundle}"
        );
    }
    let expected_version = identity
        .product_version
        .as_deref()
        .context("desktop execution identity has no CFBundleShortVersionString")?;
    let observed_version = metadata("CFBundleShortVersionString")?;
    if observed_version != expected_version {
        bail!(
            "desktop product version changed immediately before launch: expected {expected_version:?}, observed {observed_version:?}"
        );
    }
    let expected_sha = identity
        .executable_sha256
        .as_deref()
        .context("desktop execution identity has no executable SHA-256")?;
    let observed_sha = hash_file(path)?;
    if !observed_sha.eq_ignore_ascii_case(expected_sha) {
        bail!(
            "desktop executable SHA-256 changed immediately before launch: expected {expected_sha}, observed {observed_sha}"
        );
    }
    let observation = readiness_observation(manifest, expected_bundle, helper)?;
    if observation.get("product_version").and_then(Value::as_str) != Some(expected_version)
        || !observation
            .get("executable_sha256")
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case(expected_sha))
    {
        bail!("fresh desktop readiness identity differs from the exact manifest executable: {observation}");
    }
    Ok(observation)
}

pub(crate) struct LaunchedApp {
    pub(crate) pid: i64,
    pub(crate) window_id: i64,
    pub(crate) executable_path: Option<String>,
    pub(crate) response: Value,
}

pub(crate) fn launch(
    driver: &CuaDriver,
    record: &Record,
    bundle_id: &str,
    session: &str,
) -> Result<LaunchedApp> {
    let launched = call(
        driver,
        "launch_app",
        &json!({
            "session": session,
            "bundle_id": bundle_id,
        }),
    )?;
    let response = DriverResponse(&launched);
    let pid = response
        .integer("pid")
        .ok_or_else(|| anyhow!("{} launch returned no anchored pid", record.name))?;
    let window_id = response
        .integer("window_id")
        .or_else(|| {
            call(
                driver,
                "list_windows",
                &json!({"pid": pid, "session": session}),
            )
            .ok()
            .and_then(|value| DriverResponse(&value).integer("window_id"))
        })
        .ok_or_else(|| anyhow!("{} launch returned no window", record.name))?;
    // Anchored typed read of the launched executable, never a recursive key
    // search through the response tree (findings 5b and 13).
    let executable_path = response
        .text("executable_path")
        .or_else(|| response.text("executable"))
        .map(str::to_string);
    Ok(LaunchedApp {
        pid,
        window_id,
        executable_path,
        response: launched,
    })
}

/// Terminate every pre-existing instance of the bundle so the launch is cold.
/// Reusing a stale instance means the screenshot is not evidence of a clean
/// launch (finding 5).
pub(crate) fn terminate_pre_existing(
    driver: &CuaDriver,
    session: &str,
    bundle_id: &str,
) -> Result<Vec<i64>> {
    let pre_existing = running_instances(driver, session, bundle_id)?;
    for pid in &pre_existing {
        call_briefly(
            driver,
            "terminate_app",
            &json!({"session": session, "pid": pid}),
        )
        .with_context(|| format!("terminate pre-existing {bundle_id} instance {pid}"))?;
    }
    if !pre_existing.is_empty() {
        let deadline = SystemTime::now() + Duration::from_secs(15);
        loop {
            let remaining = running_instances(driver, session, bundle_id)?;
            if remaining.is_empty() {
                break;
            }
            if SystemTime::now() >= deadline {
                return Err(anyhow::Error::new(RecordFailure {
                    code: "desktop_stale_instance_survived",
                    message: format!(
                        "{bundle_id} instances {remaining:?} survived termination; the launch would not be cold"
                    ),
                }));
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    Ok(pre_existing)
}
