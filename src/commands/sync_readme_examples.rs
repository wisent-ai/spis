//! `spis sync-readme-examples` — refresh the curated README example snapshots
//! from GitHub (1:1 port of sync-readme-examples.py).
//!
//! Writes verbatim README snapshots plus source metadata under
//! `readme-examples/`. Requires an authenticated GitHub CLI (`gh auth login`)
//! and makes read-only GitHub API calls.

use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

const API_UA: &str = "wisent-product-guidelines-readme-curation";

const DEFINITION_PATH: &str = "readme-examples/scrape-definition.json";

/// Load the declarative scrape definition (what to capture, from which repos).
fn load_definition() -> Result<Vec<(usize, String, String)>> {
    let raw = std::fs::read_to_string(DEFINITION_PATH)
        .with_context(|| format!("read {}", DEFINITION_PATH))?;
    let doc: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", DEFINITION_PATH))?;
    let records = doc["records"]
        .as_array()
        .context("scrape-definition lacks records array")?;
    let mut out = Vec::new();
    for rec in records {
        let number = rec["number"].as_u64().context("record lacks number")? as usize;
        let repo = rec["repo"].as_str().context("record lacks repo")?.to_string();
        let category = rec["category"].as_str().unwrap_or_default().to_string();
        out.push((number, repo, category));
    }
    out.sort();
    Ok(out)
}

fn output_dir() -> PathBuf {
    PathBuf::from("readme-examples")
}

fn which(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

fn github_token() -> Result<String> {
    let gh = which("gh").context("GitHub CLI is not installed")?;
    let out = Command::new(gh)
        .arg("auth")
        .arg("token")
        .output()
        .context("run gh auth token")?;
    if !out.status.success() {
        bail!(
            "gh auth token failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if token.is_empty() {
        bail!("gh auth token returned an empty token");
    }
    Ok(token)
}

fn get_json(url: &str, token: &str) -> Result<Value> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(30))
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("User-Agent", API_UA)
        .call()
        .map_err(|e| anyhow::anyhow!("GET {url}: {e}"))?;
    let mut body = String::new();
    resp.into_reader()
        .take(64 * 1024 * 1024)
        .read_to_string(&mut body)
        .with_context(|| format!("read body of {url}"))?;
    serde_json::from_str(&body).with_context(|| format!("parse JSON from {url}"))
}


/// pathlib.Path.suffix equivalent for a repo file path (lowercased).
fn path_suffix(path: &str) -> String {
    let component = path.rsplit('/').next().unwrap_or(path);
    match component.rfind('.') {
        Some(0) | None => String::new(),
        Some(dot) => component[dot..].to_lowercase(),
    }
}

struct Fetched {
    entry: Value,
    content: String,
}

fn fetch_source(index: usize, requested_repo: &str, category: &str, token: &str) -> Result<Fetched> {
    let repo = get_json(&format!("https://api.github.com/repos/{requested_repo}"), token)?;
    let readme = get_json(
        &format!("https://api.github.com/repos/{requested_repo}/readme"),
        token,
    )?;
    let canonical_repo = repo["full_name"]
        .as_str()
        .context("repo response lacks full_name")?
        .to_string();
    let filename_repo = canonical_repo.to_lowercase().replace(['/', '.'], "-");
    let readme_path = readme["path"]
        .as_str()
        .context("readme response lacks path")?
        .to_string();
    let suffix = {
        let s = path_suffix(&readme_path);
        if s.is_empty() { ".md".to_string() } else { s }
    };
    let content_b64 = readme["content"]
        .as_str()
        .context("readme response lacks content")?;
    let content_bytes = base64_decode(content_b64)?;
    let content =
        String::from_utf8(content_bytes).context("readme content is not UTF-8")?;
    let spdx = repo
        .pointer("/license/spdx_id")
        .and_then(|v| v.as_str())
        .unwrap_or("NOASSERTION");
    let entry = json!({
        "number": index,
        "filename": format!("{index:02}-{filename_repo}{suffix}"),
        "repository": canonical_repo,
        "category": category,
        "description": repo.get("description").and_then(Value::as_str).unwrap_or(""),
        "repository_url": repo["html_url"].clone(),
        "default_branch": repo["default_branch"].clone(),
        "license_spdx": spdx,
        "stars_at_capture": repo.get("stargazers_count").cloned().unwrap_or(Value::Null),
        "readme_path": readme_path,
        "readme_blob_sha": readme["sha"].clone(),
        "readme_url": readme["html_url"].clone(),
    });
    Ok(Fetched { entry, content })
}

fn render_index(entries: &[Value], captured_at: &str) -> String {
    let mut lines = vec![
        "# Open-source README examples".to_string(),
        String::new(),
        "Fifty verbatim README snapshots from established open-source repositories. This is a curated reference set, not a ranking. Use it to compare information architecture, product positioning, quick starts, trust signals, support boundaries, and contribution paths.".to_string(),
        String::new(),
        "The snapshots remain the work of their respective projects and are governed by each source repository's license. Review patterns; do not copy project names, artwork, badges, or claims. Relative images and links may only render correctly in the upstream repository.".to_string(),
        String::new(),
        format!("Captured from GitHub on `{captured_at}`. `sources.json` records the README blob SHA, upstream URL, repository license identifier, and capture-time metadata for every file. Run `../sync-readme-examples.py` to refresh the catalog."),
        "Derived guidance: [README Best Practices](../readme-best-practices.md). Generated measurements: [analysis.json](analysis.json).".to_string(),
        String::new(),
        "| # | Repository | Category | Snapshot | Source | License |".to_string(),
        "|---:|---|---|---|---|---|".to_string(),
    ];
    for entry in entries {
        lines.push(format!(
            "| {} | `{}` | {} | [{}]({}) | [upstream]({}) | `{}` |",
            entry["number"],
            entry["repository"].as_str().unwrap_or_default(),
            entry["category"].as_str().unwrap_or_default(),
            entry["filename"].as_str().unwrap_or_default(),
            entry["filename"].as_str().unwrap_or_default(),
            entry["readme_url"].as_str().unwrap_or_default(),
            entry["license_spdx"].as_str().unwrap_or_default(),
        ));
    }
    lines.extend([
        String::new(),
        "## What to study".to_string(),
        String::new(),
        "- **First screen:** name, one-sentence promise, visual identity, and trust signals.".to_string(),
        "- **Audience and problem:** how quickly the intended user and concrete outcome become clear.".to_string(),
        "- **Progressive disclosure:** the transition from promise to proof, quick start, deeper docs, and contribution guidance.".to_string(),
        "- **Operational honesty:** maturity, prerequisites, platform limits, security, support, and licensing.".to_string(),
        "- **Actionability:** whether commands are copyable and whether the expected result is visible.".to_string(),
        "- **Navigation:** how a large project keeps the root README useful without duplicating its documentation site.".to_string(),
        String::new(),
    ]);
    lines.join("\n")
}

/// Minimal standard-alphabet base64 decoder (whitespace tolerated, padding ignored).
fn base64_decode(input: &str) -> Result<Vec<u8>> {
    fn val(b: u8) -> Option<u32> {
        match b {
            b'A'..=b'Z' => Some((b - b'A') as u32),
            b'a'..=b'z' => Some((b - b'a') as u32 + 26),
            b'0'..=b'9' => Some((b - b'0') as u32 + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let cleaned: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace() && *b != b'=')
        .collect();
    let mut out = Vec::with_capacity(cleaned.len() * 3 / 4 + 3);
    for chunk in cleaned.chunks(4) {
        let invalid = || anyhow::anyhow!("invalid base64 input");
        match chunk.len() {
            4 => {
                let n = (val(chunk[0]).ok_or_else(invalid)? << 18)
                    | (val(chunk[1]).ok_or_else(invalid)? << 12)
                    | (val(chunk[2]).ok_or_else(invalid)? << 6)
                    | val(chunk[3]).ok_or_else(invalid)?;
                out.push((n >> 16) as u8);
                out.push((n >> 8) as u8);
                out.push(n as u8);
            }
            3 => {
                let n = (val(chunk[0]).ok_or_else(invalid)? << 10)
                    | (val(chunk[1]).ok_or_else(invalid)? << 4)
                    | (val(chunk[2]).ok_or_else(invalid)? >> 2);
                out.push((n >> 8) as u8);
                out.push(n as u8);
            }
            2 => {
                let n = (val(chunk[0]).ok_or_else(invalid)? << 2)
                    | (val(chunk[1]).ok_or_else(invalid)? >> 4);
                out.push(n as u8);
            }
            _ => bail!("truncated base64 input"),
        }
    }
    Ok(out)
}

pub fn run(_rest: &[String]) -> Result<()> {
    let sources = load_definition()?;
    let repos: std::collections::HashSet<&str> =
        sources.iter().map(|(_, repo, _)| repo.as_str()).collect();
    if repos.len() != sources.len() {
        bail!("The curated source list contains a duplicate repository");
    }

    let token = github_token()?;
    let mut fetched: Vec<(usize, Value, String)> = Vec::with_capacity(sources.len());
    for (number, repo, category) in &sources {
        let result = fetch_source(*number, repo, category, &token)?;
        fetched.push((*number, result.entry, result.content));
    }
    fetched.sort_by_key(|(number, _, _)| *number);
    let entries: Vec<Value> = fetched.iter().map(|(_, e, _)| e.clone()).collect();

    let output = output_dir();
    std::fs::create_dir_all(&output)?;

    let expected: std::collections::HashSet<&str> = entries
        .iter()
        .filter_map(|e| e["filename"].as_str())
        .collect();
    for dir_entry in std::fs::read_dir(&output)?.filter_map(|e| e.ok()) {
        let name = dir_entry.file_name().to_string_lossy().to_string();
        let bytes = name.as_bytes();
        if bytes.len() > 2 && bytes[0].is_ascii_digit() && bytes[1].is_ascii_digit()
            && !expected.contains(name.as_str())
        {
            std::fs::remove_file(dir_entry.path())?;
        }
    }

    for (_, entry, content) in &fetched {
        let filename = entry["filename"].as_str().context("entry lacks filename")?;
        std::fs::write(output.join(filename), content)?;
    }

    let captured_at = crate::now_iso_utc()[..10].to_string();
    let sources: Vec<Map<String, Value>> = entries
        .iter()
        .map(|e| e.as_object().cloned().unwrap_or_default())
        .collect();
    let metadata = json!({
        "schema": "wisent.readme-examples",
        "captured_at": captured_at,
        "count": sources.len(),
        "sources": sources,
    });
    std::fs::write(
        output.join("sources.json"),
        serde_json::to_string_pretty(&metadata)? + "\n",
    )?;
    std::fs::write(
        output.join("README.md"),
        render_index(&entries, &captured_at),
    )?;

    // Scrape-run record: one auditable object per execution.
    let definition_hash = crate::sha256_hex(std::fs::read(DEFINITION_PATH).unwrap_or_default().as_slice());
    let tool_sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let run_record = json!({
        "schema": "wisent.scrape-run.v1",
        "tool": "spis sync-readme-examples",
        "tool_commit": tool_sha,
        "definition_path": DEFINITION_PATH,
        "definition_sha256": definition_hash,
        "started_at": captured_at,
        "finished_at": crate::now_iso_utc(),
        "records_attempted": sources.len(),
        "records_succeeded": fetched.len(),
        "records_failed": 0,
    });
    std::fs::write(
        output.join("scrape-run.json"),
        serde_json::to_string_pretty(&run_record)? + "\n",
    )?;

    println!("Wrote {} README snapshots to {}", entries.len(), output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_base64() {
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello".to_vec());
        // GitHub wraps content with newlines.
        assert_eq!(
            base64_decode("aGVs\nbG8=").unwrap(),
            b"hello".to_vec()
        );
        assert_eq!(base64_decode("YQ==").unwrap(), b"a".to_vec());
        assert_eq!(base64_decode("YWI=").unwrap(), b"ab".to_vec());
        assert_eq!(base64_decode("YWJj").unwrap(), b"abc".to_vec());
        assert!(base64_decode("!!!").is_err());
    }

    #[test]
    fn extracts_suffixes() {
        assert_eq!(path_suffix("README.md"), ".md");
        assert_eq!(path_suffix("docs/README.RST"), ".rst");
        assert_eq!(path_suffix("README"), "");
        assert_eq!(path_suffix(".github/README"), "");
        assert_eq!(path_suffix("a.b/README"), "");
    }

    #[test]
    fn source_list_has_no_duplicates() {
        let repos: std::collections::HashSet<&str> =
            SOURCES.iter().map(|(repo, _)| *repo).collect();
        assert_eq!(repos.len(), SOURCES.len());
        assert_eq!(SOURCES.len(), 50);
    }
}
