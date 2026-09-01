//! Real native desktop application crawler through Cua Driver.
//!
//! Runs on the Stado-selected desktop host. Every action is based on a fresh
//! accessibility snapshot, is addressed with that snapshot's element token,
//! and is followed by another snapshot. Paths are replayed with fresh tokens;
//! screenshots, trees and action recordings are retained per discovered state.

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

#[derive(Clone, Debug)]
struct Record {
    slug: String,
    name: String,
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
                .with_context(|| format!("read desktop input fixtures from {}", path.display()))?,
        )
        .with_context(|| format!("parse desktop input fixtures from {}", path.display()))?;
        let mut inputs = Vec::new();
        for fixture in document.inputs {
            let value = match (fixture.value, fixture.value_env) {
                (Some(value), None) => value,
                (None, Some(variable)) => std::env::var(&variable).with_context(|| {
                    format!("desktop input fixture {} needs ${variable}", fixture.key)
                })?,
                _ => bail!(
                    "desktop input fixture {} must set exactly one of value or value_env",
                    fixture.key
                ),
            };
            inputs.push(FixtureRule {
                key: fixture.key,
                matcher: Regex::new(&fixture.matcher).with_context(|| {
                    format!("invalid desktop input matcher {}", fixture.matcher)
                })?,
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

#[derive(Clone, Debug, serde::Serialize)]
struct Step {
    role: String,
    label: String,
    destructive: bool,
    input_fixture: Option<String>,
}

#[derive(Clone, Debug)]
struct Action {
    role: String,
    label: String,
    token: String,
    destructive: bool,
    input: Option<InputValue>,
}

fn call(tool: &str, payload: &Value) -> Result<Value> {
    let output = Command::new("cua-driver")
        .arg(tool)
        .arg(serde_json::to_string(payload)?)
        .output()
        .with_context(|| format!("start cua-driver {tool}"))?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        bail!("cua-driver {tool} failed: {}", error.trim());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .rev()
        .filter_map(|line| serde_json::from_str(line).ok())
        .next()
        .or_else(|| serde_json::from_str(&text).ok())
        .ok_or_else(|| anyhow!("cua-driver {tool} returned no JSON"))
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
    active: bool,
}

impl RecordingGuard {
    fn stop(&mut self) -> Option<Value> {
        if !self.active {
            return None;
        }
        self.active = false;
        call("stop_recording", &json!({})).ok()
    }
}

impl Drop for RecordingGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = call("stop_recording", &json!({}));
        }
    }
}

fn find_i64(value: &Value, key: &str) -> Option<i64> {
    match value {
        Value::Object(map) => map
            .get(key)
            .and_then(Value::as_i64)
            .or_else(|| map.values().find_map(|value| find_i64(value, key))),
        Value::Array(values) => values.iter().find_map(|value| find_i64(value, key)),
        _ => None,
    }
}

fn find_elements(value: &Value) -> Option<&Vec<Value>> {
    match value {
        Value::Object(map) => map
            .get("elements")
            .and_then(Value::as_array)
            .or_else(|| map.values().find_map(find_elements)),
        Value::Array(values) => values.iter().find_map(find_elements),
        _ => None,
    }
}

fn find_bool(value: &Value, key: &str) -> Option<bool> {
    match value {
        Value::Object(map) => map
            .get(key)
            .and_then(Value::as_bool)
            .or_else(|| map.values().find_map(|value| find_bool(value, key))),
        Value::Array(values) => values.iter().find_map(|value| find_bool(value, key)),
        _ => None,
    }
}

fn preflight() -> Result<()> {
    if std::env::consts::OS != "macos" {
        return Ok(());
    }
    let output = Command::new("cua-driver")
        .args(["permissions", "status", "--json"])
        .output()
        .context("read Cua Driver permission status")?;
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
    let directory = Path::new(catalog).join("references");
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
    call(
        "get_window_state",
        &json!({
            "session": session,
            "pid": pid,
            "window_id": window_id,
            "screenshot_out_file": screenshot,
            "max_elements": 4000,
            "max_depth": 40,
        }),
    )
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn actions(snapshot: &Value, fixtures: &Fixtures) -> Vec<Action> {
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
            let editable = matches!(
                role.as_str(),
                "textfield" | "textarea" | "searchfield" | "securetextfield"
            );
            found.push(Action {
                role,
                destructive: destructive.is_match(&label),
                input: editable.then(|| fixtures.input_for(&label)).flatten(),
                label,
                token,
            });
        }
    }
    found
}

fn matching_action(snapshot: &Value, step: &Step, fixtures: &Fixtures) -> Option<Action> {
    actions(snapshot, fixtures).into_iter().find(|action| {
        action.role == step.role
            && normalize(&action.label) == normalize(&step.label)
            && action.input.as_ref().map(|input| &input.key) == step.input_fixture.as_ref()
    })
}

fn apply(session: &str, pid: i64, window_id: i64, action: &Action) -> Result<()> {
    let (tool, payload) = match &action.input {
        Some(input) => (
            "type_text",
            json!({
                "session": session,
                "pid": pid,
                "window_id": window_id,
                "element_token": action.token,
                "text": input.value,
                "delivery_mode": "background",
            }),
        ),
        None => (
            "click",
            json!({
                "session": session,
                "pid": pid,
                "window_id": window_id,
                "element_token": action.token,
                "delivery_mode": "background",
            }),
        ),
    };
    call(tool, &payload)?;
    Ok(())
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

fn launch(record: &Record, session: &str) -> Result<(i64, i64)> {
    let launched = call(
        "launch_app",
        &json!({
            "name": record.name,
            "creates_new_application_instance": true,
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
    fixtures: &Fixtures,
    root: &Path,
    max_states: usize,
    max_depth: usize,
) -> Result<Value> {
    let session = format!("spis-{}", record.slug.replace('_', "-"));
    call(
        "start_session",
        &json!({"session": session, "capture_scope": "window"}),
    )?;
    let _session_guard = SessionGuard {
        session: session.clone(),
    };
    let output = root.join(&record.slug);
    std::fs::create_dir_all(&output)?;
    let mut queue = VecDeque::from([Vec::<Step>::new()]);
    let mut seen_states = HashSet::new();
    let mut graph = Vec::new();
    let mut blocked = Vec::new();
    let mut attempts = 0usize;

    while let Some(path) = queue.pop_front() {
        if seen_states.len() >= max_states {
            break;
        }
        attempts += 1;
        let (pid, window_id) = match launch(record, &session) {
            Ok(value) => value,
            Err(error) => {
                blocked.push(json!({"path": path, "reason": error.to_string()}));
                break;
            }
        };
        let recording_dir = output.join("attempts").join(format!("{attempts:04}"));
        std::fs::create_dir_all(&recording_dir)?;
        // Cua recordings retain full action arguments. Never record a replay
        // path that enters a fixture value; state screenshots remain, while
        // credentials stay out of action.json and the video.
        let record_actions = path.iter().all(|step| step.input_fixture.is_none());
        if record_actions {
            call(
                "start_recording",
                &json!({
                    "output_dir": recording_dir,
                    "record_video": true,
                }),
            )?;
        }
        let mut recording_guard = RecordingGuard {
            active: record_actions,
        };
        let scratch = output.join("replay.png");
        let mut replay_failed = None;
        for step in &path {
            let current = snapshot(&session, pid, window_id, &scratch)?;
            let Some(action) = matching_action(&current, step, fixtures) else {
                replay_failed = Some(format!("could not replay {} {}", step.role, step.label));
                break;
            };
            if let Err(error) = apply(&session, pid, window_id, &action) {
                replay_failed = Some(error.to_string());
                break;
            }
            if let Err(error) = snapshot(&session, pid, window_id, &scratch) {
                replay_failed = Some(format!("verify {} {}: {error}", step.role, step.label));
                break;
            }
        }
        if let Some(reason) = replay_failed {
            blocked.push(json!({"path": path, "reason": reason}));
            continue;
        }
        let index = seen_states.len() + 1;
        let state_dir = output.join(format!("state-{index:04}"));
        std::fs::create_dir_all(&state_dir)?;
        let current = snapshot(&session, pid, window_id, &state_dir.join("screenshot.png"))?;
        let hash = state_hash(&current);
        if !seen_states.insert(hash.clone()) {
            continue;
        }
        let available = actions(&current, fixtures);
        std::fs::write(
            state_dir.join("snapshot.json"),
            serde_json::to_string_pretty(&current)? + "\n",
        )?;
        let recording = recording_guard.stop();
        if let Some(recording) = recording {
            std::fs::write(
                state_dir.join("recording.json"),
                serde_json::to_string_pretty(&recording)? + "\n",
            )?;
        }
        graph.push(json!({
            "state": hash,
            "index": index,
            "depth": path.len(),
            "path": path,
            "actions": available.iter().map(|action| json!({
                "role": action.role,
                "label": action.label,
                "destructive": action.destructive,
                "kind": if action.input.is_some() { "input" } else { "click" },
                "input_fixture": action.input.as_ref().map(|input| &input.key),
            })).collect::<Vec<_>>(),
        }));
        if path.len() < max_depth {
            let entered_destructive_flow = path.iter().any(|step| step.destructive);
            for action in available {
                if entered_destructive_flow {
                    blocked.push(json!({
                        "state": hash,
                        "role": action.role,
                        "label": action.label,
                        "reason": "confirmation state retained; no control after a destructive edge is committed",
                    }));
                    continue;
                }
                let mut next = path.clone();
                next.push(Step {
                    role: action.role,
                    label: action.label,
                    destructive: action.destructive,
                    input_fixture: action.input.as_ref().map(|input| input.key.clone()),
                });
                queue.push_back(next);
            }
        }
    }
    let report = json!({
        "schema": "wisent.desktop-crawl-run.v1",
        "record": record.slug,
        "name": record.name,
        "driver": "cua-driver",
        "states": graph,
        "states_seen": seen_states.len(),
        "blocked_edges": blocked,
        "max_states": max_states,
        "input_fixtures": fixtures.inputs.len(),
        "max_depth": max_depth,
        "completed_at": crate::now_iso_utc(),
    });
    std::fs::write(
        output.join("crawl.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
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

fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        bail!("{flag} must contain only letters, digits, '-' or '_'");
    }
    Ok(())
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
        .context("publish desktop crawl fixture")?;
    if !output.status.success() {
        bail!(
            "stado storage put refused desktop crawl fixture: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

struct DesktopSubmission<'a> {
    host: &'a str,
    catalog: &'a str,
    record: Option<&'a str>,
    fixtures: Option<&'a Path>,
    secret_env: &'a [String],
    max_states: usize,
    max_depth: usize,
}

fn submit_worker(request: DesktopSubmission<'_>) -> Result<()> {
    safe_job_value(request.host, "--host")?;
    safe_job_value(request.catalog, "catalog")?;
    if let Some(record) = request.record {
        safe_job_value(record, "--record")?;
    }
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
    let mut command = format!(
        "cargo run --release -- crawl-desktop {} --worker --max-states {} --max-depth {} --artifact-uri {}",
        request.catalog, request.max_states, request.max_depth, artifact
    );
    if let Some(record) = request.record {
        command.push_str(&format!(" --record {record}"));
    }
    if let Some(fixtures) = request.fixtures {
        let uri = format!(
            "stado://spis-crawls/{}/fixtures/{stamp}.json",
            request.catalog
        );
        publish_file(fixtures, &uri)?;
        let remote = format!("$HOME/.stado/work/{stamp}-desktop-fixtures.json");
        command = format!(
            "$HOME/.stado/bin/stado storage get {uri} {remote} && {command} --fixtures {remote}"
        );
    }
    let output_uri = format!(
        "stado://spis-crawls/{}/{stamp}/job-output",
        request.catalog
    );
    let mut arguments = vec![
        "submit".to_string(),
        command,
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
        .context("submit desktop crawl through Stado")?;
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

fn publish_artifact(run_root: &Path, uri: &str) -> Result<()> {
    let archive = run_root.with_extension("tar.gz");
    let status = Command::new("stado")
        .args(["storage", "archive"])
        .arg(run_root)
        .arg(&archive)
        .status()
        .context("archive desktop crawl")?;
    if !status.success() {
        bail!("stado storage archive refused desktop crawl artifacts");
    }
    let status = Command::new("stado")
        .args(["storage", "put", "--if-absent", uri])
        .arg(&archive)
        .status()
        .context("publish desktop crawl")?;
    if !status.success() {
        bail!("stado storage put refused desktop crawl artifacts");
    }
    Ok(())
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut record: Option<String> = None;
    let mut fixtures_path: Option<PathBuf> = None;
    let mut secret_env = Vec::new();
    let mut max_states = 200usize;
    let mut max_depth = 8usize;
    let mut host: Option<String> = None;
    let mut worker = false;
    let mut artifact_uri: Option<String> = None;
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
                println!("usage: spis crawl-desktop <macos-app-examples|desktop-app-examples> --host TARGET [--record SLUG] [--fixtures FILE] [--secret-env NAME=SKARBIEC_ITEM] [--max-states N] [--max-depth N]\nworker mode: spis crawl-desktop <catalog> --worker [--artifact-uri stado://...] [--output DIR]");
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
    if !worker {
        let host =
            host.context("--host is required; desktop crawls execute as pinned Stado jobs")?;
        return submit_worker(DesktopSubmission {
            host: &host,
            catalog: &catalog,
            record: record.as_deref(),
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
    preflight()?;
    let fixtures = Fixtures::load(fixtures_path.as_deref())?;
    let run_root = output
        .join(&catalog)
        .join(crate::now_iso_utc().replace(':', "-"));
    std::fs::create_dir_all(&run_root)?;
    let mut reports = Vec::new();
    for entry in records(&catalog, record.as_deref())? {
        match crawl_record(&entry, &fixtures, &run_root, max_states, max_depth) {
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
        "schema": "wisent.desktop-crawl-batch.v1",
        "catalog": catalog,
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
        bail!("{failures} desktop records could not be crawled");
    }
    Ok(())
}
