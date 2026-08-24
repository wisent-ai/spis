//! `spis crawl-docs` — full-text crawl of the 50-reference documentation set.
//!
//! Reads documentation-site-examples/content-structure/<NN-slug>.json,
//! resolves page URLs per site (robots sitemaps, sitemap indexes, llms.txt,
//! or recorded landing navigation), fetches politely, extracts readable
//! text, and appends gzipped JSONL under ~/.spis/docs-corpus/<slug>/.
//! Resumable via state.json; nothing here talks to a model.

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

struct Override {
    sitemaps: &'static [&'static str],
    prefixes: &'static [&'static str],
    llms: &'static [&'static str],
}

fn overrides() -> HashMap<&'static str, Override> {
    HashMap::from([
        (
            "01-mdn-web-docs",
            Override {
                sitemaps: &["https://developer.mozilla.org/sitemap.xml"],
                prefixes: &["/en-US/docs/Web"],
                llms: &[],
            },
        ),
        (
            "12-net-documentation",
            Override { sitemaps: &[], prefixes: &["/dotnet"], llms: &[] },
        ),
        (
            "21-google-cloud-documentation",
            Override { sitemaps: &[], prefixes: &["/docs"], llms: &[] },
        ),
        (
            "22-microsoft-azure-documentation",
            Override { sitemaps: &[], prefixes: &["/azure"], llms: &[] },
        ),
        (
            "35-openai-api-documentation",
            Override {
                sitemaps: &[],
                prefixes: &[],
                llms: &["https://platform.claude.com/llms.txt"],
            },
        ),
        (
            "38-postgresql-documentation",
            Override {
                sitemaps: &[],
                prefixes: &["/docs/current", "/docs/17"],
                llms: &[],
            },
        ),
        (
            "48-atlassian-design-system",
            Override {
                sitemaps: &[],
                prefixes: &[],
                llms: &["https://atlassian.design/llms.txt"],
            },
        ),
    ])
}

#[derive(serde::Deserialize)]
struct SiteMeta {
    name: String,
    source_url: String,
    #[serde(default)]
    inventory_source: String,
    #[serde(default)]
    landing_nav: Vec<LandingNav>,
}

#[derive(serde::Deserialize)]
struct LandingNav {
    path: String,
}

struct Args {
    site: Option<String>,
    all: bool,
    exclude: Vec<String>,
    max_pages: usize,
    delay: f64,
    refresh: bool,
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut args = Args {
        site: None,
        all: false,
        exclude: Vec::new(),
        max_pages: 0,
        delay: 1.0,
        refresh: false,
    };
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--site" => {
                i += 1;
                args.site = Some(rest.get(i).context("--site needs a value")?.clone());
            }
            "--all" => args.all = true,
            "--exclude" => {
                i += 1;
                args.exclude.push(rest.get(i).context("--exclude needs a value")?.clone());
            }
            "--max-pages" => {
                i += 1;
                args.max_pages = rest.get(i).context("--max-pages needs a value")?.parse()?;
            }
            "--delay" => {
                i += 1;
                args.delay = rest.get(i).context("--delay needs a value")?.parse()?;
            }
            "--refresh" => args.refresh = true,
            other => bail!("unknown argument: {other}"),
        }
        i += 1;
    }
    if args.site.is_none() && !args.all {
        bail!("pass --site <NN-slug> or --all");
    }

    let structure_dir = PathBuf::from("documentation-site-examples/content-structure");
    let mut slugs: Vec<String> = std::fs::read_dir(&structure_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|f| f.ends_with(".json"))
        .map(|f| f.trim_end_matches(".json").to_string())
        .collect();
    slugs.sort();

    let chosen: Vec<String> = match &args.site {
        Some(s) => vec![s.clone()],
        None => slugs.iter().filter(|s| !args.exclude.contains(s)).cloned().collect(),
    };
    for c in &chosen {
        if !slugs.contains(c) {
            bail!("unknown site: {c}");
        }
    }

    let map = overrides();
    let mut results: Vec<serde_json::Value> = Vec::new();
    for slug in &chosen {
        let meta_path = structure_dir.join(format!("{slug}.json"));
        let meta: SiteMeta = lib::read_json(meta_path.to_str().unwrap())?;
        match crawl_site(slug, &meta, &map, &args) {
            Ok(r) => match serde_json::to_value(&r) {
                Ok(v) => results.push(v),
                Err(e) => anyhow::bail!("serialize crawl result: {e}"),
            },
            Err(e) => {
                eprintln!("[fatal] {slug}: {e:#}");
                results.push(json!({"slug": slug, "error": format!("{e:#}")}));
                break;
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}

struct SiteRules {
    sitemaps: Vec<String>,
    llms: Vec<String>,
    prefixes: Vec<String>,
}

fn site_rules(slug: &str, meta: &SiteMeta, map: &HashMap<&'static str, Override>) -> SiteRules {
    let ov = map.get(slug);
    let mut prefixes: Vec<String> =
        ov.map(|o| o.prefixes.iter().map(|s| s.to_string()).collect()).unwrap_or_default();
    if prefixes.is_empty() {
        if let Some(inv) = meta.inventory_source.strip_prefix("scoped sitemap (") {
            if let Some(inner) = inv.strip_suffix(')') {
                prefixes = inner.split(',').map(|p| p.trim().to_string()).collect();
            }
        }
    }
    SiteRules {
        sitemaps: ov
            .map(|o| o.sitemaps.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default(),
        llms: ov.map(|o| o.llms.iter().map(|s| s.to_string()).collect()).unwrap_or_default(),
        prefixes,
    }
}

fn resolve_urls(
    slug: &str,
    meta: &SiteMeta,
    rules: &SiteRules,
    delay: f64,
) -> Vec<(String, Option<String>)> {
    let base = &meta.source_url;
    let (origin, _) = lib::split_url(base);
    let mut urls: Vec<(String, Option<String>)> = Vec::new();
    let mut visited_sources: Vec<String> = Vec::new();

    let mut queue: Vec<String> = Vec::new();
    queue.extend(rules.sitemaps.clone());
    if rules.sitemaps.is_empty() && rules.llms.is_empty() {
        let robots_txt = format!("{origin}/robots.txt");
        if lib::robots_allows(&robots_txt) {
            match lib::http_get_with_retry(&robots_txt, 2) {
                Ok((_, body)) => {
                    for line in body.lines() {
                        let t = line.trim();
                        if let Some(v) = t.strip_prefix("Sitemap:") {
                            queue.push(v.trim().to_string());
                        } else if let Some(v) = t.strip_prefix("sitemap:") {
                            queue.push(v.trim().to_string());
                        }
                    }
                }
                Err(e) => eprintln!("  robots fetch failed: {e}"),
            }
        }
        queue.push(format!("{origin}/sitemap.xml"));
    }
    queue.extend(rules.llms.clone());

    while let Some(src) = queue.pop() {
        if visited_sources.contains(&src) || visited_sources.len() > 64 {
            continue;
        }
        visited_sources.push(src.clone());
        if !lib::robots_allows(&src) {
            continue;
        }
        eprintln!("  sitemap source: {src}");
        match lib::http_get_with_retry(&src, 2) {
            Ok((_, body)) => {
                let payload = body;
                if payload.contains("<urlset") || payload.contains("<sitemapindex") {
                    let (children, pages) = lib::parse_sitemap(payload.as_bytes());
                    for c in children {
                        queue.push(c);
                    }
                    urls.extend(pages);
                } else if src.ends_with(".txt") && src.contains("llms") {
                    for line in payload.lines() {
                        let t = line.trim();
                        if let Some(rest) = t.strip_prefix("- [") {
                            if let Some(close) = rest.find("](") {
                                let after = &rest[close + 2..];
                                if let Some(endq) = after.find(')') {
                                    let link = after[..endq].to_string();
                                    let joined = if link.starts_with("http") {
                                        link
                                    } else {
                                        format!("{origin}{link}")
                                    };
                                    urls.push((joined, None));
                                }
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs_f64(delay.max(0.25)));
            }
            Err(e) => eprintln!("  source error {src}: {e}"),
        }
    }

    if !rules.prefixes.is_empty() {
        urls.retain(|(u, _)| {
            let (_, path_q) = lib::split_url(u);
            rules.prefixes.iter().any(|p| path_q.contains(p.as_str()))
        });
    }

    let mut deduped: Vec<(String, Option<String>)> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (u, lm) in urls {
        let u = u.trim_end_matches('/').to_string();
        if !u.is_empty() && seen.insert(u.clone()) {
            deduped.push((u, lm));
        }
    }
    if deduped.is_empty() && meta.inventory_source.starts_with("landing-nav") {
        for item in &meta.landing_nav {
            deduped.push((format!("{origin}{}", item.path), None));
        }
    }
    deduped
}

fn data_dir(slug: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".spis/docs-corpus").join(slug)
}

#[derive(serde::Serialize)]
struct CrawlResult {
    slug: String,
    candidates: usize,
    fetched_this_run: usize,
    cumulative_ok: usize,
    seen: usize,
}

fn crawl_site(slug: &str, meta: &SiteMeta, map: &HashMap<&'static str, Override>, args: &Args) -> Result<CrawlResult> {
    let dir = data_dir(slug);
    std::fs::create_dir_all(&dir)?;
    let out_path = dir.join("pages.jsonl.gz");
    let state_path = dir.join("state.json");

    let done: HashMap<String, serde_json::Value> = if args.refresh || !state_path.exists() {
        HashMap::new()
    } else {
        lib::read_json(state_path.to_str().unwrap())?
    };

    eprintln!(
        "[{}] {slug}: resolving URL inventory ({})",
        lib::now_iso_utc(),
        meta.inventory_source
    );
    let rules = site_rules(slug, meta, map);
    let targets = resolve_urls(slug, meta, &rules, args.delay);
    eprintln!("[{}] {slug}: {} candidate URLs", lib::now_iso_utc(), targets.len());

    let append_mode = out_path.exists();
    let file = std::fs::OpenOptions::new().create(true).append(true).open(&out_path)?;
    let mut out = flate2::write::GzEncoder::new(file, flate2::Compression::default());

    let mut done_mut = done;
    let mut fetched = 0usize;
    let sleep = std::time::Duration::from_secs_f64(args.delay);

    for (url, lastmod) in &targets {
        let h = lib::sha256_hex(url.as_bytes())[..16].to_string();
        if !args.refresh && done_mut.contains_key(&h) {
            continue;
        }
        if args.max_pages > 0 && fetched >= args.max_pages {
            break;
        }
        if !lib::robots_allows(url) {
            done_mut.insert(h.clone(), json!({"url": url, "status": "robots_disallowed"}));
            continue;
        }
        match lib::http_get_with_retry(url, 3) {
            Ok((status, body)) => {
                let head = body.chars().take(4096).collect::<String>().to_lowercase();
                let looks_html = head.contains("<html") || head.contains("<title");
                let (text, title) = if looks_html {
                    lib::extract_text(&body)
                } else {
                    (String::new(), String::new())
                };
                let brace_density = if text.is_empty() {
                    0.0
                } else {
                    text.matches('{').count() as f64 * 100.0 / text.len() as f64
                };
                let quality = if brace_density > 1.0 { "css_js_noise" } else { "ok" };
                let keep_text = quality == "ok" && !text.is_empty();
                let rec = json!({
                    "url": url,
                    "fetched_at": lib::now_iso_utc(),
                    "status": status,
                    "quality": quality,
                    "sha256": lib::sha256_hex(body.as_bytes()),
                    "bytes": body.len(),
                    "title": (!title.is_empty()).then_some(title),
                    "text": keep_text.then_some(text),
                    "lastmod": lastmod,
                });
                writeln!(out, "{rec}")?;
                done_mut.insert(h.clone(), json!({"url": url, "status": status}));
                fetched += 1;
                std::thread::sleep(sleep);
            }
            Err(e) => {
                let msg = format!("{e:#}");
                let code = msg
                    .split("HTTP ")
                    .nth(1)
                    .and_then(|r| r.split(' ').next())
                    .and_then(|c| c.parse::<u16>().ok());
                let entry = match code {
                    Some(code) => json!({"url": url, "status": code}),
                    None => json!({"url": url, "status": "error", "detail": msg.chars().take(200).collect::<String>()}),
                };
                done_mut.insert(h, entry);
                eprintln!("  {msg}");
            }
        }
        if fetched % 25 == 0 {
            lib::write_pretty_json(state_path.to_str().unwrap(), &serde_json::to_value(&done_mut)?)?;
        }
    }
    out.finish()?;
    lib::write_pretty_json(state_path.to_str().unwrap(), &serde_json::to_value(&done_mut)?)?;

    let cumulative_ok = done_mut
        .values()
        .filter(|v| v.get("status").and_then(|s| s.as_u64()) == Some(200))
        .count();
    eprintln!(
        "[{}] {slug}: fetched this run {}; cumulative ok {cumulative_ok} of {} seen",
        lib::now_iso_utc(),
        fetched,
        done_mut.len()
    );
    Ok(CrawlResult {
        slug: slug.to_string(),
        candidates: targets.len(),
        fetched_this_run: fetched,
        cumulative_ok,
        seen: done_mut.len(),
    })
}
