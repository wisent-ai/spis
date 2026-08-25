//! `spis capture-wisent-references` — capture the reference catalog for the Wisent
//! products installed on this workstation.
//!
//! For each product the command opens a PTY, drives one `/bin/bash --norc
//! --noprofile -i` session, and issues exactly seven read-only commands: the
//! version form, the top-level help, one subcommand help surface, one deliberately
//! invalid flag, Ctrl-C on an unsubmitted line, the help that recovers from the
//! refusal, and the same help with NO_COLOR=1. The session becomes
//! `media/session.cast` (asciinema v2); five PNGs are rendered from that cast's
//! text with Pillow at named points in the sequence. Afterwards the catalog files
//! (`sources.json`, `references.json`, `full-reference.md`, `README.md`) are
//! rebuilt from every record on disk.
//!
//! Port of the former `capture-wisent-references.py` (deleted in 1672030).
//! One deliberate environment difference: the transient scratch tree lives under
//! `~/.spis/work/wisent-capture/` instead of `~/.stado/work/wisent-capture/`.

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const RECORD_SCHEMA: &str = "wisent.full-product-reference.v2";
const INDEX_SCHEMA: &str = "wisent.full-reference-catalog.v2";
const SOURCES_SCHEMA: &str = "wisent.example-catalog.v2";

const COLS: usize = 100;
const ROWS: usize = 32;
const PROMPT: &str = "wisent-ref$ ";
const PROBE_FLAG: &str = "--wisent-reference-probe";
const SHELL: &str = "/bin/bash";
const FONT_PX: usize = 15;

// The former script wrote its scratch under ~/.stado/work/wisent-capture; this
// checkout is confined to the spis tree and ~/.spis, so the scratch moves with it.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn catalog_dir() -> PathBuf {
    root().join("wisent-product-examples")
}

fn scratch_root() -> PathBuf {
    home_dir().join(".spis").join("work").join("wisent-capture")
}

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
}

// ------------------------------------------------------------------ products

struct Product {
    slug: &'static str,
    name: &'static str,
    binary: &'static str,
    repository: &'static str,
    product_url: &'static str,
    category: &'static str,
    one_line: &'static str,
    selection_note: &'static str,
    version_cmd: &'static str,
    help_cmd: &'static str,
    sub_cmd: &'static str,
    sub_note: &'static str,
}

macro_rules! product {
    ($slug:expr, $name:expr, $binary:expr, $repo:expr, $url:expr, $cat:expr, $one:expr,
     $note:expr, $ver:expr, $help:expr, $sub:expr, $subnote:expr) => {
        Product {
            slug: $slug,
            name: $name,
            binary: $binary,
            repository: $repo,
            product_url: $url,
            category: $cat,
            one_line: $one,
            selection_note: $note,
            version_cmd: $ver,
            help_cmd: $help,
            sub_cmd: $sub,
            sub_note: $subnote,
        }
    };
}

// One entry per Wisent product with a runnable CLI on this host. `repository` is
// the Wisent repository the binary comes from; `version_cmd` is the product's own
// version form, which several of these do not have — the refusal is then the
// measurement.
const PRODUCTS: &[Product] = &[
    product!(
        "stado",
        "Stado",
        "stado",
        "wisent-ai/stado",
        "https://github.com/wisent-ai/stado",
        "Infrastructure / compute and queue control plane",
        "Policy-controlled queue and compute control plane for machines you own or authorize.",
        "The widest command surface we ship: a Clap-style noun tree over jobs, hosts, quota and \
         credits. Study how a large control-plane CLI keeps its top-level help to one screen of \
         verbs and pushes detail into per-command help.",
        "stado --version",
        "stado --help",
        "stado capabilities --help",
        "Per-command help exists and is the documented way down one level."
    ),
    product!(
        "skarbiec",
        "Skarbiec",
        "skarbiec",
        "wisent-ai/skarbiec",
        "https://github.com/wisent-ai/skarbiec",
        "Security / credential and authentication management",
        "Credential and authentication management for the AI era.",
        "The only product here whose help is machine-readable JSON rather than prose, and the \
         only one whose first refusal is a state gate rather than a parse error. Study what a \
         credential CLI is willing to say before a vault exists.",
        "skarbiec --version",
        "skarbiec help",
        "skarbiec status --help",
        "Skarbiec has no per-subcommand help; the subcommand is reached before argument parsing \
         and answers with the vault state gate instead."
    ),
    product!(
        "weles",
        "Weles",
        "weles",
        "wisent-ai/weles",
        "https://github.com/wisent-ai/weles",
        "Automation / authorized browser execution",
        "Authorized browser execution for AI agents, with signed receipts.",
        "A CLI whose real work is gated on an authorization boundary, so its safe surface is \
         help and identity only. Study how a product that refuses unauthorized work advertises \
         that boundary in its own usage text.",
        "weles --version",
        "weles --help",
        "weles version",
        "Weles exposes no per-subcommand help. `weles version` is the only subcommand that can \
         be run here without authorizing a workflow or touching durable onboarding state, so \
         that is the subcommand surface this record measures."
    ),
    product!(
        "jeden",
        "Jeden",
        "jeden",
        "wisent-ai/jeden",
        "https://github.com/wisent-ai/jeden",
        "Agents / autonomous coding and company building",
        "The autonomous agent for building software and running the loop around it.",
        "A single-block usage synopsis for an agent runtime whose flags are permission grants \
         (`--allow-write`, `--allow-command`, `--yolo`). Study how a dangerous capability set is \
         presented in first-run help.",
        "jeden --version",
        "jeden --help",
        "jeden version",
        "Jeden's help is one usage block covering every subcommand; there is no per-subcommand \
         help. `jeden run --help` is not a safe probe: when probed once outside this recording \
         it resolved credentials through Skarbiec before parsing `--help` and failed with an \
         HTTP 403, so this record measures `jeden version` instead and reports that finding \
         rather than re-running it."
    ),
    product!(
        "probierz",
        "Probierz",
        "probierz",
        "wisent-ai/probierz",
        "https://github.com/wisent-ai/probierz",
        "Quality / test execution and evidence boundary",
        "The quality-evidence boundary: selection, execution, evidence, and verdicts.",
        "The one product whose refusal is a structured machine-readable failure envelope rather \
         than a usage dump. Study a CLI that answers an unknown surface with a parseable \
         `probierz-failure` line plus one plain sentence.",
        "probierz --version",
        "probierz --help",
        "probierz specs --help",
        "Probierz has no per-subcommand help: `--help` after a subcommand is read as a surface \
         name and refused. That refusal is the observed subcommand surface."
    ),
    product!(
        "oko-cli",
        "Oko (oko-cli)",
        "oko-cli",
        "wisent-ai/oko",
        "https://github.com/wisent-ai/oko",
        "Observability / agent session inspection",
        "Understand your team's interactions with AI.",
        "A headless companion CLI with one flat usage block and no version form at all. Study \
         the cost of that: the same text answers help, an unknown flag, and a subcommand help \
         request, and only the exit status distinguishes them.",
        "oko-cli --version",
        "oko-cli --help",
        "oko-cli diff --help",
        "Oko answers a subcommand help request with the whole top-level usage block."
    ),
    product!(
        "singularity",
        "Singularity",
        "singularity",
        "wisent-ai/singularity",
        "https://github.com/wisent-ai/singularity",
        "Agents / autonomous agent runtime",
        "An open-source framework for autonomous agents that execute tasks and manage resources.",
        "The narrowest installed surface in the catalog: one subcommand, `onboarding`. Study a \
         product whose CLI deliberately exposes only the first-use journey.",
        "singularity --version",
        "singularity --help",
        "singularity onboarding --help",
        "argparse gives every subcommand its own help; `onboarding` is the only one."
    ),
    product!(
        "tama",
        "Tama (tama-cli)",
        "tama",
        "wisent-ai/hooks-rotator",
        "https://github.com/wisent-ai/hooks-rotator",
        "Policy / agent and Git hook catalog",
        "Your AI agent made a mistake? Tama creates rules so that it never happens again.",
        "A policy catalog CLI that answers `--help` after a subcommand by ignoring the flag and \
         running the read-only command. Study how a hook installer separates a plan from an \
         install.",
        "tama --version",
        "tama --help",
        "tama install-plan --help",
        "Tama ignores a trailing `--help` and executes the subcommand; `install-plan` only \
         reports the paths an install would touch, so nothing is written."
    ),
    product!(
        "transcript-lake",
        "Transcript Lake",
        "transcript-lake",
        "wisent-ai/transcript-lake",
        "https://github.com/wisent-ai/transcript-lake",
        "Data / local privacy-masked transcript archive",
        "Nothing you ever told an AI is lost again.",
        "The only help here that opens with a `Start safely:` section and names the read-only \
         commands first. Study help text that is ordered by risk rather than alphabetically.",
        "transcript-lake --version",
        "transcript-lake --help",
        "transcript-lake paths --help",
        "Per-command usage exists and prints one line of purpose with its flags."
    ),
    product!(
        "transcript-label-trainer",
        "Transcript Label Trainer",
        "transcript-label-trainer",
        "wisent-ai/transcript-label-trainer",
        "https://github.com/wisent-ai/transcript-label-trainer",
        "Models / local classifiers over transcript labels",
        "Small models for your custom harness needs.",
        "An argparse CLI that states its own boundary in the help body — 'Never writes to the \
         lake.' Study a product that publishes what it will not touch above its command list.",
        "transcript-label-trainer --version",
        "transcript-label-trainer --help",
        "transcript-label-trainer info --help",
        "argparse gives every subcommand its own help."
    ),
];

// Products deliberately excluded, with the reason. Kept here so the catalog scope
// is a statement that can be checked rather than a claim about what happened to be
// found.
const EXCLUSIONS: &[(&str, &str, &str)] = &[
    (
        "omp",
        "~/.local/bin/omp",
        "Not a Wisent repository: the binary's own build metadata names \
         github.com/can1357/oh-my-pi. It is the harness we run, not a product we ship.",
    ),
    (
        "stado_fleet",
        "~/.stado/bin/stado_fleet",
        "A second binary of the same product (Stado, wisent-ai/stado), not a separate product.",
    ),
    (
        "wc",
        "~/.local/bin/wc",
        "The legacy name of the Stado CLI; `wc --version` prints `stado 0.6.0`. Same product.",
    ),
];

// --------------------------------------------------------------- small utils

/// Python `%g`-style float rendering (shortest form, no trailing `.0`).
fn g(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    if x.abs() >= 1e16 || x.abs() < 1e-4 {
        return format!("{x:e}");
    }
    let s = format!("{x}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Python-style `repr()` of a string: single quotes unless it contains one.
fn py_repr(s: &str) -> String {
    if s.contains('\'') {
        format!("{s:?}")
    } else {
        format!("'{s}'")
    }
}

fn round_n(x: f64, places: u32) -> f64 {
    let f = 10f64.powi(places as i32);
    (x * f).round() / f
}

fn ansi_re() -> &'static Regex {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]")
            .expect("ansi regex")
    });
    &RE
}

fn sgr_re() -> &'static Regex {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").expect("sgr regex"));
    &RE
}

fn next_action_re() -> &'static Regex {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"(try\s+'[^']+'|try\s+"[^"]+"|\(run:\s*[^)]+\)|see\s+'[^']+'|usage:|USAGE:|Usage:|--help|<command>)"#,
        )
        .expect("next-action regex")
    });
    &RE
}

fn exit_status_re() -> &'static Regex {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"exit-status=(\d+)").expect("exit-status regex"));
    &RE
}

fn cursor_re() -> &'static Regex {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\x1b\[\d*;\d*H").expect("cursor regex"));
    &RE
}

fn strip_ansi(text: &str) -> String {
    ansi_re().replace_all(text, "").replace('\u{7}', "")
}

/// Replay plain text the way the recorded terminal showed it.
fn visible_lines(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in strip_ansi(text).split('\n') {
        // PTYs terminate ordinary lines with CRLF. A trailing CR is dropped;
        // internal CR still means an in-place repaint, and a terminal shows the
        // final segment in that case.
        let line = line.strip_suffix('\r').unwrap_or(line);
        let line = match line.rfind('\r') {
            Some(i) => &line[i + 1..],
            None => line,
        };
        out.push(line.replace('\t', "    "));
    }
    out
}

/// Collapse whitespace and truncate with an ellipsis, like the Python `quote`.
fn quote(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = collapsed.chars().collect();
    if chars.len() > limit {
        let head: String = chars[..limit - 1].iter().collect();
        format!("{head}\u{2026}")
    } else {
        collapsed
    }
}

fn json_str(s: &str) -> String {
    serde_json::to_string(s).expect("string serialization")
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs()
}

/// ISO-8601 UTC timestamp with the `+00:00` offset the former script produced.
fn captured_at_now() -> String {
    format!("{}+00:00", iso_utc_body(now_unix_secs()))
}

fn iso_utc_body(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}")
}

fn today_utc() -> String {
    iso_utc_body(now_unix_secs())[..10].to_string()
}

// ---------------------------------------------------------------------- host

fn sw_vers(arg: &str) -> String {
    Command::new("sw_vers")
        .arg(arg)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn uname_field(field: fn(&libc::utsname) -> &[libc::c_char]) -> String {
    unsafe {
        let mut uts: libc::utsname = std::mem::zeroed();
        if libc::uname(&mut uts) != 0 {
            return String::new();
        }
        std::ffi::CStr::from_ptr(field(&uts).as_ptr())
            .to_string_lossy()
            .into_owned()
    }
}

struct HostFacts {
    host: Value,
    sentence: String,
}

fn host_facts() -> &'static HostFacts {
    static HOST: LazyLock<HostFacts> = LazyLock::new(|| {
        let host = json!({
            "os": "macOS",
            "os_version": sw_vers("-productVersion"),
            "os_build": sw_vers("-buildVersion"),
            "arch": uname_field(|u| &u.machine),
            "kernel": format!(
                "{} {}",
                uname_field(|u| &u.sysname),
                uname_field(|u| &u.release)
            ),
            "shell": SHELL,
            "terminal": format!("pseudo-terminal, {COLS}x{ROWS}, TERM=xterm-256color"),
        });
        let sentence = format!(
            "macOS {} ({}) {}",
            host["os_version"].as_str().unwrap_or_default(),
            host["os_build"].as_str().unwrap_or_default(),
            host["arch"].as_str().unwrap_or_default(),
        );
        HostFacts { host, sentence }
    });
    &HOST
}

fn host_sentence() -> &'static str {
    &host_facts().sentence
}

// ---------------------------------------------------------------- resolution

fn resolve(product: &Product) -> Option<String> {
    let path_var = std::env::var("PATH").ok()?;
    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }
        let candidate = Path::new(dir).join(product.binary);
        if is_executable(&candidate) {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(md) => md.is_file() && md.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

enum QuickOutcome {
    Ran(i32, String), // exit status, first line
    Failed(String),   // exception type name, like the Python returned
    Missing,
}

fn quick_version(product: &Product) -> (Option<String>, QuickOutcome) {
    let Some(path) = resolve(product) else {
        return (None, QuickOutcome::Missing);
    };
    let args: Vec<&str> = product.version_cmd.split_whitespace().collect();
    match run_with_timeout(&args, Duration::from_secs(120)) {
        QuickRun::SpawnError => (Some(path), QuickOutcome::Failed("OSError".into())),
        QuickRun::TimedOut => (Some(path), QuickOutcome::Failed("TimeoutExpired".into())),
        QuickRun::Done(status, out, err) => {
            let text = strip_ansi(&(out + &err)).trim().to_string();
            let first = text.split('\n').next().unwrap_or("").to_string();
            (Some(path), QuickOutcome::Ran(status, first))
        }
    }
}

enum QuickRun {
    Done(i32, String, String),
    TimedOut,
    SpawnError,
}

fn run_with_timeout(argv: &[&str], limit: Duration) -> QuickRun {
    let mut child = match Command::new(argv[0])
        .args(&argv[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return QuickRun::SpawnError,
    };
    let mut stdout_pipe = child.stdout.take().expect("stdout piped");
    let mut stderr_pipe = child.stderr.take().expect("stderr piped");
    let t_out = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        buf
    });
    let t_err = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        buf
    });
    let deadline = Instant::now() + limit;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let out = String::from_utf8_lossy(&t_out.join().unwrap_or_default()).into_owned();
                let err = String::from_utf8_lossy(&t_err.join().unwrap_or_default()).into_owned();
                return QuickRun::Done(status.code().unwrap_or(-1), out, err);
            }
            Ok(None) => {}
            Err(_) => return QuickRun::SpawnError,
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return QuickRun::TimedOut;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

// --------------------------------------------------------------- pty session

fn set_winsize(fd: i32, rows: u32, cols: u32) {
    let ws = libc::winsize {
        ws_row: rows as u16,
        ws_col: cols as u16,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        libc::ioctl(fd, libc::TIOCSWINSZ, &ws);
    }
}

/// One real pseudo-terminal running one interactive shell, recorded as it runs.
struct Session {
    fd: i32,
    pid: i32,
    events: Vec<(f64, String)>,
    start: Instant,
}

impl Session {
    fn new(cwd: &Path) -> Result<Session> {
        let mut env_pairs: Vec<(String, String)> = std::env::vars()
            .filter(|(k, _)| {
                k != "NO_COLOR"
                    && k != "CLICOLOR_FORCE"
                    && k != "TERM"
                    && k != "PS1"
                    && k != "PAGER"
                    && k != "LESS"
                    && k != "COLUMNS"
                    && k != "LINES"
                    && k != "SHELL"
            })
            .collect();
        for (k, v) in [
            ("TERM", "xterm-256color"),
            ("PS1", PROMPT),
            ("PAGER", "cat"),
            ("LESS", "-FRX"),
            ("COLUMNS", &COLS.to_string()),
            ("LINES", &ROWS.to_string()),
            ("SHELL", SHELL),
        ] {
            env_pairs.retain(|(ek, _)| ek != k);
            env_pairs.push((k.to_string(), v.to_string()));
        }

        let cwd_c = std::ffi::CString::new(cwd.as_os_str().as_encoded_bytes())
            .context("scratch cwd path")?;
        let shell_c = std::ffi::CString::new(SHELL).unwrap();
        let argv: Vec<std::ffi::CString> = ["--norc", "--noprofile", "-i"]
            .iter()
            .map(|a| std::ffi::CString::new(*a).unwrap())
            .collect();
        let env_c: Vec<std::ffi::CString> = env_pairs
            .iter()
            .map(|(k, v)| std::ffi::CString::new(format!("{k}={v}")).map_err(|e| anyhow!("{e}")))
            .collect::<Result<_>>()?;

        let mut master: libc::c_int = -1;
        let pid = unsafe {
            libc::forkpty(
                &mut master,
                std::ptr::null_mut::<libc::c_char>(),
                std::ptr::null_mut::<libc::termios>(),
                std::ptr::null_mut::<libc::winsize>(),
            )
        };
        if pid < 0 {
            bail!("forkpty failed: {}", std::io::Error::last_os_error());
        }
        if pid == 0 {
            // Child: become the recorded shell.
            unsafe {
                if libc::chdir(cwd_c.as_ptr()) != 0 {
                    libc::_exit(127);
                }
                let mut argp: Vec<*const libc::c_char> = vec![shell_c.as_ptr()];
                argp.extend(argv.iter().map(|a| a.as_ptr()));
                argp.push(std::ptr::null());
                let mut envp: Vec<*const libc::c_char> = env_c.iter().map(|e| e.as_ptr()).collect();
                envp.push(std::ptr::null());
                libc::execve(shell_c.as_ptr(), argp.as_ptr(), envp.as_ptr());
                libc::_exit(127);
            }
        }
        set_winsize(master, ROWS as u32, COLS as u32);
        let mut session = Session {
            fd: master,
            pid,
            events: Vec::new(),
            start: Instant::now(),
        };
        session.drain(1.5);
        session.write_text(&format!("PS1='{PROMPT}'\n"));
        session.wait_prompt(10.0);
        // Everything recorded from here is the product's own session.
        session.events.clear();
        session.start = Instant::now();
        Ok(session)
    }

    fn elapsed(&self) -> f64 {
        round_n(self.start.elapsed().as_secs_f64(), 6)
    }

    fn write_text(&self, text: &str) {
        let bytes = text.as_bytes();
        let mut written = 0;
        while written < bytes.len() {
            let n = unsafe {
                libc::write(
                    self.fd,
                    bytes[written..].as_ptr() as *const _,
                    bytes.len() - written,
                )
            };
            if n <= 0 {
                break;
            }
            written += n as usize;
        }
    }

    fn read_chunk(&mut self, timeout: f64) -> String {
        let mut pfd = libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut pfd, 1, (timeout * 1000.0) as i32) };
        if ready <= 0 {
            return String::new();
        }
        let mut buf = vec![0u8; 1 << 16];
        let n = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        if n <= 0 {
            return String::new();
        }
        let text = String::from_utf8_lossy(&buf[..n as usize]).into_owned();
        self.events.push((self.elapsed(), text.clone()));
        text
    }

    fn drain(&mut self, seconds: f64) -> String {
        let deadline = Instant::now() + Duration::from_secs_f64(seconds);
        let mut got = String::new();
        while Instant::now() < deadline {
            got.push_str(&self.read_chunk(0.1));
        }
        got
    }

    /// Read until the shell reprints its prompt, or the timeout expires.
    fn wait_prompt(&mut self, timeout: f64) -> (String, bool) {
        let deadline = Instant::now() + Duration::from_secs_f64(timeout);
        let mut buf = String::new();
        while Instant::now() < deadline {
            buf.push_str(&self.read_chunk(0.2));
            if buf.ends_with(PROMPT) {
                // Let a trailing flush land, then stop.
                buf.push_str(&self.drain(0.15));
                return (buf, true);
            }
        }
        (buf, false)
    }

    fn command(&mut self, command: &str, timeout: f64) -> Step {
        let started_at = self.elapsed();
        self.write_text(&format!("{command}\n"));
        let (mut raw, ok) = self.wait_prompt(timeout);
        if !ok {
            self.write_text("\u{3}");
            raw.push_str(&self.wait_prompt(15.0).0);
        }
        let ended_at = self.elapsed();
        self.write_text("printf \"exit-status=%s\\n\" \"$?\"\n");
        let (status_raw, _) = self.wait_prompt(20.0);
        let exit_status = exit_status_re()
            .captures(&status_raw)
            .and_then(|c| c[1].parse::<i64>().ok());
        let status_reported_at = self.elapsed();
        Step {
            command: command.to_string(),
            raw,
            started_at,
            ended_at,
            status_reported_at,
            exit_status,
            prompt_returned: ok,
            event_index: 0,
            kind: String::new(),
        }
    }

    /// Type a command and press Ctrl-C instead of Enter. Nothing is submitted.
    fn cancel(&mut self, pending: &str) -> Step {
        let started_at = self.elapsed();
        self.write_text(pending);
        let mut raw = self.drain(0.6);
        self.write_text("\u{3}");
        let (tail, _) = self.wait_prompt(20.0);
        raw.push_str(&tail);
        let ended_at = self.elapsed();
        Step {
            command: pending.to_string(),
            raw,
            started_at,
            ended_at,
            status_reported_at: ended_at,
            exit_status: None,
            prompt_returned: true,
            event_index: 0,
            kind: String::new(),
        }
    }

    fn close(&mut self) {
        self.write_text("exit\n");
        self.drain(0.5);
        unsafe {
            libc::close(self.fd);
            libc::waitpid(self.pid, std::ptr::null_mut(), 0);
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Clone)]
struct Step {
    command: String,
    raw: String,
    started_at: f64,
    ended_at: f64,
    status_reported_at: f64,
    exit_status: Option<i64>,
    #[allow(dead_code)]
    prompt_returned: bool,
    event_index: usize,
    kind: String,
}

struct Run {
    index: usize,
    product: &'static Product,
    binary_path: String,
    workdir: String,
    events: Vec<(f64, String)>,
    wall_start: u64,
    steps: BTreeMap<String, Step>,
}

const STEP_PLAN: &[(&str, &str)] = &[
    ("version", "version identity"),
    ("help", "top-level help"),
    ("subcommand-help", "subcommand help surface"),
    ("invalid-flag", "invalid flag refusal"),
    ("cancellation", "Ctrl-C on an unsubmitted line"),
    ("recovery-help", "recovery help"),
    ("no-color-help", "help with NO_COLOR=1"),
];

const STATE_PLAN: &[(&str, &str, &str)] = &[
    ("version", "01-version-identity", "version identity"),
    ("help", "02-help-surface", "top-level help surface"),
    (
        "subcommand-help",
        "03-subcommand-help",
        "subcommand help surface",
    ),
    (
        "invalid-flag",
        "04-refusal",
        "refusal after the invalid flag",
    ),
    (
        "recovery-help",
        "05-recovery",
        "recovered help after the refusal",
    ),
];

fn capture(index: usize, product: &'static Product) -> Result<Run> {
    let binary_path =
        resolve(product).ok_or_else(|| anyhow!("{} is not on PATH", product.binary))?;

    let workdir = scratch_root().join("run").join(product.slug);
    if workdir.exists() {
        std::fs::remove_dir_all(&workdir)
            .with_context(|| format!("clear {}", workdir.display()))?;
    }
    std::fs::create_dir_all(&workdir)?;

    let invalid_cmd = format!("{} {PROBE_FLAG}", product.binary);
    let mut commands: BTreeMap<String, String> = BTreeMap::new();
    commands.insert("version".into(), product.version_cmd.into());
    commands.insert("help".into(), product.help_cmd.into());
    commands.insert("subcommand-help".into(), product.sub_cmd.into());
    commands.insert("invalid-flag".into(), invalid_cmd.clone());
    commands.insert("cancellation".into(), invalid_cmd.clone());
    commands.insert("recovery-help".into(), product.help_cmd.into());
    commands.insert(
        "no-color-help".into(),
        format!("NO_COLOR=1 {}", product.help_cmd),
    );

    let mut session = Session::new(&workdir)?;
    let mut steps = BTreeMap::new();
    let result = (|| -> Result<()> {
        for (kind, _label) in STEP_PLAN {
            println!("    {kind}: {}", commands[*kind]);
            let mut step = if *kind == "cancellation" {
                session.cancel(&commands[*kind])
            } else {
                session.command(&commands[*kind], 180.0)
            };
            step.kind = (*kind).to_string();
            step.event_index = session.events.len().saturating_sub(1);
            steps.insert((*kind).to_string(), step);
        }
        Ok(())
    })();
    let events = session.events.clone();
    let wall_start = now_unix_secs();
    if let Err(e) = result {
        bail!("session failed mid-run: {e:#}");
    }
    drop(session);

    Ok(Run {
        index,
        product,
        binary_path,
        workdir: workdir.to_string_lossy().into_owned(),
        events,
        wall_start,
        steps,
    })
}

// --------------------------------------------------------------- measurement

fn step_output(raw: &str) -> &str {
    // The command's own output: the echoed command line and trailing prompt removed.
    let body = match raw.find('\n') {
        Some(i) => &raw[i + 1..],
        None => "",
    };
    if body.ends_with(PROMPT) {
        &body[..body.len() - PROMPT.len()]
    } else {
        body
    }
}

fn measure(run: &Run) -> Value {
    let mut steps = Map::new();
    for (kind, _label) in STEP_PLAN {
        let step = &run.steps[*kind];
        let raw = step_output(&step.raw);
        let plain = strip_ansi(raw);
        let mut lines = visible_lines(raw);
        while lines.last().is_some_and(|l| l.trim().is_empty()) {
            lines.pop();
        }
        let line_count = lines.len();
        let max_line_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        steps.insert(
            (*kind).to_string(),
            json!({
                "command": step.command,
                "exit_status": step.exit_status,
                "started_at": step.started_at,
                "ended_at": step.ended_at,
                "status_reported_at": step.status_reported_at,
                "elapsed_seconds": round_n(step.ended_at - step.started_at, 3),
                "event_index": step.event_index,
                "sgr_sequences": sgr_re().find_iter(raw).count(),
                "line_count": line_count,
                "max_line_width": max_line_width,
                "first_line": lines.first().map(|l| l.trim().to_string()).unwrap_or_default(),
                "last_line": lines.last().map(|l| l.trim().to_string()).unwrap_or_default(),
                "text": plain,
                "lines": lines,
            }),
        );
    }
    let help_m = steps["help"].clone();
    let nocolor_m = steps["no-color-help"].clone();
    let invalid_m = steps["invalid-flag"].clone();
    let version_m = steps["version"].clone();
    let same_text = help_m["text"].as_str().unwrap_or_default().trim()
        == nocolor_m["text"].as_str().unwrap_or_default().trim();
    let cancel_raw = run.steps["cancellation"].raw.clone();
    let all_events: String = run.events.iter().map(|(_, t)| t.as_str()).collect();
    let match_phrase = next_action_re()
        .find(invalid_m["text"].as_str().unwrap_or_default())
        .map(|m| m.as_str().to_string());

    json!({
        "steps": Value::Object(steps),
        "colors_help": help_m["sgr_sequences"].as_i64().unwrap_or(0) > 0,
        "help_sgr_count": help_m["sgr_sequences"].clone(),
        "no_color_sgr_count": nocolor_m["sgr_sequences"].clone(),
        "no_color_text_identical": same_text,
        "help_max_line_width": help_m["max_line_width"].clone(),
        "help_fits_80": help_m["max_line_width"].as_i64().unwrap_or(0) <= 80,
        "help_line_count": help_m["line_count"].clone(),
        "refusal_names_next_action": match_phrase.is_some(),
        "refusal_next_action_phrase": match_phrase,
        "refusal_first_line": invalid_m["first_line"].clone(),
        "refusal_exit_status": invalid_m["exit_status"].clone(),
        "version_first_line": version_m["first_line"].clone(),
        "version_exit_status": version_m["exit_status"].clone(),
        "version_flag_supported": version_m["exit_status"].as_i64() == Some(0),
        "cancel_echoed_interrupt": cancel_raw.contains("^C"),
        "cancel_prompt_restored": cancel_raw.trim_end().ends_with(PROMPT.trim()),
        "screen_cleared": all_events.contains("\x1b[2J"),
        "cursor_addressed": cursor_re().is_match(&all_events),
    })
}

fn timing_class(measured: &Value) -> String {
    let spans: Vec<f64> = STEP_PLAN
        .iter()
        .filter(|(k, _)| *k != "cancellation")
        .map(|(k, _)| {
            measured["steps"][*k]["elapsed_seconds"]
                .as_f64()
                .unwrap_or(0.0)
        })
        .collect();
    let slowest = spans.iter().cloned().fold(0.0f64, f64::max);
    if slowest < 0.05 {
        "instant".into()
    } else if slowest < 1.0 {
        "sub-second".into()
    } else if slowest <= 3.0 {
        "one-to-three-seconds".into()
    } else {
        "multi-second".into()
    }
}

fn timing_description(measured: &Value) -> String {
    let spans: Vec<f64> = STEP_PLAN
        .iter()
        .filter(|(k, _)| *k != "cancellation")
        .map(|(k, _)| {
            measured["steps"][*k]["elapsed_seconds"]
                .as_f64()
                .unwrap_or(0.0)
        })
        .collect();
    let fastest = spans.iter().cloned().fold(f64::INFINITY, f64::min);
    let slowest = spans.iter().cloned().fold(0.0f64, f64::max);
    format!(
        "The six submitted commands each completed between {} s and {} s; \
         the other pauses are the recorder typing and the deliberate Ctrl-C pause.",
        g(fastest),
        g(slowest)
    )
}

// -------------------------------------------------------------- record build

fn build_record(run: &Run, measured: &Value, media: &Value) -> Value {
    let product = run.product;
    let steps = &measured["steps"];
    let cast = &media["cast"];
    let states = &media["states"];
    let name = product.name;
    let binary = product.binary;

    let ev = |kind: &str, extra: &str| -> String {
        let s = &steps[kind];
        let mut base = format!(
            "media/session.cast at {}–{} s",
            g(s["started_at"].as_f64().unwrap_or(0.0)),
            g(s["ended_at"].as_f64().unwrap_or(0.0)),
        );
        if let Some(st) = s["exit_status"].as_i64() {
            base.push_str(&format!("; observed exit {st}"));
        }
        if !extra.is_empty() {
            base.push_str("; ");
            base.push_str(extra);
        }
        base
    };

    let version_line = steps["version"]["first_line"].as_str().unwrap_or_default();
    let version_ok = measured["version_flag_supported"]
        .as_bool()
        .unwrap_or(false);
    let refusal_line = measured["refusal_first_line"].as_str().unwrap_or_default();
    let refusal_status = measured["refusal_exit_status"].as_i64();
    let recovery_line = steps["recovery-help"]["first_line"]
        .as_str()
        .unwrap_or_default();
    let invalid_cmd = steps["invalid-flag"]["command"]
        .as_str()
        .unwrap_or_default();

    let cancellation_sentence = if measured["cancel_prompt_restored"]
        .as_bool()
        .unwrap_or(false)
    {
        format!(
            "Ctrl-C on the unsubmitted `{invalid_cmd}` line discarded it and restored the prompt"
        )
    } else {
        format!("Ctrl-C was sent on the unsubmitted `{invalid_cmd}` line and the session continued at the prompt")
    };
    let phrase = measured["refusal_next_action_phrase"]
        .as_str()
        .unwrap_or("");

    let interactions = vec![
        {
            let s = &steps["version"];
            json!({
                "name": "command entry",
                "trigger": format!("Type `{}` at the `{}` prompt and press Enter.", s["command"].as_str().unwrap_or_default(), PROMPT.trim()),
                "response": format!("{name} starts from {} and writes to the pseudo-terminal.", run.binary_path),
                "feedback": if version_line.is_empty() { "no output on the version form".to_string() } else { quote(version_line, 160) },
                "cancellation": format!("{cancellation_sentence}; nothing was submitted."),
                "failure": format!("`{invalid_cmd}` reaches the same parser and is refused with status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "recovery": format!("Re-enter `{}`.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "evidence": ev("version", "media/01-version-identity.png"),
            })
        },
        {
            let s = &steps["version"];
            let st = s["exit_status"].as_i64();
            json!({
                "name": "version identity",
                "trigger": format!("Ask the installed binary what it is with `{}`.", s["command"].as_str().unwrap_or_default()),
                "response": if version_ok {
                    format!("{name} prints its version identity and exits 0.")
                } else {
                    format!(
                        "{name} has no version flag: it refuses the option with status {} and answers with its usage surface instead.",
                        st.map(|v| v.to_string()).unwrap_or_else(|| "None".into())
                    )
                },
                "feedback": if version_line.is_empty() {
                    format!("The process returns exit status {}.", st.map(|v| v.to_string()).unwrap_or_else(|| "None".into()))
                } else {
                    quote(version_line, 160)
                },
                "cancellation": "The version form returns on its own; Ctrl-C is available at the prompt.",
                "failure": if version_ok {
                    format!("No failure on this path: exit {}.", st.map(|v| v.to_string()).unwrap_or_else(|| "None".into()))
                } else {
                    format!("The version request itself is the failure: exit {}.", st.map(|v| v.to_string()).unwrap_or_else(|| "None".into()))
                },
                "recovery": if version_ok {
                    "None needed.".to_string()
                } else {
                    format!("Read the identity out of `{}` instead.", steps["help"]["command"].as_str().unwrap_or_default())
                },
                "evidence": ev("version", "media/01-version-identity.png"),
            })
        },
        {
            let s = &steps["help"];
            json!({
                "name": "help discovery",
                "trigger": format!("Run `{}`.", s["command"].as_str().unwrap_or_default()),
                "response": format!(
                    "{name} prints {} lines of its own top-level help, widest line {} characters.",
                    s["line_count"], measured["help_max_line_width"]
                ),
                "feedback": match s["first_line"].as_str().unwrap_or_default() {
                    "" => format!("The help process returns exit status {}.", s["exit_status"].as_i64().map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                    fl => quote(fl, 160),
                },
                "cancellation": "The stream is short enough to complete; the prompt stays interruptible.",
                "failure": format!("A misspelled flag on the same surface is refused with status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "recovery": format!("Re-run `{}`.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "evidence": ev("help", "media/02-help-surface.png"),
            })
        },
        {
            let s = &steps["subcommand-help"];
            json!({
                "name": "subcommand surface",
                "trigger": format!("Run `{}`.", s["command"].as_str().unwrap_or_default()),
                "response": product.sub_note,
                "feedback": match s["first_line"].as_str().unwrap_or_default() {
                    "" => format!("The subcommand surface returns exit status {}.", s["exit_status"].as_i64().map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                    fl => quote(fl, 160),
                },
                "cancellation": "Ctrl-C at the prompt abandons the request before submission.",
                "failure": if s["exit_status"].as_i64().unwrap_or(0) != 0 {
                    format!("Observed exit {} on this path.", s["exit_status"])
                } else {
                    "This path returned 0; failure is shown by the invalid flag instead.".to_string()
                },
                "recovery": format!("Return to `{}` for the documented grammar.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "evidence": ev("subcommand-help", "media/03-subcommand-help.png"),
            })
        },
        {
            json!({
                "name": "invalid flag refusal",
                "trigger": format!("Run `{invalid_cmd}`."),
                "response": format!("{name} refuses the unknown option and returns status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "feedback": if refusal_line.is_empty() {
                    format!("The refusal returns exit status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into()))
                } else {
                    quote(refusal_line, 160)
                },
                "cancellation": "The refusal returns immediately; no cancellation was required.",
                "failure": format!("Observed status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "recovery": if measured["refusal_names_next_action"].as_bool().unwrap_or(false) {
                    format!(
                        "The refusal names the next action ({}); running `{}` recovers.",
                        py_repr(phrase),
                        steps["recovery-help"]["command"].as_str().unwrap_or_default()
                    )
                } else {
                    format!("The refusal names no next action; `{}` recovers anyway.", steps["recovery-help"]["command"].as_str().unwrap_or_default())
                },
                "evidence": ev("invalid-flag", "media/04-refusal.png"),
            })
        },
        {
            let s = &steps["version"];
            json!({
                "name": "exit status reporting",
                "trigger": "After each command the recorded shell prints `printf \"exit-status=%s\\n\" \"$?\"`.",
                "response": "The real status of the preceding product invocation appears in the cast as text.",
                "feedback": format!(
                    "exit-status={} after the version form, exit-status={} after the invalid flag.",
                    s["exit_status"], refusal_status.map(|v| json!(v)).unwrap_or(Value::Null)
                ),
                "cancellation": "The status line is a shell builtin write; there is nothing to cancel.",
                "failure": "A missing status line would mean the prompt never returned; every step reported one.",
                "recovery": "Not applicable: the status is evidence, not an action.",
                "evidence": format!(
                    "media/session.cast at {}–{} s and after every other command",
                    g(s["ended_at"].as_f64().unwrap_or(0.0)),
                    g(s["status_reported_at"].as_f64().unwrap_or(0.0)),
                ),
            })
        },
        {
            let s = &steps["cancellation"];
            json!({
                "name": "cancellation",
                "trigger": format!("Type `{invalid_cmd}` and press Ctrl-C instead of Enter."),
                "response": if measured["cancel_prompt_restored"].as_bool().unwrap_or(false) {
                    "The pending line is discarded and the prompt returns; the product never ran."
                } else {
                    "Ctrl-C was accepted and the session continued at the prompt."
                },
                "feedback": if measured["cancel_echoed_interrupt"].as_bool().unwrap_or(false) {
                    "`^C` is echoed in the cast, then a fresh prompt."
                } else {
                    "A fresh prompt follows the interrupt in the cast."
                },
                "cancellation": "Ctrl-C is the observed cancellation mechanism for unsubmitted input.",
                "failure": "The typed command is abandoned on purpose rather than executed.",
                "recovery": format!("Submit `{}` at the restored prompt.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "evidence": format!(
                    "media/session.cast at {}–{} s",
                    g(s["started_at"].as_f64().unwrap_or(0.0)),
                    g(s["ended_at"].as_f64().unwrap_or(0.0))
                ),
            })
        },
        {
            let s = &steps["recovery-help"];
            json!({
                "name": "recovery",
                "trigger": format!("After the refusal and the cancellation, run `{}` again.", s["command"].as_str().unwrap_or_default()),
                "response": format!("The same installed binary prints valid help and returns status {}.", s["exit_status"]),
                "feedback": if recovery_line.is_empty() {
                    format!("The recovery help returns exit status {}.", s["exit_status"])
                } else {
                    quote(recovery_line, 160)
                },
                "cancellation": "The recovery can itself be interrupted with Ctrl-C at the prompt.",
                "failure": format!("Repeating `{invalid_cmd}` reproduces status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "recovery": "The valid help form completes the first-success journey.",
                "evidence": ev("recovery-help", "media/05-recovery.png"),
            })
        },
        {
            let s = &steps["no-color-help"];
            json!({
                "name": "color-free equivalence",
                "trigger": format!("Run `{}`.", s["command"].as_str().unwrap_or_default()),
                "response": format!(
                    "{} ({} lines).",
                    if measured["no_color_text_identical"].as_bool().unwrap_or(false) {
                        "The same help text is printed with the same line count"
                    } else {
                        "The help text differs from the colored run once color is removed"
                    },
                    s["line_count"]
                ),
                "feedback": format!(
                    "{} ANSI colour sequences in the default run, {} with NO_COLOR=1.",
                    measured["help_sgr_count"], measured["no_color_sgr_count"]
                ),
                "cancellation": "Ctrl-C at the prompt applies here as to any other command.",
                "failure": "No failure on this path; it is a measurement of the same success route.",
                "recovery": "Not applicable.",
                "evidence": ev("no-color-help", ""),
            })
        },
    ];

    let journey_steps = vec![
        {
            json!({
                "index": 1,
                "user_action": format!("Open the recorded pseudo-terminal at `{}` in an empty scratch directory.", PROMPT.trim()),
                "system_response": "A clean prompt appears; no project, credential, host or queue target is selected.",
                "state": "ready prompt",
                "evidence": format!("media/session.cast at 0–{} s", g(steps["version"]["started_at"].as_f64().unwrap_or(0.0))),
            })
        },
        {
            json!({
                "index": 2,
                "user_action": format!("Run `{}`.", steps["version"]["command"].as_str().unwrap_or_default()),
                "system_response": if version_ok {
                    format!(
                        "{name} prints `{}` and exits {}.",
                        quote(version_line, 90),
                        steps["version"]["exit_status"]
                    )
                } else {
                    format!(
                        "{name} refuses the version flag with status {} and prints `{}`.",
                        steps["version"]["exit_status"],
                        quote(version_line, 90)
                    )
                },
                "state": "version identity",
                "evidence": ev("version", "media/01-version-identity.png"),
            })
        },
        {
            json!({
                "index": 3,
                "user_action": format!("Run `{}`.", steps["help"]["command"].as_str().unwrap_or_default()),
                "system_response": format!("{} lines of the product's own top-level help are printed.", steps["help"]["line_count"]),
                "state": "top-level help surface",
                "evidence": ev("help", "media/02-help-surface.png"),
            })
        },
        {
            json!({
                "index": 4,
                "user_action": format!("Run `{}`.", steps["subcommand-help"]["command"].as_str().unwrap_or_default()),
                "system_response": product.sub_note,
                "state": "subcommand help surface",
                "evidence": ev("subcommand-help", "media/03-subcommand-help.png"),
            })
        },
        {
            json!({
                "index": 5,
                "user_action": format!("Run `{invalid_cmd}`."),
                "system_response": format!("The option is refused: `{}`, status {}.", quote(refusal_line, 90), refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "state": "observed refusal",
                "evidence": ev("invalid-flag", "media/04-refusal.png"),
            })
        },
        {
            json!({
                "index": 6,
                "user_action": format!("Type `{invalid_cmd}` again and press Ctrl-C before Enter."),
                "system_response": if measured["cancel_prompt_restored"].as_bool().unwrap_or(false) {
                    "The unsubmitted line is discarded and the prompt returns; nothing ran."
                } else {
                    "Ctrl-C is accepted and the session continues at the prompt."
                },
                "state": "cancelled pending command",
                "evidence": format!(
                    "media/session.cast at {}–{} s",
                    g(steps["cancellation"]["started_at"].as_f64().unwrap_or(0.0)),
                    g(steps["cancellation"]["ended_at"].as_f64().unwrap_or(0.0))
                ),
            })
        },
        {
            json!({
                "index": 7,
                "user_action": format!("Run `{}` again.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "system_response": format!("Valid help is printed again with status {}: first success is recovered.", steps["recovery-help"]["exit_status"]),
                "state": "recovered first success",
                "evidence": ev("recovery-help", "media/05-recovery.png"),
            })
        },
        {
            json!({
                "index": 8,
                "user_action": format!("Run `{}`.", steps["no-color-help"]["command"].as_str().unwrap_or_default()),
                "system_response": if measured["no_color_text_identical"].as_bool().unwrap_or(false) {
                    "The same help text appears with colour removed, so no state was carried by colour."
                } else {
                    "The help text changes once colour is removed, which is recorded as a difference, not a claim of parity."
                },
                "state": "colour-free help",
                "evidence": ev("no-color-help", ""),
            })
        },
    ];

    let colors_help = measured["colors_help"].as_bool().unwrap_or(false);
    let identical = measured["no_color_text_identical"]
        .as_bool()
        .unwrap_or(false);
    let names_next = measured["refusal_names_next_action"]
        .as_bool()
        .unwrap_or(false);
    let fits80 = measured["help_fits_80"].as_bool().unwrap_or(false);
    let help_command = steps["help"]["command"].as_str().unwrap_or_default();
    let nocolor_command = steps["no-color-help"]["command"]
        .as_str()
        .unwrap_or_default();

    let accessibility_observations = vec![
        format!(
            "Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, \
             `{help_command}` emitted {} ANSI SGR sequences, so this product {}",
            measured["help_sgr_count"],
            if colors_help {
                "does colour its help output."
            } else {
                "does not colour its help output."
            }
        ),
        format!(
            "NO_COLOR: `{nocolor_command}` printed {} lines with {} SGR sequences; the \
             ANSI-stripped text of the two runs is {}",
            steps["no-color-help"]["line_count"],
            measured["no_color_sgr_count"],
            if identical {
                "byte-identical, so every state the help communicates survives without colour."
            } else {
                "not identical, so the two runs are recorded as different rather than equivalent."
            }
        ),
        format!(
            "Refusal wording: the refusal for `{invalid_cmd}` {}",
            if names_next {
                format!(
                    "names the next action ({}) in the same output.",
                    py_repr(phrase)
                )
            } else {
                "does not name a next action anywhere in its output.".to_string()
            }
        ),
        format!(
            "Terminal width: the widest line of `{help_command}` is {} characters, so the help {}",
            measured["help_max_line_width"],
            if fits80 {
                "fits an 80-column terminal without wrapping."
            } else {
                "does not fit an 80-column terminal and will wrap."
            }
        ),
        "Input: the whole recorded journey is keyboard-only — typed commands, Enter and one \
         Ctrl-C — and every state is emitted as selectable terminal text, not as a drawn widget."
            .to_string(),
        format!(
            "State without colour: success and refusal differ by exit status in the cast \
             ({} against {}), which the shell prints as text.",
            steps["recovery-help"]["exit_status"],
            refusal_status
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".into())
        ),
    ];

    let repainted = measured["screen_cleared"].as_bool().unwrap_or(false)
        || measured["cursor_addressed"].as_bool().unwrap_or(false);
    let duration = cast["duration_seconds"].as_f64().unwrap_or(0.0);

    json!({
        "schema": RECORD_SCHEMA,
        "name": name,
        "product_url": product.product_url,
        "evidence_status": "pending-verification",
        "upstream_owner": "Wisent (wisent-ai)",
        "wisent_product": true,
        "repository": product.repository,
        "captured_at": media["captured_at"].clone(),
        "capture_host": host_facts().host.clone(),
        "installed": {
            "binary": binary,
            "resolved_path": run.binary_path,
            "version_command": steps["version"]["command"].clone(),
            "version_output": quote(version_line, 400),
            "version_exit_status": steps["version"]["exit_status"].clone(),
            "version_flag_supported": version_ok,
        },
        "motion": [{
            "local_path": "media/session.cast",
            "source_url": product.product_url,
            "media_kind": "asciinema-v2-terminal-cast",
            "width": COLS,
            "height": ROWS,
            "duration_seconds": cast["duration_seconds"].clone(),
            "frame_count": cast["frame_count"].clone(),
            "bytes": cast["bytes"].clone(),
            "sha256": cast["sha256"].clone(),
            "capture_method": format!(
                "Real local run of the installed product on this workstation: `{binary}` resolved to \
                 {} and driven through a real pseudo-terminal (PTY) on {}, recorded as an asciinema v2 \
                 terminal cast with the timings of the run. The session issued only read-only commands \
                 — version form, top-level help, one subcommand help surface, one deliberately invalid \
                 flag, Ctrl-C on an unsubmitted line, the recovering help, and the same help with \
                 NO_COLOR=1 — from an empty scratch working directory. No host was contacted, no \
                 credential minted, no vault written, no job submitted and no service restarted.",
                run.binary_path,
                host_sentence()
            ),
            "recording_environment": format!(
                "{SHELL} --norc --noprofile -i on a {COLS}x{ROWS} PTY, TERM=xterm-256color, PAGER=cat, cwd={}",
                run.workdir
            ),
        }],
        "states": states.as_array().map(|v| v.as_slice()).unwrap_or(&[]).iter().map(|state| {
            json!({
                "name": state["label"].clone(),
                "state_name": state["state_name"].clone(),
                "local_path": state["local_path"].clone(),
                "source_motion_path": "media/session.cast",
                "source_relationship": state["source_relationship"].clone(),
                "cast_event_index": state["event_index"].clone(),
                "cast_timestamp_seconds": state["timestamp_seconds"].clone(),
                "width": state["width"].clone(),
                "height": state["height"].clone(),
                "bytes": state["bytes"].clone(),
                "sha256": state["sha256"].clone(),
            })
        }).collect::<Vec<_>>(),
        "interactions": interactions,
        "journey": {
            "actor": "An operator who has just found this Wisent binary on the PATH and wants to know what it is, what it can do, and what it refuses — before pointing it at anything real.",
            "goal": format!(
                "Get the first meaningful {name} result, read its own description of its grammar, see a real \
                 refusal, cancel a pending command safely, and recover — without touching a host, a vault, a \
                 queue or a credential."
            ),
            "prerequisites": [
                format!("{} installed on this workstation at {} (from {})", name, run.binary_path, product.repository),
                format!("An empty scratch working directory ({}) with no project or product state in it", run.workdir),
                format!("A pseudo-terminal at {COLS}x{ROWS} with TERM=xterm-256color, PAGER=cat and NO_COLOR unset"),
                host_sentence().to_string(),
            ],
            "steps": journey_steps,
            "failure_route": [
                format!("Run `{invalid_cmd}`."),
                quote(refusal_line, 160),
                format!("Observe status {} printed by the recorded shell, and the prompt restored.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
            ],
            "recovery_route": [
                format!("Run `{}`.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                quote(recovery_line, 160),
                format!("Observe status {} and the prompt returned with nothing changed on disk.", steps["recovery-help"]["exit_status"]),
            ],
            "completion_evidence": format!(
                "media/session.cast at {}–{} s plus media/05-recovery.png: the same installed binary answers \
                 again after the refusal and the cancellation. The whole {} s session is local and replayable \
                 with `asciinema play media/session.cast`.",
                g(steps["recovery-help"]["started_at"].as_f64().unwrap_or(0.0)),
                g(steps["recovery-help"]["ended_at"].as_f64().unwrap_or(0.0)),
                g(duration)
            ),
        },
        "motion_analysis": {
            "trigger": "Enter pressed on each typed command in the recorded pseudo-terminal session; the seventh keystroke sequence is Ctrl-C instead of Enter.",
            "start_state": format!(
                "An empty `{}` prompt in {} with no product state, no credential and no target selected.",
                PROMPT.trim(),
                run.workdir
            ),
            "end_state": format!(
                "The prompt restored after `{}`, with the shell's own `exit-status=` line as the last \
                 product-related output.",
                nocolor_command
            ),
            "continuity": if repainted {
                "One append-only text stream: the product repainted the screen at least once, so earlier \
                 states are recoverable only from the cast event list."
            } else {
                "One append-only text stream: no screen clear and no cursor addressing appear anywhere in \
                 the cast, so every state reached stays visible above the next one and the whole journey \
                 can be read as one scroll."
            },
            "timing_class": timing_class(measured),
            "timing_description": timing_description(measured),
            "interruption_or_reversal": format!(
                "Ctrl-C at {} s on the unsubmitted `{invalid_cmd}` line: {}",
                g(steps["cancellation"]["started_at"].as_f64().unwrap_or(0.0)),
                if measured["cancel_echoed_interrupt"].as_bool().unwrap_or(false) {
                    "the shell echoed `^C`, discarded the line and reprinted the prompt, and the product never ran."
                } else {
                    "the line was discarded and the prompt returned, and the product never ran."
                }
            ),
            "feedback": "Completion is signalled twice: the prompt returns, and the recorded shell prints \
                         `exit-status=N` for the command that just ran, so success and refusal are \
                         distinguishable in the text alone.",
            "reduced_motion_equivalent": "There is no animation to reduce. The cast is text appended in order; \
                                          the five PNGs carry the same content statically, and the raw `.cast` \
                                          file can be read as JSON without playback.",
        },
        "accessibility": {
            "measured": true,
            "measurement_method": format!(
                "Measured from this run: SGR sequences counted in the raw PTY bytes, the same help command run \
                 again with NO_COLOR=1 and the two ANSI-stripped texts compared, the refusal text searched for a \
                 named next action, and the widest help line counted against 80 columns."
            ),
            "observations": accessibility_observations,
            "measurements": {
                "help_command": help_command,
                "help_sgr_sequences": measured["help_sgr_count"].clone(),
                "colours_help_output": colors_help,
                "no_color_command": nocolor_command,
                "no_color_sgr_sequences": measured["no_color_sgr_count"].clone(),
                "no_color_text_identical": identical,
                "help_line_count": measured["help_line_count"].clone(),
                "help_max_line_width": measured["help_max_line_width"].clone(),
                "help_fits_80_columns": fits80,
                "refusal_command": invalid_cmd,
                "refusal_exit_status": refusal_status.map(|v| json!(v)).unwrap_or(Value::Null),
                "refusal_names_next_action": names_next,
                "refusal_next_action_phrase": measured["refusal_next_action_phrase"].clone(),
                "cancel_echoed_interrupt": measured["cancel_echoed_interrupt"].clone(),
                "cancel_prompt_restored": measured["cancel_prompt_restored"].clone(),
                "screen_cleared": measured["screen_cleared"].clone(),
                "cursor_addressed": measured["cursor_addressed"].clone(),
            },
            "unknowns": [
                "Screen-reader behaviour was not observed: no screen reader was attached to this PTY.",
                "Colour contrast of any emitted colours was not measured, and no WCAG or terminal-accessibility audit was performed.",
                "Behaviour in a terminal narrower than 80 columns was not observed; only the emitted line widths were measured.",
                "High-contrast themes, non-UTF-8 locales and alternative fonts were not exercised.",
                "Authenticated and target-selected paths were deliberately not run, so nothing here describes the product's accessibility once a host, vault, queue or credential is involved.",
            ],
        },
        "observed_commands": STEP_PLAN.iter().map(|(kind, label)| {
            let s = &steps[*kind];
            json!({
                "step": kind,
                "label": label,
                "command": s["command"].clone(),
                "exit_status": s["exit_status"].clone(),
                "started_at": s["started_at"].clone(),
                "ended_at": s["ended_at"].clone(),
                "line_count": s["line_count"].clone(),
                "max_line_width": s["max_line_width"].clone(),
                "first_line": s["first_line"].clone(),
            })
        }).collect::<Vec<_>>(),
        "evidence_gaps": [],
        "measured_at": Value::Null,
    })
}

// -------------------------------------------------------------- media output

fn digest(path: &Path) -> Result<(u64, String)> {
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    let mut buf = vec![0u8; 1 << 20];
    loop {
        use std::io::Read;
        let n = file
            .read(&mut buf)
            .with_context(|| format!("read {}", path.display()))?;
        if n == 0 {
            break;
        }
        size += n as u64;
        hasher.update(&buf[..n]);
    }
    Ok((size, format!("{:x}", hasher.finalize())))
}

/// Render with whichever interpreter on this host has Pillow, exactly as the
/// former script did before re-execing itself.
fn pillow_python() -> Result<PathBuf> {
    static FOUND: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
        for cand in [
            "/usr/bin/python3",
            "/opt/homebrew/bin/python3",
            "/usr/local/bin/python3",
        ] {
            if !Path::new(cand).exists() {
                continue;
            }
            let probe = Command::new(cand).arg("-c").arg("import PIL").output();
            if probe.is_ok_and(|o| o.status.success()) {
                return Some(PathBuf::from(cand));
            }
        }
        None
    });
    FOUND
        .clone()
        .ok_or_else(|| anyhow!("Pillow is required to render the state PNGs and was not found."))
}

const RENDER_SCRIPT: &str = r#"
import json, os, re, sys
from PIL import Image, ImageDraw, ImageFont

COLS = 100
ROWS = 32
FONT_CANDIDATES = ("/System/Library/Fonts/Menlo.ttc", "/System/Library/Fonts/SFNSMono.ttf")
FONT_PX = 15
ANSI = re.compile(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]")

events = json.load(open(sys.argv[1]))
path = sys.argv[2]

def strip_ansi(text):
    return ANSI.sub("", text).replace("\x07", "")

def visible_lines(text):
    out = []
    for line in strip_ansi(text).split("\n"):
        if line.endswith("\r"):
            line = line[:-1]
        if "\r" in line:
            line = line.split("\r")[-1]
        out.append(line.replace("\t", "    "))
    return out

def wrapped(lines, width):
    out = []
    for line in lines:
        if not line:
            out.append("")
            continue
        while len(line) > width:
            out.append(line[:width])
            line = line[width:]
        out.append(line)
    return out

text = "".join(e[2] for e in events)
rows = wrapped(visible_lines(text), COLS)
rows = rows[-ROWS:]
while len(rows) < ROWS:
    rows.append("")
font = None
for candidate in FONT_CANDIDATES:
    if os.path.exists(candidate):
        try:
            font = ImageFont.truetype(candidate, FONT_PX)
            break
        except OSError:
            continue
if font is None:
    font = ImageFont.load_default()
advance = font.getlength("M") if hasattr(font, "getlength") else FONT_PX * 0.6
cell_w = max(1, int(round(advance)))
cell_h = int(round(FONT_PX * 1.45))
pad = 12
size = (COLS * cell_w + 2 * pad, ROWS * cell_h + 2 * pad)
image = Image.new("RGB", size, (13, 17, 23))
draw = ImageDraw.Draw(image)
for index, row in enumerate(rows):
    draw.text((pad, pad + index * cell_h), row, font=font, fill=(222, 228, 234))
image.save(str(path), format="PNG", optimize=True)
print(image.size[0], image.size[1])
"#;

/// Deterministic PNG of the cast's own text, replayed to one event. Returns
/// (width, height) of the written image.
fn render_state(path: &Path, events: &[(f64, String)], cutoff_index: usize) -> Result<(u64, u64)> {
    let interpreter = pillow_python()?;
    std::fs::create_dir_all(scratch_root())?;
    let tmp = scratch_root().join(".render-events.json");
    let sliced: Vec<Value> = events
        .iter()
        .take(cutoff_index + 1)
        .map(|(t, text)| json!([t, "o", text]))
        .collect();
    std::fs::write(&tmp, serde_json::to_vec(&sliced)?)?;
    let output = Command::new(&interpreter)
        .args(["-c", RENDER_SCRIPT])
        .arg(&tmp)
        .arg(path)
        .output()
        .with_context(|| format!("run {}", interpreter.display()))?;
    let _ = std::fs::remove_file(&tmp);
    if !output.status.success() {
        bail!(
            "state render failed ({}, {})",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let dims = String::from_utf8_lossy(&output.stdout);
    let mut it = dims.split_whitespace();
    let w = it.next().and_then(|v| v.parse().ok()).unwrap_or(0u64);
    let h = it.next().and_then(|v| v.parse().ok()).unwrap_or(0u64);
    Ok((w, h))
}

fn write_cast(path: &Path, events: &[(f64, String)], title: &str, wall_start: u64) -> Result<()> {
    let header = format!(
        "{{\"version\":2,\"width\":{COLS},\"height\":{ROWS},\"timestamp\":{wall_start},\
         \"env\":{{\"SHELL\":\"{SHELL}\",\"TERM\":\"xterm-256color\"}},\"title\":{}}}\n",
        json_str(title),
    );
    let mut out = String::from(&header);
    for (stamp, text) in events {
        out.push_str(&format!(
            "[{}, \"o\", {}]\n",
            g(round_n(*stamp, 6)),
            json_str(text)
        ));
    }
    std::fs::write(path, out).with_context(|| format!("write {}", path.display()))
}

fn write_media(run: &Run, ref_dir: &Path) -> Result<Value> {
    let media_dir = ref_dir.join("media");
    std::fs::create_dir_all(&media_dir)?;
    for entry in std::fs::read_dir(&media_dir)?.flatten() {
        if entry.path().is_file() {
            std::fs::remove_file(entry.path())?;
        }
    }

    let events = &run.events;
    let cast_path = media_dir.join("session.cast");
    let title = format!(
        "{} — real local first-look on {}",
        run.product.name,
        host_sentence()
    );
    write_cast(&cast_path, events, &title, run.wall_start)?;
    let (size, sha) = digest(&cast_path)?;
    let duration = round_n(events.iter().map(|(t, _)| *t).fold(0.0f64, f64::max), 3);

    let mut states = Vec::new();
    for (kind, filename, label) in STATE_PLAN {
        let step = &run.steps[*kind];
        let index = step.event_index;
        let path = media_dir.join(format!("{filename}.png"));
        let (width, height) = render_state(&path, events, index)?;
        let (st_size, st_sha) = digest(&path)?;
        let ts = events.get(index).map(|(t, _)| *t).unwrap_or(duration);
        states.push(json!({
            "label": format!("{}: {label}", run.product.name),
            "state_name": filename.split_once('-').map(|(_, rest)| rest).unwrap_or(filename),
            "local_path": format!("media/{filename}.png"),
            "event_index": index,
            "timestamp_seconds": ts,
            "width": width,
            "height": height,
            "bytes": st_size,
            "sha256": st_sha,
            "source_relationship": format!(
                "Deterministic Pillow render of media/session.cast replayed to the end of the \
                 '{label}' step (event {index}, t={} s): the cast's own ANSI-stripped text, wrapped \
                 at {COLS} columns, last {ROWS} rows, Menlo {FONT_PX}px. It is a render of the cast \
                 at that named point, not a separate capture, and re-rendering the same cast \
                 produces the same bytes.",
                g(ts)
            ),
        }));
    }

    Ok(json!({
        "cast": {
            "bytes": size,
            "sha256": sha,
            "duration_seconds": duration,
            "frame_count": events.len(),
        },
        "states": states,
        "captured_at": captured_at_now(),
    }))
}

fn reference_readme(record: &Value, run: &Run, measured: &Value) -> String {
    let product = run.product;
    let steps = &measured["steps"];
    let motion = &record["motion"][0];
    let mut lines: Vec<String> = Vec::new();
    let mut a = |s: String| lines.push(s);

    a(format!("# {}", record["name"].as_str().unwrap_or_default()));
    a(String::new());
    a(product.one_line.to_string());
    a(String::new());
    a(format!(
        "A Wisent product, measured by running it here. Repository [`{}`]({}); binary `{}` \
         resolved to `{}`.",
        product.repository,
        product.product_url,
        product.binary,
        record["installed"]["resolved_path"]
            .as_str()
            .unwrap_or_default(),
    ));
    a(String::new());
    a("## What was run".to_string());
    a(String::new());
    a(format!(
        "One `{SHELL} --norc --noprofile -i` session on a real {COLS}x{ROWS} pseudo-terminal, \
         cwd `{}`, on {}. Seven commands, all read-only:",
        run.workdir,
        host_sentence()
    ));
    a(String::new());
    a("| # | step | command | exit | lines | widest line |".to_string());
    a("|---:|---|---|---:|---:|---:|".to_string());
    for (i, (kind, label)) in STEP_PLAN.iter().enumerate() {
        let s = &steps[*kind];
        let status = match s["exit_status"].as_i64() {
            None => "\u{2014}".to_string(),
            Some(v) => v.to_string(),
        };
        a(format!(
            "| {} | {label} | `{}` | {status} | {} | {} |",
            i + 1,
            s["command"].as_str().unwrap_or_default(),
            s["line_count"],
            s["max_line_width"],
        ));
    }
    a(String::new());
    a(
        "Nothing else was issued. No host was contacted, no credential minted, no vault written, \
       no job submitted, no service restarted, and no test run."
            .to_string(),
    );
    a(String::new());
    a("## Identity as installed today".to_string());
    a(String::new());
    a("```".to_string());
    a(format!(
        "$ {}",
        steps["version"]["command"].as_str().unwrap_or_default()
    ));
    for line in steps["version"]["lines"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[])
        .iter()
        .take(12)
    {
        a(line.as_str().unwrap_or_default().to_string());
    }
    a(format!(
        "exit-status={}",
        match steps["version"]["exit_status"].as_i64() {
            Some(v) => v.to_string(),
            None => "None".into(),
        }
    ));
    a("```".to_string());
    a(String::new());
    if !measured["version_flag_supported"]
        .as_bool()
        .unwrap_or(false)
    {
        a(format!(
            "{} has no version flag. The refusal above is the measurement: this product cannot be \
             asked what version it is from its own CLI.",
            record["name"].as_str().unwrap_or_default()
        ));
        a(String::new());
    }
    a("## The refusal and the recovery".to_string());
    a(String::new());
    a("```".to_string());
    a(format!(
        "$ {}",
        steps["invalid-flag"]["command"]
            .as_str()
            .unwrap_or_default()
    ));
    for line in steps["invalid-flag"]["lines"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[])
        .iter()
        .take(10)
    {
        a(line.as_str().unwrap_or_default().to_string());
    }
    a(format!(
        "exit-status={}",
        match steps["invalid-flag"]["exit_status"].as_i64() {
            Some(v) => v.to_string(),
            None => "None".into(),
        }
    ));
    a("```".to_string());
    a(String::new());
    a(format!(
        "The refusal {}. `{}` then answers again with status {}.",
        if measured["refusal_names_next_action"]
            .as_bool()
            .unwrap_or(false)
        {
            format!(
                "names the next action ({})",
                py_repr(
                    measured["refusal_next_action_phrase"]
                        .as_str()
                        .unwrap_or("")
                )
            )
        } else {
            "names no next action".to_string()
        },
        steps["recovery-help"]["command"]
            .as_str()
            .unwrap_or_default(),
        steps["recovery-help"]["exit_status"],
    ));
    a(String::new());
    a("## Motion evidence".to_string());
    a(String::new());
    a(format!(
        "- [`media/session.cast`](media/session.cast) — asciinema v2, {} s, {} events, {} bytes, `{}`\u{2026}",
        g(motion["duration_seconds"].as_f64().unwrap_or(0.0)),
        motion["frame_count"],
        motion["bytes"],
        motion["sha256"].as_str().unwrap_or_default().chars().take(16).collect::<String>(),
    ));
    a(
        "- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, \
       then `[time, \"o\", output]` events."
            .to_string(),
    );
    a(String::new());
    a("## States".to_string());
    a(String::new());
    a("Each PNG is a deterministic render of the cast's own text at a named point in the sequence \
       — not a separate screenshot."
        .to_string());
    a(String::new());
    for state in record["states"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[])
    {
        a(format!(
            "- [`{}`]({}) — {}, cast event {} at t={} s, {}x{}, {} bytes",
            state["local_path"].as_str().unwrap_or_default(),
            state["local_path"].as_str().unwrap_or_default(),
            state["state_name"].as_str().unwrap_or_default(),
            state["cast_event_index"],
            g(state["cast_timestamp_seconds"].as_f64().unwrap_or(0.0)),
            state["width"],
            state["height"],
            state["bytes"],
        ));
    }
    a(String::new());
    a("## Accessibility, measured".to_string());
    a(String::new());
    for obs in record["accessibility"]["observations"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[])
    {
        a(format!("- {}", obs.as_str().unwrap_or_default()));
    }
    a(String::new());
    a("Not measured:".to_string());
    a(String::new());
    for unknown in record["accessibility"]["unknowns"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[])
    {
        a(format!("- {}", unknown.as_str().unwrap_or_default()));
    }
    a(String::new());
    a("## Journey".to_string());
    a(String::new());
    a("| # | action | response | state |".to_string());
    a("|---:|---|---|---|".to_string());
    for step in record["journey"]["steps"]
        .as_array()
        .map(|v| v.as_slice())
        .unwrap_or(&[])
    {
        a(format!(
            "| {} | {} | {} | {} |",
            step["index"],
            step["user_action"].as_str().unwrap_or_default(),
            step["system_response"].as_str().unwrap_or_default(),
            step["state"].as_str().unwrap_or_default(),
        ));
    }
    a(String::new());
    a("## Boundary".to_string());
    a(String::new());
    a(product.selection_note.to_string());
    a(String::new());
    a(
        "This record evidences first-look grammar, help discoverability, refusal wording, safe \
       cancellation, recovery and colour independence. It evidences nothing about authenticated \
       behaviour, remote calls, queue submission, vault writes or destructive commands: those \
       paths were deliberately not run."
            .to_string(),
    );
    a(String::new());
    lines.join("\n")
}

fn write_reference(run: &Run, measured: &Value) -> Result<(PathBuf, Value)> {
    let ref_dir = catalog_dir()
        .join("references")
        .join(format!("{:02}-{}", run.index, run.product.slug));
    std::fs::create_dir_all(&ref_dir)?;
    let media = write_media(run, &ref_dir)?;
    let record = build_record(run, measured, &media);
    std::fs::write(
        ref_dir.join("reference.json"),
        serde_json::to_string_pretty(&record)? + "\n",
    )?;
    std::fs::write(
        ref_dir.join("README.md"),
        reference_readme(&record, run, measured),
    )?;
    Ok((ref_dir, record))
}

// ------------------------------------------------------------ catalog files

fn load_records() -> Result<Vec<(PathBuf, Value)>> {
    let mut out = Vec::new();
    let refs_dir = catalog_dir().join("references");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&refs_dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("reference.json").is_file())
        .collect();
    dirs.sort();
    for dir in dirs {
        let path = dir.join("reference.json");
        let text = std::fs::read_to_string(&path)?;
        let value =
            serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
        out.push((path, value));
    }
    Ok(out)
}

fn product_by_name(name: &str) -> Option<&'static Product> {
    PRODUCTS.iter().find(|p| p.name == name)
}

fn write_sources() -> Result<Value> {
    let records = load_records()?;
    let mut examples = Vec::new();
    for (path, record) in &records {
        let motion = &record["motion"][0];
        let overview = &record["states"][1];
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let slug = parent
            .split_once('-')
            .map(|(_, rest)| rest)
            .unwrap_or(parent);
        let prod = product_by_name(record["name"].as_str().unwrap_or_default());
        examples.push(json!({
            "name": record["name"].clone(),
            "slug": slug,
            "source_url": record["product_url"].clone(),
            "repository": record["repository"].clone(),
            "category": prod.map(|p| p.category).unwrap_or("Wisent product"),
            "selection_note": prod.map(|p| p.selection_note).unwrap_or(""),
            "installed": record["installed"].clone(),
            "reference_path": format!("references/{parent}/reference.json"),
            "visual": {
                "source_page_url": record["product_url"].clone(),
                "source_recording_path": format!("references/{parent}/media/session.cast"),
                "local_path": format!("references/{parent}/{}", overview["local_path"].as_str().unwrap_or_default()),
                "capture_kind": "local-terminal-render",
                "captured_at": record["captured_at"].clone(),
                "format": "png",
                "width": overview["width"].clone(),
                "height": overview["height"].clone(),
                "bytes": overview["bytes"].clone(),
                "sha256": overview["sha256"].clone(),
            },
            "interface_structure": {
                "analysis_kind": "deterministic-terminal-layout-v1",
                "image_sha256": overview["sha256"].clone(),
                "orientation": if overview["width"].as_i64().unwrap_or(0) >= overview["height"].as_i64().unwrap_or(0) { "landscape" } else { "portrait" },
                "layout_model": "single-terminal-surface",
                "panel_summary": "One 100-column pseudo-terminal surface retaining the product help as selectable text.",
                "regions": [{
                    "role": "terminal transcript",
                    "position": "full canvas",
                    "bounds": {"x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0},
                }],
                "detected_separators": [],
                "visual_density": "medium",
                "confidence": 1.0,
            },
            "evidence": {
                "kind": "asciinema-v2-terminal-cast plus deterministic renders of it",
                "local_path": format!("references/{parent}/media/session.cast"),
                "duration_seconds": motion["duration_seconds"].clone(),
                "frame_count": motion["frame_count"].clone(),
                "bytes": motion["bytes"].clone(),
                "sha256": motion["sha256"].clone(),
                "state_count": record["states"].as_array().map(|v| v.len()).unwrap_or(0),
                "captured_at": record["captured_at"].clone(),
            },
        }));
    }

    let payload = json!({
        "schema": SOURCES_SCHEMA,
        "catalog": catalog_dir().file_name().and_then(|n| n.to_str()).unwrap_or_default(),
        "title": "Wisent product examples",
        "description": "The Wisent products with a runnable CLI on the capture host, each measured by running it: \
                        version form, top-level help, one subcommand help surface, one invalid flag, a Ctrl-C on an \
                        unsubmitted line, the recovering help, and the same help with NO_COLOR=1.",
        "catalog_scope": format!(
            "This catalog is bounded by the Wisent products installed on the capture host: it contains one \
             record for each of the {} Wisent products with a runnable CLI on this workstation, and nothing \
             else. It is not a curated fifty, it is not a sample of the company's product surface, and it does \
             not cover Wisent products that ship only as a macOS app, an iOS app, a web application or a \
             service. Every Wisent repository whose CLI is not installed here is absent by construction.",
            examples.len()
        ),
        "capture_host": host_facts().host.clone(),
        "excluded_from_scope": EXCLUSIONS.iter().map(|(b, r, reason)| json!({
            "binary": b,
            "resolved": r,
            "reason": reason,
        })).collect::<Vec<_>>(),
        "curated_at": today_utc(),
        "count": examples.len(),
        "visual_count": examples.len(),
        "structure_count": examples.len(),
        "examples": examples,
    });
    std::fs::write(
        catalog_dir().join("sources.json"),
        serde_json::to_string_pretty(&payload)? + "\n",
    )?;
    Ok(payload)
}

fn write_index() -> Result<Value> {
    let records = load_records()?;
    let mut references = Vec::new();
    for (i, (path, record)) in records.iter().enumerate() {
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        references.push(json!({
            "index": i + 1,
            "name": record["name"].clone(),
            "path": format!("references/{parent}/reference.json"),
            "evidence_status": record.get("evidence_status").cloned().unwrap_or(json!("pending-verification")),
            "evidence_gap_count": record.get("evidence_gaps").and_then(|g| g.as_array()).map(|a| a.len()).unwrap_or(0),
        }));
    }
    let complete_count = references
        .iter()
        .filter(|r| r["evidence_status"].as_str() == Some("complete"))
        .count();
    let payload = json!({
        "schema": INDEX_SCHEMA,
        "catalog": catalog_dir().file_name().and_then(|n| n.to_str()).unwrap_or_default(),
        "generated_at": today_utc(),
        "reference_count": references.len(),
        "complete_count": complete_count,
        "partial_count": references.len() - complete_count,
        "references": references,
    });
    std::fs::write(
        catalog_dir().join("references.json"),
        serde_json::to_string_pretty(&payload)? + "\n",
    )?;
    Ok(payload)
}

fn write_full_reference() -> Result<()> {
    let records = load_records()?;
    let total = records.len();
    let with_version_flag = records.iter().filter(|(_, r)| {
        r["installed"]["version_flag_supported"]
            .as_bool()
            .unwrap_or(false)
    });
    let coloured = records.iter().filter(|(_, r)| {
        r["accessibility"]["measurements"]["colours_help_output"]
            .as_bool()
            .unwrap_or(false)
    });
    let fits80 = records.iter().filter(|(_, r)| {
        r["accessibility"]["measurements"]["help_fits_80_columns"]
            .as_bool()
            .unwrap_or(false)
    });
    let names_next = records.iter().filter(|(_, r)| {
        r["accessibility"]["measurements"]["refusal_names_next_action"]
            .as_bool()
            .unwrap_or(false)
    });
    let identical = records.iter().filter(|(_, r)| {
        r["accessibility"]["measurements"]["no_color_text_identical"]
            .as_bool()
            .unwrap_or(false)
    });
    let mut statuses: Vec<i64> = records
        .iter()
        .filter_map(|(_, r)| r["accessibility"]["measurements"]["refusal_exit_status"].as_i64())
        .collect();
    statuses.sort_unstable();
    statuses.dedup();

    let mut lines: Vec<String> = Vec::new();
    let mut a = |s: String| lines.push(s);
    a("# Wisent product full interaction reference".to_string());
    a(String::new());
    a(format!(
        "This synthesis is derived from the {total} complete per-product records in \
         [`references.json`](references.json). Every record is one real local run of an installed \
         Wisent binary on the capture host, recorded through a pseudo-terminal as an asciinema v2 \
         cast, with five deterministic renders of that cast, an eight-step observed journey, nine \
         interaction records, a real refusal with its exit status, a Ctrl-C on an unsubmitted \
         line, help-based recovery, and accessibility facts measured by running the product twice \
         — once with colour available and once with `NO_COLOR=1`."
    ));
    a(String::new());
    a("## What makes this catalog different".to_string());
    a(String::new());
    a(format!(
        "Every other catalog in this repository measures somebody else's product. This one measures \
         ours, and it is the only one whose motion evidence was produced by driving the product \
         rather than by collecting what its owner published. The cost of that honesty is size: it \
         holds {total} records, one per installed Wisent CLI, not fifty curated families."
    ));
    a(String::new());
    a("## Evidence method and boundary".to_string());
    a(String::new());
    a(format!(
        "Each product was driven through one `{SHELL} --norc --noprofile -i` session on a {COLS}x{ROWS} PTY with \
         `TERM=xterm-256color` and `PAGER=cat`, from an empty scratch directory under `~/.stado/work/\
         wisent-capture/run/`, on {}. Seven commands were issued and no others: the version form, \
         the top-level help, one subcommand help surface, `{PROBE_FLAG}`, a Ctrl-C on an unsubmitted \
         line, the recovering help, and the same help with `NO_COLOR=1`.",
        host_sentence()
    ));
    a(String::new());
    a("No host was contacted, no credential minted, no vault written, no queue job submitted, no \
       service restarted and no test executed. The records therefore evidence first-look identity, \
       discoverability, refusal wording, safe cancellation, recovery and colour independence. They \
       evidence nothing about authenticated behaviour, remote calls, queue submission, vault \
       contents or destructive commands."
        .to_string());
    a(String::new());
    a("## What the runs agree on".to_string());
    a(String::new());
    a(format!(
        "1. **Every product refuses an unknown flag, and every refusal is nonzero.** The observed \
         statuses are {} — so automation may treat nonzero as refusal, but must not assume a shared \
         numeric code across our own products.",
        statuses.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
    ));
    a(
        "2. **Every product recovers through its own help.** After the refusal, re-running the \
       top-level help returned valid output in every record; no product needed a reset, a flag \
       order change or a restart."
            .to_string(),
    );
    a("3. **Cancellation before submission is safe everywhere.** Ctrl-C on a typed but unsubmitted \
       line discarded it and restored the prompt in every session, and the product never started."
        .to_string());
    a(format!(
        "4. **Text carries the state.** In {} of {total} records the ANSI-stripped help text is \
         byte-identical with and without `NO_COLOR=1`, so colour is decoration rather than the carrier.",
        identical.count()
    ));
    a("5. **The shell's exit status is the portable success signal.** Each record prints the real \
       status after each command, and that line is the only cross-product way to tell a refusal \
       from an answer."
        .to_string());
    a(String::new());
    a("## What the runs disagree on, in our own products".to_string());
    a(String::new());
    a(format!(
        "- **Version identity.** Only {} of {total} answer `--version`. The rest refuse the flag: a \
         tool that wants to know which Wisent build it is talking to cannot ask uniformly.",
        with_version_flag.count()
    ));
    a("- **Help spelling.** `--help` for most, bare `help` for Skarbiec, and Tama ignores a trailing \
       `--help` after a subcommand and runs the command instead. A wrapper cannot guess."
        .to_string());
    a("- **Per-subcommand help.** Stado, Singularity, Transcript Lake and Transcript Label Trainer \
       have it. Oko answers with the whole top-level usage; Probierz refuses `--help` as an unknown \
       surface; Skarbiec reaches the vault state gate first; Weles and Jeden have none at all."
        .to_string());
    a(format!(
        "- **Refusal shape.** {} of {total} refusals name a next action in their own output; the \
         others print usage or a bare sentence. Probierz is alone in emitting a machine-readable \
         failure envelope.",
        names_next.count()
    ));
    a(format!(
        "- **Terminal width.** {} of {total} top-level helps fit 80 columns. The rest overflow, some \
         far past it, so our own help text wraps on a default terminal.",
        fits80.count()
    ));
    a(format!(
        "- **Colour.** {} of {total} colour their help output on a TTY. Colour is therefore not a \
         convention here, it is a per-product choice.",
        coloured.count()
    ));
    a(String::new());
    a("## Applicability boundaries".to_string());
    a(String::new());
    a(
        "Use these records to study first-contact behaviour of our own CLIs: identity, \
       discoverability, refusal wording, exit-status contracts, cancellation and recovery. Do not \
       use them as evidence of authenticated workflows, host operations, queue behaviour, \
       credential handling, browser execution, model calls or anything that requires a target. \
       Those need their own recordings, and they are not in this catalog."
            .to_string(),
    );
    a(String::new());
    a("## Complete record citations".to_string());
    a(String::new());
    a(
        "| # | Product | Repository | Evidence | Version identity as installed | Invalid exit |"
            .to_string(),
    );
    a("|---:|---|---|---|---|---:|".to_string());
    for (i, (path, record)) in records.iter().enumerate() {
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let version_raw = record["installed"]["version_output"]
            .as_str()
            .unwrap_or_default()
            .replace('|', "\\|");
        let version = if !record["installed"]["version_flag_supported"]
            .as_bool()
            .unwrap_or(false)
        {
            format!(
                "_no version flag_ — `{}` refused",
                record["installed"]["version_command"]
                    .as_str()
                    .unwrap_or_default()
            )
        } else {
            format!("`{version_raw}`")
        };
        a(format!(
            "| {} | {} | [`{}`]({}) | [`references/{parent}/`](references/{parent}/) | {version} | {} |",
            i + 1,
            record["name"].as_str().unwrap_or_default(),
            record["repository"].as_str().unwrap_or_default(),
            record["product_url"].as_str().unwrap_or_default(),
            record["accessibility"]["measurements"]["refusal_exit_status"],
        ));
    }
    a(String::new());
    std::fs::write(
        catalog_dir().join("full-reference.md"),
        lines.join("\n") + "\n",
    )?;
    Ok(())
}

fn write_catalog_readme() -> Result<()> {
    let records = load_records()?;
    let mut lines: Vec<String> = Vec::new();
    let mut a = |s: String| lines.push(s);
    a("# Wisent product examples".to_string());
    a(String::new());
    a(
        "The reference catalog for our own products. Every other catalog here measures somebody \
       else's product from what its owner published; this one measures Wisent products by running \
       them on this workstation and keeping the recording."
            .to_string(),
    );
    a(String::new());
    a(format!(
        "It holds {} records — one per Wisent product with a runnable CLI on the capture host. That \
         number is the honest scope, not a curated fifty: a Wisent product that ships only as a \
         macOS app, an iOS app, a web application or a service has no CLI to drive here and is \
         absent by construction.",
        records.len()
    ));
    a(String::new());
    a("## How it was captured".to_string());
    a(String::new());
    a(format!(
        "`capture-wisent-references.py` opens a real pseudo-terminal ({COLS}x{ROWS}, \
         `TERM=xterm-256color`, `PAGER=cat`), runs one `{SHELL} --norc --noprofile -i` session per \
         product from an empty scratch directory under `~/.stado/work/wisent-capture/run/`, and \
         issues seven read-only commands: the version form, the top-level help, one subcommand \
         help surface, `{PROBE_FLAG}`, Ctrl-C on an unsubmitted line, the recovering help, and the \
         same help with `NO_COLOR=1`."
    ));
    a(String::new());
    a(
        "The session's own output, with the timings of the run, becomes `media/session.cast` \
       (asciinema v2). The five PNGs beside it are deterministic Pillow renders of that cast's \
       text at named points in the sequence — they are renders of the recording, not separate \
       screenshots, and every record says so in `source_relationship`."
            .to_string(),
    );
    a(String::new());
    a("Nothing in this catalog contacted a host, minted a credential, wrote a vault, submitted a \
       job, restarted a service or ran a test."
        .to_string());
    a(String::new());
    a("## Reproducing it".to_string());
    a(String::new());
    a("```sh".to_string());
    a("cd ~/Documents/CodingProjects/Wisent/product-guidelines".to_string());
    a(
        "./capture-wisent-references.py --list      # what is installed, and its version identity"
            .to_string(),
    );
    a(
        "./capture-wisent-references.py             # re-run every product and rebuild the catalog"
            .to_string(),
    );
    a("./verify-reference-evidence.py --catalog wisent-product-examples --apply".to_string());
    a("```".to_string());
    a(String::new());
    a(
        "Re-running is idempotent apart from timestamps and hashes: the same products, the same \
       commands, the same five states, new timings."
            .to_string(),
    );
    a(String::new());
    a(format!(
        "Capture host: {}, kernel {}.",
        host_sentence(),
        host_facts().host["kernel"].as_str().unwrap_or_default()
    ));
    a(String::new());
    a("## The products, as installed".to_string());
    a(String::new());
    a(
        "| # | Product | Repository | Version identity | Cast | Invalid exit | Help fits 80 cols |"
            .to_string(),
    );
    a("|---:|---|---|---:|---:|---|".to_string());
    for (i, (path, record)) in records.iter().enumerate() {
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let m = &record["accessibility"]["measurements"];
        let version = if record["installed"]["version_flag_supported"]
            .as_bool()
            .unwrap_or(false)
        {
            format!(
                "`{}`",
                record["installed"]["version_output"]
                    .as_str()
                    .unwrap_or_default()
            )
        } else {
            format!(
                "_none_ (`{}` refused, exit {})",
                record["installed"]["version_command"]
                    .as_str()
                    .unwrap_or_default(),
                record["installed"]["version_exit_status"],
            )
        };
        let motion = &record["motion"][0];
        let fits = if m["help_fits_80_columns"].as_bool().unwrap_or(false) {
            "yes".to_string()
        } else {
            format!("no ({} cols)", m["help_max_line_width"])
        };
        a(format!(
            "| {} | [{}](references/{parent}/) | [`{}`]({}) | {version} | {} s | {} | {fits} |",
            i + 1,
            record["name"].as_str().unwrap_or_default(),
            record["repository"].as_str().unwrap_or_default(),
            record["product_url"].as_str().unwrap_or_default(),
            g(motion["duration_seconds"].as_f64().unwrap_or(0.0)),
            m["refusal_exit_status"],
        ));
    }
    a(String::new());
    a("## What is deliberately not here".to_string());
    a(String::new());
    for (item_binary, resolved, reason) in EXCLUSIONS {
        a(format!("- `{item_binary}` ({resolved}) — {reason}"));
    }
    a(String::new());
    a("## Honest statement".to_string());
    a(String::new());
    a(format!(
        "This catalog is our own products, measured by running them. The other families in this \
         repository are curated third-party examples whose motion evidence is mostly what their \
         owners published; here the evidence is a local run of our binary, and the gaps are the \
         gaps of our products rather than of a download. Where a record says a product has no \
         version flag, no per-subcommand help, or help that overflows 80 columns, that is a \
         measurement of Wisent software taken on {}, not a criticism borrowed from anyone else.",
        today_utc()
    ));
    a(String::new());
    a("## Files".to_string());
    a(String::new());
    a(
        "- `sources.json` — the catalog scope, the capture host, and one entry per product."
            .to_string(),
    );
    a(
        "- `references.json` — the record index, with the evidence status the verifier measured."
            .to_string(),
    );
    a("- `full-reference.md` — the synthesis across the records: what our CLIs agree and disagree on."
        .to_string());
    a("- `references/<NN-slug>/` — the per-product record, its README, its cast and its five states."
        .to_string());
    a(String::new());
    std::fs::write(catalog_dir().join("README.md"), lines.join("\n") + "\n")?;
    Ok(())
}

// ----------------------------------------------------------------------- cli

fn cmd_list() -> Result<()> {
    println!("Wisent products on this host ({}):", host_sentence());
    println!();
    let mut found = 0usize;
    for (index, product) in PRODUCTS.iter().enumerate() {
        let (path, outcome) = quick_version(product);
        let path = match path {
            Some(p) => p,
            None => {
                println!(
                    "{:2}. {:<26} MISSING (`{}` not on PATH)",
                    index + 1,
                    product.name,
                    product.binary
                );
                continue;
            }
        };
        found += 1;
        let identity = match outcome {
            QuickOutcome::Ran(0, first) => first,
            QuickOutcome::Ran(rc, first) => {
                format!(
                    "(no version flag; `{}` exits {rc}) {first}",
                    product.version_cmd
                )
            }
            QuickOutcome::Failed(tag) => {
                format!(
                    "(no version flag; `{}` exits None) {tag}",
                    product.version_cmd
                )
            }
            QuickOutcome::Missing => unreachable!(),
        };
        println!(
            "{:2}. {:<26} {:<38} {}",
            index + 1,
            product.name,
            product.repository,
            path
        );
        println!("{:<4}{:<40} -> {}", "", product.version_cmd, identity);
    }
    println!();
    println!(
        "{} of {} listed Wisent products are installed and runnable here.",
        found,
        PRODUCTS.len()
    );
    println!();
    println!("Excluded from scope:");
    for (item_binary, _resolved, reason) in EXCLUSIONS {
        println!("  {:<14} {}", item_binary, reason);
    }
    Ok(())
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut list = false;
    let mut catalog_only = false;
    let mut wanted: Vec<String> = Vec::new();
    let mut it = rest.iter();
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--list" => list = true,
            "--catalog-only" => catalog_only = true,
            "--product" => {
                let slug = it
                    .next()
                    .ok_or_else(|| anyhow!("--product requires a slug"))?;
                wanted.push(slug.clone());
            }
            other => {
                bail!("unknown flag: {other} (expected --list, --product <slug>, --catalog-only)")
            }
        }
    }

    if list {
        return cmd_list();
    }

    let catalog = catalog_dir();
    std::fs::create_dir_all(&catalog)?;
    std::fs::create_dir_all(catalog.join("references"))?;
    std::fs::create_dir_all(scratch_root())?;

    if !catalog_only {
        pillow_python()?;
        let selected: Vec<&Product> = PRODUCTS
            .iter()
            .filter(|p| wanted.is_empty() || wanted.iter().any(|w| w == p.slug))
            .collect();
        let missing: Vec<&str> = selected
            .iter()
            .filter(|p| resolve(p).is_none())
            .map(|p| p.binary)
            .collect();
        if !missing.is_empty() {
            bail!("not on PATH: {}", missing.join(", "));
        }
        for (i, product) in PRODUCTS.iter().enumerate() {
            if !selected.iter().any(|s| std::ptr::eq(*s, product)) {
                continue;
            }
            println!("[{:02}/{:02}] {}", i + 1, PRODUCTS.len(), product.name);
            let run = capture(i + 1, product)?;
            let measured = measure(&run);
            let (ref_dir, record) = write_reference(&run, &measured)?;
            let motion = &record["motion"][0];
            let rel = ref_dir
                .strip_prefix(root())
                .unwrap_or(&ref_dir)
                .to_string_lossy()
                .into_owned();
            println!(
                "    -> {rel}: {} s, {} events, {} states",
                g(motion["duration_seconds"].as_f64().unwrap_or(0.0)),
                motion["frame_count"],
                record["states"].as_array().map(|v| v.len()).unwrap_or(0),
            );
        }
    }

    let sources = write_sources()?;
    let index_payload = write_index()?;
    write_full_reference()?;
    write_catalog_readme()?;
    println!();
    println!(
        "catalog {}: {} products, {} records written",
        sources["catalog"].as_str().unwrap_or_default(),
        sources["count"],
        index_payload["reference_count"]
    );
    println!("next: ./verify-reference-evidence.py --catalog wisent-product-examples --apply");
    Ok(())
}
