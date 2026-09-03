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

/// Only the absolute bundle paths the coordinator itself admits. Inherited
/// PATH entries are deliberately absent: any writable earlier PATH entry would
/// be arbitrary code execution with accessibility privileges (finding 10).
pub(crate) fn cua_driver_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications/CuaDriver.app/Contents/MacOS/cua-driver"),
        PathBuf::from("/Applications/CuaDriver.app/Contents/MacOS/CuaDriver"),
    ]
}

/// The driver binary pinned for the whole record: path, digest and version are
/// observed once and every later call runs this exact file.
#[derive(Clone, Debug)]
struct CuaDriver {
    path: PathBuf,
    sha256: String,
    version: String,
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash {}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex::encode(digest.finalize()))
}

/// Resolve, canonicalize, hash and version-stamp the driver exactly once per
/// record. Re-resolving inside every call left the binary unstable even within
/// one record, and following a symlink out of the bundle defeated the pin
/// entirely (finding 10).
fn pin_cua_driver() -> Result<CuaDriver> {
    let path = cua_driver_candidates()
        .into_iter()
        .find(|candidate| candidate.is_file())
        .context("Cua Driver executable is absent from the admitted /Applications bundle paths")?;
    if !path.is_absolute() {
        bail!("Cua Driver candidate {} is not absolute", path.display());
    }
    if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
        bail!("Cua Driver executable {} is a symlink", path.display());
    }
    let canonical = std::fs::canonicalize(&path)?;
    if canonical != path {
        bail!(
            "Cua Driver executable is not canonical: declared {}, canonical {}",
            path.display(),
            canonical.display()
        );
    }
    let sha256 = hash_file(&path)?;
    let mut version_command = Command::new(&path);
    version_command.arg("--version");
    let output = super::crawl::bounded_command_output(
        &mut version_command,
        "read pinned Cua Driver version",
        Duration::from_secs(15),
        64 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "pinned Cua Driver {} refused --version: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(CuaDriver {
        path,
        sha256,
        version: String::from_utf8_lossy(&output.stdout).trim().to_string(),
    })
}

/// One helper binary, pinned exactly once per record.
#[derive(Clone, Debug)]
struct PinnedHelper {
    path: PathBuf,
    sha256: String,
    version: String,
}

/// The readiness helper is resolved from absolute directories only, never from
/// the inherited PATH, and its digest and version are retained (finding 10).
fn pinned_readiness_helper() -> Result<PinnedHelper> {
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
            "pinned desktop readiness helper {} is a symlink",
            path.display()
        );
    }
    let canonical = std::fs::canonicalize(&path)?;
    if canonical != path {
        bail!(
            "pinned desktop readiness helper is not canonical: declared {}, canonical {}",
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
        "read pinned desktop readiness helper version",
        Duration::from_secs(15),
        64 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "pinned desktop readiness helper {} refused --version: {}",
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

fn call(driver: &CuaDriver, tool: &str, payload: &Value) -> Result<Value> {
    call_with_cli_options(driver, tool, payload, &[], Duration::from_secs(30))
}

/// Cleanup deadline. A guard must never block the worker (finding 4).
fn call_briefly(driver: &CuaDriver, tool: &str, payload: &Value) -> Result<Value> {
    call_with_cli_options(driver, tool, payload, &[], Duration::from_secs(5))
}

fn call_with_cli_options(
    driver: &CuaDriver,
    tool: &str,
    payload: &Value,
    options: &[&std::ffi::OsStr],
    timeout: Duration,
) -> Result<Value> {
    let mut command = Command::new(&driver.path);
    command.arg(tool).arg(serde_json::to_string(payload)?);
    command.args(options);
    let output = super::crawl::bounded_command_output(
        &mut command,
        &format!("cua-driver {tool}"),
        timeout,
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
    driver: CuaDriver,
    session: String,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        let _ = call_briefly(&self.driver, "end_session", &json!({"session": self.session}));
    }
}

/// The launched application is terminated with the record. Without this the app
/// outlives the crawl, and across a catalog every record leaves another live GUI
/// app contending for focus (finding 5).
struct AppGuard {
    driver: CuaDriver,
    session: String,
    pid: i64,
}

impl Drop for AppGuard {
    fn drop(&mut self) {
        let _ = call_briefly(
            &self.driver,
            "terminate_app",
            &json!({"session": self.session, "pid": self.pid}),
        );
    }
}

struct RecordingGuard {
    driver: CuaDriver,
    session: String,
    active: bool,
}

impl RecordingGuard {
    fn stop(&mut self) -> Option<Result<Value>> {
        if !self.active {
            return None;
        }
        self.active = false;
        Some(call(
            &self.driver,
            "stop_recording",
            &json!({"session": self.session}),
        ))
    }
}

impl Drop for RecordingGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = call_briefly(
                &self.driver,
                "stop_recording",
                &json!({"session": self.session}),
            );
        }
    }
}

/// Typed record failure so the worker report can name a stable machine code
/// instead of a free-text diagnostic.
#[derive(Debug)]
struct RecordFailure {
    code: &'static str,
    message: String,
}

impl std::fmt::Display for RecordFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for RecordFailure {}

fn failure_code(error: &anyhow::Error) -> &'static str {
    error
        .chain()
        .find_map(|cause| {
            cause
                .downcast_ref::<RecordFailure>()
                .map(|failure| failure.code)
        })
        .unwrap_or("desktop_record_failed")
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

fn find_array<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    let object = value.as_object()?;
    let direct = object.get(key).and_then(Value::as_array);
    let result = object
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get(key))
        .and_then(Value::as_array);
    match (direct, result) {
        (Some(left), Some(right)) if left != right => None,
        (Some(value), None) | (None, Some(value)) | (Some(value), Some(_)) => Some(value),
        _ => None,
    }
}

/// Anchored, typed view of one cua-driver response. Every read consults only
/// the root object and its `result` child, so a child element, a nested
/// accessibility node or a sibling window list can never answer an identity or
/// ownership question by serde_json map order (finding 13).
struct DriverResponse<'a>(&'a Value);

impl<'a> DriverResponse<'a> {
    fn integer(&self, key: &str) -> Option<i64> {
        find_i64(self.0, key)
    }

    fn text(&self, key: &str) -> Option<&'a str> {
        find_string(self.0, key)
    }

    fn bundle(&self) -> Option<&'a str> {
        ["owner_bundle_id", "bundle_id", "bundle_identifier"]
            .iter()
            .find_map(|key| self.text(key))
    }

    fn elements(&self) -> Option<&'a Vec<Value>> {
        find_array(self.0, "elements")
    }

    fn apps(&self) -> Option<&'a Vec<Value>> {
        find_array(self.0, "apps")
    }
}

/// The window ownership triple, read only from anchored positions.
struct WindowOwnership {
    pid: i64,
    window_id: i64,
    owner_bundle_id: String,
}

impl WindowOwnership {
    fn read(snapshot: &Value) -> Result<Self> {
        let response = DriverResponse(snapshot);
        Ok(Self {
            pid: response
                .integer("pid")
                .context("Cua window snapshot has no anchored owner pid")?,
            window_id: response
                .integer("window_id")
                .context("Cua window snapshot has no anchored window id")?,
            owner_bundle_id: response
                .bundle()
                .context("Cua window snapshot has no anchored owner bundle identifier")?
                .to_string(),
        })
    }
}

/// One entry of an anchored `list_apps` response.
struct DriverApp<'a>(&'a Value);

impl DriverApp<'_> {
    fn pid(&self) -> Option<i64> {
        self.0.get("pid").and_then(Value::as_i64)
    }

    fn bundle(&self) -> Option<&str> {
        ["bundle_id", "bundle_identifier", "owner_bundle_id"]
            .iter()
            .find_map(|key| self.0.get(*key).and_then(Value::as_str))
    }

    fn frontmost(&self) -> bool {
        ["frontmost", "is_frontmost", "active", "is_active"]
            .iter()
            .any(|key| self.0.get(*key).and_then(Value::as_bool) == Some(true))
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

fn preflight(driver: &CuaDriver) -> Result<()> {
    if std::env::consts::OS != "macos" {
        return Ok(());
    }
    let mut command = Command::new(&driver.path);
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

fn snapshot(
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

fn matching_action(snapshot: &Value, step: &Step) -> Option<Action> {
    actions(snapshot).into_iter().find(|action| {
        action.role == step.role && normalize(&action.label) == normalize(&step.label)
    })
}

fn apply(
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

fn state_hash(snapshot: &Value) -> String {
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
/// Read the frontmost owner from the anchored `apps` array only. The previous
/// recursive descent returned the first matching key anywhere in the tree, so
/// map order decided which app answered (finding 13).
fn global_active_owner(driver: &CuaDriver, session: &str) -> Result<String> {
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
fn running_instances(driver: &CuaDriver, session: &str, bundle_id: &str) -> Result<Vec<i64>> {
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

fn assert_target_surface(
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

fn readiness_observation(
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

fn verify_desktop_executable(
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

struct LaunchedApp {
    pid: i64,
    window_id: i64,
    executable_path: Option<String>,
    response: Value,
}

fn launch(
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
fn terminate_pre_existing(
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

/// Prove the process now running is the exact executable the manifest bound:
/// the path comes from an anchored field of the launch response, must be an
/// absolute canonical non-symlink file, and is re-hashed against the manifest
/// digest (finding 5b).
fn launched_executable_proof(
    launched: &LaunchedApp,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<Value> {
    let identity = manifest
        .execution_identity
        .as_ref()
        .context("desktop runtime manifest has no resolved execution identity")?;
    let expected_path = identity
        .executable_path
        .as_deref()
        .context("desktop execution identity has no exact executable path")?;
    let observed = launched.executable_path.as_deref().context(
        "cua-driver launch response has no anchored executable path for the launched pid",
    )?;
    if observed != expected_path {
        return Err(anyhow::Error::new(RecordFailure {
            code: "desktop_executable_identity_mismatch",
            message: format!(
                "launched pid {} runs {observed}, not the manifest-bound executable {expected_path}",
                launched.pid
            ),
        }));
    }
    let path = Path::new(observed);
    if !path.is_absolute() {
        bail!("launched executable path {observed} is not absolute");
    }
    if std::fs::symlink_metadata(path)?.file_type().is_symlink() {
        bail!("launched executable path {observed} is a symlink");
    }
    let canonical = std::fs::canonicalize(path)?;
    if canonical != path {
        bail!(
            "launched executable path is not canonical: declared {observed}, canonical {}",
            canonical.display()
        );
    }
    let expected_sha = identity
        .executable_sha256
        .as_deref()
        .context("desktop execution identity has no executable SHA-256")?;
    let observed_sha = hash_file(path)?;
    if !observed_sha.eq_ignore_ascii_case(expected_sha) {
        bail!(
            "launched executable SHA-256 differs from the manifest: expected {expected_sha}, observed {observed_sha}"
        );
    }
    Ok(json!({
        "pid": launched.pid,
        "executable_path": canonical,
        "executable_sha256": observed_sha,
        "matches_manifest_execution_identity": true,
    }))
}

fn crawl_record(
    driver: &CuaDriver,
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
        driver,
        "start_session",
        &json!({"session": session, "capture_scope": "window"}),
    )?;
    let _session_guard = SessionGuard {
        driver: driver.clone(),
        session: session.clone(),
    };
    let session_state = call(driver, "get_session_state", &json!({"session": session}))?;
    if DriverResponse(&session_state).text("capture_scope") != Some("window") {
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
    let readiness_helper = pinned_readiness_helper()?;
    let readiness = verify_desktop_executable(&bundle_id, manifest, &readiness_helper)?;
    // Cold launch: any pre-existing instance of this bundle is terminated
    // first, because launching an already-running bundle only activates the
    // stale instance and the screenshot would not be evidence of a clean
    // launch (finding 5).
    let pre_existing = terminate_pre_existing(driver, &session, &bundle_id)?;
    let launched = launch(driver, record, &bundle_id, &session)?;
    // Declared after the session guard so it drops first: the app is
    // terminated before the driver session is ended (finding 5).
    let _app_guard = AppGuard {
        driver: driver.clone(),
        session: session.clone(),
        pid: launched.pid,
    };
    if pre_existing.contains(&launched.pid) {
        return Err(anyhow::Error::new(RecordFailure {
            code: "desktop_launch_not_cold",
            message: format!(
                "launched pid {} was already running before the launch; the app was not started cold",
                launched.pid
            ),
        }));
    }
    let executable_proof = launched_executable_proof(&launched, manifest)?;
    let cold_launch_proof = json!({
        "pre_existing_pids": pre_existing,
        "terminated_pre_existing_pids": pre_existing,
        "launched_pid": launched.pid,
        "pid_existed_before_launch": false,
        "launch_response": launched.response,
    });
    let (pid, window_id) = (launched.pid, launched.window_id);
    call(
        driver,
        "start_recording",
        &json!({
            "session": session,
            "output_dir": recording_dir,
            "record_video": false,
        }),
    )?;
    let mut recording_guard = RecordingGuard {
        driver: driver.clone(),
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
        let current = snapshot(driver, &session, pid, window_id, &observation_path)?;
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
        let pre = snapshot(driver, &session, pid, window_id, &pre_image)?;
        assert_target_surface(&pre, pid, window_id, &bundle_id)?;
        let active_owner_before = global_active_owner(driver, &session)?;
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
        let driver_response = match apply(driver, &session, pid, window_id, &action) {
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
        let driver_effect = DriverResponse(&driver_response)
            .text("effect")
            .unwrap_or("missing")
            .to_string();
        let driver_route = DriverResponse(&driver_response)
            .text("route")
            .unwrap_or("missing")
            .to_string();
        let post_image = transitions_dir.join(format!("step-{transition_index:04}-after.png"));
        let post = match snapshot(driver, &session, pid, window_id, &post_image) {
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
        let active_owner_after = global_active_owner(driver, &session)?;
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
        "pinned_driver": {
            "path": driver.path,
            "sha256": driver.sha256,
            "version": driver.version,
        },
        "pinned_readiness_helper": {
            "path": readiness_helper.path,
            "sha256": readiness_helper.sha256,
            "version": readiness_helper.version,
        },
        "cold_launch_proof": cold_launch_proof,
        "launched_executable_proof": executable_proof,
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
    // The absolute path this host executes cargo at, never the bare name.
    // Every worker in this repository is `cargo run --release`, and the job's
    // shell is a non-login `/bin/sh` that reads no profile, so a bare name
    // resolves to nothing however the host installs Rust -- the defect that
    // cost job-545551889f9e88be30daa81f sixteen minutes of a claimed slot in
    // the documentation engine, still open in this one.
    let cargo = super::crawl::resolved_worker_program(request.host)?;
    let command = format!(
        "{cargo} run --release -- crawl-desktop {} --worker --record {} --max-states {} --max-depth {} --artifact-uri {} --runtime-manifest-base64 '{}'",
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
        super::crawl::STADO_REPO_WORKDIR.to_string(),
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

fn worker_report(
    manifest: &super::crawl::RuntimeManifest,
    artifact: Option<Value>,
    failure: Option<Value>,
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
    if let Some(artifact) = artifact.as_ref() {
        if artifact.get("uri").and_then(Value::as_str) != Some(manifest.artifact_uri.as_str()) {
            bail!("published desktop artifact URI does not match the immutable runtime manifest");
        }
    }
    Ok(json!({
        "schema": "wisent.native-worker-report.v1",
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "record_key": manifest.record_key,
        "attempt": attempt,
        "attempt_id": attempt_id,
        "engine": manifest.engine,
        "state": if failure.is_some() { "failed" } else { "artifact_published" },
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "reference_sha256": manifest.reference_sha256,
        "bindings_file_sha256": bindings_file_sha256,
        "bindings_sha256": bindings_sha256,
        "execution_identity": execution_identity,
        "artifact": artifact,
        "failure": failure,
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
    let run_root = attempt_root(
        &output,
        &manifest,
    )?;
    std::fs::create_dir_all(&run_root)?;
    let entry = records(&catalog, Some(&record))?
        .into_iter()
        .next()
        .context("runtime manifest record is absent from catalog")?;
    // Driver pinning and the permission preflight run inside the failure-handled
    // region, so a refused driver still produces a typed failure artifact and a
    // published attempt archive like any other record failure.
    let (record_report, failure) =
        match (|| -> Result<Value> {
            // Resolved, canonicalized, hashed and version-stamped exactly once
            // for this record; every later call runs that same file (finding 10).
            let driver = pin_cua_driver()?;
            preflight(&driver)?;
            crawl_record(&driver, &entry, &manifest, &run_root, max_states, max_depth)
        })() {
            Ok(report) => (report, None),
            Err(error) => {
                let code = failure_code(&error);
                let message = format!("{error:#}");
                // Diagnostics never share stdout with the one worker report line.
                eprintln!("desktop record {} failed: {message}", entry.slug);
                (
                    json!({
                        "record": entry.slug,
                        "name": entry.name,
                        "status": "failed",
                        "source_revision": manifest.source_revision,
                        "source_input_sha256": manifest.source_input_sha256,
                        "runtime_manifest": manifest,
                        "error": message,
                    }),
                    Some((code, message)),
                )
            }
        };
    let summary = json!({
        "schema": "wisent.desktop-crawl-batch.v1",
        "catalog": catalog,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "records": [record_report],
        "failed": usize::from(failure.is_some()),
    });
    std::fs::write(
        run_root.join("batch.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    // A failed record still retains a typed failure artifact and still
    // publishes the attempt archive, so every attempt has exactly one archive
    // and exactly one worker report line.
    if let Some((code, message)) = failure.as_ref() {
        std::fs::write(
            run_root.join("failure.json"),
            serde_json::to_string_pretty(&json!({
                "schema": "wisent.native-worker-failure.v1",
                "code": code,
                "message": message,
                "run_id": manifest.run_id,
                "catalog": manifest.catalog,
                "record": manifest.record,
                "attempt": manifest.attempt,
                "attempt_id": manifest.attempt_id,
                "engine": manifest.engine,
            }))? + "\n",
        )?;
    }
    let artifact = super::crawl::publish_attempt_archive(&run_root, &artifact_uri)?;
    let failure = failure.map(|(code, message)| json!({"code": code, "message": message}));
    let report = worker_report(&manifest, Some(artifact), failure.clone())?;
    println!("{}", serde_json::to_string(&report)?);
    if failure.is_some() {
        bail!("the exact desktop record could not be crawled");
    }
    Ok(())
}
