//! Real terminal-application crawler.
//!
//! The worker retains only the initial exact terminal state and applies a
//! default-deny policy to all keyboard, mouse, pointer, paste, and text input.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

const REPRESENTATIVE_INPUT_CLASSES: &[(&str, &str)] = &[
    ("arrow keys", "context-ambiguous navigation or mutation input"),
    ("focus keys", "context-ambiguous focus or commit input"),
    ("escape/cancel keys", "context-ambiguous cancel or discard input"),
    ("paging keys", "context-ambiguous navigation input"),
    ("activation keys", "ambiguous activation, confirmation, or toggle input"),
    ("digit keys", "application-specific numeric input"),
    ("text keys", "application-specific letter, search, help, or command input"),
    ("mouse and pointer", "context-ambiguous pointing, scrolling, or activation input"),
    ("paste and other input", "unclassified input has no authorized exception"),
];


struct TmuxSession {
    name: String,
    socket: PathBuf,
    environment: BTreeMap<OsString, OsString>,
}

impl Drop for TmuxSession {
    fn drop(&mut self) {
        // Short deadline: cleanup must never block the worker (finding 4).
        let mut command = Command::new("tmux");
        command
            .env_clear()
            .env("PATH", "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin")
            .args(["-S", self.socket.to_string_lossy().as_ref()])
            .args(["kill-session", "-t", &self.name]);
        let _ = super::crawl::bounded_command_output(
            &mut command,
            "close private TUI PTY",
            Duration::from_secs(5),
            64 * 1024,
        );
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
        .unwrap_or("tui_record_failed")
}

fn safe_component(value: &str, name: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        bail!("{name} must contain only letters, digits, '-' or '_'");
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

fn delivery_secret_bindings(
    manifest: &super::crawl::RuntimeManifest,
) -> Result<Vec<(String, String)>> {
    let value = serde_json::to_value(manifest)?;
    let secrets = value
        .pointer("/delivery/secret_env")
        .and_then(Value::as_object)
        .context("TUI runtime manifest has no typed delivery.secret_env map")?;
    let mut bindings = Vec::with_capacity(secrets.len());
    for (name, reference) in secrets {
        let reference = reference
            .as_str()
            .filter(|value| !value.is_empty())
            .with_context(|| format!("TUI delivery secret reference is invalid for environment key {name:?}"))?;
        if name.is_empty()
            || !name
                .chars()
                .enumerate()
                .all(|(index, character)| {
                    character == '_'
                        || character.is_ascii_alphabetic()
                        || (index > 0 && character.is_ascii_digit())
                })
        {
            bail!("TUI delivery secret binding has invalid environment key {name:?}");
        }
        if [
            "PATH",
            "HOME",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "XDG_STATE_HOME",
            "XDG_RUNTIME_DIR",
            "GIT_CONFIG_GLOBAL",
            "GIT_CONFIG_NOSYSTEM",
            "KUBECONFIG",
            "DOCKER_HOST",
            "AWS_EC2_METADATA_DISABLED",
            "TERM",
            "NO_COLOR",
            "LANG",
        ]
        .contains(&name.as_str())
        {
            bail!("TUI delivery secret key {name:?} would override the isolated worker environment");
        }
        bindings.push((name.clone(), reference.to_string()));
    }
    Ok(bindings)
}

fn isolated_environment(
    manifest: &super::crawl::RuntimeManifest,
    fixture: &Path,
) -> Result<BTreeMap<OsString, OsString>> {
    let home = fixture.join("home");
    let mut environment = BTreeMap::new();
    environment.insert("PATH".into(), "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin".into());
    environment.insert("HOME".into(), home.clone().into_os_string());
    environment.insert("XDG_CONFIG_HOME".into(), home.join(".config").into_os_string());
    environment.insert("XDG_DATA_HOME".into(), home.join(".local/share").into_os_string());
    environment.insert("XDG_CACHE_HOME".into(), home.join(".cache").into_os_string());
    environment.insert("XDG_STATE_HOME".into(), home.join(".local/state").into_os_string());
    environment.insert("XDG_RUNTIME_DIR".into(), fixture.join("runtime").into_os_string());
    environment.insert("GIT_CONFIG_GLOBAL".into(), fixture.join("gitconfig").into_os_string());
    environment.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    environment.insert("KUBECONFIG".into(), fixture.join("kubeconfig").into_os_string());
    environment.insert("DOCKER_HOST".into(), format!("unix://{}", fixture.join("docker.sock").display()).into());
    environment.insert("AWS_EC2_METADATA_DISABLED".into(), "true".into());
    environment.insert("TERM".into(), "xterm-256color".into());
    environment.insert("NO_COLOR".into(), "1".into());
    environment.insert("LANG".into(), "C.UTF-8".into());
    for (name, _) in delivery_secret_bindings(manifest)? {
        let value = std::env::var_os(&name)
            .with_context(|| format!("TUI worker did not receive manifest-bound secret environment key {name}"))?;
        environment.insert(name.into(), value);
    }
    Ok(environment)
}

fn verify_exact_executable(
    manifest: &super::crawl::RuntimeManifest,
    expected_filename: &str,
    environment: &BTreeMap<OsString, OsString>,
) -> Result<PathBuf> {
    let identity = manifest
        .execution_identity
        .as_ref()
        .context("TUI runtime manifest has no resolved execution identity")?;
    if identity.platform != "terminal" {
        bail!(
            "TUI execution identity platform differs: expected \"terminal\", observed {:?}",
            identity.platform
        );
    }
    if manifest.runtime_product.identifier != expected_filename {
        bail!(
            "TUI manifest product identifier differs: expected {expected_filename:?}, observed {:?}",
            manifest.runtime_product.identifier
        );
    }
    if identity.host.is_empty() {
        bail!("TUI execution identity has no registry host alias");
    }
    let identity_value = serde_json::to_value(identity)?;
    let expected_hostname = identity_value
        .get("observed_hostname")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("TUI execution identity has no typed observed_hostname")?;
    let mut hostname_command = Command::new("hostname");
    hostname_command.env_clear().env("PATH", "/usr/bin:/bin");
    let hostname = super::crawl::bounded_command_output(
        &mut hostname_command,
        "read TUI worker hostname",
        Duration::from_secs(10),
        64 * 1024,
    )?;
    if !hostname.status.success() {
        bail!(
            "TUI worker hostname command failed: status={}; stdout={:?}; stderr={:?}",
            hostname.status,
            String::from_utf8_lossy(&hostname.stdout),
            String::from_utf8_lossy(&hostname.stderr)
        );
    }
    let observed_hostname = String::from_utf8_lossy(&hostname.stdout).trim().to_string();
    if observed_hostname != expected_hostname {
        bail!(
            "TUI observed hostname differs: expected {expected_hostname:?}, observed {observed_hostname:?}"
        );
    }
    let configured = identity
        .executable_path
        .as_deref()
        .context("TUI execution identity has no exact executable path")?;
    let path = PathBuf::from(configured);
    if !path.is_absolute() || !path.is_file() {
        bail!("TUI execution identity path is not an absolute executable file: {configured}");
    }
    let canonical = std::fs::canonicalize(&path)
        .with_context(|| format!("canonicalize exact TUI executable {}", path.display()))?;
    if canonical != path {
        bail!(
            "TUI execution identity path is not canonical: declared {}, canonical {}",
            path.display(),
            canonical.display()
        );
    }
    if path.file_name().and_then(|value| value.to_str()) != Some(expected_filename) {
        bail!(
            "TUI execution identity filename differs: expected {expected_filename}, observed {}",
            path.display()
        );
    }
    let expected_sha = identity
        .executable_sha256
        .as_deref()
        .context("TUI execution identity has no executable SHA-256")?;
    let mut file = std::fs::File::open(&path)
        .with_context(|| format!("open exact TUI executable {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash exact TUI executable {}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let observed_sha = hex::encode(digest.finalize());
    if !observed_sha.eq_ignore_ascii_case(expected_sha) {
        bail!(
            "TUI executable SHA-256 changed immediately before trajectory launch: expected {expected_sha}, observed {observed_sha}"
        );
    }
    let expected_version = identity
        .product_version
        .as_deref()
        .context("TUI execution identity has no exact product version")?;
    let mut version_command = Command::new(&path);
    version_command
        .arg("--version")
        .env_clear()
        .envs(environment);
    let version = super::crawl::bounded_command_output(
        &mut version_command,
        "read exact TUI version",
        Duration::from_secs(30),
        1024 * 1024,
    )
    .with_context(|| format!("read exact TUI version from {}", path.display()))?;
    if !version.status.success() {
        bail!(
            "exact TUI version command failed immediately before launch: status={}; stdout={:?}; stderr={:?}",
            version.status,
            String::from_utf8_lossy(&version.stdout),
            String::from_utf8_lossy(&version.stderr)
        );
    }
    let observed_version = String::from_utf8_lossy(&version.stdout).trim().to_string();
    if observed_version != expected_version {
        bail!(
            "TUI product version changed immediately before trajectory launch: expected {expected_version:?}, observed {observed_version:?}"
        );
    }
    Ok(path)
}

fn tmux(
    socket: &Path,
    environment: &BTreeMap<OsString, OsString>,
    args: &[&str],
    context: &str,
) -> Result<String> {
    let mut command = Command::new("tmux");
    command
        .env_clear()
        .envs(environment)
        .args(["-S", socket.to_string_lossy().as_ref()])
        .args(args);
    let output = super::crawl::bounded_command_output(
        &mut command,
        context,
        Duration::from_secs(30),
        MAXIMUM_CAPTURE_BYTES,
    )?;
    if !output.status.success() {
        bail!(
            "{context}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// A capture larger than this is refused rather than digested as evidence.
const MAXIMUM_CAPTURE_BYTES: usize = 4 * 1024 * 1024;

fn capture_range(session: &TmuxSession, start: &str, context: &'static str) -> Result<String> {
    let screen = tmux(
        &session.socket,
        &session.environment,
        &["capture-pane", "-t", &session.name, "-p", "-e", "-S", start],
        context,
    )?;
    if screen.len() > MAXIMUM_CAPTURE_BYTES {
        bail!(
            "{context} returned {} bytes, beyond the {MAXIMUM_CAPTURE_BYTES}-byte bound",
            screen.len()
        );
    }
    Ok(screen)
}

/// Readiness polling reads only the visible tail; `-S -` would re-read the
/// entire scrollback on every poll (finding 8).
fn capture_tail(session: &TmuxSession) -> Result<String> {
    capture_range(session, "-50", "poll TUI pane tail")
}

/// Exactly one bounded full capture per record (finding 8).
fn capture(session: &TmuxSession) -> Result<String> {
    capture_range(session, "-2000", "capture TUI pane")
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
    for directory in [
        home.join(".config"),
        home.join(".local/share"),
        home.join(".cache"),
        home.join(".local/state"),
        fixture.join("runtime"),
    ] {
        std::fs::create_dir_all(directory)?;
    }
    std::fs::write(fixture.join("seed.txt"), "Spis TUI crawl fixture\n")?;
    std::fs::write(fixture.join("tracked.txt"), "committed fixture state\n")?;
    let run_git = |arguments: &[&str]| -> Result<()> {
        let mut command = Command::new("git");
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .args(arguments)
            .current_dir(fixture)
            .env("HOME", &home)
            .env("GIT_CONFIG_GLOBAL", fixture.join("gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1");
        let output = super::crawl::bounded_command_output(
            &mut command,
            "prepare TUI fixture with git",
            Duration::from_secs(60),
            1024 * 1024,
        )
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
    environment: BTreeMap<OsString, OsString>,
) -> Result<TmuxSession> {
    let name = format!("spis-tui-{}-{attempt}-{record_slug}", std::process::id());
    let socket = fixture.join("tmux.sock");
    // exists() follows symlinks and is false for a dangling one, so a planted
    // link would survive and tmux would place its socket at the link target
    // (finding 18).
    if std::fs::symlink_metadata(&socket).is_ok() {
        std::fs::remove_file(&socket)
            .with_context(|| format!("remove stale private TUI tmux socket {}", socket.display()))?;
    }
    if std::fs::symlink_metadata(&socket).is_ok() {
        bail!(
            "private TUI tmux socket path {} reappeared before launch",
            socket.display()
        );
    }
    if std::fs::symlink_metadata(raw).is_ok() {
        bail!(
            "TUI raw terminal log {} exists before this attempt recorded anything",
            raw.display()
        );
    }
    // Bounded scrollback on this private server (finding 8).
    let configuration = fixture.join("tmux.conf");
    std::fs::write(&configuration, "set -g history-limit 2000\n")?;
    tmux(
        &socket,
        &environment,
        &[
            "-f",
            configuration.to_string_lossy().as_ref(),
            "new-session",
            "-d",
            "-s",
            &name,
            "-x",
            "120",
            "-y",
            "40",
            "-c",
            fixture.to_string_lossy().as_ref(),
            "--",
            binary.to_string_lossy().as_ref(),
        ],
        "launch TUI in private tmux PTY",
    )?;
    let session = TmuxSession {
        name,
        socket,
        environment,
    };
    // `>` truncates, so terminal.raw holds this attempt only (finding 11).
    let pipe = format!("cat > {}", shell_quote(raw.to_string_lossy().as_ref()));
    tmux(
        &session.socket,
        &session.environment,
        &["pipe-pane", "-t", &session.name, "-o", &pipe],
        "record TUI byte stream",
    )?;
    // Bounded readiness poll instead of trusting a fixed sleep: the exact
    // initial state is only captured once the program has actually drawn
    // something (finding 19). A dynamic TUI keeps repainting, so this waits for
    // first paint rather than for a stable screen.
    let floor = Duration::from_secs(1);
    let started = Instant::now();
    loop {
        let blank = capture_tail(&session)?.trim().is_empty();
        if !blank && started.elapsed() >= floor {
            break;
        }
        if started.elapsed() >= Duration::from_secs(20) {
            // The hung program is killed with the session by TmuxSession::drop;
            // nothing downstream may reuse this PTY (finding 6).
            return Err(anyhow::Error::new(RecordFailure {
                code: "tui_launch_not_ready",
                message: format!(
                    "the exact TUI executable drew nothing in the private PTY within 20s for record {record_slug}"
                ),
            }));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(session)
}


fn crawl_one(
    slug: &str,
    name: &str,
    manifest: &super::crawl::RuntimeManifest,
    output: &Path,
) -> Result<Value> {
    let configured_path = manifest
        .execution_identity
        .as_ref()
        .and_then(|identity| identity.executable_path.as_deref())
        .context("TUI runtime manifest has no exact executable path")?;
    let expected_filename = Path::new(configured_path)
        .file_name()
        .and_then(|value| value.to_str())
        .context("TUI executable path has no UTF-8 filename")?
        .to_string();
    // Attempt-clean tree: `pipe-pane` appends, so without this a retried record
    // would blend the previous attempt's raw terminal bytes with this attempt's
    // overwritten state files (finding 11).
    match std::fs::symlink_metadata(output) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            bail!("TUI attempt output {} is a symlink", output.display())
        }
        Ok(_) => std::fs::remove_dir_all(output)
            .with_context(|| format!("clear TUI attempt output {}", output.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let states = output.join("states");
    let trajectory_root = output.join("trajectories/trajectory-00001");
    let fixture = trajectory_root.join("fixture");
    let raw = trajectory_root.join("terminal.raw");
    std::fs::create_dir_all(&states)?;
    std::fs::create_dir_all(&fixture)?;
    prepare_fixture(&fixture)?;
    let environment = isolated_environment(manifest, &fixture)?;
    let binary = verify_exact_executable(manifest, &expected_filename, &environment)?;
    let session = launch(slug, &binary, &fixture, &raw, 1, environment)?;
    let screen = capture(&session)?;
    let digest = hash(&screen);
    let state_path = states.join("state-00001.ansi");
    std::fs::write(&state_path, &screen)?;
    let graph = vec![json!({
        "state": digest,
        "index": 1,
        "trajectory_depth": 0,
        "delivered_inputs": [],
        "observed_state": {
            "changed_from_unobserved_initial_state": Value::Null,
            "exact_terminal_state": state_path.strip_prefix(output).unwrap_or(&state_path),
            "raw_terminal_stream": raw.strip_prefix(output).unwrap_or(&raw),
            "isolated_fixture": fixture.strip_prefix(output).unwrap_or(&fixture),
        },
    })];
    let blocked = REPRESENTATIVE_INPUT_CLASSES
        .iter()
        .map(|(class, reason)| {
            json!({
                "state": digest,
                "delivered_input": Value::Null,
                "observed_state_change": Value::Null,
                "representative_class": class,
                "reason": reason,
                "representative_not_exhaustive": true,
            })
        })
        .collect::<Vec<_>>();
    let representative_classes = REPRESENTATIVE_INPUT_CLASSES
        .iter()
        .map(|(class, reason)| json!({"class": class, "reason": reason}))
        .collect::<Vec<_>>();
    let report = json!({
        "schema": "wisent.tui-crawl-run.v1",
        "slug": slug,
        "name": name,
        "binary": configured_path,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "runtime_execution_identity": manifest.execution_identity,
        "terminal": {"columns": 120, "rows": 40, "term": "xterm-256color"},
        "states": graph,
        "states_seen": 1,
        "blocked_paths": blocked,
        "input_policy": {
            "default_decision": "withhold",
            "scope": "all keyboard, mouse, pointer, paste, text, and other input",
            "authorized_exceptions": [],
            "authorized_exception_count": 0,
            "delivered_input_count": 0,
            "representative_classes": representative_classes,
        },
        "evidence_observations": {
            "executed_trajectories": [[]],
            "terminal_streams": graph.iter().filter_map(|state| state.pointer("/observed_state/raw_terminal_stream").cloned()).collect::<Vec<_>>(),
            "canonical_interactions": [],
            "canonical_journey": Value::Null,
            "canonical_accessibility": Value::Null,
            "canonical_motion_analysis": Value::Null,
            "gaps": [
                "A default-deny policy withheld all keyboard, mouse, pointer, paste, text, and other input.",
                "No input class or key had an authorized exception.",
                "Representative input classes are reported without claiming an exhaustive key list."
            ]
        },
    });
    std::fs::write(
        output.join("crawl.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}

fn records(selected: Option<&str>) -> Result<Vec<(String, String)>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tui-examples")
        .join("references");
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

fn worker_report(
    manifest: &super::crawl::RuntimeManifest,
    artifact: Option<Value>,
    failure: Option<Value>,
) -> Result<Value> {
    let value = serde_json::to_value(manifest)?;
    let attempt = value
        .get("attempt")
        .and_then(Value::as_u64)
        .context("TUI runtime manifest has no attempt for worker report")?;
    let attempt_id = value
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("TUI runtime manifest has no attempt_id for worker report")?;
    let bindings_file_sha256 = value
        .get("bindings_file_sha256")
        .and_then(Value::as_str)
        .context("TUI runtime manifest has no bindings_file_sha256")?;
    let bindings_sha256 = value
        .get("bindings_sha256")
        .and_then(Value::as_str)
        .context("TUI runtime manifest has no bindings_sha256")?;
    let execution_identity = value
        .get("execution_identity")
        .filter(|identity| identity.is_object())
        .context("TUI runtime manifest has no typed execution_identity")?
        .clone();
    if let Some(artifact) = artifact.as_ref() {
        if artifact.get("uri").and_then(Value::as_str) != Some(manifest.artifact_uri.as_str()) {
            bail!("published TUI artifact URI does not match the immutable runtime manifest");
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

fn submit(
    host: &str,
    selected: &str,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<()> {
    safe_component(host, "--host")?;
    safe_component(selected, "--record")?;
    let _attempt_binding = attempt_root(Path::new("."), manifest)?;
    if revision()? != manifest.source_revision {
        bail!("TUI coordinator revision does not match immutable runtime manifest");
    }
    let artifact = manifest.artifact_uri.clone();
    let output_uri = manifest.output_uri.clone();
    // The absolute path this host executes cargo at, never the bare name.
    // Every worker in this repository is `cargo run --release`, and the job's
    // shell is a non-login `/bin/sh` that reads no profile, so a bare name
    // resolves to nothing however the host installs Rust -- the defect that
    // cost job-545551889f9e88be30daa81f sixteen minutes of a claimed slot in
    // the documentation engine, still open in this one.
    let cargo = super::crawl::resolved_worker_program(host)?;
    let worker = format!(
        "{cargo} run --release -- crawl-tui --worker --record {selected} --artifact-uri {artifact} --runtime-manifest-base64 '{}'",
        manifest.encoded()?
    );
    let mut stado = super::crawl::stado_command();
    stado.args([
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
        super::crawl::STADO_REPO_WORKDIR,
        "--repo-extras",
        "",
        "--output-uri",
        &output_uri,
    ]);
    for (name, reference) in delivery_secret_bindings(manifest)? {
        stado.arg("--secret-env").arg(format!("{name}={reference}"));
    }
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "submit TUI crawl through Stado",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
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
                println!("usage: spis crawl-tui --host TARGET --record SLUG --runtime-manifest-base64 DATA\nworker mode requires the same immutable runtime manifest and exact record.");
                return Ok(());
            }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    let selected = selected.context("--record is required for one exact per-record job")?;
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
    let root = attempt_root(
        &Path::new("target").join("spis-tui-crawls"),
        &manifest,
    )?;
    std::fs::create_dir_all(&root)?;
    let (slug, name) = records(Some(&selected))?.into_iter().next().context("runtime manifest record is absent")?;
    let output = root.join(&slug);
    std::fs::create_dir_all(&output)?;
    let (record_report, failure) = match crawl_one(&slug, &name, &manifest, &output) {
        Ok(report) => (report, None),
        Err(error) => {
            let code = failure_code(&error);
            let message = format!("{error:#}");
            // Diagnostics never share stdout with the one worker report line.
            eprintln!("TUI record {slug} failed: {message}");
            (
                json!({
                    "slug": slug,
                    "name": name,
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
        "schema": "wisent.tui-crawl-batch.v1",
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "records": [record_report],
        "failed": usize::from(failure.is_some()),
    });
    std::fs::write(
        root.join("batch.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    // A failed record still retains a typed failure artifact and still
    // publishes the attempt archive, so every attempt has exactly one archive
    // and exactly one worker report line.
    if let Some((code, message)) = failure.as_ref() {
        std::fs::write(
            root.join("failure.json"),
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
    let artifact = super::crawl::publish_attempt_archive(&root, &artifact_uri)?;
    let failure = failure.map(|(code, message)| json!({"code": code, "message": message}));
    let report = worker_report(&manifest, Some(artifact), failure.clone())?;
    println!("{}", serde_json::to_string(&report)?);
    if failure.is_some() {
        bail!("the exact TUI record could not be crawled");
    }
    Ok(())
}
