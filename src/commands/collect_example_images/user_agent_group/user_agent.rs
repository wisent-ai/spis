use super::*;

pub(crate) const USER_AGENT: &str = crate::USER_AGENT;

pub(crate) const MAX_PAGE_BYTES: usize = 8 * 1024 * 1024;

pub(crate) const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;

pub(crate) const MAX_CANDIDATES: usize = 14;

// TARGET_SIZE (1400, 1000) applied only during Pillow resampling; kept for
// reference because the thum.io fallback URL embeds 1400/1000.

pub(crate) const CATALOGS: &[&str] = &[
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
pub(crate) struct Candidate {
    pub(crate) url: String,
    pub(crate) hint: String,
    pub(crate) order: usize,
    pub(crate) origin: &'static str,
}

pub(crate) struct Probe {
    pub(crate) format: &'static str,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) payload: Vec<u8>,
    pub(crate) final_url: String,
}

// ---------------------------------------------------------------------------
// Fetch helpers

pub(crate) struct Fetched {
    pub(crate) data: Vec<u8>,
    pub(crate) content_type: String,
    pub(crate) final_url: String,
}

pub(crate) fn fetch(url: &str, maximum: usize, accept: &str) -> Result<Fetched> {
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
pub(crate) fn parse_image_header(data: &[u8]) -> Option<(&'static str, u32, u32)> {
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

pub(crate) fn parse_jpeg_size(data: &[u8]) -> Option<(u32, u32)> {
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

pub(crate) fn parse_webp_size(data: &[u8]) -> Option<(u32, u32)> {
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

pub(crate) fn attr_value(tag_text: &str, name: &str) -> String {
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
