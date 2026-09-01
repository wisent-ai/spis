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
use std::time::Duration;

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
        let _ = Command::new("tmux")
            .env_clear()
            .env("PATH", "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin")
            .args(["-S", self.socket.to_string_lossy().as_ref()])
            .args(["kill-session", "-t", &self.name])
            .output();
    }
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
    let value = serde_json::to_value(manifest)?;
    let attempt = value
        .get("attempt")
        .and_then(Value::as_u64)
        .filter(|value| (1..=u32::MAX as u64).contains(value))
        .context("TUI runtime manifest has no typed u32 attempt coordinate")?;
    let attempt_id = value
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("TUI runtime manifest has no typed attempt_id coordinate")?;
    for (name, component) in [
        ("run_id", manifest.run_id.as_str()),
        ("catalog", manifest.catalog.as_str()),
        ("record", manifest.record.as_str()),
        ("record_key", manifest.record_key.as_str()),
        ("attempt_id", attempt_id),
    ] {
        safe_component(component, &format!("runtime manifest {name}"))?;
    }
    let coordinate = format!(
        "stado://spis-crawls/{}/{}/{}/{}/attempts/{}/{}/",
        manifest.run_id,
        manifest.catalog,
        manifest.record,
        manifest.record_key,
        attempt,
        attempt_id,
    );
    for (name, observed, leaf) in [
        ("artifact_uri", manifest.artifact_uri.as_str(), "artifact"),
        ("output_uri", manifest.output_uri.as_str(), "output"),
    ] {
        let expected = format!("{coordinate}{leaf}");
        if observed != expected {
            bail!("TUI runtime manifest {name} must equal exact canonical coordinate {expected:?}");
        }
    }
    Ok(base
        .join(&manifest.run_id)
        .join(&manifest.catalog)
        .join(&manifest.record)
        .join(&manifest.record_key)
        .join("attempts")
        .join(attempt.to_string())
        .join(attempt_id))
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
    let hostname = Command::new("hostname")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .context("read TUI worker hostname")?;
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
    let version = version_command
        .output()
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
    let output = Command::new("tmux")
        .env_clear()
        .envs(environment)
        .args(["-S", socket.to_string_lossy().as_ref()])
        .args(args)
        .output()
        .with_context(|| context.to_string())?;
    if !output.status.success() {
        bail!(
            "{context}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn capture(session: &TmuxSession) -> Result<String> {
    tmux(
        &session.socket,
        &session.environment,
        &["capture-pane", "-t", &session.name, "-p", "-e", "-S", "-"],
        "capture TUI pane",
    )
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
        let output = Command::new("git")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
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
    environment: BTreeMap<OsString, OsString>,
) -> Result<TmuxSession> {
    let name = format!("spis-tui-{}-{attempt}-{record_slug}", std::process::id());
    let socket = fixture.join("tmux.sock");
    if socket.exists() {
        std::fs::remove_file(&socket)
            .with_context(|| format!("remove stale private TUI tmux socket {}", socket.display()))?;
    }
    tmux(
        &socket,
        &environment,
        &[
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
    let pipe = format!("cat >> {}", shell_quote(raw.to_string_lossy().as_ref()));
    tmux(
        &session.socket,
        &session.environment,
        &["pipe-pane", "-t", &session.name, "-o", &pipe],
        "record TUI byte stream",
    )?;
    std::thread::sleep(Duration::from_secs(1));
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

fn hash_artifact(path: &Path) -> Result<(String, u64)> {
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("open TUI artifact {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash TUI artifact {}", path.display()))?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        digest.update(&buffer[..count]);
    }
    Ok((hex::encode(digest.finalize()), bytes))
}

fn publish(root: &Path, uri: &str) -> Result<Value> {
    let attempt_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .context("TUI attempt artifact root has no UTF-8 name")?;
    let archive = root.with_file_name(format!("{attempt_name}.tar.gz"));
    if !archive.is_file() {
        let output = super::crawl::stado_command()
            .args(["storage", "archive"])
            .arg(root)
            .arg(&archive)
            .output()?;
        if !output.status.success() {
            bail!(
                "stado storage archive refused TUI artifacts: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
    }
    let (sha256, bytes) = hash_artifact(&archive)?;
    let output = super::crawl::stado_command()
        .args(["storage", "put", "--if-absent", uri])
        .arg(&archive)
        .output()?;
    if !output.status.success() {
        bail!(
            "stado storage put refused TUI artifacts: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let readback = root.with_file_name(format!("{attempt_name}.readback.tar.gz"));
    if readback.exists() {
        std::fs::remove_file(&readback)?;
    }
    let output = super::crawl::stado_command()
        .args(["storage", "get", uri])
        .arg(&readback)
        .output()?;
    if !output.status.success() {
        bail!(
            "stado storage readback refused TUI artifacts: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let (observed_sha256, observed_bytes) = hash_artifact(&readback)?;
    std::fs::remove_file(&readback)?;
    if observed_sha256 != sha256 || observed_bytes != bytes {
        bail!(
            "TUI artifact readback differs: expected sha256={sha256} bytes={bytes}, observed sha256={observed_sha256} bytes={observed_bytes}"
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
    let worker = format!(
        "cargo run --release -- crawl-tui --worker --record {selected} --artifact-uri {artifact} --runtime-manifest-base64 '{}'",
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
        "spis",
        "--repo-extras",
        "",
        "--output-uri",
        &output_uri,
    ]);
    for (name, reference) in delivery_secret_bindings(manifest)? {
        stado.arg("--secret-env").arg(format!("{name}={reference}"));
    }
    let output = stado.output()?;
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
    let reports = vec![match crawl_one(&slug, &name, &manifest, &output) {
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
    });
    std::fs::write(
        root.join("batch.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    if failures > 0 {
        bail!("{failures} TUI records could not be crawled");
    }
    let artifact = publish(&root, &artifact_uri)?;
    let worker_report = worker_report(&manifest, artifact)?;
    println!("{}", serde_json::to_string(&worker_report)?);
    Ok(())
}
