//! Real iOS/Android application crawler through an Appium device endpoint.
//!
//! The crawler launches the installed application, replays one path per fresh
//! session, records the screen, stores the accessibility source and screenshot,
//! and breadth-first explores actionable controls. A destructive control may be
//! opened once so its confirmation flow is observed, but a second destructive
//! confirmation is retained as a blocked edge rather than committed.

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

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
    product_url: String,
    path: PathBuf,
}

#[derive(Clone, Debug)]
struct InputValue {
    key: String,
    value: String,
}

#[derive(Default)]
struct Fixtures {
    inputs: Vec<FixtureRule>,
}

struct FixtureRule {
    key: String,
    matcher: Regex,
    value: String,
}

#[derive(serde::Deserialize)]
struct FixtureDocument {
    #[serde(default)]
    inputs: Vec<FixtureSpec>,
}

#[derive(serde::Deserialize)]
struct FixtureSpec {
    key: String,
    matcher: String,
    value: Option<String>,
    value_env: Option<String>,
}

impl Fixtures {
    fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        let document: FixtureDocument = serde_json::from_slice(
            &std::fs::read(path)
                .with_context(|| format!("read mobile input fixtures from {}", path.display()))?,
        )
        .with_context(|| format!("parse mobile input fixtures from {}", path.display()))?;
        let mut inputs = Vec::new();
        for fixture in document.inputs {
            let value = match (fixture.value, fixture.value_env) {
                (Some(value), None) => value,
                (None, Some(variable)) => std::env::var(&variable).with_context(|| {
                    format!("mobile input fixture {} needs ${variable}", fixture.key)
                })?,
                _ => bail!(
                    "mobile input fixture {} must set exactly one of value or value_env",
                    fixture.key
                ),
            };
            inputs.push(FixtureRule {
                key: fixture.key,
                matcher: Regex::new(&fixture.matcher)
                    .with_context(|| format!("invalid mobile input matcher {}", fixture.matcher))?,
                value,
            });
        }
        Ok(Self { inputs })
    }

    fn input_for(&self, label: &str) -> Option<InputValue> {
        self.inputs
            .iter()
            .find(|fixture| fixture.matcher.is_match(label))
            .map(|fixture| InputValue {
                key: fixture.key.clone(),
                value: fixture.value.clone(),
            })
    }
}

#[derive(Clone, Debug)]
struct Action {
    selector: String,
    label: String,
    destructive: bool,
    input: Option<InputValue>,
}

#[derive(Clone, Debug)]
struct PathStep {
    selector: String,
    label: String,
    destructive: bool,
    input: Option<InputValue>,
}

#[derive(Clone, Debug)]
struct Pending {
    path: Vec<PathStep>,
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

struct Appium {
    base: String,
    agent: ureq::Agent,
}

impl Appium {
    fn new(base: &str) -> Result<Self> {
        Ok(Self {
            base: canonical_driver_url(base)?,
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(45))
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
        let response = if let Some(body) = body {
            request.send_json(body.clone())
        } else {
            request.call()
        }
        .map_err(|error| anyhow!("Appium {method} {path}: {error}"))?;
        response
            .into_json()
            .map_err(|error| anyhow!("Appium {method} {path} returned invalid JSON: {error}"))
    }

    fn create_session(&self, platform: Platform, app_id: &str) -> Result<String> {
        let mut always = serde_json::Map::new();
        always.insert("platformName".into(), json!(platform.appium_name()));
        always.insert("appium:automationName".into(), json!(platform.automation()));
        always.insert(platform.app_key().into(), json!(app_id));
        always.insert("appium:noReset".into(), json!(true));
        always.insert("appium:newCommandTimeout".into(), json!(180));
        let payload = json!({ "capabilities": { "alwaysMatch": always } });
        let response = self.call("POST", "/session", Some(&payload))?;
        response
            .pointer("/value/sessionId")
            .or_else(|| response.get("sessionId"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium did not return a session id"))
    }

    fn source(&self, session: &str) -> Result<String> {
        self.call("GET", &format!("/session/{session}/source"), None)?
            .get("value")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Appium returned no accessibility source"))
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

    fn start_recording(&self, session: &str) {
        let _ = self.call(
            "POST",
            &format!("/session/{session}/appium/start_recording_screen"),
            Some(&json!({"options": {"timeLimit": 180}})),
        );
    }

    fn stop_recording(&self, session: &str) -> Option<Vec<u8>> {
        let response = self
            .call(
                "POST",
                &format!("/session/{session}/appium/stop_recording_screen"),
                Some(&json!({})),
            )
            .ok()?;
        let encoded = response.get("value")?.as_str()?;
        STANDARD.decode(encoded).ok()
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

    fn click(&self, session: &str, selector: &str) -> Result<()> {
        let element = self.element(session, selector)?;
        self.call(
            "POST",
            &format!("/session/{session}/element/{element}/click"),
            Some(&json!({})),
        )?;
        std::thread::sleep(Duration::from_millis(700));
        Ok(())
    }

    fn input(&self, session: &str, selector: &str, text: &str) -> Result<()> {
        let element = self.element(session, selector)?;
        let _ = self.call(
            "POST",
            &format!("/session/{session}/element/{element}/clear"),
            Some(&json!({})),
        );
        self.call(
            "POST",
            &format!("/session/{session}/element/{element}/value"),
            Some(&json!({
                "text": text,
                "value": text.chars().map(|character| character.to_string()).collect::<Vec<_>>(),
            })),
        )?;
        std::thread::sleep(Duration::from_millis(700));
        Ok(())
    }

    fn restart_app(&self, session: &str, app_id: &str) -> Result<()> {
        let payload = json!({"appId": app_id, "bundleId": app_id});
        let _ = self.call(
            "POST",
            &format!("/session/{session}/appium/device/terminate_app"),
            Some(&payload),
        );
        self.call(
            "POST",
            &format!("/session/{session}/appium/device/activate_app"),
            Some(&payload),
        )?;
        std::thread::sleep(Duration::from_secs(1));
        Ok(())
    }

    fn delete_session(&self, session: &str) {
        let _ = self.call("DELETE", &format!("/session/{session}"), None);
    }
}

fn records(catalog: &str, selected: Option<&str>) -> Result<Vec<Record>> {
    let directory = Path::new(catalog).join("references");
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
            product_url: document
                .get("product_url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            path,
        });
    }
    if found.is_empty() {
        bail!("no matching records in {catalog}");
    }
    Ok(found)
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn ios_bundle_id(record: &Record) -> Result<(String, String)> {
    let query = record.name.replace(" for iOS", "");
    let url = format!(
        "https://itunes.apple.com/search?entity=software&limit=10&term={}",
        url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>()
    );
    let response: Value = ureq::get(&url)
        .timeout(Duration::from_secs(20))
        .call()
        .context("resolve iOS bundle id through Apple's catalog")?
        .into_json()?;
    let wanted = normalized(&query);
    let results = response
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Apple catalog returned no results"))?;
    let mut candidates = results.iter().filter(|entry| {
        let name = entry
            .get("trackName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let actual = normalized(name);
        actual == wanted || actual.starts_with(&wanted) || wanted.starts_with(&actual)
    });
    let candidate = candidates
        .next()
        .ok_or_else(|| anyhow!("Apple catalog has no match for {}", record.name))?;
    if candidates.next().is_some() {
        bail!(
            "Apple catalog has several matches for {}; pass --app-id explicitly",
            record.name
        );
    }
    let bundle = candidate
        .get("bundleId")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Apple catalog match has no bundleId"))?;
    let source = candidate
        .get("trackViewUrl")
        .and_then(Value::as_str)
        .unwrap_or(&url);
    Ok((bundle.to_string(), source.to_string()))
}

fn android_package(record: &Record) -> Result<(String, String)> {
    let parsed = url::Url::parse(&record.product_url).context("Android product_url is invalid")?;
    let package = parsed
        .query_pairs()
        .find(|(key, _)| key == "id")
        .map(|(_, value)| value.into_owned())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("Android product_url carries no package id"))?;
    Ok((package, record.product_url.clone()))
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

fn actions(source: &str, platform: Platform, fixtures: &Fixtures) -> Vec<Action> {
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
                input: editable.then(|| fixtures.input_for(&label)).flatten(),
                selector,
                destructive: destructive.is_match(&label),
                label,
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
                "destructive": step.destructive,
                "kind": if step.input.is_some() { "input" } else { "click" },
                "input_fixture": step.input.as_ref().map(|input| &input.key),
            })).collect::<Vec<_>>(),
            "actions": actions.iter().map(|action| json!({
                "selector": action.selector,
                "label": action.label,
                "destructive": action.destructive,
                "kind": if action.input.is_some() { "input" } else { "click" },
                "input_fixture": action.input.as_ref().map(|input| &input.key),
            })).collect::<Vec<_>>(),
        }))? + "\n",
    )?;
    Ok(())
}

fn crawl_record(
    appium: &Appium,
    platform: Platform,
    record: &Record,
    app_id_override: Option<&str>,
    fixtures: &Fixtures,
    root: &Path,
    max_states: usize,
    max_depth: usize,
) -> Result<Value> {
    let (app_id, identity_source) = if let Some(app_id) = app_id_override {
        (app_id.to_string(), "operator argument".to_string())
    } else {
        match platform {
            Platform::Ios => ios_bundle_id(record)?,
            Platform::Android => android_package(record)?,
        }
    };
    let output = root.join(&record.slug);
    std::fs::create_dir_all(&output)?;
    let mut queue = VecDeque::from([Pending { path: Vec::new() }]);
    let mut states = HashSet::new();
    let mut graph = Vec::new();
    let mut blocked = Vec::new();

    while let Some(pending) = queue.pop_front() {
        if states.len() >= max_states {
            break;
        }
        let session = appium
            .create_session(platform, &app_id)
            .with_context(|| format!("launch {} ({app_id})", record.name))?;
        if let Err(error) = appium.restart_app(&session, &app_id) {
            appium.delete_session(&session);
            return Err(error).with_context(|| format!("restart {} ({app_id})", record.name));
        }
        appium.start_recording(&session);
        let replay = pending.path.iter().try_for_each(|step| {
            let result = match &step.input {
                Some(input) => appium.input(&session, &step.selector, &input.value),
                None => appium.click(&session, &step.selector),
            };
            result.with_context(|| format!("replay {}", step.label))
        });
        if let Err(error) = replay {
            let _ = appium.stop_recording(&session);
            appium.delete_session(&session);
            blocked.push(json!({
                "path": pending.path.iter().map(|step| &step.label).collect::<Vec<_>>(),
                "reason": error.to_string(),
            }));
            continue;
        }
        let capture = appium.source(&session).and_then(|source| {
            let state_id = hash_text(&source);
            if states.contains(&state_id) {
                Ok((source, state_id, None, Vec::new()))
            } else {
                let screenshot = appium.screenshot(&session)?;
                let available = actions(&source, platform, fixtures);
                Ok((source, state_id, Some(screenshot), available))
            }
        });
        let recording = appium.stop_recording(&session);
        appium.delete_session(&session);
        let (source, state_id, screenshot, available) = capture?;
        let Some(screenshot) = screenshot else {
            continue;
        };
        states.insert(state_id.clone());
        let index = states.len();
        write_state(
            &output,
            index,
            &source,
            &screenshot,
            recording.as_deref(),
            &pending.path,
            &available,
        )?;
        graph.push(json!({
            "state": state_id,
            "index": index,
            "depth": pending.path.len(),
            "path": pending.path.iter().map(|step| &step.label).collect::<Vec<_>>(),
            "actions": available.len(),
        }));
        if pending.path.len() >= max_depth {
            continue;
        }
        let entered_destructive_flow = pending.path.iter().any(|step| step.destructive);
        for action in available {
            if entered_destructive_flow {
                blocked.push(json!({
                    "state": state_id,
                    "label": action.label,
                    "selector": action.selector,
                    "reason": "confirmation state retained; no control after a destructive edge is committed",
                }));
                continue;
            }
            let mut path = pending.path.clone();
            path.push(PathStep {
                selector: action.selector,
                label: action.label,
                destructive: action.destructive,
                input: action.input,
            });
            queue.push_back(Pending { path });
        }
    }

    let report = json!({
        "schema": "wisent.mobile-crawl-run.v1",
        "catalog": platform.appium_name(),
        "record": record.slug,
        "name": record.name,
        "record_path": record.path,
        "app_id": app_id,
        "app_identity_source": identity_source,
        "driver_url": appium.base,
        "input_fixtures": fixtures.inputs.len(),
        "states": graph,
        "states_seen": states.len(),
        "blocked_edges": blocked,
        "max_states": max_states,
        "max_depth": max_depth,
        "completed_at": crate::now_iso_utc(),
    });
    std::fs::write(
        output.join("crawl.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}

const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    if value.is_empty()
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | ':')
        })
    {
        bail!("{flag} contains characters that cannot be submitted to a worker");
    }
    Ok(())
}

fn revision() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .context("read Spis source revision")?;
    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success()
        || revision.len() != 40
        || !revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("Spis checkout has no exact Git revision");
    }
    Ok(revision)
}

fn publish_file(path: &Path, uri: &str) -> Result<()> {
    let output = Command::new("stado")
        .args([
            "storage",
            "put",
            "--if-absent",
            "--content-type",
            "application/json",
            uri,
        ])
        .arg(path)
        .output()
        .context("publish mobile crawl fixture")?;
    if !output.status.success() {
        bail!(
            "stado storage put refused mobile crawl fixture: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn publish_artifact(root: &Path, uri: &str) -> Result<()> {
    let archive = root.with_extension("tar.gz");
    let status = Command::new("stado")
        .args(["storage", "archive"])
        .arg(root)
        .arg(&archive)
        .status()
        .context("archive mobile crawl")?;
    if !status.success() {
        bail!("stado storage archive refused mobile crawl artifacts");
    }
    let status = Command::new("stado")
        .args(["storage", "put", "--if-absent", uri])
        .arg(&archive)
        .status()
        .context("publish mobile crawl")?;
    if !status.success() {
        bail!("stado storage put refused mobile crawl artifacts");
    }
    Ok(())
}

struct MobileSubmission<'a> {
    host: &'a str,
    catalog: &'a str,
    record: Option<&'a str>,
    driver_url: &'a str,
    app_id: Option<&'a str>,
    fixtures: Option<&'a Path>,
    secret_env: &'a [String],
    max_states: usize,
    max_depth: usize,
}

fn submit_worker(request: MobileSubmission<'_>) -> Result<()> {
    safe_job_value(request.host, "--host")?;
    safe_job_value(request.catalog, "catalog")?;
    if let Some(record) = request.record {
        safe_job_value(record, "--record")?;
    }
    if let Some(app_id) = request.app_id {
        safe_job_value(app_id, "--app-id")?;
    }
    Appium::new(request.driver_url)?;
    for binding in request.secret_env {
        let (name, item) = binding
            .split_once('=')
            .ok_or_else(|| anyhow!("--secret-env must be NAME=SKARBIEC_ITEM"))?;
        safe_job_value(name, "--secret-env name")?;
        if item.is_empty()
            || !item.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, '-' | '_' | '.' | '#' | '/' | ':')
            })
        {
            bail!("--secret-env item is invalid");
        }
    }
    let revision = revision()?;
    let stamp = crate::now_iso_utc().replace(':', "-");
    let artifact = format!("stado://spis-crawls/{}/{stamp}.tar.gz", request.catalog);
    let mut worker = format!(
        "cargo run --release -- crawl-mobile {} --worker --driver-url {} --max-states {} --max-depth {} --artifact-uri {}",
        request.catalog, request.driver_url, request.max_states, request.max_depth, artifact
    );
    if let Some(record) = request.record {
        worker.push_str(&format!(" --record {record}"));
    }
    if let Some(app_id) = request.app_id {
        worker.push_str(&format!(" --app-id {app_id}"));
    }
    if let Some(fixtures) = request.fixtures {
        let fixture_uri = format!(
            "stado://spis-crawls/{}/fixtures/{stamp}.json",
            request.catalog
        );
        publish_file(fixtures, &fixture_uri)?;
        let remote = format!("$HOME/.stado/work/{stamp}-fixtures.json");
        worker = format!(
            "$HOME/.stado/bin/stado storage get {fixture_uri} {remote} && {worker} --fixtures {remote}"
        );
    }
    let output_uri = format!(
        "stado://spis-crawls/{}/{stamp}/job-output",
        request.catalog
    );
    let mut arguments = vec![
        "submit".to_string(),
        worker,
        "--pinned-host".to_string(),
        request.host.to_string(),
        "--repo".to_string(),
        REPOSITORY.to_string(),
        "--repo-ref".to_string(),
        revision,
        "--repo-workdir".to_string(),
        "spis".to_string(),
        "--repo-extras".to_string(),
        String::new(),
        "--output-uri".to_string(),
        output_uri.clone(),
    ];
    for binding in request.secret_env {
        arguments.push("--secret-env".to_string());
        arguments.push(binding.clone());
    }
    let output = Command::new("stado")
        .args(arguments)
        .output()
        .context("submit mobile crawl through Stado")?;
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
    let mut app_id: Option<String> = None;
    let mut fixtures_path: Option<PathBuf> = None;
    let mut secret_env = Vec::new();
    let mut host: Option<String> = None;
    let mut worker = false;
    let mut artifact_uri: Option<String> = None;
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
            "--app-id" => {
                i += 1;
                app_id = Some(rest.get(i).context("--app-id needs a value")?.clone());
            }
            "--fixtures" => {
                i += 1;
                fixtures_path = Some(PathBuf::from(
                    rest.get(i).context("--fixtures needs a value")?,
                ));
            }
            "--secret-env" => {
                i += 1;
                secret_env.push(rest.get(i).context("--secret-env needs a value")?.clone());
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
                println!("usage: spis crawl-mobile <ios-app-examples|android-app-examples> --host TARGET [--record SLUG] [--driver-url URL] [--app-id ID] [--fixtures FILE] [--secret-env NAME=SKARBIEC_ITEM] [--max-states N] [--max-depth N]\nworker mode: spis crawl-mobile <catalog> --worker [--artifact-uri stado://...]");
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
    if app_id.is_some() && record.is_none() {
        bail!("--app-id requires --record so one identifier cannot launch several records");
    }
    if !worker {
        let host =
            host.context("--host is required; mobile crawls execute as pinned Stado jobs")?;
        return submit_worker(MobileSubmission {
            host: &host,
            catalog: &catalog,
            record: record.as_deref(),
            driver_url: &driver_url,
            app_id: app_id.as_deref(),
            fixtures: fixtures_path.as_deref(),
            secret_env: &secret_env,
            max_states,
            max_depth,
        });
    }
    if host.is_some() || !secret_env.is_empty() {
        bail!("--host and --secret-env are coordinator-only");
    }
    if let Some(uri) = &artifact_uri {
        let namespace = format!("stado://spis-crawls/{catalog}/");
        if !uri.starts_with(&namespace) {
            bail!("--artifact-uri must be under {namespace}");
        }
    }
    let appium = Appium::new(&driver_url)?;
    let fixtures = Fixtures::load(fixtures_path.as_deref())?;
    let run_root = output
        .join(&catalog)
        .join(crate::now_iso_utc().replace(':', "-"));
    std::fs::create_dir_all(&run_root)?;
    let mut reports = Vec::new();
    for entry in records(&catalog, record.as_deref())? {
        match crawl_record(
            &appium,
            platform,
            &entry,
            app_id.as_deref(),
            &fixtures,
            &run_root,
            max_states,
            max_depth,
        ) {
            Ok(report) => reports.push(report),
            Err(error) => reports.push(json!({
                "record": entry.slug,
                "name": entry.name,
                "status": "failed",
                "error": error.to_string(),
            })),
        }
    }
    let failures = reports
        .iter()
        .filter(|report| report.get("status").and_then(Value::as_str) == Some("failed"))
        .count();
    let summary = json!({
        "schema": "wisent.mobile-crawl-batch.v1",
        "catalog": catalog,
        "driver_url": driver_url,
        "input_fixtures": fixtures.inputs.len(),
        "records": reports,
        "failed": failures,
        "completed_at": crate::now_iso_utc(),
    });
    std::fs::write(
        run_root.join("batch.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    if let Some(uri) = &artifact_uri {
        publish_artifact(&run_root, uri)?;
    }
    println!("{}", serde_json::to_string_pretty(&summary)?);
    if failures > 0 {
        bail!("{failures} mobile records could not be crawled");
    }
    Ok(())
}
