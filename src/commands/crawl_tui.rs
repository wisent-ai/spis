//! Real terminal-application crawler.
//!
//! The coordinator submits an exact-revision Stado job to the selected host.
//! The worker launches each installed TUI inside isolated tmux PTYs, replays
//! paths from a fresh process, breadth-first explores non-destructive keyboard
//! controls, records raw terminal bytes, and retains every distinct screen.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";
const KEYS: &[(&str, &str)] = &[
    ("down", "Down"),
    ("up", "Up"),
    ("left", "Left"),
    ("right", "Right"),
    ("next-region", "Tab"),
    ("previous-region", "BTab"),
    ("inspect", "Enter"),
    ("back", "Escape"),
    ("toggle", "Space"),
    ("help-question", "?"),
    ("help-h", "h"),
    ("vim-down", "j"),
    ("vim-up", "k"),
    ("vim-left", "h"),
    ("vim-right", "l"),
    ("home", "Home"),
    ("end", "End"),
    ("page-up", "PageUp"),
    ("page-down", "PageDown"),
    ("search", "/"),
    ("view-1", "1"),
    ("view-2", "2"),
    ("view-3", "3"),
    ("view-4", "4"),
    ("view-5", "5"),
];

#[derive(Clone, serde::Serialize)]
struct Step {
    label: String,
    key: String,
}

struct TmuxSession {
    name: String,
}

impl Drop for TmuxSession {
    fn drop(&mut self) {
        let _ = Command::new("tmux")
            .args(["kill-session", "-t", &self.name])
            .status();
    }
}

fn safe_component(value: &str, name: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        bail!("{name} must contain only letters, digits, '-' or '_'");
    }
    Ok(())
}

fn revision() -> Result<String> { super::crawl::build_revision() }

pub(crate) fn binary_candidates(name: &str) -> Vec<String> {
    let lower = name.to_lowercase();
    vec![
        match lower.as_str() {
            "midnight commander" => "mc".to_string(),
            "github cli dashboard" => "gh-dash".to_string(),
            "bottom" => "btm".to_string(),
            other => other.to_string(),
        },
        lower.replace(' ', "-"),
        lower.replace(' ', ""),
    ]
}

fn executable(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let candidates = binary_candidates(name);
    std::env::split_paths(&path)
        .flat_map(|directory| candidates.iter().map(move |name| directory.join(name)))
        .find(|candidate| candidate.is_file())
}

fn command(mut command: Command, context: &str) -> Result<String> {
    let output = command.output().with_context(|| context.to_string())?;
    if !output.status.success() {
        bail!(
            "{context}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn capture(session: &str) -> Result<String> {
    let mut request = Command::new("tmux");
    request.args(["capture-pane", "-t", session, "-p", "-e", "-S", "-"]);
    command(request, "capture TUI pane")
}

fn hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    hex::encode(digest.finalize())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn prepare_fixture(fixture: &Path) -> Result<()> {
    let home = fixture.join("home");
    std::fs::create_dir_all(&home)?;
    std::fs::write(fixture.join("seed.txt"), "Spis TUI crawl fixture\n")?;
    std::fs::write(fixture.join("tracked.txt"), "committed fixture state\n")?;
    let run_git = |arguments: &[&str]| -> Result<()> {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(fixture)
            .env("HOME", &home)
            .env("GIT_CONFIG_GLOBAL", fixture.join("gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .with_context(|| format!("prepare TUI fixture: git {}", arguments.join(" ")))?;
        if !output.status.success() {
            bail!(
                "prepare TUI fixture: git {}: {}",
                arguments.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(())
    };
    run_git(&["init", "--quiet"])?;
    run_git(&["config", "user.name", "Spis crawler"])?;
    run_git(&["config", "user.email", "spis-crawler@invalid"])?;
    run_git(&["add", "seed.txt", "tracked.txt"])?;
    run_git(&["commit", "--quiet", "-m", "Seed isolated crawl fixture"])?;
    std::fs::write(
        fixture.join("tracked.txt"),
        "committed fixture state\nmodified fixture state\n",
    )?;
    std::fs::write(fixture.join("untracked.txt"), "untracked fixture state\n")?;
    Ok(())
}

fn launch(
    record_slug: &str,
    binary: &Path,
    fixture: &Path,
    raw: &Path,
    attempt: usize,
) -> Result<TmuxSession> {
    let session = format!("spis-tui-{}-{attempt}-{record_slug}", std::process::id());
    let home = fixture.join("home");
    let config = home.join(".config");
    let data = home.join(".local/share");
    let cache = home.join(".cache");
    let state = home.join(".local/state");
    let runtime = fixture.join("runtime");
    for directory in [&config, &data, &cache, &state, &runtime] {
        std::fs::create_dir_all(directory)?;
    }
    let path = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string());
    let mut launch = Command::new("tmux");
    launch
        .args([
            "new-session",
            "-d",
            "-s",
            &session,
            "-x",
            "120",
            "-y",
            "40",
            "-c",
        ])
        .arg(fixture)
        .arg("--")
        .arg("env")
        .arg("-i")
        .arg(format!("PATH={path}"))
        .arg(format!("HOME={}", home.display()))
        .arg(format!("XDG_CONFIG_HOME={}", config.display()))
        .arg(format!("XDG_DATA_HOME={}", data.display()))
        .arg(format!("XDG_CACHE_HOME={}", cache.display()))
        .arg(format!("XDG_STATE_HOME={}", state.display()))
        .arg(format!("XDG_RUNTIME_DIR={}", runtime.display()))
        .arg(format!(
            "GIT_CONFIG_GLOBAL={}",
            fixture.join("gitconfig").display()
        ))
        .arg("GIT_CONFIG_NOSYSTEM=1")
        .arg(format!(
            "KUBECONFIG={}",
            fixture.join("kubeconfig").display()
        ))
        .arg(format!(
            "DOCKER_HOST=unix://{}",
            fixture.join("docker.sock").display()
        ))
        .arg("AWS_EC2_METADATA_DISABLED=true")
        .arg("TERM=xterm-256color")
        .arg("NO_COLOR=1")
        .arg("LANG=C.UTF-8")
        .arg(binary);
    command(launch, "launch TUI in tmux PTY")?;
    let pipe = format!("cat >> {}", shell_quote(raw.to_string_lossy().as_ref()));
    let mut pipe_pane = Command::new("tmux");
    pipe_pane.args(["pipe-pane", "-t", &session, "-o", &pipe]);
    command(pipe_pane, "record TUI byte stream")?;
    std::thread::sleep(Duration::from_secs(1));
    Ok(TmuxSession { name: session })
}

fn replay(session: &str, path: &[Step]) -> Result<()> {
    for step in path {
        let mut send = Command::new("tmux");
        send.args(["send-keys", "-t", session, &step.key]);
        command(send, "replay TUI key")?;
        std::thread::sleep(Duration::from_millis(300));
    }
    Ok(())
}

fn crawl_one(
    slug: &str,
    name: &str,
    manifest: &super::crawl::RuntimeManifest,
    output: &Path,
    max_states: usize,
    max_depth: usize,
) -> Result<Value> {
    let binary = executable(&manifest.runtime_product.identifier)
        .ok_or_else(|| anyhow!("{} is not installed on this host", manifest.runtime_product.identifier))?;
    if binary.file_name().and_then(|value| value.to_str())
        != Some(manifest.runtime_product.identifier.as_str())
    {
        bail!("TUI executable differs from immutable runtime manifest");
    }
    let fixture = output.join("fixture");
    let states = output.join("states");
    let attempts = output.join("attempts");
    std::fs::create_dir_all(&fixture)?;
    std::fs::create_dir_all(&states)?;
    std::fs::create_dir_all(&attempts)?;
    prepare_fixture(&fixture)?;

    let mut queue = VecDeque::from([Vec::<Step>::new()]);
    let mut seen = HashSet::new();
    let mut graph = Vec::new();
    let mut blocked = Vec::new();
    let mut attempt = 0usize;
    while let Some(path) = queue.pop_front() {
        if seen.len() >= max_states {
            break;
        }
        attempt += 1;
        let raw = attempts.join(format!("attempt-{attempt:05}.raw"));
        let session = match launch(slug, &binary, &fixture, &raw, attempt) {
            Ok(session) => session,
            Err(error) => {
                blocked.push(json!({"path": path, "reason": error.to_string()}));
                continue;
            }
        };
        if let Err(error) = replay(&session.name, &path) {
            blocked.push(json!({"path": path, "reason": error.to_string()}));
            continue;
        }
        let screen = match capture(&session.name) {
            Ok(screen) => screen,
            Err(error) => {
                blocked.push(json!({"path": path, "reason": error.to_string()}));
                continue;
            }
        };
        let digest = hash(&screen);
        if !seen.insert(digest.clone()) {
            continue;
        }
        let index = seen.len();
        let state_path = states.join(format!("state-{index:05}.ansi"));
        std::fs::write(&state_path, &screen)?;
        graph.push(json!({
            "state": digest,
            "index": index,
            "depth": path.len(),
            "path": path,
            "screen": state_path.strip_prefix(output).unwrap_or(&state_path),
            "raw_terminal_stream": raw.strip_prefix(output).unwrap_or(&raw),
        }));
        if path.len() < max_depth {
            for (label, key) in KEYS {
                let mut next = path.clone();
                next.push(Step {
                    label: (*label).to_string(),
                    key: (*key).to_string(),
                });
                queue.push_back(next);
            }
        }
    }
    let report = json!({
        "schema": "wisent.tui-crawl-run.v1",
        "slug": slug,
        "name": name,
        "binary": binary,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "runtime_execution_identity": manifest.execution_identity,
        "terminal": {"columns": 120, "rows": 40, "term": "xterm-256color"},
        "states": graph,
        "states_seen": seen.len(),
        "blocked_paths": blocked,
        "max_states": max_states,
        "max_depth": max_depth,
        "evidence_observations": {
            "executed_paths": graph.iter().filter_map(|state| state.get("path").cloned()).collect::<Vec<_>>(),
            "variant_events": [],
            "terminal_streams": graph.iter().filter_map(|state| state.get("raw_terminal_stream").cloned()).collect::<Vec<_>>(),
            "canonical_interactions": [],
            "canonical_journey": Value::Null,
            "canonical_accessibility": Value::Null,
            "canonical_motion_analysis": Value::Null,
            "gaps": [
                "No complete trigger/response/feedback/cancellation/failure/recovery interaction set was observed.",
                "No timed terminal cast or rendered state image was retained.",
                "Terminal keyboard accessibility equivalents and reduced-motion behavior remain unmeasured."
            ]
        },
        "completed_at": crate::now_iso_utc(),
    });
    std::fs::write(
        output.join("crawl.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}

fn records(selected: Option<&str>) -> Result<Vec<(String, String)>> {
    let directory = Path::new("tui-examples/references");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(directory)?
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
        if selected.is_some_and(|value| {
            value != slug && value != slug.split_once('-').map(|(_, tail)| tail).unwrap_or(slug)
        }) {
            continue;
        }
        let record: Value = serde_json::from_slice(&std::fs::read(path.join("reference.json"))?)?;
        let name = record
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        records.push((slug.to_string(), name.to_string()));
    }
    if records.is_empty() {
        bail!("no matching TUI records");
    }
    Ok(records)
}

fn publish(root: &Path, uri: &str) -> Result<()> {
    let archive = root.with_extension("tar.gz");
    let status = super::crawl::stado_command()
        .args(["storage", "archive"])
        .arg(root)
        .arg(&archive)
        .status()?;
    if !status.success() {
        bail!("stado storage archive refused TUI artifacts");
    }
    let status = super::crawl::stado_command()
        .args(["storage", "put", "--if-absent", uri])
        .arg(&archive)
        .status()?;
    if !status.success() {
        bail!("stado storage put refused TUI artifacts");
    }
    Ok(())
}

fn submit(
    host: &str,
    selected: &str,
    max_states: usize,
    max_depth: usize,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<()> {
    safe_component(host, "--host")?;
    safe_component(selected, "--record")?;
    if revision()? != manifest.source_revision {
        bail!("TUI coordinator revision does not match immutable runtime manifest");
    }
    let artifact = manifest.artifact_uri.clone();
    let output_uri = manifest.output_uri.clone();
    let worker = format!(
        "cargo run --release -- crawl-tui --worker --record {selected} --max-records 1 --max-states {max_states} --max-depth {max_depth} --artifact-uri {artifact} --runtime-manifest-base64 '{}'",
        manifest.encoded()?
    );
    let output = super::crawl::stado_command()
        .args([
            "submit",
            &worker,
            "--run-id",
            &manifest.stado_run_id,
            "--pinned-host",
            host,
            "--repo",
            REPOSITORY,
            "--repo-ref",
            &manifest.source_revision,
            "--repo-workdir",
            "spis",
            "--repo-extras",
            "",
            "--output-uri",
            &output_uri,
        ])
        .output()?;
    if !output.status.success() {
        bail!("Stado refused TUI crawl: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    super::crawl::print_submission(
        "tui-examples",
        "tui",
        host,
        Some(&artifact),
        &output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut host: Option<String> = None;
    let mut selected: Option<String> = None;
    let mut worker = false;
    let mut max_records = 50usize;
    let mut max_states = 200usize;
    let mut max_depth = 4usize;
    let mut artifact_uri: Option<String> = None;
    let mut runtime_manifest_base64: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--record" => {
                i += 1;
                selected = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--max-records" => {
                i += 1;
                max_records = rest
                    .get(i)
                    .context("--max-records needs a value")?
                    .parse()?;
            }
            "--max-states" => {
                i += 1;
                max_states = rest.get(i).context("--max-states needs a value")?.parse()?;
            }
            "--max-depth" => {
                i += 1;
                max_depth = rest.get(i).context("--max-depth needs a value")?.parse()?;
            }
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            "--runtime-manifest-base64" => {
                i += 1;
                runtime_manifest_base64 =
                    Some(rest.get(i).context("--runtime-manifest-base64 needs a value")?.clone());
            }
            "--worker" => worker = true,
            "--help" | "-h" => {
                println!("usage: spis crawl-tui --host TARGET --record SLUG --runtime-manifest-base64 DATA [--max-states N] [--max-depth N]\nworker mode requires the same immutable runtime manifest and exact record.");
                return Ok(());
            }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    if max_records == 0
        || max_records > 50
        || max_states == 0
        || max_states > 5_000
        || max_depth > 12
    {
        bail!("--max-records must be 1..50, --max-states 1..5000 and --max-depth 0..12");
    }
    let selected = selected.context("--record is required for one exact per-record job")?;
    if max_records != 1 {
        bail!("per-record TUI jobs require --max-records 1");
    }
    let manifest = super::crawl::decode_runtime_manifest(
        runtime_manifest_base64.as_deref().context("--runtime-manifest-base64 is required")?,
        "tui-examples",
        "tui",
        Some(&selected),
    )?;
    if !worker {
        return submit(
            &host.context("--host is required; TUI crawls execute as pinned Stado jobs")?,
            &selected,
            max_states,
            max_depth,
            &manifest,
        );
    }
    if host.is_some() {
        bail!("--host cannot be used with --worker");
    }
    let artifact_uri = artifact_uri.context("--artifact-uri is required in worker mode")?;
    if artifact_uri != manifest.artifact_uri {
        bail!("worker artifact URI does not match immutable runtime manifest");
    }
    let root = Path::new("target")
        .join("spis-tui-crawls")
        .join(&manifest.run_id)
        .join(&manifest.record_key);
    std::fs::create_dir_all(&root)?;
    let (slug, name) = records(Some(&selected))?.into_iter().next().context("runtime manifest record is absent")?;
    let output = root.join(&slug);
    std::fs::create_dir_all(&output)?;
    let reports = vec![match crawl_one(&slug, &name, &manifest, &output, max_states, max_depth) {
        Ok(report) => report,
        Err(error) => json!({
            "slug": slug,
            "name": name,
            "status": "failed",
            "source_revision": manifest.source_revision,
            "source_input_sha256": manifest.source_input_sha256,
            "runtime_manifest": manifest,
            "error": error.to_string()
        }),
    }];
    let failures = reports
        .iter()
        .filter(|report| report.get("status").and_then(Value::as_str) == Some("failed"))
        .count();
    let summary = json!({
        "schema": "wisent.tui-crawl-batch.v1",
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "records": reports,
        "failed": failures,
        "completed_at": crate::now_iso_utc(),
    });
    std::fs::write(
        root.join("batch.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    publish(&root, &artifact_uri)?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    if failures > 0 {
        bail!("{failures} TUI records could not be crawled");
    }
    Ok(())
}
