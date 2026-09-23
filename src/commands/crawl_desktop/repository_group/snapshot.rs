use super::*;

pub(crate) fn snapshot(
    driver: &CuaDriver,
    session: &str,
    pid: i64,
    window_id: i64,
    screenshot: &Path,
) -> Result<Value> {
    if let Some(parent) = screenshot.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match std::fs::symlink_metadata(screenshot) {
        Ok(_) => std::fs::remove_file(screenshot)
            .with_context(|| format!("remove stale screenshot {}", screenshot.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let started = SystemTime::now();
    let mut response = call_with_cli_options(
        driver,
        "get_window_state",
        &json!({
            "session": session,
            "pid": pid,
            "window_id": window_id,
            "max_elements": 4000,
            "max_depth": 40,
        }),
        &[
            std::ffi::OsStr::new("--screenshot-out-file"),
            screenshot.as_os_str(),
        ],
        Duration::from_secs(30),
    )?;
    let metadata = std::fs::symlink_metadata(screenshot)
        .with_context(|| format!("read fresh screenshot metadata {}", screenshot.display()))?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_file()
        || metadata.len() == 0
        || metadata.len() > 16 * 1024 * 1024
        || metadata.modified().is_ok_and(|modified| modified < started)
    {
        bail!("Cua screenshot is not a fresh bounded regular file");
    }
    let bytes = std::fs::read(screenshot)?;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        bail!("Cua screenshot is not an exact PNG byte stream");
    }
    let evidence = json!({
        "path": screenshot,
        "sha256": hex::encode(Sha256::digest(&bytes)),
        "bytes": bytes.len(),
        "media_type": "image/png",
    });
    response
        .as_object_mut()
        .context("Cua window-state response must be an object")?
        .insert("screenshot_evidence".into(), evidence);
    Ok(response)
}

pub(crate) fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub(crate) fn actions(snapshot: &Value) -> Vec<Action> {
    let roles = [
        "button",
        "link",
        "menuitem",
        "tab",
        "checkbox",
        "radiobutton",
        "switch",
        "cell",
        "row",
        "disclosuretriangle",
        "combobox",
        "popupbutton",
        "textfield",
        "textarea",
        "searchfield",
        "securetextfield",
    ];
    let destructive = regex::Regex::new(
        r"(?i)\b(delete|remove|erase|close account|purchase|buy|pay|send|publish|post|confirm deletion|log ?out|sign ?out)\b",
    )
    .expect("static destructive regex");
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for element in DriverResponse(snapshot).elements().into_iter().flatten() {
        let role = element
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim_start_matches("AX")
            .to_lowercase();
        if !roles.contains(&role.as_str()) || element.get("enabled") == Some(&Value::Bool(false)) {
            continue;
        }
        let label = ["label", "title", "placeholder", "identifier", "value"]
            .iter()
            .filter_map(|key| element.get(*key).and_then(Value::as_str))
            .find(|value| !value.trim().is_empty())
            .unwrap_or_default()
            .trim()
            .to_string();
        let token = element
            .get("element_token")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if label.is_empty() || token.is_empty() {
            continue;
        }
        let identity = format!("{}:{}", role, normalize(&label));
        if seen.insert(identity) {
            found.push(Action {
                role,
                destructive: destructive.is_match(&label),
                label,
                token,
            });
        }
    }
    found
}

pub(crate) fn matching_action(snapshot: &Value, step: &Step) -> Option<Action> {
    actions(snapshot).into_iter().find(|action| {
        action.role == step.role && normalize(&action.label) == normalize(&step.label)
    })
}

pub(crate) fn apply(
    driver: &CuaDriver,
    session: &str,
    pid: i64,
    window_id: i64,
    action: &Action,
) -> Result<Value> {
    call(
        driver,
        "click",
        &json!({
            "session": session,
            "pid": pid,
            "window_id": window_id,
            "element_token": action.token,
            "delivery_mode": "background",
        }),
    )
}

pub(crate) fn state_hash(snapshot: &Value) -> String {
    let mut rows: Vec<String> = DriverResponse(snapshot)
        .elements()
        .into_iter()
        .flatten()
        .map(|element| {
            format!(
                "{}|{}|{}|{}",
                element
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                element
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                element
                    .get("value")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                element
                    .get("selected")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            )
        })
        .collect();
    rows.sort();
    let mut digest = Sha256::new();
    digest.update(rows.join("\n").as_bytes());
    hex::encode(digest.finalize())
}

pub(crate) fn action_block_reason(action: &Action) -> Option<&'static str> {
    if action.destructive {
        return Some("destructive action withheld before delivery");
    }
    let label = normalize(&action.label);
    if action.role == "button" && matches!(label.as_str(), "back" | "cancel" | "dismiss" | "close") {
        return None;
    }
    Some("action is not independently safe navigation or cancellation and was withheld before delivery")
}

pub(crate) fn system_bundle(bundle: &str) -> bool {
    [
        "com.apple.SecurityAgent",
        "com.apple.UserNotificationCenter",
        "com.apple.notificationcenterui",
        "com.apple.systempreferences",
        "com.apple.systemsettings",
        "com.apple.CoreAuthUI",
    ]
    .iter()
    .any(|candidate| bundle == *candidate || bundle.starts_with(&format!("{candidate}.")))
}

/// Read the frontmost owner from the anchored `apps` array only. The previous
/// recursive descent returned the first matching key anywhere in the tree, so
/// map order decided which app answered (finding 13).
pub(crate) fn global_active_owner(driver: &CuaDriver, session: &str) -> Result<String> {
    let apps = call(driver, "list_apps", &json!({"session": session}))?;
    let entries = DriverResponse(&apps)
        .apps()
        .ok_or_else(|| anyhow!("Cua Driver list_apps returned no anchored apps array: {apps}"))?;
    let mut frontmost = entries
        .iter()
        .map(DriverApp)
        .filter(|entry| entry.frontmost());
    let owner = frontmost
        .next()
        .ok_or_else(|| anyhow!("Cua Driver list_apps reported no frontmost app: {apps}"))?;
    if frontmost.next().is_some() {
        bail!("Cua Driver list_apps reported more than one frontmost app: {apps}");
    }
    owner
        .bundle()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Cua Driver frontmost app has no exact bundle identifier: {apps}"))
}

/// Every running pid of `bundle_id`, read from the anchored `apps` array.
pub(crate) fn running_instances(driver: &CuaDriver, session: &str, bundle_id: &str) -> Result<Vec<i64>> {
    let apps = call(driver, "list_apps", &json!({"session": session}))?;
    let entries = DriverResponse(&apps)
        .apps()
        .ok_or_else(|| anyhow!("Cua Driver list_apps returned no anchored apps array: {apps}"))?;
    Ok(entries
        .iter()
        .map(DriverApp)
        .filter(|entry| entry.bundle() == Some(bundle_id))
        .filter_map(|entry| entry.pid())
        .collect())
}
