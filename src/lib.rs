pub mod commands;
pub mod weles_provenance;

use anyhow::{bail, Context, Result};
use std::time::Duration;

pub const USER_AGENT: &str =
    concat!("Spis/", env!("CARGO_PKG_VERSION"), " (evidence-grade interface corpus; +https://spis.wisent.com/docs)");

/// Every object a crawl attempt publishes lives under this one prefix.
///
/// The `runs/` segment is not decoration. Stado authorizes object traffic per
/// namespace *prefix* (`object_api.namespaces`, `ObjectPrefixPolicy`), and a
/// prefix only matches as a prefix when it ends in `/`; a key that begins with
/// a per-run identifier can therefore be granted by nothing narrower than the
/// empty prefix, which is the whole namespace. Every one of the seventeen
/// namespaces Stado already declares grants a named first segment instead, so
/// the attempt tree carries one too and `spis-crawls` can be granted exactly
/// `runs/` with exactly `get`, `put` and `stat`.
pub const CRAWL_ATTEMPT_ROOT: &str = "stado://spis-crawls/runs";

/// The immutable coordinate of one record inside one run.
pub fn crawl_record_base_uri(
    run_id: &str,
    catalog: &str,
    record: &str,
    record_key: &str,
) -> String {
    format!("{CRAWL_ATTEMPT_ROOT}/{run_id}/{catalog}/{record}/{record_key}")
}

/// The immutable coordinate of one attempt of one record. Every producer and
/// every verifier of an attempt URI derives it here, so a change of shape can
/// never leave one side of a digest comparison spelling it the old way.
pub fn crawl_attempt_base_uri(
    run_id: &str,
    catalog: &str,
    record: &str,
    record_key: &str,
    // The attempt number is a u32 in the runtime manifest, a u64 in the
    // durable state and a serde integer in a receipt. It renders identically
    // in all three, and accepting them by display keeps one derivation
    // instead of a cast at every call site.
    attempt: impl std::fmt::Display,
    attempt_id: &str,
) -> String {
    format!(
        "{}/attempts/{attempt}/{attempt_id}",
        crawl_record_base_uri(run_id, catalog, record, record_key)
    )
}

mod robots {
    use super::USER_AGENT;
    use parking_lot::Mutex;
    use std::collections::HashMap;
    use std::time::Duration;

    #[derive(Clone)]
    struct Rule {
        pattern: regex::Regex,
        specificity: usize,
        allow: bool,
    }

    #[derive(Clone)]
    struct Rules {
        directives: Vec<Rule>,
    }

    impl Rules {
        fn allows(&self, path: &str) -> bool {
            self.directives.iter()
                .filter(|rule| rule.pattern.is_match(path))
                .max_by(|left, right| left.specificity.cmp(&right.specificity).then(left.allow.cmp(&right.allow)))
                .is_none_or(|rule| rule.allow)
        }
    }

    fn compute(origin: &str) -> Rules {
        let mut groups: Vec<(Vec<String>, Vec<Rule>)> = Vec::new();
        let url = format!("{origin}/robots.txt");
        let response = ureq::get(&url)
            .timeout(Duration::from_secs(15))
            .set("User-Agent", USER_AGENT)
            .call();
        let body = match response {
            Ok(response) => response.into_string().ok(),
            Err(ureq::Error::Status(status, _)) if status == 401 || status == 403 => {
                return Rules { directives: vec![Rule {
                    pattern: regex::Regex::new("^/").expect("fixed robots pattern"),
                    specificity: 1,
                    allow: false,
                }] };
            }
            Err(ureq::Error::Status(status, _)) if (400..500).contains(&status) => None,
            Err(_) => {
                return Rules { directives: vec![Rule {
                    pattern: regex::Regex::new("^/").expect("fixed robots pattern"),
                    specificity: 1,
                    allow: false,
                }] };
            }
        };
        if let Some(body) = body {
            let mut agents = Vec::new();
                let mut directives = Vec::new();
                for raw in body.lines() {
                    let line = raw.split('#').next().unwrap_or("").trim();
                    let Some((field, value)) = line.split_once(':') else { continue };
                    let field = field.trim();
                    let value = value.trim();
                    if field.eq_ignore_ascii_case("user-agent") {
                        if !directives.is_empty() {
                            groups.push((std::mem::take(&mut agents), std::mem::take(&mut directives)));
                        }
                        agents.push(value.to_ascii_lowercase());
                    } else if field.eq_ignore_ascii_case("allow") || field.eq_ignore_ascii_case("disallow") {
                        if !agents.is_empty() && !value.is_empty() {
                            let terminal = value.ends_with('$');
                            let source = value.strip_suffix('$').unwrap_or(value);
                            let expression = format!(
                                "^{}{}",
                                regex::escape(source).replace(r"\*", ".*"),
                                if terminal { "$" } else { "" }
                            );
                            if let Ok(pattern) = regex::Regex::new(&expression) {
                                directives.push(Rule {
                                    pattern,
                                    specificity: source.chars().filter(|character| *character != '*').count(),
                                    allow: field.eq_ignore_ascii_case("allow"),
                                });
                            }
                        }
                    }
                }
                if !agents.is_empty() {
                    groups.push((agents, directives));
                }
            }
        let user_agent = USER_AGENT.to_ascii_lowercase();
        let specificity = groups.iter().flat_map(|(agents, _)| agents)
            .filter_map(|agent| if agent == "*" { Some(0) } else if user_agent.starts_with(agent) { Some(agent.len()) } else { None })
            .max();
        let directives = specificity.map(|wanted| groups.into_iter()
            .filter(|(agents, _)| agents.iter().any(|agent| {
                (agent == "*" && wanted == 0) || (agent != "*" && agent.len() == wanted && user_agent.starts_with(agent))
            }))
            .flat_map(|(_, rules)| rules)
            .collect()).unwrap_or_default();
        Rules { directives }
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
        Some((scheme, rest)) => {
            format!("{scheme}://{}", rest.split('/').next().unwrap_or_default())
        }
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

/// Transparently decompress a gzipped payload (magic 1f 8b); passthrough otherwise.
pub fn maybe_gunzip(body: &[u8]) -> Vec<u8> {
    if body.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        let mut dec = flate2::read::MultiGzDecoder::new(body);
        if std::io::Read::read_to_end(&mut dec, &mut out).is_err() {
            return body.to_vec();
        }
        out
    } else {
        body.to_vec()
    }
}

pub fn http_get_with_retry(url: &str, tries: u32) -> Result<(u16, String)> {
    // Some servers stream bodies slower than any sane timeout; ureq's own
    // deadline does not always fire during body streaming, so each attempt
    // runs under a hard join-timeout. A timed-out attempt leaks its thread
    // until the server closes — accepted because such stragglers are rare.
    const ATTEMPT_DEADLINE: Duration = Duration::from_secs(45);

    for attempt in 1..=tries {
        let (tx, rx) = std::sync::mpsc::channel::<Result<(u16, String)>>();
        let url_owned = url.to_string();
        std::thread::spawn(move || {
            use std::io::Read as _;
            let resp = ureq::get(&url_owned)
                .timeout(ATTEMPT_DEADLINE)
                .set("User-Agent", USER_AGENT)
                .call();
            let result = match resp {
                Ok(r) => {
                    let status = r.status() as u16;
                    let mut body = Vec::new();
                    let read = r.into_reader().take(256 << 20).read_to_end(&mut body);
                    match read {
                        Ok(_) => Ok((status, String::from_utf8_lossy(&body).to_string())),
                        Err(e) => Err(anyhow::anyhow!("body read: {e}")),
                    }
                }
                Err(ureq::Error::Status(code, _)) => Err(anyhow::anyhow!("HTTP {code}")),
                Err(e) => Err(anyhow::anyhow!("{e}")),
            };
            let _ = tx.send(result);
        });
        match rx.recv_timeout(ATTEMPT_DEADLINE) {
            Ok(Ok(pair)) => return Ok(pair),
            Ok(Err(e)) => {
                let msg = format!("{e:#}");
                let retryable = ["HTTP 429", "HTTP 500", "HTTP 502", "HTTP 503", "HTTP 504"]
                    .iter()
                    .any(|m| msg.contains(m));
                if retryable && attempt < tries {
                    std::thread::sleep(Duration::from_secs(2));
                    continue;
                }
                bail!("{msg}");
            }
            Err(_) => bail!(
                "attempt timed out after {}s: {url}",
                ATTEMPT_DEADLINE.as_secs()
            ),
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
    "script", "style", "nav", "footer", "header", "aside", "form", "svg", "noscript", "template",
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
        let name: String = bare
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();

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
            self.out
                .push_str(if closing { "\n```\n" } else { "\n```\n" });
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
                    if self.skip_depth == 0 {
                        self.buf.push_str(&html[pos..]);
                    }
                    pos = html.len();
                }
                Some(rel) => {
                    let tag_start = pos + rel;
                    if tag_start > pos && self.skip_depth == 0 {
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
        let cleaned = raw
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n");
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
