use super::*;

/// One helper binary, pinned exactly once per record.
#[derive(Clone, Debug)]
pub(crate) struct PinnedHelper {
    pub(crate) path: PathBuf,
    pub(crate) sha256: String,
    pub(crate) version: String,
}

/// Resolve the readiness helper from absolute directories only. The inherited
/// PATH never participates, the path must be canonical and must not be a
/// symlink, and its digest and version are retained, so no writable earlier
/// PATH entry can substitute the binary that observes device readiness
/// (finding 10).
pub(crate) fn pinned_readiness_helper() -> Result<PinnedHelper> {
    const PROGRAM: &str = "stado-runtime-readiness";
    let path = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"]
        .iter()
        .map(|directory| Path::new(directory).join(PROGRAM))
        .find(|candidate| candidate.is_file())
        .with_context(|| {
            format!("{PROGRAM} is absent from the pinned absolute helper directories")
        })?;
    if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
        bail!(
            "pinned mobile readiness helper {} is a symlink",
            path.display()
        );
    }
    let canonical = std::fs::canonicalize(&path)?;
    if canonical != path {
        bail!(
            "pinned mobile readiness helper is not canonical: declared {}, canonical {}",
            path.display(),
            canonical.display()
        );
    }
    let sha256 = hash_file(&path)?;
    let mut version_command = Command::new(&path);
    version_command
        .arg("--version")
        .env_clear()
        .env("PATH", "/usr/bin:/bin");
    let output = crate::commands::crawl::command_output(
        &mut version_command,
        "read pinned mobile readiness helper version",
    )?;
    if !output.status.success() {
        bail!(
            "pinned mobile readiness helper {} refused --version: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(PinnedHelper {
        path,
        sha256,
        version: String::from_utf8_lossy(&output.stdout).trim().to_string(),
    })
}

pub(crate) fn readiness_observation(
    manifest: &super::crawl::RuntimeManifest,
    app_id: &str,
    helper: &PinnedHelper,
) -> Result<Value> {
    let execution = manifest
        .execution_identity
        .as_ref()
        .context("mobile runtime manifest has no exact execution identity")?;
    let expected_version = execution
        .product_version
        .as_deref()
        .context("mobile execution identity has no installed product version")?;
    let expected_sha = execution
        .executable_sha256
        .as_deref()
        .context("mobile execution identity has no installed package/binary SHA-256")?;
    let device = execution
        .device_id
        .as_deref()
        .context("mobile execution identity has no exact device id")?;
    let proof = manifest
        .prepared_proof
        .as_ref()
        .context("mobile runtime manifest has no prepared-runtime proof")?;
    let proof_value = serde_json::to_value(proof)?;
    if proof.product_identifier != app_id
        || proof.device_id.as_deref() != Some(device)
        || proof.pending_permission_prompts != 0
        || proof.pending_notification_prompts != 0
        || !manifest.constraints.no_system_permission_prompts
        || !manifest.constraints.no_notifications
        || [
            "notification_delivery_disabled",
            "permission_prompt_invocation_disabled",
            "notification_prompt_invocation_disabled",
        ]
        .iter()
        .any(|field| proof_value.get(*field).and_then(Value::as_bool) != Some(true))
    {
        bail!(
            "prepared-runtime proof does not bind the exact mobile app/device with prompt invocation and notification delivery disabled"
        );
    }
    let mut readiness = Command::new(&helper.path);
    readiness.args([
        "verify",
        "--json",
        "--product",
        app_id,
        "--device",
        device,
        "--evidence-uri",
        &proof.evidence_uri,
        "--evidence-sha256",
        &proof.evidence_sha256,
    ]);
    let output = crate::commands::crawl::command_output(
        &mut readiness,
        "run fresh mobile runtime-readiness verification",
    )?;
    if !output.status.success() {
        bail!(
            "fresh mobile runtime-readiness verification failed: status={}; stdout={:?}; stderr={:?}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let observation: Value = serde_json::from_slice(&output.stdout)
        .context("fresh mobile runtime-readiness output is not JSON")?;
    if observation.get("ready").and_then(Value::as_bool) != Some(true)
        || observation
            .get("product_identifier")
            .and_then(Value::as_str)
            != Some(app_id)
        || observation.get("device_id").and_then(Value::as_str) != Some(device)
        || observation
            .get("pending_permission_prompts")
            .and_then(Value::as_u64)
            != Some(0)
        || observation
            .get("pending_notification_prompts")
            .and_then(Value::as_u64)
            != Some(0)
        || observation.get("evidence_sha256").and_then(Value::as_str)
            != Some(proof.evidence_sha256.as_str())
        || observation.get("product_version").and_then(Value::as_str) != Some(expected_version)
        || !observation
            .get("executable_sha256")
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case(expected_sha))
        || [
            "notification_delivery_disabled",
            "permission_prompt_invocation_disabled",
            "notification_prompt_invocation_disabled",
        ]
        .iter()
        .any(|field| observation.get(*field).and_then(Value::as_bool) != Some(true))
    {
        bail!("fresh mobile readiness identity/safety observation differs from the immutable manifest: {observation}");
    }
    Ok(observation)
}

/// Why a control is not pressed, from what Brama judged pressing it does.
pub(crate) fn action_block_reason(action: &Action) -> Option<&'static str> {
    action.safety.withheld()
}

/// The attribute `field` of `tag` when it names the element.
fn named_by(tag: &str, field: &'static str) -> Option<(&'static str, String)> {
    attribute(tag, field)
        .filter(|value| !value.trim().is_empty())
        .map(|value| (field, value))
}

/// Every named element of `source`, each with what pressing it does as
/// Brama judged it (`control_safety`). A screen nobody could judge is
/// refused, never pressed.
pub(crate) fn actions(source: &str, platform: Platform) -> Result<Vec<Action>> {
    let tags = Regex::new(r"<[^!?][^>]*>").expect("static tag regex");
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    for tag in tags.find_iter(source).map(|value| value.as_str()) {
        let node_name: String = tag
            .trim_start_matches('<')
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
            })
            .collect();
        let kind = attribute(tag, "type").unwrap_or_else(|| node_name.clone());
        if attribute(tag, "enabled").as_deref() == Some("false") {
            continue;
        }
        // Android says which elements take a tap; an input takes text.
        if matches!(platform, Platform::Android)
            && attribute(tag, "clickable").as_deref() != Some("true")
            && !kind.ends_with("EditText")
        {
            continue;
        }
        // The attribute that names an element on each platform, in the
        // order its accessibility tree states them.
        let named = match platform {
            Platform::Android => named_by(tag, "resource-id")
                .or_else(|| named_by(tag, "content-desc"))
                .or_else(|| named_by(tag, "hint"))
                .or_else(|| named_by(tag, "text")),
            Platform::Ios => named_by(tag, "name")
                .or_else(|| named_by(tag, "label"))
                .or_else(|| named_by(tag, "placeholder"))
                .or_else(|| named_by(tag, "value")),
        };
        let Some((field, label)) = named else {
            continue;
        };
        let Some(literal) = xpath_literal(&label) else {
            continue;
        };
        let selector = format!("//*[@{field}={literal}]");
        if seen.insert(selector.clone()) {
            found.push((selector, label, kind));
        }
    }
    let named: Vec<(String, String)> = found
        .iter()
        .map(|(_, label, kind)| (kind.clone(), label.clone()))
        .collect();
    let classes = crate::commands::control_safety::judge("crawl-mobile", &named)?;
    Ok(found
        .into_iter()
        .zip(classes)
        .filter(|(_, safety)| *safety != crate::commands::control_safety::Safety::Inert)
        .map(|((selector, label, kind), safety)| Action {
            selector,
            label,
            safety,
            kind,
        })
        .collect())
}

pub(crate) fn hash_text(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    hex::encode(digest.finalize())
}
