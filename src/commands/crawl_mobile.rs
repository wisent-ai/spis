//! Real iOS/Android application crawler through an Appium device endpoint.
//!
//! The crawler validates the exact returned Appium session binding and a fresh
//! read-only runtime-readiness observation, then retains one genuinely
//! sequential trajectory. Every delivered action is bracketed by alert,
//! accessibility-source, and active-owner observations.

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

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
        .unwrap_or("mobile_record_failed")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Platform {
    Ios,
    Android,
}

impl Platform {
    fn from_catalog(catalog: &str) -> Result<Self> {
        match catalog {
            "ios-app-examples" => Ok(Self::Ios),
            "android-app-examples" => Ok(Self::Android),
            _ => bail!("crawl-mobile accepts ios-app-examples or android-app-examples"),
        }
    }

    fn appium_name(self) -> &'static str {
        match self {
            Self::Ios => "iOS",
            Self::Android => "Android",
        }
    }

    fn automation(self) -> &'static str {
        match self {
            Self::Ios => "XCUITest",
            Self::Android => "UiAutomator2",
        }
    }

    fn app_key(self) -> &'static str {
        match self {
            Self::Ios => "appium:bundleId",
            Self::Android => "appium:appPackage",
        }
    }
}

#[derive(Clone, Debug)]
struct Record {
    slug: String,
    name: String,
    path: PathBuf,
}


#[derive(Clone, Debug)]
struct Action {
    selector: String,
    label: String,
    destructive: bool,
    kind: String,
}

#[derive(Clone, Debug, serde::Serialize)]
struct PathStep {
    selector: String,
    label: String,
}


fn canonical_driver_url(value: &str) -> Result<String> {
    let parsed = url::Url::parse(value).context("--driver-url must be a URL")?;
    let local = matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    if parsed.host_str().is_none()
        || (parsed.scheme() != "https" && !(parsed.scheme() == "http" && local))
    {
        bail!("--driver-url must be HTTPS or loopback HTTP");
    }
    if !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !matches!(parsed.path(), "" | "/")
    {
        bail!("--driver-url may contain only scheme, host and port");
    }
    Ok(parsed.as_str().trim_end_matches('/').to_string())
}

/// Per-endpoint response bound in bytes.
fn response_limit(path: &str) -> u64 {
    if path.ends_with("/screenshot") {
        32 * 1024 * 1024
    } else if path.ends_with("/stop_recording_screen") {
        256 * 1024 * 1024
    } else {
        8 * 1024 * 1024
    }
}

struct Appium {
    base: String,
    agent: ureq::Agent,
}

impl Appium {
    fn new(base: &str) -> Result<Self> {
        Ok(Self {
            base: canonical_driver_url(base)?,
            // Zero redirects: an Appium reply may not send this worker, its
            // session id or its screenshots to another origin (finding 12b).
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(45))
                .redirects(0)
                .build(),
        })
    }

    fn call(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{}{}", self.base, path);
        let request = match method {
            "GET" => self.agent.get(&url),
            "POST" => self.agent.post(&url),
            "DELETE" => self.agent.delete(&url),
            _ => bail!("unsupported Appium method {method}"),
        };
        let response = match if let Some(body) = body {
            request.send_json(body.clone())
        } else {
            request.call()
        } {
            Ok(response) => response,
            Err(ureq::Error::Status(status, response)) => {
                let diagnostic = response
                    .into_string()
                    .unwrap_or_else(|error| format!("<unreadable response body: {error}>"));
                bail!("Appium {method} {path} returned HTTP {status}: {diagnostic}");
            }
            Err(ureq::Error::Transport(error)) => {
                bail!("Appium {method} {path} transport failed: {error}");
            }
        };
        if (300..400).contains(&response.status()) {
            bail!(
                "Appium {method} {path} returned redirect HTTP {}; this agent follows no redirects",
                response.status()
            );
        }
        // `into_json` has no length bound, so a screenshot or screen-recording
        // body would be materialized whole inside a Value before any decode
        // could reject it (finding 12).
        let limit = response_limit(path);
        serde_json::from_reader(response.into_reader().take(limit)).map_err(|error| {
            anyhow!(
                "Appium {method} {path} returned invalid JSON within its {limit}-byte response bound: {error}"
            )
        })
    }

    fn create_session(
        &self,
        platform: Platform,
        app_id: &str,
        execution: &super::crawl::RuntimeExecutionIdentity,
    ) -> Result<(String, Value)> {
        let mut always = serde_json::Map::new();
        always.insert("platformName".into(), json!(platform.appium_name()));
        always.insert("appium:automationName".into(), json!(platform.automation()));
        always.insert(platform.app_key().into(), json!(app_id));
        always.insert("appium:noReset".into(), json!(true));
        always.insert("appium:autoGrantPermissions".into(), json!(false));
        always.insert("appium:autoAcceptAlerts".into(), json!(false));
        always.insert("appium:autoDismissAlerts".into(), json!(false));
        always.insert("appium:newCommandTimeout".into(), json!(180));
        let device_id = execution
            .device_id
            .as_deref()
            .context("mobile execution identity has no exact UDID")?;
        always.insert("appium:udid".into(), json!(device_id));
        let payload = json!({ "capabilities": { "alwaysMatch": always } });
        let response = self.call("POST", "/session", Some(&payload))?;
        let session = response
            .pointer("/value/sessionId")
            .or_else(|| response.get("sessionId"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium did not return a session id: {response}"))?;
        let capabilities = response
            .pointer("/value/capabilities")
            .or_else(|| response.get("capabilities"))
            .cloned()
            .ok_or_else(|| anyhow!("Appium did not return session capabilities: {response}"))?;
        Ok((session, capabilities))
    }

    fn source(&self, session: &str) -> Result<String> {
        self.call("GET", &format!("/session/{session}/source"), None)?
            .get("value")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium returned no accessibility source"))
    }

    fn active_app_identity(&self, session: &str) -> Result<String> {
        let response = self.call(
            "POST",
            &format!("/session/{session}/execute/sync"),
            Some(&json!({"script": "mobile: activeAppInfo", "args": []})),
        )?;
        response
            .pointer("/value/bundleId")
            .or_else(|| response.pointer("/value/package"))
            .or_else(|| response.pointer("/value/appPackage"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium activeAppInfo returned no bundle/package identity"))
    }

    fn screenshot(&self, session: &str) -> Result<Vec<u8>> {
        let value = self
            .call("GET", &format!("/session/{session}/screenshot"), None)?
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("Appium returned no screenshot"))?
            .to_string();
        STANDARD
            .decode(value)
            .context("Appium screenshot was not base64")
    }
    fn alert_text(&self, session: &str) -> Result<Option<String>> {
        let path = format!("/session/{session}/alert/text");
        let response = match self.agent.get(&format!("{}{}", self.base, path)).call() {
            Ok(response) => response,
            Err(ureq::Error::Status(status, response)) => {
                let body = response
                    .into_string()
                    .unwrap_or_else(|error| format!("<unreadable response body: {error}>"));
                if serde_json::from_str::<Value>(&body)
                    .ok()
                    .and_then(|value| {
                        value
                            .pointer("/value/error")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                    .as_deref()
                    == Some("no such alert")
                {
                    return Ok(None);
                }
                bail!("Appium GET {path} returned HTTP {status}: {body}");
            }
            Err(ureq::Error::Transport(error)) => {
                bail!("Appium GET {path} transport failed: {error}");
            }
        };
        if (300..400).contains(&response.status()) {
            bail!(
                "Appium GET {path} returned redirect HTTP {}; this agent follows no redirects",
                response.status()
            );
        }
        let limit = response_limit(&path);
        let value: Value = serde_json::from_reader(response.into_reader().take(limit))
            .with_context(|| {
                format!("Appium GET {path} alert text exceeded its {limit}-byte bound or was invalid JSON")
            })?;
        Ok(value
            .get("value")
            .and_then(Value::as_str)
            .map(str::to_string))
    }

    fn start_recording(&self, session: &str) -> Result<()> {
        self.call(
            "POST",
            &format!("/session/{session}/appium/start_recording_screen"),
            Some(&json!({"options": {"timeLimit": 180}})),
        )?;
        Ok(())
    }

    fn stop_recording(&self, session: &str) -> Result<Option<Vec<u8>>> {
        let response = self.call(
            "POST",
            &format!("/session/{session}/appium/stop_recording_screen"),
            Some(&json!({})),
        )?;
        let Some(encoded) = response.get("value").and_then(Value::as_str) else {
            return Ok(None);
        };
        Ok(Some(
            STANDARD
                .decode(encoded)
                .context("Appium screen recording was not base64")?,
        ))
    }

    fn element(&self, session: &str, selector: &str) -> Result<String> {
        let found = self.call(
            "POST",
            &format!("/session/{session}/element"),
            Some(&json!({"using": "xpath", "value": selector})),
        )?;
        let value = found
            .get("value")
            .and_then(Value::as_object)
            .ok_or_else(|| anyhow!("Appium found no element for {selector}"))?;
        value
            .get("element-6066-11e4-a52e-4f735466cecf")
            .or_else(|| value.get("ELEMENT"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium element response had no id"))
    }

    /// Poll the accessibility source until two consecutive reads agree. A fixed
    /// 700ms sleep was the only settling wait before a screenshot, so a slower
    /// transition was captured mid-animation; 700ms is now the floor of a
    /// bounded wait, not the whole wait (finding 19).
    fn settle(&self, session: &str) -> Result<String> {
        let started = Instant::now();
        let mut previous = hash_text(&self.source(session)?);
        loop {
            std::thread::sleep(Duration::from_millis(200));
            let current = self.source(session)?;
            let digest = hash_text(&current);
            if digest == previous && started.elapsed() >= Duration::from_millis(700) {
                return Ok(current);
            }
            if started.elapsed() >= Duration::from_secs(20) {
                return Err(anyhow::Error::new(RecordFailure {
                    code: "mobile_surface_never_settled",
                    message:
                        "the mobile surface produced no two consecutive identical accessibility sources within 20s"
                            .to_string(),
                }));
            }
            previous = digest;
        }
    }

    fn click(&self, session: &str, selector: &str) -> Result<()> {
        let element = self.element(session, selector)?;
        self.call(
            "POST",
            &format!("/session/{session}/element/{element}/click"),
            Some(&json!({})),
        )?;
        self.settle(session)?;
        Ok(())
    }


    fn delete_session(&self, session: &str) {
        let _ = self.call("DELETE", &format!("/session/{session}"), None);
    }
}

fn records(catalog: &str, selected: Option<&str>) -> Result<Vec<Record>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(catalog)
        .join("references");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&directory)
        .with_context(|| format!("read {}", directory.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    entries.sort();
    let mut found = Vec::new();
    for path in entries {
        let slug = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        if selected.is_some_and(|value| {
            value != slug && value != slug.split_once('-').map(|(_, tail)| tail).unwrap_or(&slug)
        }) {
            continue;
        }
        let record_path = path.join("reference.json");
        let document: Value = serde_json::from_slice(&std::fs::read(&record_path)?)?;
        found.push(Record {
            slug,
            name: document
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            path: record_path,
        });
    }
    if found.is_empty() {
        bail!("no matching records in {catalog}");
    }
    Ok(found)
}


pub(crate) fn ios_bundle_id_for(product_url: &str) -> Result<(String, String)> {
    let track = product_url.rsplit("/id").next()
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .ok_or_else(|| anyhow!("{product_url} has no exact /idNNN App Store identity"))?;
    let track_id: u64 = track.parse().context("parse App Store track id")?;
    let url = format!("https://itunes.apple.com/lookup?id={track_id}");
    let cache = Path::new(".wisent-output/product-resolution/ios").join(format!("{track_id}.json"));
    let response: Value = if cache.is_file() {
        serde_json::from_slice(&std::fs::read(&cache)?)?
    } else {
        // Same bounded, redirect-free treatment as every Appium read: no
        // unbounded into_json and no cross-origin redirect (findings 12/12b).
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(20))
            .redirects(0)
            .build();
        let response = agent
            .get(&url)
            .call()
            .context("resolve exact iOS bundle id through Apple's lookup API")?;
        if (300..400).contains(&response.status()) {
            bail!(
                "Apple lookup returned redirect HTTP {}; this agent follows no redirects",
                response.status()
            );
        }
        let limit = response_limit(&url);
        let value: Value = serde_json::from_reader(response.into_reader().take(limit))
            .with_context(|| {
                format!("Apple lookup response exceeded its {limit}-byte bound or was invalid JSON")
            })?;
        super::crawl::atomic_json_write(&cache, &value)?;
        value
    };
    let results = response.get("results").and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Apple lookup response has no results array"))?;
    if response.get("resultCount").and_then(Value::as_u64) != Some(1) || results.len() != 1 {
        bail!("Apple lookup for track id {track_id} did not return exactly one result");
    }
    let candidate = &results[0];
    if candidate.get("trackId").and_then(Value::as_u64) != Some(track_id) {
        bail!("Apple lookup result does not match requested track id {track_id}");
    }
    let bundle = candidate.get("bundleId").and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Apple lookup result has no bundleId"))?;
    Ok((bundle.to_string(), url))
}


fn attribute(tag: &str, name: &str) -> Option<String> {
    let pattern = Regex::new(&format!(r#"\b{}="([^"]*)""#, regex::escape(name))).ok()?;
    pattern
        .captures(tag)
        .and_then(|capture| capture.get(1))
        .map(|value| value.as_str().replace("&quot;", "\"").replace("&amp;", "&"))
}

fn xpath_literal(value: &str) -> Option<String> {
    if value.is_empty() || value.contains('\'') || value.contains('\n') || value.len() > 200 {
        return None;
    }
    Some(format!("'{value}'"))
}

fn system_owner(platform: Platform, identity: &str) -> bool {
    match platform {
        Platform::Android => [
            "com.android.permissioncontroller",
            "com.google.android.permissioncontroller",
            "com.android.systemui",
            "com.android.packageinstaller",
        ]
        .iter()
        .any(|owner| identity == *owner || identity.starts_with(&format!("{owner}."))),
        Platform::Ios => [
            "com.apple.springboard",
            "com.apple.Preferences",
            "com.apple.UserNotificationsUI",
            "com.apple.CoreAuthUI",
        ]
        .iter()
        .any(|owner| identity == *owner || identity.starts_with(&format!("{owner}."))),
    }
}

#[derive(serde::Serialize)]
struct SurfaceObservation {
    active_owner_before: String,
    source: String,
    alert_text: Option<String>,
    active_owner_after: String,
}

impl SurfaceObservation {
    fn refusal_reason(&self, platform: Platform, expected: &str) -> Option<String> {
        for observed in [&self.active_owner_before, &self.active_owner_after] {
            if observed != expected {
                return Some(if system_owner(platform, observed) {
                    format!(
                        "system-owned surface {observed} replaced exact app {expected}; further input withheld"
                    )
                } else {
                    format!(
                        "other-owner surface {observed} replaced exact app {expected}; further input withheld"
                    )
                });
            }
        }
        self.alert_text.as_ref().map(|text| {
            format!(
                "Appium reported an alert surface with exact text {text:?}; it was not classified or clicked"
            )
        })
    }
}

fn inspect_surface(appium: &Appium, session: &str) -> Result<SurfaceObservation> {
    let active_owner_before = appium.active_app_identity(session)?;
    let source = appium.source(session)?;
    let alert_text = appium.alert_text(session)?;
    let active_owner_after = appium.active_app_identity(session)?;
    Ok(SurfaceObservation {
        active_owner_before,
        source,
        alert_text,
        active_owner_after,
    })
}
fn exact_surface_screenshot(
    appium: &Appium,
    session: &str,
    platform: Platform,
    expected_owner: &str,
) -> Result<(Vec<u8>, String, String)> {
    // The screenshot is only taken from a settled surface (finding 19).
    appium.settle(session)?;
    let active_owner_before = appium.active_app_identity(session)?;
    if active_owner_before != expected_owner {
        let owner_class = if system_owner(platform, &active_owner_before) {
            "system-owned"
        } else {
            "other-owner"
        };
        bail!(
            "{owner_class} surface {active_owner_before:?} was active immediately before the state screenshot, not exact app {expected_owner:?}; screenshot withheld"
        );
    }
    let screenshot = appium.screenshot(session)?;
    let active_owner_after = appium.active_app_identity(session)?;
    if active_owner_after != expected_owner {
        let owner_class = if system_owner(platform, &active_owner_after) {
            "system-owned"
        } else {
            "other-owner"
        };
        bail!(
            "{owner_class} surface {active_owner_after:?} was active immediately after the state screenshot, not exact app {expected_owner:?}; screenshot rejected"
        );
    }
    Ok((screenshot, active_owner_before, active_owner_after))
}


fn returned_capability<'a>(
    capabilities: &'a Value,
    prefixed: &str,
    alias: &str,
) -> Result<&'a str> {
    let prefixed_value = capabilities.get(prefixed).and_then(Value::as_str);
    let alias_value = capabilities.get(alias).and_then(Value::as_str);
    if let (Some(left), Some(right)) = (prefixed_value, alias_value) {
        if left != right {
            bail!(
                "Appium returned conflicting scalar capabilities {prefixed}={left:?} and {alias}={right:?}"
            );
        }
    }
    prefixed_value
        .or(alias_value)
        .ok_or_else(|| anyhow!("Appium returned neither {prefixed} nor {alias}: {capabilities}"))
}

fn verify_session_capabilities(
    capabilities: &Value,
    platform: Platform,
    app_id: &str,
    execution: &super::crawl::RuntimeExecutionIdentity,
) -> Result<()> {
    let platform_name = capabilities
        .get("platformName")
        .and_then(Value::as_str)
        .context("Appium returned no scalar platformName capability")?;
    if platform_name != platform.appium_name() {
        bail!(
            "Appium session platform differs: expected {:?}, observed {platform_name:?}",
            platform.appium_name()
        );
    }
    let expected_udid = execution
        .device_id
        .as_deref()
        .context("mobile execution identity has no exact UDID")?;
    let observed_udid = returned_capability(capabilities, "appium:udid", "udid")?;
    if observed_udid != expected_udid {
        bail!(
            "Appium session UDID differs: expected {expected_udid:?}, observed {observed_udid:?}"
        );
    }
    let (prefixed, alias) = match platform {
        Platform::Ios => ("appium:bundleId", "bundleId"),
        Platform::Android => ("appium:appPackage", "appPackage"),
    };
    let observed_app = returned_capability(capabilities, prefixed, alias)?;
    if observed_app != app_id {
        bail!(
            "Appium session app identity differs: expected {app_id:?}, observed {observed_app:?}"
        );
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("open {}", path.display()))?;
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

/// One helper binary, pinned exactly once per record.
#[derive(Clone, Debug)]
struct PinnedHelper {
    path: PathBuf,
    sha256: String,
    version: String,
}

/// Resolve the readiness helper from absolute directories only. The inherited
/// PATH never participates, the path must be canonical and must not be a
/// symlink, and its digest and version are retained, so no writable earlier
/// PATH entry can substitute the binary that observes device readiness
/// (finding 10).
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

fn readiness_observation(
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

fn action_block_reason(action: &Action) -> Option<&'static str> {
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

fn potential_consent_trigger(label: &str) -> bool {
    Regex::new(
        r"(?i)\b(allow while using|allow once|don'?t allow|enable (push )?notifications|turn on notifications|grant permission|camera access|microphone access|location access)\b",
    )
    .expect("static consent regex")
    .is_match(label)
}

fn actions(source: &str, platform: Platform) -> Vec<Action> {
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

fn hash_text(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    hex::encode(digest.finalize())
}

fn write_state(
    directory: &Path,
    index: usize,
    source: &str,
    screenshot: &[u8],
    video: Option<&[u8]>,
    path: &[PathStep],
    actions: &[Action],
) -> Result<()> {
    let state = directory.join(format!("state-{index:04}"));
    std::fs::create_dir_all(&state)?;
    std::fs::write(state.join("source.xml"), source)?;
    std::fs::write(state.join("screenshot.png"), screenshot)?;
    if let Some(video) = video.filter(|video| !video.is_empty()) {
        std::fs::write(state.join("trajectory.mp4"), video)?;
    }
    std::fs::write(
        state.join("state.json"),
        serde_json::to_string_pretty(&json!({
            "schema": "wisent.mobile-crawl-state.v1",
            "source_sha256": hash_text(source),
            "path": path.iter().map(|step| json!({
                "selector": step.selector,
                "label": step.label,
                "kind": "click",
            })).collect::<Vec<_>>(),
            "actions": actions.iter().map(|action| json!({
                "selector": action.selector,
                "label": action.label,
                "destructive": action.destructive,
                "kind": "click",
            })).collect::<Vec<_>>(),
        }))? + "\n",
    )?;
    Ok(())
}

fn crawl_record(
    appium: &Appium,
    platform: Platform,
    record: &Record,
    manifest: &super::crawl::RuntimeManifest,
    root: &Path,
    max_states: usize,
    max_depth: usize,
) -> Result<Value> {
    let app_id = manifest.runtime_product.identifier.clone();
    let identity_source = manifest.runtime_product.identity_source.clone();
    let execution = manifest
        .execution_identity
        .as_ref()
        .context("mobile runtime manifest has no exact device identity")?;
    // Pinned exactly once per record and reused by both readiness observations.
    let readiness_helper = pinned_readiness_helper()?;
    let readiness_before = readiness_observation(manifest, &app_id, &readiness_helper)?;
    let output = root.join(&record.slug);
    std::fs::create_dir_all(&output)?;

    let (session, capabilities) = appium
        .create_session(platform, &app_id, execution)
        .with_context(|| format!("launch {} ({app_id})", record.name))?;
    let result = (|| -> Result<Value> {
        verify_session_capabilities(&capabilities, platform, &app_id, execution)?;
        let readiness_after = readiness_observation(manifest, &app_id, &readiness_helper)?;
        appium.start_recording(&session)?;

        let mut states = HashSet::new();
        let mut attempted = HashSet::<String>::new();
        let mut reported_gaps = HashSet::<String>::new();
        let mut trajectory = Vec::<PathStep>::new();
        let mut graph = Vec::new();
        let mut transitions = Vec::new();
        let mut blocked = Vec::new();
        let mut surface_observations = 0usize;

        for action_index in 0..=max_depth {
            if states.len() >= max_states {
                blocked.push(json!({
                    "reason": "state limit reached on the single observed trajectory; unexplored branches remain explicit gaps",
                    "max_states": max_states,
                }));
                break;
            }
            let current = inspect_surface(appium, &session)?;
            surface_observations += 1;
            if let Some(reason) = current.refusal_reason(platform, &app_id) {
                blocked.push(json!({
                    "delivered_input": Value::Null,
                    "observed_surface": current,
                    "reason": reason,
                    "further_input_withheld": true,
                }));
                break;
            }
            let state_id = hash_text(&current.source);
            let available = actions(&current.source, platform);
            if states.insert(state_id.clone()) {
                let index = states.len();
                let (screenshot, screenshot_owner_before, screenshot_owner_after) =
                    exact_surface_screenshot(appium, &session, platform, &app_id)?;
                write_state(
                    &output,
                    index,
                    &current.source,
                    &screenshot,
                    None,
                    &trajectory,
                    &available,
                )?;
                graph.push(json!({
                    "state": state_id,
                    "index": index,
                    "trajectory_depth": trajectory.len(),
                    "delivered_inputs": trajectory,
                    "observed_state": {
                        "active_owner_before": current.active_owner_before,
                        "active_owner_after": current.active_owner_after,
                        "screenshot_active_owner_before": screenshot_owner_before,
                        "screenshot_active_owner_after": screenshot_owner_after,
                        "alert_text": current.alert_text,
                        "source": format!("state-{index:04}/source.xml"),
                        "screenshot": format!("state-{index:04}/screenshot.png"),
                    },
                    "available_actions": available.iter().map(|action| json!({
                        "selector": action.selector,
                        "label": action.label,
                        "kind": action.kind,
                        "destructive": action.destructive,
                        "withheld_reason": action_block_reason(action),
                    })).collect::<Vec<_>>(),
                }));
            }

            let mut safe = Vec::new();
            for action in available {
                let identity = format!("{state_id}|{}|{}", action.selector, action.label);
                if let Some(reason) = action_block_reason(&action) {
                    if reported_gaps.insert(identity) {
                        blocked.push(json!({
                            "state": state_id,
                            "selector": action.selector,
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
                        "state": state_id,
                        "selector": action.selector,
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

            let before = inspect_surface(appium, &session)?;
            surface_observations += 1;
            if let Some(reason) = before.refusal_reason(platform, &app_id) {
                blocked.push(json!({
                    "state": state_id,
                    "delivered_input": Value::Null,
                    "observed_surface": before,
                    "reason": reason,
                    "further_input_withheld": true,
                }));
                break;
            }
            let Some(action) = actions(&before.source, platform)
                .into_iter()
                .find(|action| action.selector == selected.selector && action.label == selected.label)
            else {
                blocked.push(json!({
                    "state": state_id,
                    "selector": selected.selector,
                    "label": selected.label,
                    "delivered_input": Value::Null,
                    "observed_state_change": Value::Null,
                    "reason": "fresh pre-action source no longer exposed the selected control",
                }));
                continue;
            };
            if let Some(reason) = action_block_reason(&action) {
                blocked.push(json!({
                    "state": state_id,
                    "selector": action.selector,
                    "label": action.label,
                    "delivered_input": Value::Null,
                    "observed_state_change": Value::Null,
                    "reason": reason,
                }));
                continue;
            }
            let before_hash = hash_text(&before.source);
            appium
                .click(&session, &action.selector)
                .with_context(|| format!("deliver independently safe mobile action {:?}", action.label))?;
            let after = inspect_surface(appium, &session)?;
            surface_observations += 1;
            let after_hash = hash_text(&after.source);
            let changed = before_hash != after_hash;
            transitions.push(json!({
                "step": transitions.len() + 1,
                "delivered_input": {
                    "kind": "click",
                    "selector": action.selector,
                    "label": action.label,
                    "driver_acknowledged": true,
                },
                "observed_state_change": {
                    "changed": changed,
                    "before_source_sha256": before_hash,
                    "after_source_sha256": after_hash,
                    "active_owner_before_source": before.active_owner_before,
                    "active_owner_after_source": before.active_owner_after,
                    "active_owner_before_post_source": after.active_owner_before,
                    "active_owner_after_post_source": after.active_owner_after,
                    "alert_before": before.alert_text,
                    "alert_after": after.alert_text,
                },
            }));
            trajectory.push(PathStep {
                selector: action.selector.clone(),
                label: action.label.clone(),
            });
            if let Some(reason) = after.refusal_reason(platform, &app_id) {
                blocked.push(json!({
                    "state": after_hash,
                    "delivered_input": Value::Null,
                    "observed_surface": after,
                    "reason": reason,
                    "further_input_withheld": true,
                }));
                break;
            }
            if changed {
                for (_, alternative) in safe.into_iter().skip(1) {
                    blocked.push(json!({
                        "state": before_hash,
                        "selector": alternative.selector,
                        "label": alternative.label,
                        "delivered_input": Value::Null,
                        "observed_state_change": Value::Null,
                        "reason": "single sequential trajectory took a different safe edge; this branch was not reset or inferred",
                    }));
                }
            }
        }

        let recording = appium.stop_recording(&session)?;
        let recording_path = recording
            .as_deref()
            .filter(|bytes| !bytes.is_empty())
            .map(|bytes| {
                let path = output.join("trajectory.mp4");
                std::fs::write(&path, bytes)?;
                Ok::<_, anyhow::Error>(path)
            })
            .transpose()?;
        let report = json!({
            "schema": "wisent.mobile-crawl-run.v1",
            "catalog": platform.appium_name(),
            "record": record.slug,
            "name": record.name,
            "record_path": record.path,
            "app_id": app_id,
            "app_identity_source": identity_source,
            "source_revision": manifest.source_revision,
            "source_input_sha256": manifest.source_input_sha256,
            "runtime_manifest": manifest,
            "runtime_execution_identity": execution,
            "runtime_readiness_before_appium_launch": readiness_before,
            "runtime_readiness_after_capability_verification": readiness_after,
            "pinned_readiness_helper": {
                "path": readiness_helper.path,
                "sha256": readiness_helper.sha256,
                "version": readiness_helper.version,
            },
            "appium_session_capabilities": capabilities,
            "driver_url": appium.base,
            "surface_observations": surface_observations,
            "states": graph,
            "states_seen": states.len(),
            "transitions": transitions,
            "blocked_edges": blocked,
            "max_states": max_states,
            "max_depth": max_depth,
            "evidence_observations": {
                "executed_trajectory": trajectory,
                "accessibility_artifacts": graph.iter().filter_map(|state| state.pointer("/observed_state/source").cloned()).collect::<Vec<_>>(),
                "motion_artifacts": recording_path.iter().map(|path| path.strip_prefix(&output).unwrap_or(path)).collect::<Vec<_>>(),
                "canonical_interactions": [],
                "canonical_journey": Value::Null,
                "canonical_accessibility": Value::Null,
                "canonical_motion_analysis": Value::Null,
                "gaps": [
                    "Only one genuinely sequential observed trajectory was retained; alternative branches were not reset or inferred.",
                    "Destructive, input, permission-like, notification-like, and ambiguous controls were withheld before delivery.",
                    "Alert/source/owner observations are retained as observations; no hard-coded claim that a system dialog was absent is emitted."
                ]
            },
        });
        std::fs::write(
            output.join("crawl.json"),
            serde_json::to_string_pretty(&report)? + "\n",
        )?;
        Ok(report)
    })();
    appium.delete_session(&session);
    result
}

const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | ':')
        })
    {
        bail!("{flag} contains characters that cannot be submitted to a worker");
    }
    Ok(())
}
fn attempt_root(
    base: &Path,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<PathBuf> {
    super::crawl::native_attempt_root(base, manifest)
}


fn revision() -> Result<String> { super::crawl::build_revision() }


fn worker_report(
    manifest: &super::crawl::RuntimeManifest,
    artifact: Option<Value>,
    failure: Option<Value>,
) -> Result<Value> {
    let value = serde_json::to_value(manifest)?;
    let attempt = value
        .get("attempt")
        .and_then(Value::as_u64)
        .context("mobile runtime manifest has no attempt for worker report")?;
    let attempt_id = value
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("mobile runtime manifest has no attempt_id for worker report")?;
    let bindings_file_sha256 = value
        .get("bindings_file_sha256")
        .and_then(Value::as_str)
        .context("mobile runtime manifest has no bindings_file_sha256")?;
    let bindings_sha256 = value
        .get("bindings_sha256")
        .and_then(Value::as_str)
        .context("mobile runtime manifest has no bindings_sha256")?;
    let execution_identity = value
        .get("execution_identity")
        .filter(|identity| identity.is_object())
        .context("mobile runtime manifest has no typed execution_identity")?
        .clone();
    if let Some(artifact) = artifact.as_ref() {
        if artifact.get("uri").and_then(Value::as_str) != Some(manifest.artifact_uri.as_str()) {
            bail!("published mobile artifact URI does not match the immutable runtime manifest");
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

struct MobileSubmission<'a> {
    host: &'a str,
    catalog: &'a str,
    record: &'a str,
    driver_url: &'a str,
    max_states: usize,
    max_depth: usize,
    manifest: &'a super::crawl::RuntimeManifest,
}

fn submit_worker(request: MobileSubmission<'_>) -> Result<()> {
    safe_job_value(request.host, "--host")?;
    safe_job_value(request.catalog, "catalog")?;
    safe_job_value(request.record, "--record")?;
    Appium::new(request.driver_url)?;
    let _attempt_binding = attempt_root(Path::new("."), request.manifest)?;
    if revision()? != request.manifest.source_revision {
        bail!("mobile coordinator revision does not match immutable runtime manifest");
    }
    let encoded = request.manifest.encoded()?;
    let artifact = request.manifest.artifact_uri.clone();
    let output_uri = request.manifest.output_uri.clone();
    let worker = format!(
        "cargo run --release -- crawl-mobile {} --worker --record {} --driver-url {} --max-states {} --max-depth {} --artifact-uri {} --runtime-manifest-base64 '{}'",
        request.catalog,
        request.record,
        request.driver_url,
        request.max_states,
        request.max_depth,
        artifact,
        encoded,
    );
    let arguments = vec![
        "submit".to_string(),
        worker,
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
        "submit mobile crawl through Stado",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "Stado refused mobile crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    super::crawl::print_submission(
        request.catalog,
        "mobile",
        request.host,
        Some(&artifact),
        &output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut record: Option<String> = None;
    let mut driver_url = "http://127.0.0.1:4723".to_string();
    let mut host: Option<String> = None;
    let mut worker = false;
    let mut artifact_uri: Option<String> = None;
    let mut runtime_manifest_base64: Option<String> = None;
    let mut max_states = 200usize;
    let mut max_depth = 8usize;
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
            "--driver-url" => {
                i += 1;
                driver_url = rest.get(i).context("--driver-url needs a value")?.clone();
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
                println!("usage: spis crawl-mobile <ios-app-examples|android-app-examples> --host TARGET --record SLUG --runtime-manifest-base64 DATA [--driver-url URL] [--max-states N] [--max-depth N]\nworker mode requires the same immutable runtime manifest and exact record.");
                return Ok(());
            }
            value if value.starts_with('-') => bail!("unknown argument: {value}"),
            value if catalog.is_none() => catalog = Some(value.to_string()),
            value => bail!("unexpected argument: {value}"),
        }
        i += 1;
    }
    let catalog = catalog.context("catalog is required")?;
    let platform = Platform::from_catalog(&catalog)?;
    driver_url = canonical_driver_url(&driver_url)?;
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
        "mobile",
        Some(&record),
    )?;
    if !worker {
        let host =
            host.context("--host is required; mobile crawls execute as pinned Stado jobs")?;
        return submit_worker(MobileSubmission {
            host: &host,
            catalog: &catalog,
            record: &record,
            driver_url: &driver_url,
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
    let appium = Appium::new(&driver_url)?;
    let run_root = attempt_root(
        &output,
        &manifest,
    )?;
    std::fs::create_dir_all(&run_root)?;
    let entry = records(&catalog, Some(&record))?
        .into_iter()
        .next()
        .context("runtime manifest record is absent from catalog")?;
    let (record_report, failure) = match crawl_record(
        &appium,
        platform,
        &entry,
        &manifest,
        &run_root,
        max_states,
        max_depth,
    ) {
        Ok(report) => (report, None),
        Err(error) => {
            let code = failure_code(&error);
            let message = format!("{error:#}");
            // Diagnostics never share stdout with the one worker report line.
            eprintln!("mobile record {} failed: {message}", entry.slug);
            (
                json!({
                    "record": entry.slug,
                    "name": entry.name,
                    "status": "failed",
                    "source_revision": manifest.source_revision,
                    "source_input_sha256": manifest.source_input_sha256,
                    "runtime_manifest": manifest,
                    "no_consent_diagnostic": message,
                    "error": message,
                }),
                Some((code, message)),
            )
        }
    };
    let summary = json!({
        "schema": "wisent.mobile-crawl-batch.v1",
        "catalog": catalog,
        "driver_url": driver_url,
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
        bail!("the exact mobile record could not be crawled");
    }
    Ok(())
}
