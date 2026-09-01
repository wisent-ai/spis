//! `spis check-upstream-drift` — report where stored reference evidence no
//! longer matches its upstream. Read-only checks:
//!
//! 1. README drift — current blob SHA of each snapshotted README via the
//!    GitHub API (`gh` CLI, preserved external call) against the recorded SHA.
//! 2. Source reachability — HTTP HEAD falling back to a ranged GET for every
//!    catalog `source_url`, `source_image_url`, and motion `source_url`.
//! 3. Local integrity — every recorded local media path resolves and matches
//!    its recorded SHA-256.
//!
//! Ported 1:1 from the former check-upstream-drift.py.

use crate as lib;
use anyhow::{Context, Result};
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const REPORT: &str = "upstream-drift.json";
const USER_AGENT: &str = crate::USER_AGENT;
const SCHEMA: &str = "wisent.upstream-drift-report.v1";
const TIMEOUT_SECS: u64 = 20;

const GONE_CODES: &[u16] = &[400, 404, 410];
const GUARDED_CODES: &[u16] = &[401, 403, 405, 429, 451, 501, 503];

#[derive(Default)]
struct Drift {
    readme_changed: Vec<Value>,
    readme_unreachable: Vec<Value>,
    readme_unchanged: usize,
    sources_gone: Vec<Value>,
    sources_guarded: Vec<Value>,
    sources_unresolved: Vec<Value>,
    sources_ok: usize,
    sources_skipped: usize,
    media_missing: Vec<String>,
    media_hash_mismatch: Vec<String>,
    media_ok: usize,
}

impl Drift {
    /// Guarded sources are not drift: an authenticated product answering 401
    /// is behaving as recorded. A gone or unresolvable source is drift.
    fn any_drift(&self) -> bool {
        !self.readme_changed.is_empty()
            || !self.readme_unreachable.is_empty()
            || !self.sources_gone.is_empty()
            || !self.sources_unresolved.is_empty()
            || !self.media_missing.is_empty()
            || !self.media_hash_mismatch.is_empty()
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .user_agent(USER_AGENT)
        .build()
}

/// Run `f` over `items` on a bounded worker pool (mirrors ThreadPoolExecutor).
fn pool<T: Sync, F: Fn(&T) + Sync>(items: &[T], workers: usize, f: F) {
    if items.is_empty() {
        return;
    }
    let next = AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..workers.min(items.len()) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= items.len() {
                    break;
                }
                f(&items[i]);
            });
        }
    });
}

fn gh_json(path: &str) -> Option<Value> {
    // Preserved external call: `gh api <path> --cache 0`.
    let out = Command::new("gh")
        .args(["api", path, "--cache", "0"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

fn check_readmes(drift: &Mutex<Drift>) -> Result<()> {
    let sources: Value = lib::read_json("readme-examples/sources.json")?;
    let entries = sources
        .get("repositories")
        .or_else(|| sources.get("examples"))
        .or_else(|| sources.get("sources"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    pool(&entries, 8, |entry| {
        let repo = entry.get("repository").and_then(|v| v.as_str());
        let readme_path = entry.get("readme_path").and_then(|v| v.as_str());
        let recorded = entry.get("readme_blob_sha");
        let (repo, readme_path) = match (repo, readme_path) {
            (Some(r), Some(p)) => (r, p),
            _ => return,
        };
        let mut guard = drift.lock();
        match gh_json(&format!("repos/{repo}/contents/{readme_path}")) {
            None => guard.readme_unreachable.push(json!({
                "repository": repo,
                "readme_path": readme_path,
            })),
            Some(data) => match data.get("sha").and_then(|v| v.as_str()) {
                None => guard.readme_unreachable.push(json!({
                    "repository": repo,
                    "readme_path": readme_path,
                })),
                Some(current) => {
                    if Some(current) != recorded.and_then(|v| v.as_str()) {
                        guard.readme_changed.push(json!({
                            "repository": repo,
                            "readme_path": readme_path,
                            "recorded_sha": recorded.cloned().unwrap_or(Value::Null),
                            "current_sha": current,
                            "snapshot": entry.get("filename").cloned().unwrap_or(Value::Null),
                        }));
                    } else {
                        guard.readme_unchanged += 1;
                    }
                }
            },
        }
    });
    Ok(())
}

/// Return (state, detail) where state is reachable | gone | guarded | unresolved.
fn url_state(client: &ureq::Agent, url: &str) -> (&'static str, Value) {
    match client.head(url).call() {
        Ok(resp) => ("reachable", json!(resp.status())),
        Err(ureq::Error::Status(code, resp)) => {
            if GUARDED_CODES.contains(&resp.status()) {
                // HEAD refused or rate limited; try one byte.
                match client.get(url).set("Range", "bytes=0-0").call() {
                    Ok(r) => return ("reachable", json!(r.status())),
                    Err(ureq::Error::Status(inner, _)) => {
                        if GONE_CODES.contains(&inner) {
                            return ("gone", json!(inner));
                        }
                        return ("guarded", json!(inner));
                    }
                    Err(_) => {
                        // Python reported "{code} then {ExceptionName}".
                        return (
                            "guarded",
                            json!(format!("{} then TransportError", resp.status())),
                        );
                    }
                }
            }
            if GONE_CODES.contains(&code) {
                return ("gone", json!(code));
            }
            ("guarded", json!(code))
        }
        Err(e) => ("unresolved", json!(format!("{}", e_kind(&e)))),
    }
}

fn e_kind(e: &ureq::Error) -> &'static str {
    match e {
        ureq::Error::Status(_, _) => "Status",
        ureq::Error::Transport(t) => match t.kind() {
            ureq::ErrorKind::Dns => "Dns",
            _ => "URLError",
        },
    }
}

/// (url, where, expected_state) for every recorded upstream reference.
fn collect_urls() -> Vec<(String, String, Option<String>)> {
    let mut pairs: Vec<(String, String, Option<String>)> = Vec::new();
    for sources in sorted_glob_examples_files("sources.json") {
        let Ok(data) = lib::read_json::<Value>(sources.to_str().unwrap()) else {
            continue;
        };
        let catalog = sources
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        for example in data
            .get("examples")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[])
        {
            let name = example
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();
            if let Some(url) = example.get("source_url").and_then(|v| v.as_str()) {
                pairs.push((
                    url.to_string(),
                    format!("{catalog}/{name}/source_url"),
                    example
                        .get("source_url_state")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                ));
            }
            if let Some(visual) = example.get("visual") {
                for key in ["source_page_url", "source_image_url"] {
                    if let Some(url) = visual.get(key).and_then(|v| v.as_str()) {
                        pairs.push((
                            url.to_string(),
                            format!("{catalog}/{name}/{key}"),
                            visual
                                .get(&format!("{key}_state"))
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                        ));
                    }
                }
            }
        }
    }
    for record in reference_records() {
        let Ok(data) = lib::read_json::<Value>(record.to_str().unwrap()) else {
            continue;
        };
        let catalog = record
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let dir_name = record
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        for entry in data
            .get("motion")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[])
        {
            if let Some(url) = entry.get("source_url").and_then(|v| v.as_str()) {
                pairs.push((
                    url.to_string(),
                    format!("{catalog}/{dir_name}/motion"),
                    entry
                        .get("source_url_state")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                ));
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    pairs.retain(|(url, _, _)| seen.insert(url.clone()));
    pairs
}

fn check_sources(drift: &Mutex<Drift>) {
    let pairs = collect_urls();
    pool(&pairs, 12, |pair| {
        let (url, where_, expected) = pair;
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            drift.lock().sources_skipped += 1;
            return;
        }
        let client = agent();
        let (mut state, detail) = url_state(&client, url);
        if state == "reachable" {
            drift.lock().sources_ok += 1;
            return;
        }
        // A private repository or authenticated application may deliberately
        // answer anonymous HTTP with 400/404. Its recorded classification wins
        // over the transport code.
        if expected.as_deref() == Some("guarded") && state == "gone" {
            state = "guarded";
        }
        let item = json!({
            "url": url,
            "where": where_,
            "result": detail,
            "expected": expected.clone().map(Value::String).unwrap_or(Value::Null),
        });
        let mut guard = drift.lock();
        match state {
            "gone" => guard.sources_gone.push(item),
            "guarded" => guard.sources_guarded.push(item),
            _ => guard.sources_unresolved.push(item),
        }
    });
}

fn stream_sha256(path: &std::path::Path) -> Result<String> {
    use sha2::Digest;
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut digest = sha2::Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

fn check_local_media(drift: &mut Drift) -> Result<()> {
    for record in reference_records() {
        let data: Value = lib::read_json(record.to_str().context("non-UTF8 path")?)?;
        let base = record.parent().context("record without parent")?;
        for key in ["motion", "states"] {
            for entry in data
                .get(key)
                .and_then(|v| v.as_array())
                .map(|a| a.as_slice())
                .unwrap_or(&[])
            {
                let local = base.join(
                    entry
                        .get("local_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                let rel = local.to_string_lossy().to_string();
                if !local.exists() {
                    drift.media_missing.push(rel);
                    continue;
                }
                let Some(recorded) = entry.get("sha256").and_then(|v| v.as_str()) else {
                    continue;
                };
                if stream_sha256(&local)? != recorded {
                    drift.media_hash_mismatch.push(rel);
                } else {
                    drift.media_ok += 1;
                }
            }
        }
    }
    Ok(())
}

/// Sorted `*-examples/references/*/reference.json` under the working directory.
fn reference_records() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for catalog in sorted_example_dirs() {
        let refs_dir = catalog.join("references");
        let Ok(entries) = std::fs::read_dir(&refs_dir) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let candidate = entry.path().join("reference.json");
            if candidate.is_file() {
                out.push(candidate);
            }
        }
    }
    out.sort();
    out
}

/// Sorted `*-examples` directories containing a `references` directory.
fn sorted_example_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(".")
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| {
            let p = e.path();
            p.strip_prefix(".").unwrap_or(p.as_path()).to_path_buf()
        })
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .map(|n| n.to_string_lossy().ends_with("-examples"))
                    .unwrap_or(false)
        })
        .collect();
    dirs.sort();
    dirs
}

fn sorted_glob_examples_files(name: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = sorted_example_dirs()
        .into_iter()
        .map(|d| d.join(name))
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    files
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut skip_network = false;
    let mut skip_readme = false;
    let mut write_report = false;
    let mut strict = false;
    for arg in rest {
        match arg.as_str() {
            "--skip-network" => skip_network = true,
            "--skip-readme" => skip_readme = true,
            "--write-report" => write_report = true,
            "--strict" => strict = true,
            other => anyhow::bail!("unknown argument: {other}"),
        }
    }

    let mut drift = Drift::default();
    check_local_media(&mut drift)?;
    if !skip_network {
        let shared = Mutex::new(Drift::default());
        if !skip_readme {
            check_readmes(&shared)?;
        }
        check_sources(&shared);
        let net = shared.into_inner();
        drift.readme_changed = net.readme_changed;
        drift.readme_unreachable = net.readme_unreachable;
        drift.readme_unchanged = net.readme_unchanged;
        drift.sources_gone = net.sources_gone;
        drift.sources_guarded = net.sources_guarded;
        drift.sources_unresolved = net.sources_unresolved;
        drift.sources_ok = net.sources_ok;
        drift.sources_skipped = net.sources_skipped;
    }

    println!("local media verified: {}", drift.media_ok);
    println!("local media missing: {}", drift.media_missing.len());
    println!(
        "local media hash mismatch: {}",
        drift.media_hash_mismatch.len()
    );
    if !skip_network {
        println!("README snapshots unchanged: {}", drift.readme_unchanged);
        println!(
            "README snapshots changed upstream: {}",
            drift.readme_changed.len()
        );
        println!(
            "README snapshots unreachable: {}",
            drift.readme_unreachable.len()
        );
        println!("upstream URLs reachable: {}", drift.sources_ok);
        println!("upstream URLs gone: {}", drift.sources_gone.len());
        println!(
            "upstream URLs guarded (auth, rate limit, bot wall): {}",
            drift.sources_guarded.len()
        );
        println!(
            "upstream URLs unresolved (network): {}",
            drift.sources_unresolved.len()
        );
    }
    for item in drift.media_missing.iter().take(20) {
        println!("  missing media: {item}");
    }
    for item in drift.media_hash_mismatch.iter().take(20) {
        println!("  hash mismatch: {item}");
    }
    for item in drift.readme_changed.iter().take(20) {
        println!(
            "  README changed: {} ({})",
            item["repository"].as_str().unwrap_or("?"),
            item["snapshot"].as_str().unwrap_or("null"),
        );
    }
    for item in &drift.sources_gone {
        println!(
            "  upstream gone: {} -> {} {}",
            item["where"].as_str().unwrap_or("?"),
            item["result"],
            item["url"].as_str().unwrap_or("?")
        );
    }
    for item in &drift.sources_unresolved {
        println!(
            "  upstream unresolved: {} -> {} {}",
            item["where"].as_str().unwrap_or("?"),
            item["result"],
            item["url"].as_str().unwrap_or("?")
        );
    }

    if write_report {
        let report = json!({
            "schema": SCHEMA,
            "checked_at": lib::now_iso_utc(),
            "network_checked": !skip_network,
            "local_media_verified": drift.media_ok,
            "local_media_missing": drift.media_missing,
            "local_media_hash_mismatch": drift.media_hash_mismatch,
            "readme_unchanged": drift.readme_unchanged,
            "readme_changed": drift.readme_changed,
            "readme_unreachable": drift.readme_unreachable,
            "upstream_urls_reachable": drift.sources_ok,
            "upstream_urls_gone": drift.sources_gone,
            "upstream_urls_guarded": drift.sources_guarded,
            "upstream_urls_unresolved": drift.sources_unresolved,
            "upstream_urls_skipped": drift.sources_skipped,
        });
        std::fs::write(REPORT, serde_json::to_string_pretty(&report)? + "\n")?;
        println!("\nreport written to {REPORT}");
    }

    if strict && drift.any_drift() {
        // The Python original exited 1 silently; main.rs prints "error:" for
        // Err returns, so exit directly.
        std::process::exit(1);
    }
    Ok(())
}
