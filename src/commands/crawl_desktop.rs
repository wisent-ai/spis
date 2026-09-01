//! Real native desktop application crawler through Cua Driver.
//!
//! Runs one strictly window-scoped, genuinely sequential trajectory on the
//! Stado-selected host. Each action uses a token from a fresh target-window
//! snapshot and retains the exact driver response plus a fresh observed state.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};
const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

#[derive(Clone, Debug)]
struct Record {
    slug: String,
    name: String,
}


#[derive(Clone, Debug, serde::Serialize)]
struct Step {
    role: String,
    label: String,
}

#[derive(Clone, Debug)]
struct Action {
    role: String,
    label: String,
    token: String,
    destructive: bool,
}

pub(crate) fn cua_driver_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(configured) = std::env::var_os("SPIS_CUA_DRIVER_BIN") {
        candidates.push(PathBuf::from(configured));
    }
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|directory| directory.join("cua-driver")));
    }
    for applications in [
        Some(PathBuf::from("/Applications")),
        std::env::var_os("HOME").map(PathBuf::from).map(|home| home.join("Applications")),
    ].into_iter().flatten() {
        candidates.push(applications.join("CuaDriver.app/Contents/MacOS/cua-driver"));
        candidates.push(applications.join("CuaDriver.app/Contents/MacOS/CuaDriver"));
    }
    candidates
}

fn cua_driver() -> Result<PathBuf> {
    cua_driver_candidates().into_iter().find(|candidate| candidate.is_file())
        .context("Cua Driver executable is absent from PATH, /Applications and the worker user's Applications")
}

fn call(tool: &str, payload: &Value) -> Result<Value> {
    call_with_cli_options(tool, payload, &[])
}

fn call_with_cli_options(
    tool: &str,
    payload: &Value,
    options: &[&std::ffi::OsStr],
) -> Result<Value> {
    let mut command = Command::new(cua_driver()?);
    command.arg(tool).arg(serde_json::to_string(payload)?);
    command.args(options);
    let output = super::crawl::bounded_command_output(
        &mut command,
        &format!("cua-driver {tool}"),
        Duration::from_secs(30),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "cua-driver {tool} failed: status={}; stdout={:?}; stderr={:?}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("cua-driver {tool} returned no exact JSON document"))
}

struct SessionGuard {
    session: String,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        let _ = call("end_session", &json!({"session": self.session}));
    }
}

struct RecordingGuard {
    session: String,
    active: bool,
}

impl RecordingGuard {
    fn stop(&mut self) -> Option<Result<Value>> {
        if !self.active {
            return None;
        }
        self.active = false;
        Some(call("stop_recording", &json!({"session": self.session})))
    }
}

impl Drop for RecordingGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = call("stop_recording", &json!({"session": self.session}));
        }
    }
}

fn find_i64(value: &Value, key: &str) -> Option<i64> {
    let object = value.as_object()?;
    let direct = object.get(key).and_then(Value::as_i64);
    let result = object
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get(key))
        .and_then(Value::as_i64);
    match (direct, result) {
        (Some(left), Some(right)) if left != right => None,
        (Some(value), _) | (_, Some(value)) => Some(value),
        _ => None,
    }
}

fn find_elements(value: &Value) -> Option<&Vec<Value>> {
    let object = value.as_object()?;
    let direct = object.get("elements").and_then(Value::as_array);
    let result = object
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get("elements"))
        .and_then(Value::as_array);
    match (direct, result) {
        (Some(left), Some(right)) if left != right => None,
        (Some(value), None) | (None, Some(value)) | (Some(value), Some(_)) => Some(value),
        _ => None,
    }
}

fn find_bool(value: &Value, key: &str) -> Option<bool> {
    let object = value.as_object()?;
    let direct = object.get(key).and_then(Value::as_bool);
    let result = object
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get(key))
        .and_then(Value::as_bool);
    match (direct, result) {
        (Some(left), Some(right)) if left != right => None,
        (Some(value), _) | (_, Some(value)) => Some(value),
        _ => None,
    }
}
fn find_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    let object = value.as_object()?;
    let direct = object.get(key).and_then(Value::as_str);
    let result = object
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get(key))
        .and_then(Value::as_str);
    match (direct, result) {
        (Some(left), Some(right)) if left != right => None,
        (Some(value), _) | (_, Some(value)) => Some(value),
        _ => None,
    }
}

fn preflight() -> Result<()> {
    if std::env::consts::OS != "macos" {
        return Ok(());
    }
    let mut command = Command::new(cua_driver()?);
    command.args(["permissions", "status", "--json"]);
    let output = super::crawl::bounded_command_output(
        &mut command,
        "Cua Driver permission status",
        Duration::from_secs(15),
        1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "cua-driver permission status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let status: Value =
        serde_json::from_slice(&output.stdout).context("parse Cua Driver permission status")?;
    let accessibility =
        find_bool(&status, "accessibility").or_else(|| find_bool(&status, "accessibility_granted"));
    let screen = find_bool(&status, "screen_recording")
        .or_else(|| find_bool(&status, "screen_recording_granted"));
    if accessibility != Some(true) {
        bail!("Cua Driver accessibility permission is unavailable on the selected host");
    }
    if screen != Some(true) {
        bail!("Cua Driver screen-recording permission is unavailable on the selected host");
    }
    Ok(())
}
fn records(catalog: &str, selected: Option<&str>) -> Result<Vec<Record>> {
    if !matches!(catalog, "macos-app-examples" | "desktop-app-examples") {
        bail!("crawl-desktop accepts macos-app-examples or desktop-app-examples");
    }
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(catalog)
        .join("references");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    paths.sort();
    let mut records = Vec::new();
    for path in paths {
        let slug = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if selected.is_some_and(|selected| {
            selected != slug
                && selected != slug.split_once('-').map(|(_, tail)| tail).unwrap_or(slug)
        }) {
            continue;
        }
        let record: Value = serde_json::from_slice(&std::fs::read(path.join("reference.json"))?)?;
        records.push(Record {
            slug: slug.to_string(),
            name: record
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        });
    }
    if records.is_empty() {
        bail!("no matching records in {catalog}");
    }
    Ok(records)
}

fn snapshot(session: &str, pid: i64, window_id: i64, screenshot: &Path) -> Result<Value> {
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

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn actions(snapshot: &Value) -> Vec<Action> {
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
    for element in find_elements(snapshot).into_iter().flatten() {
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

fn matching_action(snapshot: &Value, step: &Step) -> Option<Action> {
    actions(snapshot).into_iter().find(|action| {
        action.role == step.role && normalize(&action.label) == normalize(&step.label)
    })
}

fn apply(session: &str, pid: i64, window_id: i64, action: &Action) -> Result<Value> {
    call(
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

fn state_hash(snapshot: &Value) -> String {
    let mut rows: Vec<String> = find_elements(snapshot)
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
fn action_block_reason(action: &Action) -> Option<&'static str> {
    if action.destructive {
        return Some("destructive action withheld before delivery");
    }
    let label = normalize(&action.label);
    if action.role == "button" && matches!(label.as_str(), "back" | "cancel" | "dismiss" | "close") {
        return None;
    }
    Some("action is not independently safe navigation or cancellation and was withheld before delivery")
}

fn system_bundle(bundle: &str) -> bool {
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
fn find_active_owner(value: &Value) -> Option<&str> {
    match value {
        Value::Object(map) => {
            let active = ["frontmost", "is_frontmost", "active", "is_active"]
                .iter()
                .any(|key| map.get(*key).and_then(Value::as_bool) == Some(true));
            if active {
                ["bundle_id", "bundle_identifier", "owner_bundle_id"]
                    .iter()
                    .find_map(|key| map.get(*key).and_then(Value::as_str))
                    .or_else(|| map.values().find_map(find_active_owner))
            } else {
                map.values().find_map(find_active_owner)
            }
        }
        Value::Array(values) => values.iter().find_map(find_active_owner),
        _ => None,
    }
}

fn global_active_owner(session: &str) -> Result<String> {
    let apps = call("list_apps", &json!({"session": session}))?;
    find_active_owner(&apps)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Cua Driver list_apps returned no exact active-owner bundle: {apps}"))
}

fn assert_target_surface(
    snapshot: &Value,
    expected_pid: i64,
    expected_window: i64,
    expected_bundle: &str,
) -> Result<()> {
    let observed_pid = find_i64(snapshot, "pid").context("Cua window snapshot has no owner pid")?;
    let observed_window =
        find_i64(snapshot, "window_id").context("Cua window snapshot has no window id")?;
    if observed_pid != expected_pid || observed_window != expected_window {
        bail!(
            "Cua window ownership changed: expected pid={expected_pid} window={expected_window}, observed pid={observed_pid} window={observed_window}"
        );
    }
    let observed_bundle = ["owner_bundle_id", "bundle_id", "bundle_identifier"]
        .iter()
        .find_map(|key| find_string(snapshot, key))
        .context("Cua window snapshot has no exact owner bundle identifier")?;
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

fn readiness_observation(
    manifest: &super::crawl::RuntimeManifest,
    product: &str,
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
    let output = Command::new("stado-runtime-readiness")
        .args([
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
        ])
        .output()
        .context("run fresh desktop runtime-readiness verification")?;
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

fn verify_desktop_executable(
    expected_bundle: &str,
    manifest: &super::crawl::RuntimeManifest,
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
        let output = Command::new("/usr/bin/plutil")
            .args(["-extract", key, "raw", "-o", "-"])
            .arg(&info)
            .output()
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
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("open exact desktop executable {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash exact desktop executable {}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let observed_sha = hex::encode(digest.finalize());
    if !observed_sha.eq_ignore_ascii_case(expected_sha) {
        bail!(
            "desktop executable SHA-256 changed immediately before launch: expected {expected_sha}, observed {observed_sha}"
        );
    }
    let observation = readiness_observation(manifest, expected_bundle)?;
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

fn launch(record: &Record, bundle_id: &str, session: &str) -> Result<(i64, i64)> {
    let launched = call(
        "launch_app",
        &json!({
            "bundle_id": bundle_id,
        }),
    )?;
    let pid = find_i64(&launched, "pid")
        .ok_or_else(|| anyhow!("{} launch returned no pid", record.name))?;
    let window = find_i64(&launched, "window_id")
        .or_else(|| {
            call("list_windows", &json!({"pid": pid, "session": session}))
                .ok()
                .and_then(|value| find_i64(&value, "window_id"))
        })
        .ok_or_else(|| anyhow!("{} launch returned no window", record.name))?;
    Ok((pid, window))
}

fn crawl_record(
    record: &Record,
    manifest: &super::crawl::RuntimeManifest,
    root: &Path,
    max_states: usize,
    max_depth: usize,
) -> Result<Value> {
    let bundle_id = manifest
        .prepared_proof
        .as_ref()
        .context("desktop runtime manifest has no exact prepared product identity")?
        .product_identifier
        .clone();
    let session = format!("spis-{}", record.slug.replace('_', "-"));
    call(
        "start_session",
        &json!({"session": session, "capture_scope": "window"}),
    )?;
    let _session_guard = SessionGuard {
        session: session.clone(),
    };
    let session_state = call("get_session_state", &json!({"session": session}))?;
    if find_string(&session_state, "capture_scope") != Some("window") {
        bail!("Cua Driver did not establish the required strict window session: {session_state}");
    }

    let output = root.join(&record.slug);
    let states_dir = output.join("states");
    let transitions_dir = output.join("transitions");
    let recording_dir = output.join("trajectory");
    std::fs::create_dir_all(&states_dir)?;
    std::fs::create_dir_all(&transitions_dir)?;
    std::fs::create_dir_all(&recording_dir)?;

    // These are deliberately the last product checks before the exact-bundle
    // launch. No catalog display name or PATH lookup participates.
    let readiness = verify_desktop_executable(&bundle_id, manifest)?;
    let (pid, window_id) = launch(record, &bundle_id, &session)?;
    call(
        "start_recording",
        &json!({
            "session": session,
            "output_dir": recording_dir,
            "record_video": false,
        }),
    )?;
    let mut recording_guard = RecordingGuard {
        session: session.clone(),
        active: true,
    };

    let mut trajectory = Vec::<Step>::new();
    let mut seen_states = HashSet::new();
    let mut attempted = HashSet::<String>::new();
    let mut reported_gaps = HashSet::<String>::new();
    let mut graph = Vec::new();
    let mut transitions = Vec::new();
    let mut blocked = Vec::new();

    for action_index in 0..=max_depth {
        if seen_states.len() >= max_states {
            blocked.push(json!({
                "reason": "state limit reached on the single observed trajectory; unexplored branches remain explicit gaps",
                "max_states": max_states,
            }));
            break;
        }
        let observation_path = states_dir.join(format!("observation-{:04}.png", action_index + 1));
        let current = snapshot(&session, pid, window_id, &observation_path)?;
        assert_target_surface(&current, pid, window_id, &bundle_id)?;
        let hash = state_hash(&current);
        let available = actions(&current);
        if seen_states.insert(hash.clone()) {
            let index = seen_states.len();
            let snapshot_path = states_dir.join(format!("state-{index:04}.json"));
            std::fs::write(
                &snapshot_path,
                serde_json::to_string_pretty(&current)? + "\n",
            )?;
            graph.push(json!({
                "state": hash,
                "index": index,
                "trajectory_depth": trajectory.len(),
                "delivered_inputs": trajectory,
                "observed_state": {
                    "snapshot": snapshot_path.strip_prefix(&output).unwrap_or(&snapshot_path),
                    "screenshot": observation_path.strip_prefix(&output).unwrap_or(&observation_path),
                },
                "available_actions": available.iter().map(|action| json!({
                    "role": action.role,
                    "label": action.label,
                    "destructive": action.destructive,
                    "kind": "click",
                    "withheld_reason": action_block_reason(action),
                })).collect::<Vec<_>>(),
            }));
        }

        let mut safe = Vec::new();
        for action in available {
            let identity = format!("{}|{}|{}", hash, action.role, normalize(&action.label));
            if let Some(reason) = action_block_reason(&action) {
                if reported_gaps.insert(identity) {
                    blocked.push(json!({
                        "state": hash,
                        "role": action.role,
                        "label": action.label,
                        "delivered_input": Value::Null,
                        "observed_state_change": Value::Null,
                        "reason": reason,
                    }));
                }
            } else if !attempted.contains(&identity) {
                safe.push((identity, action));
            }
        }
        if action_index == max_depth {
            for (_, action) in safe {
                blocked.push(json!({
                    "state": hash,
                    "role": action.role,
                    "label": action.label,
                    "delivered_input": Value::Null,
                    "observed_state_change": Value::Null,
                    "reason": "trajectory depth limit reached before this independently safe branch",
                }));
            }
            break;
        }
        let Some((attempt_identity, selected)) = safe.first().cloned() else {
            break;
        };
        attempted.insert(attempt_identity);

        let transition_index = transitions.len() + 1;
        let pre_image = transitions_dir.join(format!("step-{transition_index:04}-before.png"));
        let pre = snapshot(&session, pid, window_id, &pre_image)?;
        assert_target_surface(&pre, pid, window_id, &bundle_id)?;
        let active_owner_before = global_active_owner(&session)?;
        if active_owner_before != bundle_id {
            blocked.push(json!({
                "state": hash,
                "delivered_input": Value::Null,
                "observed_state_change": Value::Null,
                "reason": format!("fresh global observation found active owner {active_owner_before:?}, not exact target {bundle_id:?}; input withheld"),
                "further_input_withheld": true,
            }));
            break;
        }
        let step = Step {
            role: selected.role.clone(),
            label: selected.label.clone(),
        };
        let Some(action) = matching_action(&pre, &step) else {
            blocked.push(json!({
                "state": hash,
                "role": step.role,
                "label": step.label,
                "delivered_input": Value::Null,
                "observed_state_change": Value::Null,
                "reason": "fresh pre-action snapshot no longer exposed the selected token",
            }));
            continue;
        };
        if let Some(reason) = action_block_reason(&action) {
            blocked.push(json!({
                "state": hash,
                "role": action.role,
                "label": action.label,
                "delivered_input": Value::Null,
                "observed_state_change": Value::Null,
                "reason": reason,
            }));
            continue;
        }
        let before_hash = state_hash(&pre);
        let driver_response = match apply(&session, pid, window_id, &action) {
            Ok(response) => response,
            Err(error) => {
                transitions.push(json!({
                    "step": transition_index,
                    "delivered_input": {
                        "role": action.role,
                        "label": action.label,
                        "delivery_status": "unknown",
                        "exact_driver_diagnostic": error.to_string(),
                    },
                    "observed_state_change": Value::Null,
                }));
                break;
            }
        };
        let driver_effect = find_string(&driver_response, "effect")
            .unwrap_or("missing")
            .to_string();
        let driver_route = find_string(&driver_response, "route")
            .unwrap_or("missing")
            .to_string();
        let post_image = transitions_dir.join(format!("step-{transition_index:04}-after.png"));
        let post = match snapshot(&session, pid, window_id, &post_image) {
            Ok(post) => post,
            Err(error) => {
                transitions.push(json!({
                    "step": transition_index,
                    "delivered_input": {
                        "role": action.role,
                        "label": action.label,
                        "driver_response": driver_response,
                    },
                    "observed_state_change": Value::Null,
                    "exact_observation_diagnostic": error.to_string(),
                }));
                break;
            }
        };
        let active_owner_after = global_active_owner(&session)?;
        let post_snapshot = transitions_dir.join(format!("step-{transition_index:04}-after.json"));
        std::fs::write(
            &post_snapshot,
            serde_json::to_string_pretty(&post)? + "\n",
        )?;
        if let Err(error) = assert_target_surface(&post, pid, window_id, &bundle_id) {
            transitions.push(json!({
                "step": transition_index,
                "delivered_input": {
                    "role": action.role,
                    "label": action.label,
                    "driver_response": driver_response,
                },
                "observed_state_change": {
                    "snapshot": post_snapshot.strip_prefix(&output).unwrap_or(&post_snapshot),
                    "screenshot": post_image.strip_prefix(&output).unwrap_or(&post_image),
                    "exact_diagnostic": error.to_string(),
                },
            }));
            blocked.push(json!({
                "state": before_hash,
                "reason": error.to_string(),
                "further_input_withheld": true,
            }));
            break;
        }
        let after_hash = state_hash(&post);
        let changed = before_hash != after_hash;
        transitions.push(json!({
            "step": transition_index,
            "delivered_input": {
                "role": action.role,
                "label": action.label,
                "driver_response": driver_response,
                "inspected_effect": driver_effect,
                "inspected_route": driver_route,
                "global_active_owner": active_owner_before,
            },
            "observed_state_change": {
                "changed": changed,
                "before_state": before_hash,
                "after_state": after_hash,
                "snapshot": post_snapshot.strip_prefix(&output).unwrap_or(&post_snapshot),
                "screenshot": post_image.strip_prefix(&output).unwrap_or(&post_image),
                "global_active_owner": active_owner_after,
            },
        }));
        trajectory.push(step);
        if active_owner_after != bundle_id {
            blocked.push(json!({
                "state": after_hash,
                "reason": format!("fresh global post-action observation found active owner {active_owner_after:?}, not exact target {bundle_id:?}; further input withheld"),
                "further_input_withheld": true,
            }));
            break;
        }
        if driver_effect != "confirmed" {
            blocked.push(json!({
                "state": after_hash,
                "role": action.role,
                "label": action.label,
                "reason": format!(
                    "driver action effect was {driver_effect:?}; fresh postcondition was retained, and further input was withheld"
                ),
                "further_input_withheld": true,
            }));
            break;
        }
        if changed {
            for (_, alternative) in safe.into_iter().skip(1) {
                blocked.push(json!({
                    "state": before_hash,
                    "role": alternative.role,
                    "label": alternative.label,
                    "delivered_input": Value::Null,
                    "observed_state_change": Value::Null,
                    "reason": "single sequential trajectory took a different safe edge; this branch was not reset or inferred",
                }));
            }
        }
    }

    let recording = recording_guard
        .stop()
        .transpose()?
        .context("Cua Driver did not return session-scoped recording metadata")?;
    let recording_path = output.join("recording.json");
    std::fs::write(
        &recording_path,
        serde_json::to_string_pretty(&recording)? + "\n",
    )?;
    let report = json!({
        "schema": "wisent.desktop-crawl-run.v1",
        "record": record.slug,
        "name": record.name,
        "driver": "cua-driver",
        "bundle_id": bundle_id,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "runtime_execution_identity": manifest.execution_identity,
        "fresh_runtime_readiness_observation": readiness,
        "resource_lease": manifest.resource_lease,
        "capture_scope": "window",
        "record_video": false,
        "fresh_snapshot_before_after_each_action": true,
        "states": graph,
        "states_seen": seen_states.len(),
        "transitions": transitions,
        "blocked_edges": blocked,
        "max_states": max_states,
        "max_depth": max_depth,
        "evidence_observations": {
            "executed_trajectory": trajectory,
            "accessibility_artifacts": graph.iter().filter_map(|state| state.pointer("/observed_state/snapshot").cloned()).collect::<Vec<_>>(),
            "motion_artifacts": [recording_path.strip_prefix(&output).unwrap_or(&recording_path)],
            "canonical_interactions": [],
            "canonical_journey": Value::Null,
            "canonical_accessibility": Value::Null,
            "canonical_motion_analysis": Value::Null,
            "gaps": [
                "Only one genuinely sequential observed trajectory was retained; alternative branches were not reset or inferred.",
                "Destructive, input, and ambiguous controls were withheld before delivery.",
                "No screen-reader, focus-order, live-region, or reduced-motion variant was executed."
            ]
        },
    });
    std::fs::write(
        output.join("crawl.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}

fn revision() -> Result<String> { super::crawl::build_revision() }

fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        bail!("{flag} must contain only letters, digits, '-' or '_'");
    }
    Ok(())
}
fn attempt_root(
    base: &Path,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<PathBuf> {
    super::crawl::native_attempt_root(base, manifest)
}



struct DesktopSubmission<'a> {
    host: &'a str,
    catalog: &'a str,
    record: &'a str,
    max_states: usize,
    max_depth: usize,
    manifest: &'a super::crawl::RuntimeManifest,
}

fn submit_worker(request: DesktopSubmission<'_>) -> Result<()> {
    safe_job_value(request.host, "--host")?;
    safe_job_value(request.catalog, "catalog")?;
    safe_job_value(request.record, "--record")?;
    let _attempt_binding = attempt_root(Path::new("."), request.manifest)?;
    if revision()? != request.manifest.source_revision {
        bail!("desktop coordinator revision does not match immutable runtime manifest");
    }
    let encoded = request.manifest.encoded()?;
    let artifact = request.manifest.artifact_uri.clone();
    let output_uri = request.manifest.output_uri.clone();
    let command = format!(
        "cargo run --release -- crawl-desktop {} --worker --record {} --max-states {} --max-depth {} --artifact-uri {} --runtime-manifest-base64 '{}'",
        request.catalog,
        request.record,
        request.max_states,
        request.max_depth,
        artifact,
        encoded,
    );
    let arguments = vec![
        "submit".to_string(),
        command,
        "--run-id".to_string(),
        request.manifest.stado_run_id.clone(),
        "--pinned-host".to_string(),
        request.host.to_string(),
        "--exclusive".to_string(),
        "--repo".to_string(),
        REPOSITORY.to_string(),
        "--repo-ref".to_string(),
        request.manifest.source_revision.clone(),
        "--repo-workdir".to_string(),
        "spis".to_string(),
        "--repo-extras".to_string(),
        String::new(),
        "--output-uri".to_string(),
        output_uri.clone(),
    ];
    let mut stado = super::crawl::stado_command();
    stado.args(arguments);
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "submit desktop crawl through Stado",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "Stado refused desktop crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    super::crawl::print_submission(
        request.catalog,
        "desktop",
        request.host,
        Some(&artifact),
        &output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

fn hash_artifact(path: &Path) -> Result<(String, u64)> {
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("open desktop artifact {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash desktop artifact {}", path.display()))?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        digest.update(&buffer[..count]);
    }
    Ok((hex::encode(digest.finalize()), bytes))
}

fn publish_artifact(run_root: &Path, uri: &str) -> Result<Value> {
    let attempt_name = run_root
        .file_name()
        .and_then(|value| value.to_str())
        .context("desktop attempt artifact root has no UTF-8 name")?;
    let archive = run_root.with_file_name(format!("{attempt_name}.tar.gz"));
    if !archive.is_file() {
        let mut stado = super::crawl::stado_command();
        stado
            .args(["storage", "archive"])
            .arg(run_root)
            .arg(&archive);
        let output = super::crawl::bounded_command_output(
            &mut stado,
            "archive desktop crawl",
            Duration::from_secs(120),
            4 * 1024 * 1024,
        )?;
        if !output.status.success() {
            bail!(
                "stado storage archive refused desktop crawl artifacts: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
    }
    let (sha256, bytes) = hash_artifact(&archive)?;
    let mut stado = super::crawl::stado_command();
    stado
        .args(["storage", "put", "--if-absent", uri])
        .arg(&archive);
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "publish desktop crawl",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "stado storage put refused desktop crawl artifacts: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let readback = run_root.with_file_name(format!("{attempt_name}.readback.tar.gz"));
    if readback.exists() {
        std::fs::remove_file(&readback)?;
    }
    let mut stado = super::crawl::stado_command();
    stado.args(["storage", "get", uri]).arg(&readback);
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "read back desktop crawl",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "stado storage readback refused desktop crawl artifacts: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let (observed_sha256, observed_bytes) = hash_artifact(&readback)?;
    std::fs::remove_file(&readback)?;
    if observed_sha256 != sha256 || observed_bytes != bytes {
        bail!(
            "desktop artifact readback differs: expected sha256={sha256} bytes={bytes}, observed sha256={observed_sha256} bytes={observed_bytes}"
        );
    }
    Ok(json!({
        "uri": uri,
        "sha256": sha256,
        "bytes": bytes,
        "media_type": "application/gzip",
    }))
}

fn worker_report(
    manifest: &super::crawl::RuntimeManifest,
    artifact: Value,
) -> Result<Value> {
    let value = serde_json::to_value(manifest)?;
    let attempt = value
        .get("attempt")
        .and_then(Value::as_u64)
        .context("desktop runtime manifest has no attempt for worker report")?;
    let attempt_id = value
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("desktop runtime manifest has no attempt_id for worker report")?;
    let bindings_file_sha256 = value
        .get("bindings_file_sha256")
        .and_then(Value::as_str)
        .context("desktop runtime manifest has no bindings_file_sha256")?;
    let bindings_sha256 = value
        .get("bindings_sha256")
        .and_then(Value::as_str)
        .context("desktop runtime manifest has no bindings_sha256")?;
    let execution_identity = value
        .get("execution_identity")
        .filter(|identity| identity.is_object())
        .context("desktop runtime manifest has no typed execution_identity")?
        .clone();
    Ok(json!({
        "schema": "wisent.native-worker-report.v1",
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "record_key": manifest.record_key,
        "attempt": attempt,
        "attempt_id": attempt_id,
        "engine": manifest.engine,
        "state": "artifact_published",
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "reference_sha256": manifest.reference_sha256,
        "bindings_file_sha256": bindings_file_sha256,
        "bindings_sha256": bindings_sha256,
        "execution_identity": execution_identity,
        "artifact": artifact,
    }))
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut record: Option<String> = None;
    let mut max_states = 200usize;
    let mut max_depth = 8usize;
    let mut host: Option<String> = None;
    let mut worker = false;
    let mut artifact_uri: Option<String> = None;
    let mut runtime_manifest_base64: Option<String> = None;
    let mut output = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".spis")
        .join("crawls");
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--worker" => worker = true,
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            "--runtime-manifest-base64" => {
                i += 1;
                runtime_manifest_base64 =
                    Some(rest.get(i).context("--runtime-manifest-base64 needs a value")?.clone());
            }
            "--max-states" => {
                i += 1;
                max_states = rest.get(i).context("--max-states needs a value")?.parse()?;
            }
            "--max-depth" => {
                i += 1;
                max_depth = rest.get(i).context("--max-depth needs a value")?.parse()?;
            }
            "--output" => {
                i += 1;
                output = PathBuf::from(rest.get(i).context("--output needs a value")?);
            }
            "--help" | "-h" => {
                println!("usage: spis crawl-desktop <macos-app-examples|desktop-app-examples> --host TARGET --record SLUG --runtime-manifest-base64 DATA [--max-states N] [--max-depth N]\nworker mode requires the same immutable runtime manifest and exact record.");
                return Ok(());
            }
            value if value.starts_with('-') => bail!("unknown argument: {value}"),
            value if catalog.is_none() => catalog = Some(value.to_string()),
            value => bail!("unexpected argument: {value}"),
        }
        i += 1;
    }
    let catalog = catalog.context("catalog is required")?;
    if max_states == 0 || max_states > 10_000 || max_depth > 32 {
        bail!("--max-states must be 1..10000 and --max-depth must be 0..32");
    }
    let record = record.context("--record is required for one exact per-record job")?;
    let encoded_manifest = runtime_manifest_base64
        .as_deref()
        .context("--runtime-manifest-base64 is required")?;
    let manifest = super::crawl::decode_runtime_manifest(
        encoded_manifest,
        &catalog,
        "desktop",
        Some(&record),
    )?;
    if !worker {
        let host =
            host.context("--host is required; desktop crawls execute as pinned Stado jobs")?;
        return submit_worker(DesktopSubmission {
            host: &host,
            catalog: &catalog,
            record: &record,
            max_states,
            max_depth,
            manifest: &manifest,
        });
    }
    if host.is_some() {
        bail!("--host is coordinator-only");
    }
    let artifact_uri = artifact_uri.context("--artifact-uri is required in worker mode")?;
    if artifact_uri != manifest.artifact_uri {
        bail!("worker artifact URI does not match immutable runtime manifest");
    }
    preflight()?;
    let run_root = attempt_root(
        &output,
        &manifest,
    )?;
    std::fs::create_dir_all(&run_root)?;
    let entry = records(&catalog, Some(&record))?
        .into_iter()
        .next()
        .context("runtime manifest record is absent from catalog")?;
    let reports = vec![match crawl_record(&entry, &manifest, &run_root, max_states, max_depth) {
        Ok(report) => report,
        Err(error) => json!({
            "record": entry.slug,
            "name": entry.name,
            "status": "failed",
            "source_revision": manifest.source_revision,
            "source_input_sha256": manifest.source_input_sha256,
            "runtime_manifest": manifest,
            "error": error.to_string(),
        }),
    }];
    let failures = reports
        .iter()
        .filter(|report| report.get("status").and_then(Value::as_str) == Some("failed"))
        .count();
    let summary = json!({
        "schema": "wisent.desktop-crawl-batch.v1",
        "catalog": catalog,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "records": reports,
        "failed": failures,
    });
    std::fs::write(
        run_root.join("batch.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    if failures > 0 {
        bail!("{failures} desktop records could not be crawled");
    }
    let artifact = publish_artifact(&run_root, &artifact_uri)?;
    let worker_report = worker_report(&manifest, artifact)?;
    println!("{}", serde_json::to_string(&worker_report)?);
    Ok(())
}
