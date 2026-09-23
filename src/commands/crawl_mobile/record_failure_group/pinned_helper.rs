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
        bail!("pinned mobile readiness helper {} is a symlink", path.display());
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
    let output = super::crawl::bounded_command_output(
        &mut version_command,
        "read pinned mobile readiness helper version",
        Duration::from_secs(15),
        64 * 1024,
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
        || ["notification_delivery_disabled", "permission_prompt_invocation_disabled", "notification_prompt_invocation_disabled"]
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
    let output = super::crawl::bounded_command_output(
        &mut readiness,
        "run fresh mobile runtime-readiness verification",
        Duration::from_secs(120),
        4 * 1024 * 1024,
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
        || observation.get("product_identifier").and_then(Value::as_str) != Some(app_id)
        || observation.get("device_id").and_then(Value::as_str) != Some(device)
        || observation.get("pending_permission_prompts").and_then(Value::as_u64) != Some(0)
        || observation.get("pending_notification_prompts").and_then(Value::as_u64) != Some(0)
        || observation.get("evidence_sha256").and_then(Value::as_str)
            != Some(proof.evidence_sha256.as_str())
        || observation.get("product_version").and_then(Value::as_str)
            != Some(expected_version)
        || !observation
            .get("executable_sha256")
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case(expected_sha))
        || ["notification_delivery_disabled", "permission_prompt_invocation_disabled", "notification_prompt_invocation_disabled"]
            .iter()
            .any(|field| observation.get(*field).and_then(Value::as_bool) != Some(true))
    {
        bail!("fresh mobile readiness identity/safety observation differs from the immutable manifest: {observation}");
    }
    Ok(observation)
}

pub(crate) fn action_block_reason(action: &Action) -> Option<&'static str> {
    if potential_consent_trigger(&action.label) {
        return Some("permission/notification-like control withheld before delivery");
    }
    if action.destructive {
        return Some("destructive control withheld before delivery");
    }
    if action.kind.ends_with("EditText")
        || matches!(
            action.kind.as_str(),
            "XCUIElementTypeTextField"
                | "XCUIElementTypeSecureTextField"
                | "XCUIElementTypeSearchField"
                | "XCUIElementTypeTextView"
        )
    {
        return Some("editable control withheld before delivery");
    }
    let label = action.label.trim().to_ascii_lowercase();
    if matches!(label.as_str(), "back" | "cancel" | "close" | "dismiss") {
        return None;
    }
    Some("control has no digest-bound journey authorization and was withheld before delivery")
}

pub(crate) fn potential_consent_trigger(label: &str) -> bool {
    Regex::new(
        r"(?i)\b(allow while using|allow once|don'?t allow|enable (push )?notifications|turn on notifications|grant permission|camera access|microphone access|location access)\b",
    )
    .expect("static consent regex")
    .is_match(label)
}

pub(crate) fn actions(source: &str, platform: Platform) -> Vec<Action> {
    let tags = Regex::new(r"<[^!?][^>]*>").expect("static tag regex");
    let destructive = Regex::new(
        r"(?i)\b(delete|remove|erase|close account|purchase|buy|pay|send|publish|post|confirm deletion|log ?out|sign ?out)\b",
    )
    .expect("static destructive regex");
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let mut editable_index = 0usize;
    for tag in tags.find_iter(source).map(|value| value.as_str()) {
        let node_name: String = tag[1..]
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
            })
            .collect();
        let kind = attribute(tag, "type").unwrap_or_else(|| node_name.clone());
        let editable = match platform {
            Platform::Android => kind.ends_with("EditText"),
            Platform::Ios => matches!(
                kind.as_str(),
                "XCUIElementTypeTextField"
                    | "XCUIElementTypeSecureTextField"
                    | "XCUIElementTypeSearchField"
                    | "XCUIElementTypeTextView"
            ),
        };
        let clickable = match platform {
            Platform::Android => {
                (editable || attribute(tag, "clickable").as_deref() == Some("true"))
                    && attribute(tag, "enabled").as_deref() != Some("false")
            }
            Platform::Ios => {
                (editable
                    || matches!(
                        kind.as_str(),
                        "XCUIElementTypeButton"
                            | "XCUIElementTypeCell"
                            | "XCUIElementTypeLink"
                            | "XCUIElementTypeTab"
                            | "XCUIElementTypeMenuItem"
                            | "XCUIElementTypeSwitch"
                    ))
                    && attribute(tag, "enabled").as_deref() != Some("false")
            }
        };
        if !clickable {
            continue;
        }
        let candidates: &[&str] = match platform {
            Platform::Android => &["resource-id", "content-desc", "hint", "text"],
            Platform::Ios => &["name", "label", "placeholder", "value"],
        };
        let named = candidates
            .iter()
            .filter_map(|field| attribute(tag, field).map(|value| (*field, value)))
            .find(|(_, value)| !value.trim().is_empty());
        let (selector, label) = if let Some((field, value)) = named {
            let Some(literal) = xpath_literal(&value) else {
                continue;
            };
            (format!("//*[@{field}={literal}]"), value)
        } else if editable {
            editable_index += 1;
            (
                format!("(//{kind})[{editable_index}]"),
                format!("unlabelled input {editable_index}"),
            )
        } else {
            continue;
        };
        if seen.insert(selector.clone()) {
            found.push(Action {
                selector,
                destructive: destructive.is_match(&label),
                label,
                kind,
            });
        }
    }
    found
}

pub(crate) fn hash_text(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    hex::encode(digest.finalize())
}
