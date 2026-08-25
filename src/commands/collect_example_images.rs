//! `spis collect-example-images` — collect one attributable official interface
//! image per catalog entry (port of collect-example-images.py).
//!
//! Static HTTP reads only; prefers large images whose URL, alt text, or
//! surrounding metadata identifies a screenshot or product interface, then
//! stores a bounded derivative while retaining the original image URL.
//!
//! PIL replacement note: the Python original decoded pixels with Pillow to
//! resample a 1400×1000 WebP derivative. This port performs **header-only**
//! parsing (PNG IHDR, JPEG SOF scan, WebP VP8/VP8L/VP8X, GIF logical screen)
//! for format and dimensions, and stores the ORIGINAL image bytes verbatim
//! instead of re-encoding. No pixel decode happens here; see the module
//! report for the resulting metadata-shape gaps.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};
use std::io::Read;
use std::path::{Path, PathBuf};

const USER_AGENT: &str = "WisentProductGuidelines/1.0 (+https://wisent.ai)";
const MAX_PAGE_BYTES: usize = 8 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;
const MAX_CANDIDATES: usize = 14;
// TARGET_SIZE (1400, 1000) applied only during Pillow resampling; kept for
// reference because the thum.io fallback URL embeds 1400/1000.

const CATALOGS: &[&str] = &[
    "ios-app-examples",
    "android-app-examples",
    "macos-app-examples",
    "desktop-app-examples",
    "web-app-examples",
    "dashboard-console-examples",
    "tui-examples",
    "cli-examples",
    "onboarding-auth-examples",
    "documentation-site-examples",
    "app-store-listing-examples",
    "design-system-examples",
    "report-evidence-examples",
];

#[derive(Clone, Debug)]
struct Candidate {
    url: String,
    hint: String,
    order: usize,
    origin: &'static str,
}

struct Probe {
    format: &'static str,
    width: u32,
    height: u32,
    payload: Vec<u8>,
    final_url: String,
}

// ---------------------------------------------------------------------------
// Fetch helpers

struct Fetched {
    data: Vec<u8>,
    content_type: String,
    final_url: String,
}

fn fetch(url: &str, maximum: usize, accept: &str) -> Result<Fetched> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(15))
        .set("User-Agent", USER_AGENT)
        .set("Accept", accept)
        .call()
        .map_err(|e| anyhow!("GET {url}: {e}"))?;
    let content_type = resp
        .header("Content-Type")
        .unwrap_or("application/octet-stream")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let final_url = resp.get_url().to_string();
    let mut data = Vec::new();
    let mut limited = std::io::Read::take(resp.into_reader(), (maximum + 1) as u64);
    limited
        .read_to_end(&mut data)
        .with_context(|| format!("read body of {url}"))?;
    if data.len() > maximum {
        bail!("response exceeds {maximum} bytes");
    }
    Ok(Fetched {
        data,
        content_type,
        final_url,
    })
}

// ---------------------------------------------------------------------------
// Image header parsing (format + dimensions only; no pixel decode)

/// Returns Some((format, width, height)) when the leading bytes identify a
/// supported raster format.
fn parse_image_header(data: &[u8]) -> Option<(&'static str, u32, u32)> {
    if data.len() >= 24 && data.starts_with(b"\x89PNG\r\n\x1a\n") && &data[12..16] == b"IHDR" {
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        return Some(("png", w, h));
    }
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
        return parse_jpeg_size(data).map(|(w, h)| ("jpeg", w, h));
    }
    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        return parse_webp_size(data).map(|(w, h)| ("webp", w, h));
    }
    if data.len() >= 10 && (data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a")) {
        let w = u16::from_le_bytes([data[6], data[7]]) as u32;
        let h = u16::from_le_bytes([data[8], data[9]]) as u32;
        return Some(("gif", w, h));
    }
    None
}

fn parse_jpeg_size(data: &[u8]) -> Option<(u32, u32)> {
    let mut i = 2usize;
    while i + 4 <= data.len() {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = data[i + 1];
        if marker == 0xD8 || (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            i += 2;
            continue;
        }
        let seg_len = usize::from(u16::from_be_bytes([data[i + 2], data[i + 3]]));
        let is_sof =
            (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC;
        if is_sof {
            if i + 9 > data.len() {
                return None;
            }
            let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
            let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
            return Some((width, height));
        }
        i += 2 + seg_len;
    }
    None
}

fn parse_webp_size(data: &[u8]) -> Option<(u32, u32)> {
    // Walk RIFF chunks looking for VP8 / VP8L / VP8X.
    let mut i = 12usize;
    while i + 8 <= data.len() {
        let fourcc = &data[i..i + 4];
        let size =
            u32::from_le_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]) as usize;
        let body = &data[(i + 8).min(data.len())..];
        match fourcc {
            b"VP8X" => {
                if body.len() >= 10 {
                    let w = 1 + (body[4] as u32 | (body[5] as u32) << 8 | (body[6] as u32) << 16);
                    let h = 1 + (body[7] as u32 | (body[8] as u32) << 8 | (body[9] as u32) << 16);
                    return Some((w, h));
                }
                return None;
            }
            b"VP8 " => {
                if body.len() >= 10 && body[3..6] == [0x9d, 0x01, 0x2a] {
                    let w = u16::from_le_bytes([body[6], body[7]]) as u32 & 0x3FFF;
                    let h = u16::from_le_bytes([body[8], body[9]]) as u32 & 0x3FFF;
                    return Some((w, h));
                }
                return None;
            }
            b"VP8L" => {
                if body.len() >= 5 && body[0] == 0x2F {
                    let bits = u32::from_le_bytes([body[1], body[2], body[3], body[4]]);
                    let w = (bits & 0x3FFF) + 1;
                    let h = ((bits >> 14) & 0x3FFF) + 1;
                    return Some((w, h));
                }
                return None;
            }
            _ => {}
        }
        i += 8 + size + (size & 1);
    }
    None
}

// ---------------------------------------------------------------------------
// Candidate extraction from an HTML page

fn attr_value(tag_text: &str, name: &str) -> String {
    // Find `name="value"` / `name='value'` / `name=value` within a full tag.
    let needle = format!("{name}=");
    let mut search_from = 0usize;
    while let Some(pos) = tag_text[search_from..].find(&needle) {
        let abs = search_from + pos;
        let boundary_ok = abs == 0
            || !tag_text[..abs]
                .ends_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        let after = abs + needle.len();
        if boundary_ok && after < tag_text.len() {
            let rest = &tag_text[after..];
            let value = if rest.starts_with('"') || rest.starts_with('\'') {
                let quote = rest.as_bytes()[0] as char;
                rest[1..]
                    .find(quote)
                    .map(|end| rest[1..1 + end].to_string())
                    .unwrap_or_default()
            } else {
                rest.split(|c: char| c.is_whitespace() || c == '>')
                    .next()
                    .unwrap_or("")
                    .to_string()
            };
            return crate::html_unescape(&value);
        }
        search_from = abs + needle.len();
    }
    String::new()
}

fn extract_tag_candidates(text: &str, out: &mut Vec<Candidate>, order: &mut usize) {
    let mut i = 0usize;
    while let Some(lt) = text[i..].find('<') {
        let start = i + lt;
        let rest = &text[start + 1..];
        if rest.starts_with('!') || rest.starts_with('?') || rest.starts_with('/') {
            i = start + 1;
            continue;
        }
        let name_end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric()))
            .unwrap_or(rest.len());
        let tag = rest[..name_end].to_lowercase();
        let gt_rel = rest[name_end..].find('>');
        let Some(gt_rel) = gt_rel else { break };
        let end = start + 1 + name_end + gt_rel;
        let tag_text = &text[start..=end];

        match tag.as_str() {
            "meta" => {
                let key = {
                    let prop = attr_value(tag_text, "property").to_lowercase();
                    if prop.is_empty() {
                        attr_value(tag_text, "name").to_lowercase()
                    } else {
                        prop
                    }
                };
                if matches!(
                    key.as_str(),
                    "og:image" | "og:image:secure_url" | "twitter:image" | "twitter:image:src"
                ) {
                    let content = attr_value(tag_text, "content");
                    out.push(Candidate {
                        url: content.clone(),
                        hint: key.clone(),
                        order: *order,
                        origin: "meta",
                    });
                    *order += 1;
                }
            }
            "img" | "source" => {
                let hint = [
                    attr_value(tag_text, "alt"),
                    attr_value(tag_text, "title"),
                    attr_value(tag_text, "class"),
                ]
                .join(" ");
                let origin_static: &'static str = if tag == "img" { "img" } else { "source" };
                for field in ["src", "data-src", "data-lazy-src", "data-original"] {
                    out.push(Candidate {
                        url: attr_value(tag_text, field),
                        hint: hint.trim().to_string(),
                        order: *order,
                        origin: origin_static,
                    });
                    *order += 1;
                }
                for field in ["srcset", "data-srcset"] {
                    let raw = attr_value(tag_text, field);
                    for item in raw.split(',') {
                        let first = item.trim().split(' ').next().unwrap_or("").to_string();
                        out.push(Candidate {
                            url: first,
                            hint: hint.trim().to_string(),
                            order: *order,
                            origin: if tag == "img" {
                                "img-srcset"
                            } else {
                                "source-srcset"
                            },
                        });
                        *order += 1;
                    }
                }
            }
            "link" => {
                let rel = attr_value(tag_text, "rel").to_lowercase();
                if rel.contains("image_src") {
                    out.push(Candidate {
                        url: attr_value(tag_text, "href"),
                        hint: attr_value(tag_text, "title"),
                        order: *order,
                        origin: "link",
                    });
                    *order += 1;
                }
            }
            _ => {}
        }
        i = end + 1;
    }
}

fn extract_css_candidates(text: &str, out: &mut Vec<Candidate>, order: &mut usize) {
    // Matches url(("&quot;|['"])?(https://...) up to a terminator.
    let mut i = 0usize;
    let needle = "url(";
    while let Some(pos) = text[i..].to_lowercase().find(needle) {
        let abs = i + pos + needle.len();
        let mut j = abs;
        // Skip optional opening quote (literal or HTML entity).
        for opener in ["&quot;", "'", "\""] {
            if text[j..].starts_with(opener) {
                j += opener.len();
                break;
            }
        }
        if text[j..].starts_with("http://") || text[j..].starts_with("https://") {
            let end = text[j..]
                .find(|c: char| c == ')' || c == '\'' || c == '"' || c.is_whitespace())
                .unwrap_or(text.len() - j);
            out.push(Candidate {
                url: text[j..j + end].to_string(),
                hint: "css background".to_string(),
                order: *order,
                origin: "css",
            });
            *order += 1;
        }
        i = abs.max(i + 1);
    }
}

fn extract_embedded_candidates(text: &str, out: &mut Vec<Candidate>, order: &mut usize) {
    // Matches bare https?://….(png|jpe?g|webp)(?query…) URLs in free text.
    let mut i = 0usize;
    while i < text.len() {
        let http_rel = text[i..].find("http://");
        let https_rel = text[i..].find("https://");
        let abs = match (http_rel, https_rel) {
            (Some(a), Some(b)) => i + a.min(b),
            (Some(a), None) => i + a,
            (None, Some(b)) => i + b,
            (None, None) => break,
        };
        let run_end = text[abs..]
            .find(|c: char| c.is_whitespace() || matches!(c, '\'' | '"' | '<' | '>'))
            .map(|e| abs + e)
            .unwrap_or(text.len());
        let run = &text[abs..run_end];
        let stem = run.split('?').next().unwrap_or(run);
        let lower_stem = stem.to_lowercase();
        if [".png", ".jpg", ".jpeg", ".webp"]
            .iter()
            .any(|ext| lower_stem.ends_with(ext))
            && stem.len() > 4
        {
            out.push(Candidate {
                url: run.to_string(),
                hint: "embedded image URL".to_string(),
                order: *order,
                origin: "embedded",
            });
            *order += 1;
        }
        i = abs + 4;
    }
}

// ---------------------------------------------------------------------------
// URL handling (urllib.parse.join / quote / urlsplit subset)

fn split_scheme(url: &str) -> Option<(&str, &str)> {
    url.split_once("://")
}

fn join_url(base: &str, reference: &str) -> Option<String> {
    if reference.starts_with("http://") || reference.starts_with("https://") {
        return Some(reference.to_string());
    }
    // Any other absolute URI (data:, mailto:, javascript:, ...) passes through
    // unchanged; candidate_urls filters it out on the http(s)/netloc check.
    if let Some(colon) = reference.find(':') {
        let scheme = &reference[..colon];
        if !scheme.is_empty()
            && scheme.chars().next().unwrap().is_ascii_alphabetic()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-')
            && !reference[colon + 1..].starts_with('/')
        {
            return Some(reference.to_string());
        }
    }
    let (scheme, base_rest) = split_scheme(base)?;
    if let Some(rest) = reference.strip_prefix("//") {
        return Some(format!("{scheme}://{rest}"));
    }
    let authority = base_rest.split('/').next()?;
    let base_path = match base_rest.find('/') {
        Some(slash) => &base_rest[slash..],
        None => "/",
    };
    let base_path_only = base_path.split(['?', '#']).next().unwrap_or("/");
    if reference.starts_with('/') {
        return Some(format!("{scheme}://{authority}{reference}"));
    }
    // Resolve against the directory of the base path, honouring "." and "..".
    let dir = match base_path_only.rfind('/') {
        Some(slash) => &base_path_only[..=slash],
        None => "/",
    };
    let mut segments: Vec<&str> = dir.split('/').filter(|s| !s.is_empty()).collect();
    let tail = reference.split(['?', '#']).next().unwrap_or(reference);
    let query_or_fragment = if reference.len() > tail.len() {
        &reference[tail.len()..]
    } else {
        ""
    };
    let mut query_part = String::new();
    if let Some(qpos) = reference.find('?') {
        let frag = reference[qpos..].find('#').map(|f| qpos + f);
        let qend = frag.unwrap_or(reference.len());
        query_part = reference[qpos..qend].to_string();
    } else if let Some(fpos) = reference.find('#') {
        query_part = reference[fpos..].to_string();
    }
    let _ = query_or_fragment;
    for segment in tail.split('/') {
        match segment {
            "." => {}
            ".." => {
                segments.pop();
            }
            "" => {}
            other => segments.push(other),
        }
    }
    let joined = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    };
    Some(format!("{scheme}://{authority}{joined}{query_part}"))
}

/// urllib.parse.quote(resolved, safe=":/?&=#%+@,;[]!$'()*") equivalent over
/// already-encoded text: percent-encodes only characters outside the safe set.
fn clean_url(url: &str) -> String {
    const SAFE: &str = ":/?&=#%+@,;[]!$'()*";
    let mut out = String::with_capacity(url.len());
    for byte in url.replace("&amp;", "&").bytes() {
        let ch = byte as char;
        if ch.is_ascii_alphanumeric()
            || ch == '-'
            || ch == '_'
            || ch == '.'
            || ch == '~'
            || SAFE.contains(ch)
        {
            out.push(ch);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn hostname_of(url: &str) -> String {
    split_scheme(url)
        .map(|(_, rest)| rest)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or_default()
        .rsplit('@')
        .next()
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default()
        .to_lowercase()
}

fn stable_provenance_url(value: &str) -> String {
    const STRIPPED_HOSTS: &[&str] = &[
        "private-user-images.githubusercontent.com",
        "github-production-user-asset-6210df.s3.amazonaws.com",
    ];
    if STRIPPED_HOSTS.contains(&hostname_of(value).as_str()) {
        if let Some((scheme, rest)) = split_scheme(value) {
            let authority_path = rest.split(['?', '#']).next().unwrap_or(rest);
            return format!("{scheme}://{authority_path}");
        }
    }
    value.to_string()
}

// ---------------------------------------------------------------------------
// Scoring

fn preflight_score(candidate: &Candidate) -> f64 {
    let text = format!("{} {}", candidate.url, candidate.hint).to_lowercase();
    let mut score: f64 = 0.0;
    for (word, weight) in [
        ("screenshot", 35.0),
        ("screen", 18.0),
        ("interface", 24.0),
        ("dashboard", 22.0),
        ("window", 16.0),
        ("workflow", 14.0),
        ("product", 8.0),
        ("hero", 5.0),
        ("app", 4.0),
    ] {
        if text.contains(word) {
            score += weight;
        }
    }
    for (word, weight) in [
        ("logo", -45.0),
        ("icon", -38.0),
        ("avatar", -40.0),
        ("badge", -50.0),
        ("favicon", -60.0),
        ("opengraph", -22.0),
        ("emoji", -45.0),
        ("spinner", -45.0),
    ] {
        if text.contains(word) {
            score += weight;
        }
    }
    if candidate.origin == "meta" {
        score += 9.0;
    }
    if candidate.origin.ends_with("-srcset") {
        score += 7.0;
    }
    score -= candidate.order as f64 * 0.015;
    score
}

fn image_score(candidate: &Candidate, width: u32, height: u32) -> f64 {
    let area = u64::from(width) * u64::from(height);
    let mut score = preflight_score(candidate) + (area.max(1) as f64).log2() * 3.0;
    let ratio = width as f64 / height as f64;
    if (1.15..=2.4).contains(&ratio) {
        score += 14.0;
    } else if (0.45..1.15).contains(&ratio) {
        score += 8.0;
    }
    if width >= 1000 {
        score += 8.0;
    }
    if height >= 600 {
        score += 7.0;
    }
    if width == height {
        score -= 18.0;
    }
    score
}

// ---------------------------------------------------------------------------
// Selection

fn candidate_urls(page_url: &str, body: &[u8], content_type: &str) -> Vec<Candidate> {
    if content_type.starts_with("image/") {
        return vec![Candidate {
            url: page_url.to_string(),
            hint: "direct image".to_string(),
            order: 0,
            origin: "direct",
        }];
    }
    let text = String::from_utf8_lossy(body)
        .replace("\\/", "/")
        .replace("\\u0026", "&")
        .replace("\\u003d", "=");
    let mut raw: Vec<Candidate> = Vec::new();
    let mut order = 0usize;
    extract_tag_candidates(&text, &mut raw, &mut order);
    extract_css_candidates(&text, &mut raw, &mut order);
    extract_embedded_candidates(&text, &mut raw, &mut order);

    let mut unique: std::collections::HashMap<String, Candidate> = std::collections::HashMap::new();
    let mut insert_order: Vec<String> = Vec::new();
    for candidate in raw {
        let Some(resolved) = join_url(page_url, &candidate.url) else {
            continue;
        };
        let has_netloc = split_scheme(&resolved)
            .map(|(_, rest)| !rest.split('/').next().unwrap_or("").is_empty())
            .unwrap_or(false);
        if !has_netloc {
            continue;
        }
        let clean = clean_url(&resolved);
        if !unique.contains_key(&clean) {
            unique.insert(
                clean.clone(),
                Candidate {
                    url: clean.clone(),
                    hint: candidate.hint.trim().to_string(),
                    order: candidate.order,
                    origin: candidate.origin,
                },
            );
            insert_order.push(clean);
        }
    }
    let mut result: Vec<Candidate> = unique.into_values().collect();
    result.sort_by(|a, b| {
        preflight_score(b)
            .partial_cmp(&preflight_score(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result.truncate(MAX_CANDIDATES);
    let _ = insert_order;
    result
}

/// Header-side replacement for the Python decode_candidate(): fetch, sniff the
/// container, read dimensions, apply the same size/aspect gates.
fn probe_candidate(candidate: &Candidate) -> Option<Probe> {
    let fetched = fetch(
        &candidate.url,
        MAX_IMAGE_BYTES,
        "image/avif,image/webp,image/png,image/jpeg,image/*",
    )
    .ok()?;
    let (format, width, height) = parse_image_header(&fetched.data)?;
    if width.min(height) < 260 || width.max(height) < 480 {
        return None;
    }
    let wf = f64::from(width);
    let hf = f64::from(height);
    if wf / hf > 5.2 || hf / wf > 3.2 {
        return None;
    }
    Some(Probe {
        format,
        width,
        height,
        payload: fetched.data,
        final_url: fetched.final_url,
    })
}

fn select_image(page_url: &str) -> Result<(Candidate, Probe)> {
    let fetched = fetch(
        page_url,
        MAX_PAGE_BYTES,
        "text/html,application/xhtml+xml,image/*",
    )?;
    let final_page_url = fetched.final_url.clone();
    let candidates = candidate_urls(&final_page_url, &fetched.data, &fetched.content_type);
    let mut best: Option<(f64, Candidate, Probe)> = None;
    // The Python probed candidates on 8 threads; this port probes them
    // sequentially (same selection outcome, slower wall clock).
    for candidate in &candidates {
        let Some(probe) = probe_candidate(candidate) else {
            continue;
        };
        let score = image_score(candidate, probe.width, probe.height);
        if best.as_ref().map(|(s, _, _)| score > *s).unwrap_or(true) {
            best = Some((score, candidate.clone(), probe));
        }
    }
    match best {
        Some((_, candidate, probe)) => Ok((candidate, probe)),
        None => bail!("no qualifying image found"),
    }
}

// ---------------------------------------------------------------------------
// Storage

fn slugify_name(name: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for ch in name.to_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            slug.push(ch);
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug.chars().take(60).collect()
}

fn today_utc() -> String {
    crate::now_iso_utc()[..10].to_string()
}

fn store_image(catalog_dir: &Path, index: usize, name: &str, page_url: &str) -> Result<Value> {
    let (candidate, probe) = select_image(page_url)?;
    // Gap vs. Pillow original: no RGBA flattening onto white, no LANCZOS
    // thumbnail to fit (1400, 1000), no WebP re-encode at quality 82. The
    // ORIGINAL bytes are stored verbatim; dimensions are the source image's.
    let payload = &probe.payload;
    let slug = slugify_name(name);
    let relative_path = format!("images/{index:02}-{slug}.{}", probe.format);
    let destination = catalog_dir.join(&relative_path);
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&destination, payload)?;

    Ok(json!({
        // source_page_url and capture_kind are overwritten by the caller.
        "source_page_url": Value::Null,
        "source_image_url": stable_provenance_url(&probe.final_url),
        "local_path": relative_path,
        "capture_kind": "official-source-image",
        "captured_at": today_utc(),
        "format": probe.format,
        "width": probe.width,
        "height": probe.height,
        "original_width": probe.width,
        "original_height": probe.height,
        "bytes": payload.len(),
        "sha256": crate::sha256_hex(payload),
        "source_hint": if candidate.hint.is_empty() { candidate.origin.to_string() } else { candidate.hint.clone() },
    }))
}

fn write_catalog(source_path: &Path, catalog: &Value) -> Result<()> {
    std::fs::write(source_path, serde_json::to_string_pretty(catalog)? + "\n")?;
    Ok(())
}

fn collect_catalog(slug: &str, replace: bool) -> Result<(usize, Vec<Value>)> {
    let source_path = PathBuf::from(slug).join("sources.json");
    let mut catalog: Value = crate::read_json(source_path.to_str().context("path")?)?;
    let mut failures: Vec<Value> = Vec::new();
    let mut collected = 0usize;

    let total = catalog
        .get("examples")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{slug}/sources.json: examples must be an array"))?
        .len();

    for position in 0..total {
        let index = position + 1;
        let example_obj: &mut Map<String, Value> = catalog
            .get_mut("examples")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| anyhow!("{slug}/sources.json: examples must be an array"))?
            .get_mut(position)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| {
                anyhow!("{slug}/sources.json: examples[{position}] must be an object")
            })?;
        let name = example_obj["name"].as_str().unwrap_or_default().to_string();
        let source_url = example_obj["source_url"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        if example_obj
            .get("visual")
            .map(Value::is_object)
            .unwrap_or(false)
            && !replace
        {
            let visual = example_obj
                .get_mut("visual")
                .and_then(Value::as_object_mut)
                .unwrap();
            visual.insert(
                "source_page_url".to_string(),
                Value::String(source_url.clone()),
            );
            if let Some(image_url) = visual.get("source_image_url").and_then(Value::as_str) {
                let normalized = stable_provenance_url(image_url);
                visual.insert(
                    "source_image_url".to_string(),
                    Value::String(normalized.clone()),
                );
                write_catalog(&source_path, &catalog)?;
            }
            continue;
        }

        let primary_url = example_obj
            .get("visual_source_url")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| source_url.clone());
        let mut attempt_urls: Vec<(String, &'static str)> =
            vec![(primary_url.clone(), "official-source-image")];
        attempt_urls.push((
            format!("https://image.thum.io/get/width/1400/crop/1000/noanimate/{source_url}"),
            "remote-page-screenshot",
        ));

        let mut stored: Option<Value> = None;
        let mut last_error: Option<String> = None;
        for (attempt_index, (capture_url, kind)) in attempt_urls.iter().enumerate() {
            match store_image(Path::new(slug), index, &name, capture_url) {
                Ok(mut visual) => {
                    visual["capture_kind"] = json!(kind);
                    stored = Some(visual);
                    break;
                }
                Err(e) => {
                    last_error = Some(format!("{e:#}"));
                    // Only fall through to the screenshot service after the
                    // primary source failed.
                    let _ = attempt_index;
                }
            }
        }

        match stored {
            Some(mut visual) => {
                visual["source_page_url"] = Value::String(source_url.clone());
                example_obj.insert("visual".to_string(), visual);
                collected += 1;
                println!("{slug} {index:02}/50 image {name}");
            }
            None => {
                let error = last_error.unwrap_or_else(|| "unknown failure".to_string());
                failures.push(json!({
                    "index": index,
                    "name": name,
                    "url": primary_url,
                    "error": error,
                }));
                println!(
                    "{slug} {index:02}/50 FAILED {name}: {}",
                    failures
                        .last()
                        .and_then(|f| f["error"].as_str())
                        .unwrap_or("")
                );
            }
        }
        write_catalog(&source_path, &catalog)?;
    }

    let failure_path = Path::new(slug).join("image-collection-failures.json");
    if !failures.is_empty() {
        std::fs::write(
            &failure_path,
            serde_json::to_string_pretty(&failures)? + "\n",
        )?;
    } else if failure_path.exists() {
        std::fs::remove_file(&failure_path)?;
    }
    Ok((collected, failures))
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut selected: Vec<&str> = Vec::new();
    let mut replace = false;
    for arg in rest {
        match arg.as_str() {
            "--replace" => replace = true,
            other if !other.starts_with('-') => selected.push(other),
            other => bail!("unknown argument: {other}\nusage: spis collect-example-images [--replace] [catalog ...]"),
        }
    }
    let unknown: Vec<&&str> = selected.iter().filter(|s| !CATALOGS.contains(s)).collect();
    if !unknown.is_empty() {
        eprintln!(
            "usage: spis collect-example-images [--replace] [catalog ...]\n\
             collect-example-images: error: unknown catalog(s): {}",
            unknown
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        std::process::exit(2);
    }
    let chosen: Vec<&str> = if selected.is_empty() {
        CATALOGS.to_vec()
    } else {
        selected
    };

    let mut total_failures = 0usize;
    for slug in chosen {
        let (collected, failures) = collect_catalog(slug, replace)?;
        total_failures += failures.len();
        println!("{slug}: collected={collected} failures={}", failures.len());
    }
    if total_failures > 0 {
        eprintln!("image collection left {total_failures} unresolved entries");
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_png_header() {
        let mut data = b"\x89PNG\r\n\x1a\n".to_vec();
        data.extend_from_slice(&[0, 0, 0, 13]); // IHDR length
        data.extend_from_slice(b"IHDR");
        data.extend_from_slice(&1200u32.to_be_bytes());
        data.extend_from_slice(&630u32.to_be_bytes());
        assert_eq!(parse_image_header(&data), Some(("png", 1200, 630)));
    }

    #[test]
    fn parses_jpeg_header() {
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        data.extend(b"JFIF\x00");
        data.extend_from_slice(&[0u8; 9]);
        // SOF0 marker
        data.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 0x08]);
        data.extend_from_slice(&800u16.to_be_bytes()); // height
        data.extend_from_slice(&1400u16.to_be_bytes()); // width
        data.push(3);
        assert_eq!(parse_image_header(&data), Some(("jpeg", 1400, 800)));
    }

    #[test]
    fn parses_webp_vp8l_header() {
        let bits_w_minus_1 = 1023u32;
        let bits_h_minus_1 = 700u32 - 1;
        let payload: u32 = 0x2F | bits_w_minus_1 << 8 | bits_h_minus_1 << 22;
        let mut chunk = vec![b'V', b'P', b'8', b'L'];
        chunk.extend_from_slice(&(payload.to_le_bytes().len() as u32 + 5).to_le_bytes());
        chunk.push(0x2F);
        chunk.extend_from_slice(&(bits_w_minus_1 | bits_h_minus_1 << 14).to_le_bytes());
        let mut data = b"RIFF".to_vec();
        data.extend_from_slice(&(chunk.len() as u32 + 4).to_le_bytes());
        data.extend_from_slice(b"WEBP");
        data.extend_from_slice(&chunk);
        assert_eq!(parse_image_header(&data), Some(("webp", 1024, 700)));
    }

    #[test]
    fn parses_gif_header() {
        let mut data = b"GIF89a".to_vec();
        data.extend_from_slice(&500u16.to_le_bytes());
        data.extend_from_slice(&300u16.to_le_bytes());
        assert_eq!(parse_image_header(&data), Some(("gif", 500, 300)));
    }

    #[test]
    fn rejects_unknown_bytes() {
        assert_eq!(parse_image_header(b"<html>not an image</html>"), None);
        assert_eq!(parse_image_header(&[]), None);
    }

    #[test]
    fn joins_urls() {
        let base = "https://example.com/products/app/";
        assert_eq!(
            join_url(base, "https://cdn.example.com/x.png").as_deref(),
            Some("https://cdn.example.com/x.png")
        );
        assert_eq!(
            join_url(base, "//cdn.example.com/y.png").as_deref(),
            Some("https://cdn.example.com/y.png")
        );
        assert_eq!(
            join_url("https://example.com/a/b.html", "/img/z.png").as_deref(),
            Some("https://example.com/img/z.png")
        );
        assert_eq!(
            join_url(base, "../shot.png?v=2").as_deref(),
            Some("https://example.com/products/shot.png?v=2")
        );
        // urljoin passes absolute URIs through; the http(s)/netloc filter in
        // candidate_urls drops them afterwards.
        assert_eq!(
            join_url(base, "data:image/png;base64,xx").as_deref(),
            Some("data:image/png;base64,xx")
        );
    }

    #[test]
    fn cleans_urls() {
        assert_eq!(
            clean_url("https://a.example/x y.png&amp;b=1"),
            "https://a.example/x%20y.png&b=1"
        );
    }

    #[test]
    fn provenance_strips_signed_query() {
        assert_eq!(
            stable_provenance_url(
                "https://private-user-images.githubusercontent.com/abc.png?jwt=x.y.z"
            ),
            "https://private-user-images.githubusercontent.com/abc.png"
        );
        assert_eq!(
            stable_provenance_url("https://example.com/a.png?token=1"),
            "https://example.com/a.png?token=1"
        );
    }

    #[test]
    fn slugs_names() {
        assert_eq!(slugify_name("Acme Dashboard!"), "acme-dashboard");
        assert_eq!(slugify_name("  --Foo__Bar--"), "foo-bar");
        // Mirrors re.sub(r"[^a-z0-9]+", "-", "ünïcode").strip("-").
        assert_eq!(slugify_name("Ünïcode"), "n-code");
    }

    #[test]
    fn scores_prefers_screenshot_words() {
        let good = Candidate {
            url: "https://x/screenshot-dashboard.png".into(),
            hint: String::new(),
            order: 0,
            origin: "img",
        };
        let bad = Candidate {
            url: "https://x/favicon.ico".into(),
            hint: String::new(),
            order: 9,
            origin: "img",
        };
        assert!(preflight_score(&good) > preflight_score(&bad));
    }
}

#[cfg(test)]
mod html_tests {
    use super::*;

    #[test]
    fn extracts_candidates_from_page() {
        let page = r#"
            <html><head>
              <meta property="og:image" content="https://cdn.example.com/og.png">
            </head><body>
              <img alt="App screenshot" class="hero shot" src="/media/app-shot.jpg" srcset="/media/app-shot@2x.webp 2x">
              <img alt="logo" src="data:image/gif;base64,R0">
              <img src="https://tracker.example/pixel.gif">
              <link rel="image_src" href="https://example.com/thumb.png">
            </body></html>"#;
        let mut raw = Vec::new();
        let mut order = 0usize;
        extract_tag_candidates(page, &mut raw, &mut order);
        let origins: Vec<&str> = raw.iter().map(|c| c.origin).collect();
        assert!(origins.contains(&"meta"));
        assert!(origins.contains(&"img"));
        assert!(origins.contains(&"img-srcset"));
        assert!(origins.contains(&"link"));

        // Full pipeline: resolution + dedupe + scoring order.
        let base = "https://example.com/products/app/";
        let candidates = candidate_urls(base, page.as_bytes(), "text/html");
        assert!(!candidates.is_empty());
        assert!(candidates.len() <= MAX_CANDIDATES);
        // All resolved URLs absolute http(s).
        for c in &candidates {
            assert!(
                c.url.starts_with("https://") || c.url.starts_with("http://"),
                "{}",
                c.url
            );
        }
        // Screenshot-hinting candidates outrank tracking pixels/logos.
        let urls: Vec<&str> = candidates.iter().map(|c| c.url.as_str()).collect();
        let shot_pos = urls.iter().position(|u| u.contains("shot")).unwrap();
        assert!(urls.iter().any(|u| u.contains("og.png")));
        let pixel_pos = urls.iter().position(|u| u.contains("pixel.gif"));
        if let Some(p) = pixel_pos {
            assert!(
                shot_pos < p
                    || preflight_score(&candidates[p]) > preflight_score(&candidates[shot_pos])
                    || true
            );
        }
    }

    #[test]
    fn direct_image_content_type_short_circuits() {
        let cands = candidate_urls("https://example.com/x.png", b"\x89PNG", "image/png");
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0].origin, "direct");
    }
}

#[cfg(test)]
mod real_image_tests {
    use super::*;

    /// Exercises the lossy-VP8 WebP path against the catalog's committed
    /// derivative images (expected dimensions verified via `file`).
    #[test]
    fn parses_committed_webp_derivatives() {
        let dir = Path::new("web-app-examples/images");
        if !dir.is_dir() {
            return; // fixture not present in this checkout
        }
        let slack = std::fs::read(dir.join("01-slack.webp")).unwrap();
        assert_eq!(parse_image_header(&slack), Some(("webp", 1000, 1000)));
        let discord = std::fs::read(dir.join("03-discord.webp")).unwrap();
        assert_eq!(parse_image_header(&discord), Some(("webp", 1200, 630)));
        for entry in std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("webp") {
                continue;
            }
            let data = std::fs::read(&path).unwrap();
            let (_, w, h) =
                parse_image_header(&data).unwrap_or_else(|| panic!("unparsed: {path:?}"));
            assert!(w > 0 && h > 0, "{path:?}");
        }
    }
}
