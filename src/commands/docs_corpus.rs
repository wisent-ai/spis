//! `spis docs-corpus-*` — read-only JSON views over ~/.spis/docs-corpus for
//! desktop apps and scripts. stdout carries exactly one JSON document.

use crate as lib;
use anyhow::{bail, Context as _, Result};
use serde_json::json;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

fn engine_root() -> PathBuf {
    PathBuf::from("documentation-site-examples/content-structure")
}

fn corpus_dir(slug: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".spis/docs-corpus").join(slug)
}

struct SiteInfo {
    slug: String,
    name: String,
    category: String,
    source_url: String,
    inventory_url_count: i64,
    seen: usize,
    cumulative_ok: usize,
    noise: usize,
}

fn collect_sites() -> Result<Vec<SiteInfo>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(engine_root())? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().unwrap() == "full-text-manifest.json" {
            continue;
        }
        let meta: serde_json::Value = lib::read_json(path.to_str().unwrap())?;
        let slug = path.file_stem().unwrap().to_string_lossy().to_string();
        let state_path = corpus_dir(&slug).join("state.json");
        let mut seen = 0usize;
        let mut ok200 = 0usize;
        let mut non200 = 0usize;
        if state_path.exists() {
            let state: HashMap<String, serde_json::Value> =
                lib::read_json(state_path.to_str().unwrap())?;
            for v in state.values() {
                seen += 1;
                match v.get("status").and_then(|s| s.as_i64()) {
                    Some(200) => ok200 += 1,
                    Some(_) => non200 += 1,
                    None => {}
                }
            }
        }
        let noise = seen.saturating_sub(ok200 + non200);
        out.push(SiteInfo {
            slug,
            name: meta["name"].as_str().unwrap_or_default().to_string(),
            category: meta["category"].as_str().unwrap_or_default().to_string(),
            source_url: meta["source_url"].as_str().unwrap_or_default().to_string(),
            inventory_url_count: meta["inventory_url_count"].as_i64().unwrap_or(0),
            seen,
            cumulative_ok: ok200,
            noise,
        });
    }
    out.sort_by(|a, b| b.inventory_url_count.cmp(&a.inventory_url_count));
    Ok(out)
}

fn open_jsonl(slug: &str) -> Result<Option<std::io::BufReader<flate2::read::MultiGzDecoder<std::fs::File>>>> {
    let path = corpus_dir(slug).join("pages.jsonl.gz");
    if !path.exists() {
        return Ok(None);
    }
    let f = std::fs::File::open(&path)?;
    Ok(Some(std::io::BufReader::new(flate2::read::MultiGzDecoder::new(f))))
}

fn scan_site(
    slug: &str,
    name: &str,
    query: &str,
    limit: usize,
    hits: &mut Vec<serde_json::Value>,
) -> Result<usize> {
    let Some(reader) = open_jsonl(slug)? else { return Ok(0) };
    let q = query.to_lowercase();
    let mut scanned = 0usize;
    let mut lines = reader.lines();
    loop {
        let line = match lines.next() {
            Some(Ok(l)) => l,
            // A concurrently appended gzip member can end mid-stream; serve
            // everything that decompressed cleanly and stop at the tear.
            Some(Err(_)) | None => break,
        };
        scanned += 1;
        let rec: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let text = rec["text"].as_str().unwrap_or("");
        let title = rec["title"].as_str().unwrap_or("");
        let url = rec["url"].as_str().unwrap_or("");
        if !(text.to_lowercase().contains(&q.to_lowercase())
            || title.to_lowercase().contains(&q.to_lowercase())
            || url.to_lowercase().contains(&q.to_lowercase()))
        {
            continue;
        }
        let snippet = text.to_lowercase().find(&q.to_lowercase()).map(|byte_pos| {
            let start = text
                .char_indices()
                .nth(byte_pos.saturating_sub(60))
                .map(|(b, _)| b)
                .unwrap_or(0);
            let end = text
                .char_indices()
                .nth(byte_pos + query.len() + 120)
                .map(|(b, _)| b)
                .unwrap_or(text.len());
            text[start..end].replace('\n', " ")
        });
        hits.push(json!({
            "slug": slug,
            "site": name,
            "url": url,
            "title": (!title.is_empty()).then_some(title),
            "snippet": snippet,
        }));
        if hits.len() >= limit {
            break;
        }
    }
    Ok(scanned)
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut sub = "";
    let mut query = String::new();
    let mut site: Option<String> = None;
    let mut url_filter = String::new();
    let mut limit = 20usize;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "status" | "search" | "show" => sub = rest[i].as_str(),
            "--query" => {
                i += 1;
                query = rest.get(i).context("--query needs a value")?.clone();
            }
            "--site" => {
                i += 1;
                site = Some(rest.get(i).context("--site needs a value")?.clone());
            }
            "--url" => {
                i += 1;
                url_filter = rest.get(i).context("--url needs a value")?.clone();
            }
            "--limit" => {
                i += 1;
                limit = rest.get(i).context("--limit needs a value")?.parse()?;
            }
            other => bail!("unknown argument: {other}"),
        }
        i += 1;
    }
    if sub.is_empty() {
        bail!("usage: spis docs-corpus status | search --query T [--site S] [--limit N] | show --site S --url U");
    }

    match sub {
        "status" => {
            let sites = collect_sites()?;
            let arr: Vec<serde_json::Value> = sites
                .iter()
                .map(|s| {
                    json!({
                        "slug": s.slug,
                        "name": s.name,
                        "category": s.category,
                        "source_url": s.source_url,
                        "inventory_url_count": s.inventory_url_count,
                        "seen": s.seen,
                        "cumulative_ok": s.cumulative_ok,
                        "noise": s.noise,
                        "done": s.inventory_url_count > 0 && s.seen >= s.inventory_url_count as usize,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
            Ok(())
        }
        "search" => {
            if query.is_empty() {
                bail!("--query required");
            }
            let sites = collect_sites()?;
            let mut hits = Vec::new();
            let mut scanned = 0usize;
            for site in &sites {
                scanned += scan_site(&site.slug, &site.name, &query, limit, &mut hits)?;
                if hits.len() >= limit {
                    break;
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"hits": hits, "scanned": scanned, "limit": limit}))?
            );
            Ok(())
        }
        "show" => {
            let slug = site.clone().context("show needs --site <slug>")?;
            if url_filter.is_empty() {
                bail!("show needs --url <url>");
            }
            let Some(mut reader) = open_jsonl(&slug)? else {
                bail!("no corpus file for {slug}");
            };
            let mut lines = reader.lines();
            loop {
                let line = match lines.next() {
                    Some(Ok(l)) => l,
                    Some(Err(_)) | None => {
                        bail!("url not found in the {slug} corpus (or archive still being written)");
                    }
                };
                let Ok(rec) = serde_json::from_str::<serde_json::Value>(&line) else {
                    continue;
                };
                if rec["url"].as_str() == Some(url_filter.as_str()) {
                    println!("{rec}");
                    return Ok(());
                }
            }
        }
        _ => unreachable!(),
    }
}
