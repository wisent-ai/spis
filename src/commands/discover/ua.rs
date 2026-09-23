use super::*;

pub(crate) const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) spis-discovery/1.0";

pub(crate) const THUMB: &str = "https://image.thum.io/get/width/1400/crop/1000/noanimate/";

pub(crate) const FAMILIES: &[&str] = &[
    "pricing", "docs", "signup", "about", "product", "blog", "other",
];

/// GET raw bytes with the discovery User-Agent; returns (body, content type).
pub(crate) fn fetch(url: &str, timeout_secs: u64) -> Result<(Vec<u8>, String)> {
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
pub(crate) fn ascii_lower(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_uppercase() {
                c.to_ascii_lowercase()
            } else {
                c
            }
        })
        .collect()
}

/// Insertion-ordered href -> text map, first text wins (`setdefault`).
#[derive(Default)]
pub(crate) struct Links {
    pub(crate) entries: Vec<(String, String)>,
}

impl Links {
    pub(crate) fn setdefault(&mut self, href: String, text: String) {
        if !self.entries.iter().any(|(h, _)| *h == href) {
            self.entries.push((href, text));
        }
    }
}

pub(crate) fn netloc_of(url: &str) -> &str {
    let after_scheme = match url.split_once("://") {
        Some((_, r)) => r,
        None => return "",
    };
    after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
}

pub(crate) fn scheme_of(url: &str) -> &str {
    match url.split_once("://") {
        Some((scheme, _)) => scheme,
        None => "",
    }
}

/// True when `rel` carries a URI scheme (`^[a-zA-Z][a-zA-Z0-9+.-]*:`).
pub(crate) fn has_scheme(rel: &str) -> bool {
    match rel.find(':') {
        Some(colon) if colon > 0 => {
            let scheme = &rel[..colon];
            scheme
                .chars()
                .enumerate()
                .all(|(i, c)| c.is_ascii_alphanumeric() || (i > 0 && "+.-".contains(c)))
                && scheme
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic())
        }
        _ => false,
    }
}

/// Minimal stand-in for `urllib.parse.urljoin` covering the shapes found in nav
/// markup: absolute URLs, protocol-relative, root-relative, query-only, and
/// relative paths with dot segments.
pub(crate) fn urljoin(base: &str, rel: &str) -> String {
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
pub(crate) fn remove_dot_segments(path: &str) -> String {
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

pub(crate) fn same_origin(url: &str, origin: &str) -> bool {
    matches!(scheme_of(url), "http" | "https") && netloc_of(url) == netloc_of(origin)
}

/// Pull `<a href>` -> collapsed inner text from HTML, mimicking the original
/// `HTMLParser` subclass: first text per href, unclosed anchors dropped, text
/// inside nested tags still counted.
pub(crate) fn parse_links(html: &str) -> Links {
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
        let Some(close_rel) = lower[open..].find('>') else {
            break;
        };
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
                    let text: String = pending_text
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");
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
pub(crate) fn attr_value(tag_body: &str, name: &str) -> Option<String> {
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

pub(crate) fn extract_links(start_url: &str, html_bytes: &[u8], limit: usize) -> Links {
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
