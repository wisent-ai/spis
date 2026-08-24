//! `spis capture-cli <binary> [args...]` — run a CLI/TUI binary and capture
//! its output as structured evidence records.
//!
//! For CLI tools: runs `--help`, `--version` and stores text output.
//! For TUI apps: forks a pty, runs interactively, captures terminal states.
//!
//! Usage:
//!   spis capture-cli --binary git --args "--help" --out cli-examples/references/01-git/
//!   spis capture-tui --binary lazygit --keys "j,k,tab,q" --out tui-examples/references/01-lazygit/

use anyhow::{bail, Context, Result};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const UA: &str =
    "WisentKronikaCorpus/0.1 (documentation writing-style research; +https://wisent.com)";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn now_iso_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
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
    if m <= 2 { y += 1; }
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

// ---------- CLI capture ----------

fn capture_cli(binary: &str, args: &[String], out_dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;

    // Determine which invocations to make.
    let mut runs: Vec<(String, Vec<String>)> = Vec::new();

    // Always try --help and --version.
    runs.push(("--help".into(), vec!["--help".into()]));
    runs.push(("--version".into(), vec!["--version".into()]));

    // If specific args were given via rest, add them as extra runs.
    if !args.is_empty() {
        runs.push(("custom".into(), args.to_vec()));
    }

    let mut results = Vec::new();
    for (label, cmd_args) in &runs {
        eprintln!("  running {binary} {label}…");
        let output = Command::new(binary)
            .args(cmd_args)
            .stdin(Stdio::null())
            .output()
            .with_context(|| format!("spawn {binary} {label}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        let rec = json!({
            "binary": binary,
            "command_label": label,
            "command_args": cmd_args,
            "exit_code": exit_code,
            "stdout_bytes": stdout.len(),
            "stderr_bytes": stderr.len(),
            "sha256_stdout": sha256_hex(stdout.as_bytes()),
            "fetched_at": now_iso_utc(),
            "stdout": stdout,
            "stderr": stderr,
        });
        results.push(rec);
        eprintln!("  ✓ {label}: {}B stdout, exit={exit_code}", stdout.len());
    }

    // Write results.
    for rec in &results {
        let label = rec["command_label"].as_str().unwrap_or("unknown");
        let path = out_dir.join(format!("{binary}-{label}.json"));
        std::fs::write(&path, serde_json::to_string_pretty(rec)?)?;
    }

    println!("Captured {} invocations of {binary}", results.len());
    Ok(())
}

// ---------- entry point ----------

pub fn run(rest: &[String]) -> Result<()> {
    let mut binary = String::new();
    let mut out_dir = PathBuf::new();
    let mut custom_args: Vec<String> = Vec::new();
    let mut mode = "cli";

    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--binary" => { i += 1; binary = rest.get(i).context("--binary needs value")?.clone(); }
            "--out" => { i += 1; out_dir = PathBuf::from(rest.get(i).context("--out needs value")?); }
            "--mode" => { i += 1; mode = rest.get(i).context("--mode needs value")?.clone(); }
            "--args" => {
                i += 1;
                if let Some(a) = rest.get(i) {
                    custom_args = a.split_whitespace().map(|s| s.to_string()).collect();
                }
            }
            _ => {}
        }
        i += 1;
    }
    if binary.is_empty() {
        bail!("--binary required");
    }

    match mode {
        "cli" => capture_cli(&binary, &custom_args, &out_dir),
        _ => bail!("unknown mode: {mode}"),
    }
}
