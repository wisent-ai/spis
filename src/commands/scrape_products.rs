//! `spis scrape-products <catalog-dir>` — sequential product-page scraper.
//!
//! Reads every reference.json in `<catalog>/references/*/`, fetches each
//! `product_url`, extracts readable text, stores as gzipped JSONL under
//! `~/.spis/docs-corpus/<catalog>/`. Self-contained, no model calls.
//!
//! Usage:
//!   spis scrape-products --dir ios-app-examples/references

use anyhow::{bail, Context, Result};
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

const UA: &str =
    "WisentKronikaCorpus/0.1 (documentation writing-style research; +https://wisent.com)";
const TIMEOUT_SECS: u64 = 30;

// ---------- HTTP ----------

fn http_get(url: &str) -> Result<(u16, String)> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<(u16, String)>>();
    let owned = url.to_string();
    std::thread::spawn(move || {
        let result = ureq::get(&owned)
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .set("User-Agent", UA)
            .call()
            .map(|r| {
                let status = r.status() as u16;
                match r.into_string() {
                    Ok(body) => Ok((status, body)),
                    Err(e) => Err(anyhow::anyhow!("body read: {e}")),
                }
            })
            .unwrap_or_else(|e| Err(anyhow::anyhow!("{e}")));
        let _ = tx.send(result);
    });
    match rx.recv_timeout(Duration::from_secs(TIMEOUT_SECS + 5)) {
        Ok(Ok(pair)) => Ok(pair),
        Ok(Err(e)) => Err(e),
        Err(_) => bail!("timeout after {TIMEOUT_SECS}s"),
    }
}

// ---------- HTML extraction ----------

fn extract_text(html: &str) -> (String, String) {
    #[derive(Default)]
    struct Ex {
        out: String,
        title: String,
        skip_depth: usize,
        in_pre: bool,
        in_title: bool,
        buf: String,
    }
    impl Ex {
        fn flush(&mut self) {
            if self.buf.is_empty() { return; }
            let squeezed = squeeze(&std::mem::take(&mut self.buf));
            if squeezed.is_empty() { return; }
            if self.in_title { self.title.push_str(&squeezed); }
            else if self.in_pre { self.out.push_str(&squeezed); }
            else if !self.out.is_empty() && !self.out.ends_with('\n') {
                self.out.push(' '); self.out.push_str(&squeezed);
            } else { self.out.push_str(&squeezed); }
        }
        fn handle_tag(&mut self, inner: &str) {
            let lower = inner.to_ascii_lowercase();
            let closing = lower.starts_with('/');
            let bare = lower.trim_start_matches('/');
            let name: String = bare.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
            const SKIP: &[&str] = &["script", "style", "nav", "footer", "header", "aside", "form", "svg", "noscript", "template"];
            if SKIP.contains(&name.as_str()) {
                if closing { self.skip_depth = self.skip_depth.saturating_sub(1); }
                else if !inner.ends_with('/') { self.skip_depth += 1; }
                return;
            }
            if name == "title" { self.in_title = !closing; return; }
            if name == "pre" { self.flush(); self.in_pre = !closing; self.out.push_str("\n```\n"); return; }
            if closing || self.in_pre { return; }
            match name.as_str() {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = name[1..].parse::<usize>().unwrap_or(6);
                    self.flush();
                    self.out.push_str(&format!("\n{} ", "#".repeat(level)));
                }
                "li" => { self.flush(); self.out.push_str("\n- "); }
                "p" | "tr" | "section" | "article" | "br" | "div" | "blockquote" => { self.flush(); self.out.push('\n'); }
                _ => {}
            }
        }
    }

    let mut ex = Ex::default();
    let mut pos = 0usize;
    while pos < html.len() {
        match html[pos..].find('<') {
            None => {
                if ex.skip_depth == 0 { ex.buf.push_str(&html[pos..]); }
                pos = html.len();
            }
            Some(rel) => {
                let tag_start = pos + rel;
                if ex.skip_depth == 0 && tag_start > pos {
                    ex.buf.push_str(&html[pos..tag_start]);
                    ex.flush();
                }
                match html[tag_start..].find('>') {
                    Some(gt) => {
                        let tag_src = html[tag_start + 1..tag_start + gt].trim().to_string();
                        ex.handle_tag(&tag_src);
                        pos = tag_start + gt + 1;
                    }
                    None => {
                        if ex.skip_depth == 0 { ex.buf.push_str(&html[tag_start..]); }
                        pos = html.len();
                    }
                }
            }
        }
    }
    let mut raw = ex.out;
    while raw.contains("\n\n\n") { raw = raw.replace("\n\n\n", "\n\n"); }
    (
        raw.lines().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n").trim().to_string(),
        ex.title.trim().to_string(),
    )
}

fn squeeze(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() { out.push(' '); }
            prev_space = true;
        } else { out.push(c); prev_space = false; }
    }
    out
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

// ---------- entry point ----------

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog_name = String::new();
    let mut refs_dir = PathBuf::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--catalog" => { i += 1; catalog_name = rest.get(i).context("--catalog needs value")?.clone(); }
            "--refs-dir" => { i += 1; refs_dir = PathBuf::from(rest.get(i).context("--refs-dir needs value")?); }
            _ => {}
        }
        i += 1;
    }
    if catalog_name.is_empty() || refs_dir.as_os_str().is_empty() {
        bail!("usage: scrape-products --catalog <name> --refs-dir <path>");
    }

    let data_dir = {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join(".spis/docs-corpus").join(&catalog_name)
    };
    std::fs::create_dir_all(&data_dir)?;
    let out_path = data_dir.join("pages.jsonl.gz");

    // Scan reference records.
    let mut slugs: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&refs_dir)? {
        let p = entry?.path();
        if p.is_dir() && p.join("reference.json").exists() {
            slugs.push(p.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    slugs.sort();

    // Load done set for resume.
    let done_path = data_dir.join("done.json");
    let mut done: HashMap<String, bool> = if done_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&done_path)?)?
    } else {
        HashMap::new()
    };

    eprintln!("{catalog_name}: {} records to scrape ({} already done)", slugs.len(), done.len());

    // Fetch each page sequentially.
    let client = ureq::AgentBuilder::new().build();
    for slug in &slugs {
        if done.contains_key(slug) { continue; }
        let rec_path = refs_dir.join(slug).join("reference.json");
        let rec: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&rec_path)?
        )?;
        let url = rec["product_url"].as_str().unwrap_or_default().to_string();
        if url.is_empty() || !url.starts_with("http") {
            continue;
        }

        eprintln!("  fetching {url}…");
        match http_get_string(&client, &url) {
            Ok((status, body)) => {
                let (text, title) = extract_text(&body);
                let rec_out = json!({
                    "slug": slug,
                    "url": url,
                    "status": status,
                    "title": title,
                    "text": text,
                    "sha256": sha256_hex(body.as_bytes()),
                    "fetched_at": crate::now_iso_utc(),
                });
                let line = serde_json::to_string(&rec_out)?;
                append_line(&out_path, &line)?;
                done.insert(slug.clone(), true);
                save_done(&done_path, &done)?;
                eprintln!("  ✓ {slug} ({} bytes text)", text.len());
            }
            Err(e) => {
                eprintln!("  ✗ {slug}: {e}");
                done.insert(format!("__error_{slug}"), true);
            }
        }
    }

    println!("Done.");
    Ok(())
}

fn http_get_string(client: &ureq::Agent, url: &str) -> Result<(u16, String)> {
    let resp = client.get(url).timeout(Duration::from_secs(30)).call()?;
    let status = resp.status() as u16;
    let body = resp.into_string()?;
    Ok((status, body))
}

fn append_line(path: &PathBuf, line: &str) -> Result<()> {
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{line}")?;
    Ok(())
}

fn save_done(path: &PathBuf, done: &HashMap<String, bool>) -> Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(done)?)?;
    Ok(())
}
