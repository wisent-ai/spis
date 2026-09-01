//! Real command-line application crawler.
//!
//! The worker executes only exact top-level version, help, refusal, and recovery
//! observations. Raw terminal bytes, screen states, argv, and exit statuses are
//! kept.

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";
const CATALOG: &str = "cli-examples";

#[derive(Clone)]
struct Record {
    slug: String,
    name: String,
    binary: String,
}


struct Invocation {
    argv: Vec<String>,
    output: String,
    exit_status: Option<i32>,
    timed_out: bool,
    state_path: String,
}

fn safe_component(value: &str, flag: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{flag} must contain only letters, digits, '.', '-' or '_'");
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
        .context("CLI runtime manifest has no typed u32 attempt coordinate")?;
    let attempt_id = value
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("CLI runtime manifest has no typed attempt_id coordinate")?;
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
            bail!("CLI runtime manifest {name} must equal exact canonical coordinate {expected:?}");
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

pub(crate) fn binary_for(slug: &str) -> String {
    let tail = slug.split_once('-').map(|(_, tail)| tail).unwrap_or(slug);
    match tail {
        "github-cli-gh" => "gh",
        "gitlab-cli-glab" => "glab",
        "go-command" => "go",
        "npm-cli" => "npm",
        "homebrew" => "brew",
        "docker-cli" => "docker",
        "opentofu-cli" => "tofu",
        "ansible-command-line-tools" => "ansible",
        "terraform-cli" => "terraform",
        "google-cloud-cli-gcloud" => "gcloud",
        "azure-cli" => "az",
        "digitalocean-cli-doctl" => "doctl",
        "oracle-cloud-infrastructure-cli" => "oci",
        "httpie-cli" => "http",
        "gnu-wget" => "wget",
        "iproute2" => "ip",
        "openssl-command-line-tools" => "openssl",
        "vault-cli" => "vault",
        "sqlite-command-line-shell" => "sqlite3",
        "duckdb-cli" => "duckdb",
        "mongodb-shell-mongosh" => "mongosh",
        other => other,
    }
    .to_string()
}

fn records(selected: Option<&str>) -> Result<Vec<Record>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(CATALOG)
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
        if selected.is_some_and(|value| {
            value != slug && value != slug.split_once('-').map(|(_, tail)| tail).unwrap_or(slug)
        }) {
            continue;
        }
        let document: Value = serde_json::from_slice(&std::fs::read(path.join("reference.json"))?)?;
        records.push(Record {
            slug: slug.to_string(),
            name: document
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            binary: binary_for(slug),
        });
    }
    if records.is_empty() {
        bail!("no matching CLI records");
    }
    Ok(records)
}

fn delivery_secret_bindings(
    manifest: &super::crawl::RuntimeManifest,
) -> Result<Vec<(String, String)>> {
    let value = serde_json::to_value(manifest)?;
    let secrets = value
        .pointer("/delivery/secret_env")
        .and_then(Value::as_object)
        .context("CLI runtime manifest has no typed delivery.secret_env map")?;
    let mut bindings = Vec::with_capacity(secrets.len());
    for (name, reference) in secrets {
        let reference = reference
            .as_str()
            .filter(|value| !value.is_empty())
            .with_context(|| format!("CLI delivery secret reference is invalid for environment key {name:?}"))?;
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
            bail!("CLI delivery secret binding has invalid environment key {name:?}");
        }
        if [
            "PATH",
            "HOME",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "GIT_CONFIG_GLOBAL",
            "GIT_CONFIG_NOSYSTEM",
            "KUBECONFIG",
            "DOCKER_HOST",
            "AWS_EC2_METADATA_DISABLED",
            "TERM",
            "PAGER",
            "GIT_PAGER",
            "MANPAGER",
            "NO_COLOR",
            "CI",
            "LANG",
        ]
        .contains(&name.as_str())
        {
            bail!("CLI delivery secret key {name:?} would override the isolated worker environment");
        }
        bindings.push((name.clone(), reference.to_string()));
    }
    Ok(bindings)
}

fn delivery_secret_names(manifest: &super::crawl::RuntimeManifest) -> Result<Vec<String>> {
    Ok(delivery_secret_bindings(manifest)?
        .into_iter()
        .map(|(name, _)| name)
        .collect())
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
    environment.insert("GIT_CONFIG_GLOBAL".into(), fixture.join("gitconfig").into_os_string());
    environment.insert("GIT_CONFIG_NOSYSTEM".into(), "1".into());
    environment.insert("KUBECONFIG".into(), fixture.join("kubeconfig").into_os_string());
    environment.insert("DOCKER_HOST".into(), format!("unix://{}", fixture.join("docker.sock").display()).into());
    environment.insert("AWS_EC2_METADATA_DISABLED".into(), "true".into());
    environment.insert("TERM".into(), "xterm-256color".into());
    environment.insert("PAGER".into(), "cat".into());
    environment.insert("GIT_PAGER".into(), "cat".into());
    environment.insert("MANPAGER".into(), "cat".into());
    environment.insert("NO_COLOR".into(), "1".into());
    environment.insert("CI".into(), "1".into());
    environment.insert("LANG".into(), "C.UTF-8".into());
    for name in delivery_secret_names(manifest)? {
        let value = std::env::var_os(&name)
            .with_context(|| format!("CLI worker did not receive manifest-bound secret environment key {name}"))?;
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
        .context("CLI runtime manifest has no resolved execution identity")?;
    if identity.platform != "terminal" {
        bail!(
            "CLI execution identity platform differs: expected \"terminal\", observed {:?}",
            identity.platform
        );
    }
    if manifest.runtime_product.identifier != expected_filename {
        bail!(
            "CLI manifest product identifier differs: expected {expected_filename:?}, observed {:?}",
            manifest.runtime_product.identifier
        );
    }
    if identity.host.is_empty() {
        bail!("CLI execution identity has no registry host alias");
    }
    let identity_value = serde_json::to_value(identity)?;
    let expected_hostname = identity_value
        .get("observed_hostname")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("CLI execution identity has no typed observed_hostname")?;
    let hostname = Command::new("hostname")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .context("read CLI worker hostname")?;
    if !hostname.status.success() {
        bail!(
            "CLI worker hostname command failed: status={}; stdout={:?}; stderr={:?}",
            hostname.status,
            String::from_utf8_lossy(&hostname.stdout),
            String::from_utf8_lossy(&hostname.stderr)
        );
    }
    let observed_hostname = String::from_utf8_lossy(&hostname.stdout).trim().to_string();
    if observed_hostname != expected_hostname {
        bail!(
            "CLI observed hostname differs: expected {expected_hostname:?}, observed {observed_hostname:?}"
        );
    }
    let configured = identity
        .executable_path
        .as_deref()
        .context("CLI execution identity has no exact executable path")?;
    let path = PathBuf::from(configured);
    if !path.is_absolute() || !path.is_file() {
        bail!("CLI execution identity path is not an absolute executable file: {configured}");
    }
    let canonical = std::fs::canonicalize(&path)
        .with_context(|| format!("canonicalize exact CLI executable {}", path.display()))?;
    if canonical != path {
        bail!(
            "CLI execution identity path is not canonical: declared {}, canonical {}",
            path.display(),
            canonical.display()
        );
    }
    if path.file_name().and_then(|value| value.to_str()) != Some(expected_filename) {
        bail!(
            "CLI execution identity filename differs: expected {expected_filename}, observed {}",
            path.display()
        );
    }
    let expected_sha = identity
        .executable_sha256
        .as_deref()
        .context("CLI execution identity has no executable SHA-256")?;
    let mut file = std::fs::File::open(&path)
        .with_context(|| format!("open exact CLI executable {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash exact CLI executable {}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let observed_sha = hex::encode(digest.finalize());
    if !observed_sha.eq_ignore_ascii_case(expected_sha) {
        bail!(
            "CLI executable SHA-256 changed immediately before use: expected {expected_sha}, observed {observed_sha}"
        );
    }
    let expected_version = identity
        .product_version
        .as_deref()
        .context("CLI execution identity has no exact product version")?;
    let mut version_command = Command::new(&path);
    version_command
        .arg("--version")
        .env_clear()
        .envs(environment);
    let version = version_command
        .output()
        .with_context(|| format!("read exact CLI version from {}", path.display()))?;
    if !version.status.success() {
        bail!(
            "exact CLI version command failed immediately before use: status={}; stdout={:?}; stderr={:?}",
            version.status,
            String::from_utf8_lossy(&version.stdout),
            String::from_utf8_lossy(&version.stderr)
        );
    }
    let observed_version = String::from_utf8_lossy(&version.stdout).trim().to_string();
    if observed_version != expected_version {
        bail!(
            "CLI product version changed immediately before use: expected {expected_version:?}, observed {observed_version:?}"
        );
    }
    Ok(path)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
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

fn capture(session: &TmuxSession) -> Result<String> {
    tmux(
        &session.socket,
        &session.environment,
        &["capture-pane", "-t", &session.name, "-p", "-e", "-S", "-"],
        "capture CLI PTY",
    )
}

fn clean_terminal(value: &str) -> String {
    static ANSI: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]")
            .expect("static ANSI regex")
    });
    ANSI.replace_all(value, "").replace('\r', "")
}

fn run_in_pty(
    session: &TmuxSession,
    binary: &Path,
    argv: &[String],
    output: &Path,
    index: usize,
) -> Result<Invocation> {
    let start_marker = format!("__SPIS_START_{index}__");
    let marker = format!("__SPIS_EXIT_{index}__:");
    let mut invocation = shell_quote(binary.to_string_lossy().as_ref());
    for argument in argv {
        invocation.push(' ');
        invocation.push_str(&shell_quote(argument));
    }

    let command =
        format!("printf '\\n{start_marker}\\n'; {invocation}; printf '\\n{marker}%s\\n' \"$?\"");
    tmux(
        &session.socket,
        &session.environment,
        &["send-keys", "-t", &session.name, "-l", &command],
        "type CLI invocation",
    )?;
    tmux(
        &session.socket,
        &session.environment,
        &["send-keys", "-t", &session.name, "Enter"],
        "submit CLI invocation",
    )?;
    let started = Instant::now();
    let mut timed_out = false;
    let screen = loop {
        std::thread::sleep(Duration::from_millis(100));
        let screen = capture(session)?;
        if screen.contains(&marker) {
            break screen;
        }
        if started.elapsed() >= Duration::from_secs(30) {
            let _ = tmux(
                &session.socket,
                &session.environment,
                &["send-keys", "-t", &session.name, "C-c"],
                "interrupt CLI timeout",
            );
            std::thread::sleep(Duration::from_millis(300));
            timed_out = true;
            break capture(session)?;
        }
    };
    let state_path = format!("states/state-{index:04}.ansi");
    std::fs::write(output.join(&state_path), &screen)?;
    let cleaned_screen = clean_terminal(&screen);
    let exit_status = cleaned_screen.lines().rev().find_map(|line| {
        line.split_once(&marker)
            .and_then(|(_, value)| value.trim().parse().ok())
    });
    let invocation_output = cleaned_screen
        .rsplit_once(&start_marker)
        .map(|(_, tail)| tail)
        .unwrap_or(&cleaned_screen)
        .split_once(&marker)
        .map(|(body, _)| body)
        .unwrap_or_default()
        .trim()
        .to_string();
    Ok(Invocation {
        argv: argv.to_vec(),
        output: invocation_output,
        exit_status,
        timed_out,
        state_path,
    })
}


fn invocation_json(invocation: &Invocation, kind: &str) -> Value {
    let mut digest = Sha256::new();
    digest.update(invocation.output.as_bytes());
    let output_sha256 = hex::encode(digest.finalize());
    json!({
        "kind": kind,
        "delivered_input": {
            "argv": invocation.argv,
        },
        "observed_state": {
            "exit_status": invocation.exit_status,
            "timed_out": invocation.timed_out,
            "raw_terminal_state": invocation.state_path,
            "rendered_output": invocation.output,
            "rendered_output_sha256": output_sha256,
        },
        "argv": invocation.argv,
        "exit_status": invocation.exit_status,
        "timed_out": invocation.timed_out,
        "state": invocation.state_path,
        "output_sha256": output_sha256,
    })
}

fn crawl_one(
    record: &Record,
    manifest: &super::crawl::RuntimeManifest,
    output: &Path,
) -> Result<Value> {
    let fixture = output.join("fixture");
    std::fs::create_dir_all(output.join("states"))?;
    std::fs::create_dir_all(&fixture)?;
    std::fs::write(
        fixture.join("README.txt"),
        "Spis isolated CLI crawl fixture\n",
    )?;
    let home = fixture.join("home");
    let config = home.join(".config");
    let data = home.join(".local/share");
    let cache = home.join(".cache");
    for directory in [&home, &config, &data, &cache] {
        std::fs::create_dir_all(directory)?;
    }
    let environment = isolated_environment(manifest, &fixture)?;
    let binary = verify_exact_executable(manifest, &record.binary, &environment)?;
    let session_name = format!("spis-cli-{}-{}", std::process::id(), record.slug);
    let socket = fixture.join("tmux.sock");
    if socket.exists() {
        std::fs::remove_file(&socket)
            .with_context(|| format!("remove stale private CLI tmux socket {}", socket.display()))?;
    }
    let raw = output.join("terminal.raw");
    tmux(
        &socket,
        &environment,
        &[
            "new-session",
            "-d",
            "-s",
            &session_name,
            "-x",
            "120",
            "-y",
            "40",
            "-c",
            fixture.to_string_lossy().as_ref(),
            "--",
            "/bin/sh",
        ],
        "launch private CLI PTY",
    )?;
    let session = TmuxSession {
        name: session_name,
        socket,
        environment,
    };
    let pipe = format!("cat >> {}", shell_quote(raw.to_string_lossy().as_ref()));
    tmux(
        &session.socket,
        &session.environment,
        &["pipe-pane", "-t", &session.name, "-o", &pipe],
        "record CLI terminal bytes",
    )?;
    std::thread::sleep(Duration::from_millis(200));

    let mut reports = Vec::new();
    let mut index = 1usize;
    let version = run_in_pty(
        &session,
        &binary,
        &["--version".to_string()],
        output,
        index,
    )?;
    index += 1;
    reports.push(invocation_json(&version, "version"));

    let help = run_in_pty(
        &session,
        &binary,
        &["--help".to_string()],
        output,
        index,
    )?;
    index += 1;
    reports.push(invocation_json(&help, "help"));

    let refusal = run_in_pty(
        &session,
        &binary,
        &["--spis-invalid-option".to_string()],
        output,
        index,
    )?;
    index += 1;
    reports.push(invocation_json(&refusal, "refusal"));

    let recovery = run_in_pty(
        &session,
        &binary,
        &["--help".to_string()],
        output,
        index,
    )?;
    reports.push(invocation_json(&recovery, "recovery"));

    let _ = tmux(
        &session.socket,
        &session.environment,
        &["kill-session", "-t", &session.name],
        "close CLI PTY",
    );
    let variant_events: Vec<Value> = reports.iter().enumerate().filter_map(|(position, invocation)| {
        let kind = invocation.get("kind").and_then(Value::as_str)?;
        let event_kind = if invocation.get("timed_out").and_then(Value::as_bool) == Some(true) {
            "crawler_timeout"
        } else if kind == "refusal" && invocation.get("exit_status").and_then(Value::as_i64).is_some_and(|status| status != 0) {
            "parser_refusal"
        } else if kind == "recovery" && invocation.get("exit_status").and_then(Value::as_i64) == Some(0) {
            "recovery_observation"
        } else {
            return None;
        };
        Some(json!({
            "event_id": format!("invocation-{}", position + 1),
            "event_kind": event_kind,
            "argv": invocation.get("argv"),
            "exit_status": invocation.get("exit_status"),
            "timed_out": invocation.get("timed_out"),
            "state": invocation.get("state"),
            "output_sha256": invocation.get("output_sha256"),
            "linked_interaction_id": Value::Null,
        }))
    }).collect();
    let report = json!({
        "schema": "wisent.cli-crawl-run.v1",
        "slug": record.slug,
        "name": record.name,
        "binary": binary,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "runtime_execution_identity": manifest.execution_identity,
        "terminal": {"columns": 120, "rows": 40, "term": "xterm-256color"},
        "invocations": reports,
        "commands_crawled": 4,
        "evidence_observations": {
            "executed_invocations": reports,
            "variant_events": variant_events,
            "terminal_stream": raw.strip_prefix(output).unwrap_or(&raw),
            "canonical_interactions": [],
            "canonical_journey": Value::Null,
            "canonical_accessibility": Value::Null,
            "canonical_motion_analysis": Value::Null,
            "gaps": [
                "Observed failure/recovery/cancellation events are retained independently; no eight interactions have all required variants linked.",
                "No timed terminal cast or rendered state image was retained.",
                "Terminal keyboard accessibility equivalents and reduced-motion behavior remain unmeasured."
            ]
        },
    });
    std::fs::write(
        output.join("crawl.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}

fn hash_artifact(path: &Path) -> Result<(String, u64)> {
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("open CLI artifact {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash CLI artifact {}", path.display()))?;
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
        .context("CLI attempt artifact root has no UTF-8 name")?;
    let archive = root.with_file_name(format!("{attempt_name}.tar.gz"));
    if !archive.is_file() {
        let output = super::crawl::stado_command()
            .args(["storage", "archive"])
            .arg(root)
            .arg(&archive)
            .output()?;
        if !output.status.success() {
            bail!(
                "stado storage archive refused CLI artifacts: {}",
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
            "stado storage put refused CLI artifacts: {}",
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
            "stado storage readback refused CLI artifacts: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let (observed_sha256, observed_bytes) = hash_artifact(&readback)?;
    std::fs::remove_file(&readback)?;
    if observed_sha256 != sha256 || observed_bytes != bytes {
        bail!(
            "CLI artifact readback differs: expected sha256={sha256} bytes={bytes}, observed sha256={observed_sha256} bytes={observed_bytes}"
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
        .context("CLI runtime manifest has no attempt for worker report")?;
    let attempt_id = value
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("CLI runtime manifest has no attempt_id for worker report")?;
    let bindings_file_sha256 = value
        .get("bindings_file_sha256")
        .and_then(Value::as_str)
        .context("CLI runtime manifest has no bindings_file_sha256")?;
    let bindings_sha256 = value
        .get("bindings_sha256")
        .and_then(Value::as_str)
        .context("CLI runtime manifest has no bindings_sha256")?;
    let execution_identity = value
        .get("execution_identity")
        .filter(|identity| identity.is_object())
        .context("CLI runtime manifest has no typed execution_identity")?
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


struct Submission<'a> {
    host: &'a str,
    record: &'a str,
    manifest: &'a super::crawl::RuntimeManifest,
}

fn submit(request: Submission<'_>) -> Result<()> {
    safe_component(request.host, "--host")?;
    safe_component(request.record, "--record")?;
    let _attempt_binding = attempt_root(Path::new("."), request.manifest)?;
    if revision()? != request.manifest.source_revision {
        bail!("CLI coordinator revision does not match immutable runtime manifest");
    }
    let artifact = request.manifest.artifact_uri.clone();
    let output_uri = request.manifest.output_uri.clone();
    let worker = format!(
        "cargo run --release -- crawl-cli --worker --record {} --artifact-uri {} --runtime-manifest-base64 '{}'",
        request.record,
        artifact,
        request.manifest.encoded()?,
    );
    let mut arguments = vec![
        "submit".to_string(),
        worker,
        "--run-id".to_string(),
        request.manifest.stado_run_id.clone(),
        "--pinned-host".to_string(),
        request.host.to_string(),
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
    for (name, reference) in delivery_secret_bindings(request.manifest)? {
        arguments.push("--secret-env".to_string());
        arguments.push(format!("{name}={reference}"));
    }
    let output = super::crawl::stado_command().args(arguments).output()?;
    if !output.status.success() {
        bail!("Stado refused CLI crawl: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    super::crawl::print_submission(
        CATALOG,
        "cli",
        request.host,
        Some(&artifact),
        &output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut host = None;
    let mut record = None;
    let mut worker = false;
    let mut artifact_uri = None;
    let mut runtime_manifest_base64 = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
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
                println!("usage: spis crawl-cli --host TARGET --record SLUG --runtime-manifest-base64 DATA\nworker mode requires the same immutable runtime manifest and exact record.");
                return Ok(());
            }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    let record = record.context("--record is required for one exact per-record job")?;
    let encoded_manifest = runtime_manifest_base64
        .as_deref()
        .context("--runtime-manifest-base64 is required")?;
    let manifest = super::crawl::decode_runtime_manifest(
        encoded_manifest,
        CATALOG,
        "cli",
        Some(&record),
    )?;
    if !worker {
        return submit(Submission {
            host: &host.context("--host is required; CLI crawls execute as pinned Stado jobs")?,
            record: &record,
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
    let root = attempt_root(
        &Path::new("target").join("spis-cli-crawls"),
        &manifest,
    )?;
    let entry = records(Some(&record))?.into_iter().next().context("runtime manifest record is absent")?;
    let output = root.join(&entry.slug);
    std::fs::create_dir_all(&output)?;
    let reports = vec![match crawl_one(&entry, &manifest, &output) {
        Ok(report) => report,
        Err(error) => json!({
            "slug": entry.slug,
            "name": entry.name,
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
        "schema": "wisent.cli-crawl-batch.v1",
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
        bail!("{failures} CLI records could not be crawled");
    }
    let artifact = publish(&root, &artifact_uri)?;
    let worker_report = worker_report(&manifest, artifact)?;
    println!("{}", serde_json::to_string(&worker_report)?);
    Ok(())
}
