//! Real command-line application crawler.
//!
//! The coordinator submits an exact-revision Stado job to a selected host. The
//! worker runs each installed executable inside a real tmux PTY, recursively
//! discovers its documented subcommand tree from help output, records version,
//! help, refusal and recovery states, and can execute explicitly declared safe
//! journeys. Raw terminal bytes, screen states, argv and exit statuses are kept.

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashSet, VecDeque};
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

#[derive(Clone, serde::Deserialize)]
struct Journey {
    name: String,
    argv: Vec<String>,
    #[serde(default)]
    stdin: Option<String>,
    #[serde(default)]
    destructive: bool,
}

#[derive(Default, serde::Deserialize)]
struct JourneyDocument {
    #[serde(default)]
    records: std::collections::HashMap<String, Vec<Journey>>,
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
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{flag} must contain only letters, digits, '.', '-' or '_'");
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

fn binary_for(slug: &str) -> String {
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
    let directory = Path::new(CATALOG).join("references");
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

fn executable(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|directory| directory.join(name))
            .find(|candidate| candidate.is_file())
    })
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn tmux(args: &[&str], context: &str) -> Result<String> {
    let output = Command::new("tmux")
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
}

impl Drop for TmuxSession {
    fn drop(&mut self) {
        let _ = Command::new("tmux")
            .args(["kill-session", "-t", &self.name])
            .status();
    }
}

fn capture(session: &str) -> Result<String> {
    tmux(
        &["capture-pane", "-t", session, "-p", "-e", "-S", "-"],
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
    session: &str,
    binary: &Path,
    argv: &[String],
    stdin: Option<&str>,
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
    if let Some(stdin) = stdin {
        invocation = format!("printf %s {} | {invocation}", shell_quote(stdin));
    }
    let command =
        format!("printf '\\n{start_marker}\\n'; {invocation}; printf '\\n{marker}%s\\n' \"$?\"");
    tmux(
        &["send-keys", "-t", session, "-l", &command],
        "type CLI invocation",
    )?;
    tmux(
        &["send-keys", "-t", session, "Enter"],
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
                &["send-keys", "-t", session, "C-c"],
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

fn subcommands(help: &str) -> Vec<String> {
    static COMMAND: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"^\s{2,}([a-z][a-z0-9_-]*)\s{2,}\S").expect("static command regex")
    });
    let mut in_commands = false;
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    for line in help.lines() {
        let heading = line.trim().to_ascii_lowercase();
        if heading.ends_with("commands:") || matches!(heading.as_str(), "commands" | "subcommands:")
        {
            in_commands = true;
            continue;
        }
        if in_commands && !line.trim().is_empty() && !line.starts_with(char::is_whitespace) {
            in_commands = false;
        }
        if !in_commands {
            continue;
        }
        if let Some(command) = COMMAND.captures(line).and_then(|capture| capture.get(1)) {
            let command = command.as_str();
            if !matches!(command, "help" | "version") && seen.insert(command.to_string()) {
                found.push(command.to_string());
            }
        }
    }
    found
}

fn invocation_json(invocation: &Invocation, kind: &str) -> Value {
    let mut digest = Sha256::new();
    digest.update(invocation.output.as_bytes());
    json!({
        "kind": kind,
        "argv": invocation.argv,
        "exit_status": invocation.exit_status,
        "timed_out": invocation.timed_out,
        "state": invocation.state_path,
        "output_sha256": hex::encode(digest.finalize()),
    })
}

fn crawl_one(
    record: &Record,
    output: &Path,
    journeys: &[Journey],
    max_commands: usize,
    max_depth: usize,
) -> Result<Value> {
    let binary = executable(&record.binary).ok_or_else(|| {
        anyhow!(
            "{} ({}) is not installed on this host",
            record.name,
            record.binary
        )
    })?;
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
    let home_env = format!("HOME={}", home.display());
    let config_env = format!("XDG_CONFIG_HOME={}", config.display());
    let data_env = format!("XDG_DATA_HOME={}", data.display());
    let cache_env = format!("XDG_CACHE_HOME={}", cache.display());
    let git_config_env = format!("GIT_CONFIG_GLOBAL={}", fixture.join("gitconfig").display());
    let kube_env = format!("KUBECONFIG={}", fixture.join("kubeconfig").display());
    let docker_env = format!(
        "DOCKER_HOST=unix://{}",
        fixture.join("docker.sock").display()
    );
    let session = format!("spis-cli-{}-{}", std::process::id(), record.slug);
    let raw = output.join("terminal.raw");
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    tmux(
        &[
            "new-session",
            "-d",
            "-s",
            &session,
            "-x",
            "120",
            "-y",
            "40",
            "-c",
            fixture.to_string_lossy().as_ref(),
            "--",
            "env",
            &home_env,
            &config_env,
            &data_env,
            &cache_env,
            &git_config_env,
            "GIT_CONFIG_NOSYSTEM=1",
            &kube_env,
            &docker_env,
            "AWS_EC2_METADATA_DISABLED=true",
            &shell,
        ],
        "launch CLI PTY",
    )?;
    let _session_guard = TmuxSession {
        name: session.clone(),
    };
    let pipe = format!("cat >> {}", shell_quote(raw.to_string_lossy().as_ref()));
    let _ = tmux(
        &["pipe-pane", "-t", &session, "-o", &pipe],
        "record CLI terminal bytes",
    );
    tmux(
        &[
            "send-keys",
            "-t",
            &session,
            "-l",
            "export TERM=xterm-256color PAGER=cat GIT_PAGER=cat MANPAGER=cat NO_COLOR=1 CI=1",
        ],
        "configure CLI PTY",
    )?;
    tmux(
        &["send-keys", "-t", &session, "Enter"],
        "apply CLI environment",
    )?;
    std::thread::sleep(Duration::from_millis(200));

    let mut reports = Vec::new();
    let mut index = 1usize;
    let version_candidates = [
        vec!["--version".to_string()],
        vec!["version".to_string()],
        vec!["-V".to_string()],
    ];
    for argv in version_candidates {
        let invocation = run_in_pty(&session, &binary, &argv, None, output, index)?;
        index += 1;
        let success = invocation.exit_status == Some(0) && !invocation.timed_out;
        reports.push(invocation_json(&invocation, "version"));
        if success {
            break;
        }
    }

    let mut queue = VecDeque::from([Vec::<String>::new()]);
    let mut seen = HashSet::new();
    while let Some(prefix) = queue.pop_front() {
        if reports.len() >= max_commands || prefix.len() > max_depth || !seen.insert(prefix.clone())
        {
            continue;
        }
        let argv = if record.binary == "git" && prefix.is_empty() {
            vec!["help".to_string(), "-a".to_string()]
        } else {
            prefix
                .iter()
                .cloned()
                .chain(["--help".to_string()])
                .collect()
        };
        let invocation = run_in_pty(&session, &binary, &argv, None, output, index)?;
        index += 1;
        let discovered = subcommands(&invocation.output);
        reports.push(invocation_json(&invocation, "help"));
        if prefix.len() < max_depth {
            for command in discovered {
                let mut child = prefix.clone();
                child.push(command);
                queue.push_back(child);
            }
        }
    }

    let refusal = run_in_pty(
        &session,
        &binary,
        &["--spis-invalid-option".to_string()],
        None,
        output,
        index,
    )?;
    index += 1;
    reports.push(invocation_json(&refusal, "refusal"));

    let recovery = run_in_pty(
        &session,
        &binary,
        &["--help".to_string()],
        None,
        output,
        index,
    )?;
    index += 1;
    reports.push(invocation_json(&recovery, "recovery"));

    for journey in journeys {
        if journey.destructive {
            reports.push(json!({
                "kind": "journey",
                "name": journey.name,
                "argv": journey.argv,
                "blocked": true,
                "reason": "destructive journeys require a non-destructive fixture variant",
            }));
            continue;
        }
        let invocation = run_in_pty(
            &session,
            &binary,
            &journey.argv,
            journey.stdin.as_deref(),
            output,
            index,
        )?;
        index += 1;
        let mut report = invocation_json(&invocation, "journey");
        report["name"] = json!(journey.name);
        reports.push(report);
    }
    let _ = tmux(&["kill-session", "-t", &session], "close CLI PTY");
    let variant_events: Vec<Value> = reports.iter().enumerate().filter_map(|(position, invocation)| {
        let kind = invocation.get("kind").and_then(Value::as_str)?;
        let variant = if invocation.get("timed_out").and_then(Value::as_bool) == Some(true) {
            "cancellation"
        } else if kind == "refusal" && invocation.get("exit_status").and_then(Value::as_i64).is_some_and(|status| status != 0) {
            "failure"
        } else if kind == "recovery" && invocation.get("exit_status").and_then(Value::as_i64) == Some(0) {
            "recovery"
        } else {
            return None;
        };
        Some(json!({
            "event_id": format!("invocation-{}", position + 1),
            "variant": variant,
            "argv": invocation.get("argv"),
            "exit_status": invocation.get("exit_status"),
            "timed_out": invocation.get("timed_out"),
            "state": invocation.get("state"),
            "output_sha256": invocation.get("output_sha256"),
        }))
    }).collect();
    let report = json!({
        "schema": "wisent.cli-crawl-run.v1",
        "slug": record.slug,
        "name": record.name,
        "binary": binary,
        "terminal": {"columns": 120, "rows": 40, "term": "xterm-256color"},
        "invocations": reports,
        "commands_crawled": seen.len(),
        "completed_at": crate::now_iso_utc(),
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

fn publish(root: &Path, uri: &str) -> Result<()> {
    let archive = root.with_extension("tar.gz");
    let status = Command::new("stado")
        .args(["storage", "archive"])
        .arg(root)
        .arg(&archive)
        .status()?;
    if !status.success() {
        bail!("stado storage archive refused CLI artifacts");
    }
    let status = Command::new("stado")
        .args(["storage", "put", "--if-absent", uri])
        .arg(&archive)
        .status()?;
    if !status.success() {
        bail!("stado storage put refused CLI artifacts");
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
        .output()?;
    if !output.status.success() {
        bail!(
            "stado storage put refused CLI journeys: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

struct Submission<'a> {
    host: &'a str,
    record: Option<&'a str>,
    journeys: Option<&'a Path>,
    secret_env: &'a [String],
    max_commands: usize,
    max_depth: usize,
}

fn submit(request: Submission<'_>) -> Result<()> {
    safe_component(request.host, "--host")?;
    if let Some(record) = request.record {
        safe_component(record, "--record")?;
    }
    for binding in request.secret_env {
        let (name, item) = binding
            .split_once('=')
            .ok_or_else(|| anyhow!("--secret-env must be NAME=SKARBIEC_ITEM"))?;
        safe_component(name, "--secret-env name")?;
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
    let artifact = format!("stado://spis-crawls/{CATALOG}/{stamp}.tar.gz");
    let mut worker = format!(
        "cargo run --release -- crawl-cli --worker --max-commands {} --max-depth {} --artifact-uri {}",
        request.max_commands, request.max_depth, artifact
    );
    if let Some(record) = request.record {
        worker.push_str(&format!(" --record {record}"));
    }
    if let Some(journeys) = request.journeys {
        let uri = format!("stado://spis-crawls/{CATALOG}/journeys/{stamp}.json");
        publish_file(journeys, &uri)?;
        let remote = format!("$HOME/.stado/work/{stamp}-cli-journeys.json");
        worker = format!(
            "$HOME/.stado/bin/stado storage get {uri} {remote} && {worker} --journeys {remote}"
        );
    }
    let output_uri = format!("stado://spis-crawls/{CATALOG}/{stamp}/job-output");
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
    let output = Command::new("stado").args(arguments).output()?;
    if !output.status.success() {
        bail!(
            "Stado refused CLI crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
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
    let mut journeys = None;
    let mut secret_env = Vec::new();
    let mut worker = false;
    let mut artifact_uri = None;
    let mut max_commands = 250usize;
    let mut max_depth = 4usize;
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
            "--journeys" => {
                i += 1;
                journeys = Some(PathBuf::from(
                    rest.get(i).context("--journeys needs a value")?,
                ));
            }
            "--secret-env" => {
                i += 1;
                secret_env.push(rest.get(i).context("--secret-env needs a value")?.clone());
            }
            "--max-commands" => {
                i += 1;
                max_commands = rest
                    .get(i)
                    .context("--max-commands needs a value")?
                    .parse()?;
            }
            "--max-depth" => {
                i += 1;
                max_depth = rest.get(i).context("--max-depth needs a value")?.parse()?;
            }
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            "--worker" => worker = true,
            "--help" | "-h" => {
                println!("usage: spis crawl-cli --host TARGET [--record SLUG] [--journeys FILE] [--secret-env NAME=SKARBIEC_ITEM] [--max-commands N] [--max-depth N]\nworker mode: spis crawl-cli --worker [--artifact-uri stado://...]");
                return Ok(());
            }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    if max_commands == 0 || max_commands > 5_000 || max_depth > 16 {
        bail!("--max-commands must be 1..5000 and --max-depth must be 0..16");
    }
    if !worker {
        return submit(Submission {
            host: &host.context("--host is required; CLI crawls execute as pinned Stado jobs")?,
            record: record.as_deref(),
            journeys: journeys.as_deref(),
            secret_env: &secret_env,
            max_commands,
            max_depth,
        });
    }
    if host.is_some() || !secret_env.is_empty() {
        bail!("--host and --secret-env are coordinator-only");
    }
    let journeys_document: JourneyDocument = match journeys {
        Some(path) => serde_json::from_slice(&std::fs::read(path)?)?,
        None => JourneyDocument::default(),
    };
    let stamp = crate::now_iso_utc().replace(':', "-");
    let root = Path::new("target").join("spis-cli-crawls").join(&stamp);
    std::fs::create_dir_all(&root)?;
    let mut reports = Vec::new();
    for entry in records(record.as_deref())? {
        let output = root.join(&entry.slug);
        std::fs::create_dir_all(&output)?;
        let declared = journeys_document
            .records
            .get(&entry.slug)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        reports.push(match crawl_one(&entry, &output, declared, max_commands, max_depth) {
            Ok(report) => report,
            Err(error) => json!({"slug": entry.slug, "name": entry.name, "status": "failed", "error": error.to_string()}),
        });
    }
    let failures = reports
        .iter()
        .filter(|report| report.get("status").and_then(Value::as_str) == Some("failed"))
        .count();
    let summary = json!({
        "schema": "wisent.cli-crawl-batch.v1",
        "records": reports,
        "failed": failures,
        "completed_at": crate::now_iso_utc(),
    });
    std::fs::write(
        root.join("batch.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    if let Some(uri) = artifact_uri {
        if !uri.starts_with("stado://spis-crawls/cli-examples/") {
            bail!("--artifact-uri must be under stado://spis-crawls/cli-examples/");
        }
        publish(&root, &uri)?;
    }
    println!("{}", serde_json::to_string_pretty(&summary)?);
    if failures > 0 {
        bail!("{failures} CLI records could not be crawled");
    }
    Ok(())
}
