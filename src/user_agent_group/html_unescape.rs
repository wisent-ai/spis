use super::*;

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

pub(crate) struct Extractor {
    pub(crate) out: String,
    pub(crate) title: String,
    /// The open elements whose content renders no text.
    pub(crate) hidden: Vec<super::hidden::HiddenElement>,
    pub(crate) in_pre: bool,
    pub(crate) in_title: bool,
    pub(crate) buf: String,
    /// The page marks its main content with `<main>`
    /// (https://html.spec.whatwg.org/multipage/grouping-content.html#the-main-element),
    /// so only text inside it is kept; a page without one is kept whole.
    pub(crate) main_only: bool,
    /// The `<main>` elements open at this point of the page.
    pub(crate) open_mains: Vec<String>,
}

impl Extractor {
    pub(crate) fn new(main_only: bool) -> Self {
        Self {
            out: String::new(),
            title: String::new(),
            hidden: Vec::new(),
            in_pre: false,
            in_title: false,
            buf: String::new(),
            main_only,
            open_mains: Vec::new(),
        }
    }

    pub(crate) fn flush_text(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        let squeezed = squeeze(&std::mem::take(&mut self.buf));
        if squeezed.is_empty() {
            return;
        }
        if self.in_title {
            self.title.push_str(&squeezed);
        } else if self.main_only && self.open_mains.is_empty() {
            // Outside the page's main content.
        } else if self.in_pre {
            self.out.push_str(&squeezed);
        } else if !self.out.is_empty() && !self.out.ends_with('\n') {
            self.out.push(' ');
            self.out.push_str(&squeezed);
        } else {
            self.out.push_str(&squeezed);
        }
    }

    pub(crate) fn handle_tag(&mut self, inner: &str) {
        let lower = inner.to_ascii_lowercase();
        let closing = lower.starts_with('/');
        let bare = lower.trim_start_matches('/');
        let name: String = bare
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();

        if name == "main" {
            self.flush_text();
            if closing {
                self.open_mains.pop();
            } else {
                self.open_mains.push(name);
            }
            return;
        }
        if let Some(element) = super::hidden::HiddenElement::named(&name) {
            if closing {
                self.hidden.pop();
            } else if !inner.ends_with('/') {
                self.hidden.push(element);
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
    pub(crate) fn feed(mut self, html: &str) -> Self {
        let mut pos = 0usize;
        while pos < html.len() {
            match html[pos..].find('<') {
                None => {
                    if self.hidden.is_empty() {
                        self.buf.push_str(&html[pos..]);
                    }
                    pos = html.len();
                }
                Some(rel) => {
                    let tag_start = pos + rel;
                    if tag_start > pos && self.hidden.is_empty() {
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

    pub(crate) fn result(self) -> (String, String) {
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

pub(crate) fn squeeze(s: &str) -> String {
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

/// The text and title of an HTML page: its `<main>` content when it marks
/// one, otherwise all of it, without the elements that render no text.
pub fn extract_text(html: &str) -> (String, String) {
    let main_only = html.to_ascii_lowercase().contains("<main");
    Extractor::new(main_only).feed(html).result()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The SHA-256 of one file's bytes, streamed. Every executable, helper and
/// visual digest Spis records or checks is read through this; a failure
/// names the file.
pub fn sha256_file(path: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut file =
        std::fs::File::open(path).with_context(|| format!("open {} to hash it", path.display()))?;
    let mut digest = Sha256::new();
    std::io::copy(&mut file, &mut digest).with_context(|| format!("hash {}", path.display()))?;
    Ok(hex::encode(digest.finalize()))
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

/// The JSON document at `path`; a file that cannot be read or parsed is
/// refused naming the path and the cause.
pub fn read_json<T: serde::de::DeserializeOwned>(path: impl AsRef<std::path::Path>) -> Result<T> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

/// Write `value` as pretty JSON to `path`; a failure names the path.
pub fn write_pretty_json(
    path: impl AsRef<std::path::Path>,
    value: &serde_json::Value,
) -> Result<()> {
    let path = path.as_ref();
    std::fs::write(path, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}
