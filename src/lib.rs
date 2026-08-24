pub mod commands;

use anyhow::{bail, Context, Result};
use parking_lot::Mutex;
use std::time::Duration;

pub const USER_AGENT: &str =
    "WisentKronikaCorpus/0.1 (documentation writing-style research; +https://wisent.com)";

mod robots {
    use super::USER_AGENT;
    use parking_lot::Mutex;
    use std::collections::HashMap;
    use std::io::Read;
    use std::time::Duration;

    #[derive(Clone)]
    struct Rules {
        applies_to_us: bool,
        disallow: Vec<String>,
    }

    impl Rules {
        fn allows(&self, path: &str) -> bool {
            self.disallow.iter().all(|d| !path.starts_with(d.as_str()))
        }
    }


    fn compute(origin: &str) -> Rules {
        let mut rules =
            Rules { applies_to_us: false, disallow: Vec::new() };
        let url = format!("{origin}/robots.txt");
        if let Ok(resp) = ureq::get(&url)
            .timeout(Duration::from_secs(15))
            .set("User-Agent", USER_AGENT)
            .call()
        {
            if let Ok(body) = resp.into_string() {
                for line in body.lines() {
                    let line = line.split('#').next().unwrap_or("").trim();
                    if let Some(rest) = line.strip_prefix("User-agent:") {
                        let agent = rest.trim();
                        rules.applies_to_us = agent == "*" || USER_AGENT.starts_with(agent);
                        rules.disallow.clear();
                    } else if let Some(rest) = line.strip_prefix("Disallow:") {
                        if rules.applies_to_us && !rest.trim().is_empty() {
                            rules.disallow.push(rest.trim().to_string());
                        }
                    }
                }
            }
        }
        rules
    }

    static CACHE: Mutex<Option<HashMap<String, Rules>>> = Mutex::new(None);

    pub fn allows(url: &str) -> bool {
        let origin = super::origin_of(url);
        let path_and_query = url[origin.len()..].to_string();
        let rules = {
            let mut guard = CACHE.lock();
            guard
                .get_or_insert_with(HashMap::new)
                .entry(origin.to_string())
                .or_insert_with(|| compute(&origin))
                .clone()
        };
        rules.allows(&path_and_query)
    }
}

pub fn origin_of(url: &str) -> String {
    match url.split_once("://") {
        Some((scheme, rest)) => format!("{scheme}://{}", rest.split('/').next().unwrap_or_default()),
        None => String::new(),
    }
}

pub fn split_url(url: &str) -> (String, String) {
    let (scheme, after_scheme) = match url.split_once("://") {
        Some((s, r)) => (s, r),
        None => ("http", url),
    };
    match after_scheme.find('/') {
        Some(slash) => {
            let host = format!("{scheme}://{}", &after_scheme[..slash]);
            let path_q = after_scheme[slash..].to_string();
            (host, path_q)
        }
        None => (format!("{scheme}://{after_scheme}"), "/".into()),
    }
}

pub fn http_get_with_retry(url: &str, tries: u32) -> Result<(u16, String)> {
    let mut delay = 2.0f64;
    for attempt in 1..=tries {
        let resp = ureq::get(url)
            .timeout(Duration::from_secs(30))
            .set("User-Agent", USER_AGENT)
            .call();
        match resp {
            Ok(r) => {
                let status = r.status();
                let body = r.into_string().map_err(|e| anyhow::anyhow!("body read: {e}"))?;
                return Ok((status as u16, body));
            }
            Err(ureq::Error::Status(code, r)) if code == 429 || code >= 500 => {
                if attempt == tries {
                    bail!("HTTP {code} after {tries} attempts for {url}");
                }
                let wait = r
                    .header("Retry-After")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(delay);
                eprintln!("  backoff {wait:.0}s after HTTP {code} for {url}");
                std::thread::sleep(Duration::from_secs_f64(wait));
                delay *= 2.0;
            }
            Err(e) => return Err(e.into()),
        }
    }
    unreachable!("retry loop always returns or bails")
}

pub fn robots_allows(url: &str) -> bool {
    robots::allows(url)
}

fn between<'a>(hay: &'a str, open: &str, close: &str) -> Option<String> {
    let s = hay.find(open)? + open.len();
    let e = hay[s..].find(close)? + s;
    Some(html_unescape(hay[s..e].trim()))
}

pub fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}

/// Parse a sitemap document into (child sitemaps, [(page url, lastmod)]).
pub fn parse_sitemap(body: &[u8]) -> (Vec<String>, Vec<(String, Option<String>)>) {
    let text = String::from_utf8_lossy(body).to_string();
    let is_index = text.contains("<sitemapindex");
    let (open_tag, close_tag): (&str, &str) = if is_index {
        ("<sitemap>", "</sitemap>")
    } else {
        ("<url>", "</url>")
    };
    let mut children = Vec::new();
    let mut pages: Vec<(String, Option<String>)> = Vec::new();
    let mut rest = text.as_str();
    while let Some(start) = rest.find(open_tag) {
        let tail = &rest[start..];
        let end_rel = match tail.find(close_tag) {
            Some(e) => e + close_tag.len(),
            None => break,
        };
        let entry = &tail[..end_rel];
        if let Some(loc) = between(entry, "<loc>", "</loc>") {
            if is_index {
                children.push(loc);
            } else {
                pages.push((loc, between(entry, "<lastmod>", "</lastmod>")));
            }
        }
        rest = &tail[end_rel..];
    }
    (children, pages)
}

const SKIP_TAGS: &[&str] = &[
    "script", "style", "nav", "footer", "header", "aside", "form", "svg",
    "noscript", "template",
];

struct Extractor {
    out: String,
    title: String,
    skip_depth: usize,
    in_pre: bool,
    in_title: bool,
    buf: String,
}

impl Extractor {
    fn new() -> Self {
        Self {
            out: String::new(),
            title: String::new(),
            skip_depth: 0,
            in_pre: false,
            in_title: false,
            buf: String::new(),
        }
    }

    fn flush_text(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        let squeezed = squeeze(&std::mem::take(&mut self.buf));
        if squeezed.is_empty() {
            return;
        }
        if self.in_title {
            self.title.push_str(&squeezed);
        } else if self.in_pre {
            self.out.push_str(&squeezed);
        } else if !self.out.is_empty() && !self.out.ends_with('\n') {
            self.out.push(' ');
            self.out.push_str(&squeezed);
        } else {
            self.out.push_str(&squeezed);
        }
    }

    fn handle_tag(&mut self, inner: &str) {
        let lower = inner.to_ascii_lowercase();
        let closing = lower.starts_with('/');
        let bare = lower.trim_start_matches('/');
        let name: String = bare.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();

        if SKIP_TAGS.contains(&name.as_str()) {
            if closing {
                self.skip_depth = self.skip_depth.saturating_sub(1);
            } else if !inner.ends_with('/') {
                self.skip_depth += 1;
            }
            return;
        }
        if name == "title" {
            self.in_title = !closing;
            return;
        }
        if name == "pre" {
            self.flush_text();
            self.in_pre = !closing;
            self.out.push_str(if closing { "\n```\n" } else { "\n```\n" });
            return;
        }
        if closing || self.in_pre {
            return;
        }
        match name.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = name[1..].parse::<usize>().unwrap_or(6);
                self.flush_text();
                self.out.push_str(&format!("\n{} ", "#".repeat(level)));
            }
            "li" => {
                self.flush_text();
                self.out.push_str("\n- ");
            }
            "p" | "tr" | "section" | "article" | "br" | "div" | "blockquote" => {
                self.flush_text();
                self.out.push('\n');
            }
            _ => {}
        }
    }

    /// Feed the whole HTML string, scanning tag by tag.
    fn feed(mut self, html: &str) -> Self {
        let mut pos = 0usize;
        while pos < html.len() {
            match html[pos..].find('<') {
                None => {
                    self.buf.push_str(&html[pos..]);
                    pos = html.len();
                }
                Some(rel) => {
                    let tag_start = pos + rel;
                    if tag_start > pos {
                        self.buf.push_str(&html[pos..tag_start]);
                        self.flush_text();
                    }
                    match html[tag_start..].find('>') {
                        Some(gt) => {
                            let tag_src = html[tag_start + 1..tag_start + gt].trim();
                            self.handle_tag(tag_src);
                            pos = tag_start + gt + 1;
                        }
                        None => {
                            self.buf.push_str(&html[tag_start..]);
                            self.flush_text();
                            pos = html.len();
                        }
                    }
                }
            }
        }
        self.flush_text();
        self
    }

    fn result(self) -> (String, String) {
        let mut raw = self.out;
        while raw.contains("\n\n\n") {
            raw = raw.replace("\n\n\n", "\n\n");
        }
        let cleaned = raw.lines().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n");
        (cleaned.trim().to_string(), self.title.trim().to_string())
    }
}

fn squeeze(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

pub fn extract_text(html: &str) -> (String, String) {
    Extractor::new().feed(html).result()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn now_iso_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
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
    if m <= 2 {
        y += 1;
    }
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    serde_json::from_str(&text).with_context(|| format!("parse {path}"))
}

pub fn write_pretty_json(path: &str, value: &serde_json::Value) -> Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}
