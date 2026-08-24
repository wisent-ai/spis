//! `spis discover` — Rust port of `discover.py`.
//!
//! Discover the important pages behind a start URL and turn them into records.
//!
//! 1. Fetch the start page and extract every same-origin link with its text.
//! 2. Ask Brama which pages matter for a reference corpus (pricing, docs,
//!    sign-in, about…). If Brama is unreachable or unauthenticated, fall back to
//!    deterministic keyword classification — discovery never blocks on a model.
//! 3. Download an overview screenshot per selected page and scaffold a numbered
//!    record through the same contract as `reference-record add`.

use crate as lib;
use crate::commands::reference_record::{self, AddArgs};
use anyhow::{bail, Context, Result};
use serde_json::json;
use std::io::Read;
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) spis-discovery/1.0";
const THUMB: &str = "https://image.thum.io/get/width/1400/crop/1000/noanimate/";
const FAMILIES: &[&str] = &["pricing", "docs", "signup", "about", "product", "blog", "other"];

/// GET raw bytes with the discovery User-Agent; returns (body, content type).
fn fetch(url: &str, timeout_secs: u64) -> Result<(Vec<u8>, String)> {
    let resp = ureq::get(url)
        .timeout(Duration::from_secs(timeout_secs))
        .set("User-Agent", UA)
        .call()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let content_type = resp.content_type().to_string();
    let mut bytes = Vec::new();
    resp.into_reader()
        .take(64 << 20)
        .read_to_end(&mut bytes)
        .with_context(|| format!("read body of {url}"))?;
    Ok((bytes, content_type))
}

/// Length-preserving ASCII lowercase (keeps byte offsets aligned).
fn ascii_lower(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_uppercase() { c.to_ascii_lowercase() } else { c })
        .collect()
}

/// Insertion-ordered href -> text map, first text wins (`setdefault`).
#[derive(Default)]
struct Links {
    entries: Vec<(String, String)>,
}

impl Links {
    fn setdefault(&mut self, href: String, text: String) {
        if !self.entries.iter().any(|(h, _)| *h == href) {
            self.entries.push((href, text));
        }
    }
}

fn netloc_of(url: &str) -> &str {
    let after_scheme = match url.split_once("://") {
        Some((_, r)) => r,
        None => return "",
    };
    after_scheme.split(['/', '?', '#']).next().unwrap_or_default()
}

fn scheme_of(url: &str) -> &str {
    match url.split_once("://") {
        Some((scheme, _)) => scheme,
        None => "",
    }
}

/// True when `rel` carries a URI scheme (`^[a-zA-Z][a-zA-Z0-9+.-]*:`).
fn has_scheme(rel: &str) -> bool {
    match rel.find(':') {
        Some(colon) if colon > 0 => {
            let scheme = &rel[..colon];
            scheme.chars().enumerate().all(|(i, c)| {
                c.is_ascii_alphanumeric() || (i > 0 && "+.-".contains(c))
            }) && scheme.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        }
        _ => false,
    }
}

/// Minimal stand-in for `urllib.parse.urljoin` covering the shapes found in nav
/// markup: absolute URLs, protocol-relative, root-relative, query-only, and
/// relative paths with dot segments.
fn urljoin(base: &str, rel: &str) -> String {
    if has_scheme(rel) {
        return rel.to_string();
    }
    let scheme = scheme_of(base);
    if scheme.is_empty() {
        return rel.to_string();
    }
    let authority = base.split_once("://").map(|(_, r)| r).unwrap_or_default();
    let (netloc, path_and_more) = match authority.find('/') {
        Some(slash) => (&authority[..slash], &authority[slash..]),
        None => (authority, ""),
    };
    let base_path = path_and_more.split(['?', '#']).next().unwrap_or("");
    if let Some(rest) = rel.strip_prefix("//") {
        return format!("{scheme}://{rest}");
    }
    if let Some(rest) = rel.strip_prefix('?') {
        return format!("{scheme}://{netloc}{base_path}?{rest}");
    }
    if let Some(rest) = rel.strip_prefix('/') {
        return format!("{scheme}://{netloc}/{rest}");
    }
    if rel.is_empty() {
        return base.to_string();
    }
    let dir = match base_path.rfind('/') {
        Some(slash) => &base_path[..=slash],
        None => "/",
    };
    format!(
        "{scheme}://{netloc}{}",
        remove_dot_segments(&format!("{dir}{rel}"))
    )
}

/// RFC 3986 remove_dot_segments.
fn remove_dot_segments(path: &str) -> String {
    let mut output: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "." => {}
            ".." => {
                output.pop();
            }
            other => output.push(other),
        }
    }
    let mut joined = output.join("/");
    if path.ends_with("/.") || path.ends_with("/..") {
        if !joined.ends_with('/') {
            joined.push('/');
        }
    } else if path.starts_with('/') && !joined.starts_with('/') {
        joined.insert(0, '/');
    }
    joined
}

fn same_origin(url: &str, origin: &str) -> bool {
    matches!(scheme_of(url), "http" | "https") && netloc_of(url) == netloc_of(origin)
}

/// Pull `<a href>` -> collapsed inner text from HTML, mimicking the original
/// `HTMLParser` subclass: first text per href, unclosed anchors dropped, text
/// inside nested tags still counted.
fn parse_links(html: &str) -> Links {
    let mut links = Links::default();
    let lower = ascii_lower(html);
    let mut i = 0;
    let mut pending_href: Option<String> = None;
    let mut pending_text = String::new();
    while let Some(open_rel) = lower[i..].find('<') {
        let open = i + open_rel;
        // Text chunk since the previous tag belongs to any open anchor.
        if pending_href.is_some() {
            pending_text.push_str(&html[i..open]);
        }
        let Some(close_rel) = lower[open..].find('>') else { break };
        let close = open + close_rel;
        let tag_body = &lower[open + 1..close];
        let is_end = tag_body.starts_with('/');
        let tag_name: &str = tag_body
            .trim_start_matches('/')
            .split(|c: char| c.is_whitespace())
            .next()
            .unwrap_or("");
        if tag_name == "a" {
            if is_end {
                // `</a>` commits the pending anchor.
                if let Some(href) = pending_href.take() {
                    let text: String =
                        pending_text.split_whitespace().collect::<Vec<_>>().join(" ");
                    links.setdefault(lib::html_unescape(&href), lib::html_unescape(&text));
                }
            } else if tag_body.ends_with('/') {
                // Self-closing `<a/>`: start + immediate end with no text.
                if let Some(href) = attr_value(tag_body, "href") {
                    links.setdefault(lib::html_unescape(&href), String::new());
                }
                pending_href = None;
                pending_text.clear();
            } else {
                // A new `<a>` abandons any previously unclosed anchor.
                pending_href = attr_value(tag_body, "href");
                pending_text.clear();
            }
        }
        i = close + 1;
    }
    // An anchor left unclosed at EOF is discarded, as in Python.
    links
}

/// Extract an attribute value (first occurrence) from a start-tag body.
fn attr_value(tag_body: &str, name: &str) -> Option<String> {
    let body_lower = ascii_lower(tag_body);
    let mut search_from = 0;
    loop {
        let pos = body_lower[search_from..].find(name)? + search_from;
        let boundary_ok = pos == 0
            || tag_body[..pos]
                .chars()
                .last()
                .is_some_and(char::is_whitespace);
        let after = &tag_body[pos + name.len()..];
        let after_trimmed = after.trim_start();
        if boundary_ok && after_trimmed.starts_with('=') {
            let value_part = &after_trimmed[1..];
            let value_trimmed = value_part.trim_start();
            if let Some(rest) = value_trimmed.strip_prefix('"') {
                let end = rest.find('"').unwrap_or(rest.len());
                return Some(rest[..end].to_string());
            }
            if let Some(rest) = value_trimmed.strip_prefix('\'') {
                let end = rest.find('\'').unwrap_or(rest.len());
                return Some(rest[..end].to_string());
            }
            return Some(
                value_trimmed
                    .chars()
                    .take_while(|c| !c.is_whitespace())
                    .collect(),
            );
        }
        search_from = pos + name.len();
    }
}

fn extract_links(start_url: &str, html_bytes: &[u8], limit: usize) -> Links {
    let html = String::from_utf8_lossy(html_bytes);
    let parser = parse_links(&html);
    let origin = start_url;
    let mut found = Links::default();
    const SKIP_SUFFIXES: &[&str] = &[
        ".pdf", ".zip", ".png", ".jpeg", ".jpg", ".svg", ".webp", ".gif", ".mp4", ".css", ".js",
    ];
    for (href, text) in &parser.entries {
        let no_fragment = href.split('#').next().unwrap_or("");
        let absolute = urljoin(start_url, no_fragment);
        if !same_origin(&absolute, origin)
            || absolute.trim_end_matches('/') == origin.trim_end_matches('/')
        {
            continue;
        }
        let lowered = absolute.to_lowercase();
        if SKIP_SUFFIXES.iter().any(|suffix| lowered.ends_with(suffix)) {
            continue;
        }
        let text = if text.is_empty() {
            absolute.clone()
        } else {
            text.clone()
        };
        found.setdefault(absolute, text);
        if found.entries.len() >= limit {
            break;
        }
    }
    found
}

const KEYWORDS: &[(&str, &[&str])] = &[
    ("pricing", &["pricing", "plans", "plans-and-pricing"]),
    ("docs", &["docs", "documentation", "developers", "api", "guides"]),
    ("signup", &["sign-up", "signup", "register", "get-started", "start"]),
    ("about", &["about", "company", "customers", "careers"]),
    ("product", &["product", "features", "platform", "solutions"]),
];

fn heuristic_family(url: &str, text: &str) -> &'static str {
    let blob = format!("{} {}", url, text).to_lowercase();
    for (family, words) in KEYWORDS {
        if words.iter().any(|word| blob.contains(word)) {
            return family;
        }
    }
    "other"
}

/// Ask Brama to rank pages; `None` means fall back deterministically.
fn brama_rank(start_url: &str, links: &Links, limit: usize) -> Option<Vec<(String, String)>> {
    let router = std::env::var("MODEL_ROUTER_URL").ok()?;
    if router.is_empty() {
        return None;
    }
    let endpoint = if router.contains("/v1") {
        format!("{}/chat/completions", router.trim_end_matches('/'))
    } else {
        format!("{}/v1/chat/completions", router.trim_end_matches('/'))
    };
    let listing: Vec<String> = links
        .entries
        .iter()
        .take(80)
        .map(|(url, text)| format!("- {url} | {text}"))
        .collect();
    let payload = json!({
        "model": std::env::var("MODEL_ROUTER_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into()),
        "messages": [
            {"role": "system", "content": format!(
                "You classify pages of one product's website for an interface reference corpus. \
                 Return STRICT JSON: {{\"pages\": [{{\"url\": string, \"family\": \
                 one of {FAMILIES:?}]}}}} . Only use URLs from the list. Pick at most {limit}.")},
            {"role": "user", "content": format!(
                "Start page: {start_url}\nDiscovered links:\n{}", listing.join("\n"))},
        ],
    })
    .to_string();

    let parsed: Result<serde_json::Value> = (|| {
        let mut request = ureq::post(&endpoint)
            .timeout(Duration::from_secs(60))
            .set("Content-Type", "application/json");
        if let Ok(token) = std::env::var("MODEL_ROUTER_TOKEN") {
            if !token.is_empty() {
                request = request.set("Authorization", &format!("Bearer {token}"));
            }
        }
        let response = request.send_string(&payload)?;
        let mut body_bytes = Vec::new();
        let bytes_read = response.into_reader().read_to_end(&mut body_bytes)?;
        let _ = bytes_read;
        let body: serde_json::Value = serde_json::from_slice(&body_bytes)?;
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .context("no message content")?;
        let brace_open = content.find('{').context("no JSON object")?;
        let brace_close = content.rfind('}').context("no JSON object")? + 1;
        Ok(serde_json::from_str(&content[brace_open..brace_close])?)
    })();
    match parsed {
        Ok(parsed) => {
            let mut ranked: Vec<(String, String)> = Vec::new();
            for page in parsed["pages"].as_array().unwrap_or(&Vec::new()) {
                let (Some(url), Some(family)) = (page["url"].as_str(), page["family"].as_str())
                else {
                    continue;
                };
                if links.entries.iter().any(|(known, _)| known == url)
                    && FAMILIES.contains(&family)
                    && !ranked.iter().any(|(seen, _)| seen == url)
                {
                    ranked.push((url.to_string(), family.to_string()));
                }
            }
            if ranked.is_empty() {
                None
            } else {
                Some(ranked)
            }
        }
        Err(error) => {
            eprintln!("discover: Brama ranking unavailable ({error}); using keyword fallback");
            None
        }
    }
}

/// `spis discover <start-url> --catalog <slug> [--limit <n>] [--max-links <n>]`
pub fn run(rest: &[String]) -> Result<()> {
    let mut positionals: Vec<String> = Vec::new();
    let mut catalog: Option<String> = None;
    let mut limit: usize = 6;
    let mut max_links: usize = 120;
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        match arg.as_str() {
            "--catalog" => {
                i += 1;
                catalog = Some(rest.get(i).cloned().ok_or_else(|| {
                    anyhow::anyhow!("discover: argument --catalog: expected one argument")
                })?);
            }
            "--limit" | "--max-links" => {
                let name = arg.clone();
                i += 1;
                let value = rest.get(i).cloned().ok_or_else(|| {
                    anyhow::anyhow!("discover: argument {name}: expected one argument")
                })?;
                let parsed: usize = value.parse().map_err(|_| {
                    anyhow::anyhow!("discover: argument {name}: invalid int value {value:?}")
                })?;
                if name == "--limit" {
                    limit = parsed;
                } else {
                    max_links = parsed;
                }
            }
            other => {
                if other.starts_with("--") {
                    bail!("discover: unrecognized argument {other}");
                }
                positionals.push(arg);
            }
        }
        i += 1;
    }
    let start_url = positionals.first().cloned().ok_or_else(|| {
        anyhow::anyhow!("discover: the following arguments are required: start_url")
    })?;
    let catalog = catalog.ok_or_else(|| {
        anyhow::anyhow!("discover: the following arguments are required: --catalog")
    })?;

    let slug = if catalog.ends_with("-examples") {
        catalog.clone()
    } else {
        format!("{catalog}-examples")
    };
    let directory = std::path::PathBuf::from(&slug);

    let (html_bytes, _) = fetch(&start_url, 25)?;
    let links = extract_links(&start_url, &html_bytes, max_links);
    println!(
        "discovered {} same-origin links on {start_url}",
        links.entries.len()
    );
    if links.entries.is_empty() {
        bail!("discover: no same-origin links found");
    }

    let ranked = brama_rank(&start_url, &links, limit).unwrap_or_else(|| {
        let mut ranked: Vec<(String, String)> = Vec::new();
        for (url, text) in &links.entries {
            let family = heuristic_family(url, text);
            let family_count = ranked.iter().filter(|(_, f)| f == family).count();
            if family != "other" && family_count < std::cmp::max(1, limit / 3) {
                ranked.push((url.clone(), family.to_string()));
            }
        }
        ranked
    });
    let selected: Vec<(String, String)> = ranked.into_iter().take(limit).collect();
    if selected.is_empty() {
        bail!("discover: nothing selected for this corpus");
    }
    println!("Brama/heuristics selected {} page(s):", selected.len());
    for (url, family) in &selected {
        println!("  [{family}] {url}");
    }

    // Ensure the catalog exists, then reuse the tested record scaffolder.
    if !directory.is_dir() {
        let title_base = catalog.replace("-examples", "").to_lowercase();
        let mut title_chars = title_base.chars();
        let title = match title_chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + title_chars.as_str(),
            None => title_base,
        };
        let status = std::process::Command::new("python3")
            .args([
                "catalog-type.py",
                "add",
                &catalog,
                "--title",
                &format!("{title} examples"),
            ])
            .status()
            .context("run catalog-type.py")?;
        if !status.success() {
            bail!("discover: catalog-type.py add failed with {status}");
        }
    }

    for (url, family) in &selected {
        let thumb_url = format!("{THUMB}{url}");
        let (image_bytes, _) = fetch(&thumb_url, 40)?;
        let tmp_name: String = url
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_ascii_lowercase() || c.is_ascii_digit() {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("-")
            .chars()
            .take(60)
            .collect();
        let tmp = std::path::PathBuf::from(format!("/tmp/{tmp_name}.png"));
        std::fs::write(&tmp, &image_bytes)
            .with_context(|| format!("write {}", tmp.display()))?;
        let last_segment: String = url
            .split_once("://")
            .map(|(_, r)| r)
            .unwrap_or(url)
            .split(['?', '#'])
            .next()
            .unwrap_or("")
            .trim_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
            .chars()
            .take(40)
            .collect();
        let capitalized_family = {
            let mut chars = family.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        };
        reference_record::add(&AddArgs {
            catalog: slug.clone(),
            name: format!("{capitalized_family} \u{2014} {last_segment}"),
            source_url: url.clone(),
            category: family.clone(),
            selection_note: format!("auto-discovered from {start_url}; family {family}"),
            visual: tmp.display().to_string(),
            owner: Some(netloc_of(url).to_string()),
        })?;
    }

    super::reference_contract::regenerate_index()?;
    println!("done: catalog regenerated and consistent");
    Ok(())
}
