//! `spis crawl-docs` — PARALLEL full-text crawler for the 50-reference set.
//!
//! One global work queue across all sites; a worker pool pulls from it so
//! every host is crawled simultaneously while each single host stays
//! rate-limited by its own token bucket (`--host-delay`). Per-site gzipped
//! JSONL files have exactly one writer thread each; the shared done-map is
//! flushed to state.json periodically and at the end. Resumable.

use crate as lib;
use anyhow::{bail, Context, Result};
use parking_lot::Mutex;
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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
            Override {
                sitemaps: &[],
                prefixes: &["/dotnet"],
                llms: &[],
            },
        ),
        (
            "21-google-cloud-documentation",
            Override {
                sitemaps: &[],
                prefixes: &["/docs"],
                llms: &[],
            },
        ),
        (
            "22-microsoft-azure-documentation",
            Override {
                sitemaps: &[],
                prefixes: &["/azure"],
                llms: &[],
            },
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

#[derive(serde::Deserialize, Clone)]
struct SiteMeta {
    name: String,
    source_url: String,
    #[serde(default)]
    inventory_source: String,
    #[serde(default)]
    landing_nav: Vec<LandingNavItem>,
}

#[derive(serde::Deserialize, Clone)]
struct LandingNavItem {
    path: String,
}

struct SiteRules {
    sitemaps: Vec<String>,
    llms: Vec<String>,
    prefixes: Vec<String>,
}

fn site_rules(slug: &str, meta: &SiteMeta, map: &HashMap<&'static str, Override>) -> SiteRules {
    let ov = map.get(slug);
    let mut prefixes: Vec<String> = ov
        .map(|o| o.prefixes.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();
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
        llms: ov
            .map(|o| o.llms.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default(),
        prefixes,
    }
}

fn resolve_urls(meta: &SiteMeta, rules: &SiteRules, delay: f64) -> Vec<(String, Option<String>)> {
    let base = &meta.source_url;
    let origin = lib::origin_of(base);
    let mut urls: Vec<(String, Option<String>)> = Vec::new();
    let mut visited: Vec<String> = Vec::new();

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
        if visited.contains(&src) || visited.len() > 96 {
            continue;
        }
        visited.push(src.clone());
        if !lib::robots_allows(&src) {
            continue;
        }
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
                                    urls.push((
                                        if link.starts_with("http") {
                                            link
                                        } else {
                                            format!("{origin}{link}")
                                        },
                                        None,
                                    ));
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

// ---------- parallel fetch engine ----------

struct HostGate {
    next_allowed: Mutex<HashMap<String, std::time::Instant>>,
    host_delay: f64,
}

impl HostGate {
    fn new(host_delay: f64) -> Self {
        Self {
            next_allowed: Mutex::new(HashMap::new()),
            host_delay,
        }
    }

    /// Block until this host's next slot, then reserve it.
    fn wait_turn(&self, url: &str) {
        loop {
            let now = std::time::Instant::now();
            let host = lib::origin_of(url);
            let mut slots = self.next_allowed.lock();
            let slot = slots.entry(host).or_insert(now);
            if *slot <= now {
                *slot = now + std::time::Duration::from_secs_f64(self.host_delay);
                return;
            }
            let wait = *slot - now;
            drop(slots);
            std::thread::sleep(wait);
        }
    }
}

struct SiteJob {
    slug: String,
    out_path: PathBuf,
    state_path: PathBuf,
    targets: Vec<(String, Option<String>)>,
    done: HashMap<String, serde_json::Value>,
}

struct Shared {
    queue: Mutex<std::vec::IntoIter<(String, String, Option<String>)>>,
    done: Mutex<HashMap<String, HashMap<String, serde_json::Value>>>,
    writers: Mutex<HashMap<String, std::sync::mpsc::Sender<String>>>,
    gate: HostGate,
    fetched: AtomicUsize,
    stop: std::sync::atomic::AtomicBool,
}

fn run_worker(rest: &[String]) -> Result<()> {
    let mut site: Option<String> = None;
    let mut all = false;
    let mut exclude: Vec<String> = Vec::new();
    let mut workers: usize = 64;
    let mut host_delay: f64 = 0.3;

    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--site" => {
                i += 1;
                site = Some(rest.get(i).context("--site needs a value")?.clone());
            }
            "--all" => all = true,
            "--exclude" => {
                i += 1;
                exclude.push(rest.get(i).context("--exclude needs a value")?.clone());
            }
            "--workers" => {
                i += 1;
                workers = rest.get(i).context("--workers needs a value")?.parse()?;
            }
            "--host-delay" => {
                i += 1;
                host_delay = rest.get(i).context("--host-delay needs a value")?.parse()?;
            }
            "--refresh" => {}
            other => bail!("unknown argument: {other}"),
        }
        i += 1;
    }
    if site.is_none() && !all {
        bail!("pass --site <NN-slug> or --all");
    }

    let structure_dir = PathBuf::from("documentation-site-examples/content-structure");
    let mut slugs: Vec<String> = std::fs::read_dir(&structure_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|f| f.ends_with(".json"))
        .map(|f| f.trim_end_matches(".json").to_string())
        .filter(|f| f != "full-text-manifest")
        .collect();
    slugs.sort();

    let chosen: Vec<String> = match &site {
        Some(s) => vec![s.clone()],
        None => slugs
            .iter()
            .filter(|s| !exclude.contains(s))
            .cloned()
            .collect(),
    };
    for c in &chosen {
        if !slugs.contains(c) {
            bail!("unknown site: {c}");
        }
    }

    let map = overrides();

    // Phase 1 (sequential): resolve inventories and load prior state.
    let mut jobs: Vec<SiteJob> = Vec::new();
    for slug in &chosen {
        let meta: SiteMeta =
            lib::read_json(structure_dir.join(format!("{slug}.json")).to_str().unwrap())?;
        let dir = data_dir(slug);
        std::fs::create_dir_all(&dir)?;
        let state_path = dir.join("state.json");
        let done: HashMap<String, serde_json::Value> = if !state_path.exists() {
            HashMap::new()
        } else {
            lib::read_json(state_path.to_str().unwrap())?
        };
        eprintln!(
            "[{}] {slug} ({}): resolving URL inventory ({})",
            lib::now_iso_utc(),
            meta.name,
            meta.inventory_source
        );
        let rules = site_rules(slug, &meta, &map);
        let targets = resolve_urls(&meta, &rules, 0.25);
        eprintln!(
            "[{}] {slug}: {} candidate URLs",
            lib::now_iso_utc(),
            targets.len()
        );
        jobs.push(SiteJob {
            slug: slug.clone(),
            out_path: dir.join("pages.jsonl.gz"),
            state_path,
            targets,
            done,
        });
    }

    // Phase 2: flat queue across sites.
    let mut queue: Vec<(String, String, Option<String>)> = Vec::new();
    for job in &jobs {
        for (url, lm) in &job.targets {
            let h = lib::sha256_hex(url.as_bytes())[..16].to_string();
            if job.done.contains_key(&h) {
                continue;
            }
            queue.push((job.slug.clone(), url.clone(), lm.clone()));
        }
    }
    queue.reverse(); // pop() takes from the end; restore original order
    let total_pending = queue.len();
    eprintln!(
        "[{}] queue ready: {total_pending} URLs across {} sites; workers={workers} host-delay={host_delay}s",
        lib::now_iso_utc(),
        chosen.len()
    );

    // One writer thread per site keeps each gz single-stream.
    let (tx_map, rxs): (Vec<_>, Vec<_>) = jobs
        .iter()
        .map(|j| {
            let (tx, rx) = std::sync::mpsc::channel::<String>();
            ((j.slug.clone(), tx), rx)
        })
        .unzip();
    let writer_handles: Vec<_> = jobs
        .iter()
        .zip(rxs)
        .map(|(job, rx)| {
            let rx = parking_lot::Mutex::new(rx);
            let path = job.out_path.clone();
            std::thread::spawn(move || -> Result<()> {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)?;
                let mut enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
                loop {
                    match rx.lock().try_recv() {
                        Ok(line) => writeln!(enc, "{line}")?,
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            std::thread::sleep(std::time::Duration::from_millis(25));
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                    }
                }
                enc.finish()?;
                Ok(())
            })
        })
        .collect();

    let shared = Arc::new(Shared {
        queue: Mutex::new(queue.into_iter()),
        done: Mutex::new(
            jobs.iter()
                .map(|j| (j.slug.clone(), j.done.clone()))
                .collect(),
        ),
        writers: Mutex::new(tx_map.into_iter().collect()),
        gate: HostGate::new(host_delay),
        fetched: AtomicUsize::new(0),
        stop: std::sync::atomic::AtomicBool::new(false),
    });

    // Periodic state flusher so long runs are resumable after crashes.
    {
        let flusher_shared = Arc::clone(&shared);
        let flusher_jobs_state: Vec<(String, PathBuf)> = jobs
            .iter()
            .map(|j| (j.slug.clone(), j.state_path.clone()))
            .collect();
        std::thread::spawn(move || loop {
            if flusher_shared
                .stop
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(10));
            let snapshot = flusher_shared.done.lock();
            for (slug, path) in &flusher_jobs_state {
                if let Some(dm) = snapshot.get(slug) {
                    let tmp = path.with_extension("json.tmp");
                    if std::fs::write(&tmp, serde_json::to_string_pretty(dm).unwrap_or_default())
                        .is_ok()
                    {
                        let _ = std::fs::rename(&tmp, path);
                    }
                }
            }
        });
    }

    std::thread::scope(|scope| {
        for _ in 0..workers {
            let shared = Arc::clone(&shared);
            scope.spawn(move || loop {
                let (slug, url, lastmod) = {
                    let mut q = shared.queue.lock();
                    match q.next() {
                        Some(t) => t,
                        None => break,
                    }
                };
                if !lib::robots_allows(&url) {
                    continue;
                }
                shared.gate.wait_turn(&url);
                let h = lib::sha256_hex(url.as_bytes())[..16].to_string();
                match lib::http_get_with_retry(&url, 2) {
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
                        let rec = json!({
                            "url": url,
                            "fetched_at": lib::now_iso_utc(),
                            "status": status,
                            "quality": quality,
                            "sha256": lib::sha256_hex(body.as_bytes()),
                            "bytes": body.len(),
                            "title": (!title.is_empty()).then_some(title),
                            "text": (quality == "ok").then_some(text),
                            "lastmod": lastmod,
                        });
                        if let Ok(line) = serde_json::to_string(&rec) {
                            let writers = shared.writers.lock();
                            if let Some(tx) = writers.get(&slug) {
                                let _ = tx.send(line);
                            }
                        }
                        shared.done.lock().entry(slug).or_default().insert(h, json!({"url": url, "status": status}));
                        shared.fetched.fetch_add(1, Ordering::Relaxed);
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
                        shared.done.lock().entry(slug).or_default().insert(h, entry);
                    }
                }
            });
        }
    });
    // Workers are done: drop the per-site senders so writer threads see
    // Disconnected, drain their buffers, and finish. Then flush states.
    shared.writers.lock().clear();
    for h in writer_handles {
        let _ = h.join();
    }

    let mut results = Vec::new();
    {
        let done_final = shared.done.lock();
        for job in &jobs {
            let empty = HashMap::new();
            let dm = done_final.get(&job.slug).unwrap_or(&empty);
            let ok_count = dm
                .values()
                .filter(|v| v.get("status").and_then(|s| s.as_u64()) == Some(200))
                .count();
            let tmp = job.state_path.with_extension("json.tmp");
            std::fs::write(&tmp, serde_json::to_string_pretty(dm)?)?;
            std::fs::rename(&tmp, &job.state_path)?;
            results.push(json!({
                "slug": job.slug,
                "candidates": job.targets.len(),
                "seen": dm.len(),
                "cumulative_ok": ok_count,
            }));
        }
    }
    // Scrape-run record: one auditable object per execution.
    let definition_path = structure_dir.join("full-text-manifest.json");
    let definition_hash = lib::sha256_hex(&std::fs::read(&definition_path).unwrap_or_default());
    let tool_sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let total_seen: usize = results
        .iter()
        .map(|r| r["seen"].as_u64().unwrap_or(0) as usize)
        .sum();
    let run_record = json!({
        "schema": "wisent.crawl-run.v1",
        "tool": "spis crawl-docs",
        "tool_commit": tool_sha,
        "definition_sha256": definition_hash,
        "started_at": lib::now_iso_utc(),
        "sites": chosen.len(),
        "urls_pending_at_start": total_pending,
        "urls_seen_after": total_seen,
    });
    let manifest_run = structure_dir.join("crawl-run.json");
    std::fs::write(
        &manifest_run,
        serde_json::to_string_pretty(&run_record)? + "\n",
    )?;

    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}

fn data_dir(slug: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".spis/docs-corpus").join(slug)
}

const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    if value.is_empty()
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{flag} contains characters that cannot be submitted to a worker");
    }
    Ok(())
}

fn source_revision() -> Result<String> {
    let output = std::process::Command::new("git")
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

fn publish_corpus(uri: &str) -> Result<()> {
    if !uri.starts_with("stado://spis-crawls/documentation-site-examples/") {
        bail!("documentation artifact URI is outside the Spis crawl namespace");
    }
    let root = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".spis")
        .join("docs-corpus");
    let archive = root.with_extension("tar.gz");
    let status = std::process::Command::new("stado")
        .args(["storage", "archive"])
        .arg(&root)
        .arg(&archive)
        .status()
        .context("archive documentation corpus")?;
    if !status.success() {
        bail!("stado storage archive refused documentation corpus");
    }
    let status = std::process::Command::new("stado")
        .args(["storage", "put", "--if-absent", uri])
        .arg(&archive)
        .status()
        .context("publish documentation corpus")?;
    if !status.success() {
        bail!("stado storage put refused documentation corpus");
    }
    Ok(())
}

fn submit_worker(host: &str, arguments: &[String]) -> Result<()> {
    safe_job_value(host, "--host")?;
    for value in arguments {
        safe_job_value(value, "crawl-docs argument")?;
    }
    let revision = source_revision()?;
    let stamp = crate::now_iso_utc().replace(':', "-");
    let artifact = format!("stado://spis-crawls/documentation-site-examples/{stamp}.tar.gz");
    let command = format!(
        "cargo run --release -- crawl-docs --worker {} --artifact-uri {}",
        arguments.join(" "),
        artifact
    );
    let output = std::process::Command::new("stado")
        .args([
            "submit",
            &command,
            "--pinned-host",
            host,
            "--repo",
            REPOSITORY,
            "--repo-ref",
            &revision,
            "--repo-workdir",
            "spis",
            "--repo-extras",
            "",
            "--output-uri",
            &format!("stado://spis-crawls/documentation-site-examples/{stamp}/job-output"),
        ])
        .output()
        .context("submit documentation crawl through Stado")?;
    if !output.status.success() {
        bail!(
            "Stado refused documentation crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut host = None;
    let mut worker = false;
    let mut artifact_uri = None;
    let mut forwarded = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--worker" => worker = true,
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            value => forwarded.push(value.to_string()),
        }
        i += 1;
    }
    if !worker {
        return submit_worker(
            &host
                .context("--host is required; documentation crawls execute as pinned Stado jobs")?,
            &forwarded,
        );
    }
    if host.is_some() {
        bail!("--host cannot be used with --worker");
    }
    run_worker(&forwarded)?;
    if let Some(uri) = artifact_uri {
        publish_corpus(&uri)?;
    }
    Ok(())
}
