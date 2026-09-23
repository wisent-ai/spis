use super::*;

pub(crate) fn extract_tag_candidates(text: &str, out: &mut Vec<Candidate>, order: &mut usize) {
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

pub(crate) fn extract_css_candidates(text: &str, out: &mut Vec<Candidate>, order: &mut usize) {
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

pub(crate) fn extract_embedded_candidates(text: &str, out: &mut Vec<Candidate>, order: &mut usize) {
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

pub(crate) fn split_scheme(url: &str) -> Option<(&str, &str)> {
    url.split_once("://")
}

pub(crate) fn join_url(base: &str, reference: &str) -> Option<String> {
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
pub(crate) fn clean_url(url: &str) -> String {
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

pub(crate) fn hostname_of(url: &str) -> String {
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
