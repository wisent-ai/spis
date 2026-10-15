//! The same-origin page links of a fetched start page. Every href is resolved
//! against the page by the URL standard the `url` crate implements and kept
//! when its origin is the page's; no part of a URL is cut out of its text.

use super::*;

/// What a start page links to: each same-origin page once with the text of
/// its first anchor, and each href that does not resolve against the page
/// with the reason, so no link is dropped unsaid.
#[derive(Default)]
pub(crate) struct PageLinks {
    pub(crate) entries: Vec<(String, String)>,
    pub(crate) unresolved: Vec<(String, String)>,
}

/// An anchor's `href` value, quoted either way or bare.
const HREF: &str =
    r#"(?i)(?:^|\s)href\s*=\s*(?:"(?P<double>[^"]*)"|'(?P<single>[^']*)'|(?P<bare>[^\s"'>]+))"#;

/// Every `<a href>` of `html` with the text inside it, in document order. An
/// anchor left open when the next one starts, or at the end, is not a link.
fn anchors(html: &str) -> Result<Vec<(String, String)>> {
    let href = regex::Regex::new(HREF).context("discover: the href pattern does not compile")?;
    let mut found = Vec::new();
    let mut open: Option<(String, String)> = None;
    let mut chunks = html.split('<');
    // What precedes the first tag lies outside every anchor.
    chunks.next();
    for chunk in chunks {
        let Some((tag, text)) = chunk.split_once('>') else {
            break;
        };
        let is_anchor = tag
            .trim_start_matches('/')
            .split(|character: char| character.is_whitespace() || character == '/')
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case("a"));
        if is_anchor {
            if tag.starts_with('/') {
                if let Some(anchor) = open.take() {
                    found.push(anchor);
                }
            } else {
                let target = href.captures(tag).and_then(|captured| {
                    captured
                        .name("double")
                        .or_else(|| captured.name("single"))
                        .or_else(|| captured.name("bare"))
                        .map(|value| value.as_str().to_string())
                });
                open = None;
                match target {
                    Some(target) if tag.ends_with('/') => found.push((target, String::new())),
                    Some(target) => open = Some((target, String::new())),
                    None => {}
                }
            }
        }
        if let Some((_, collected)) = open.as_mut() {
            collected.push_str(text);
        }
    }
    Ok(found)
}

/// Every page `start` links to on its own origin, other than itself.
pub(crate) fn page_links(start: &url::Url, html: &str) -> Result<PageLinks> {
    let mut links = PageLinks::default();
    for (target, text) in anchors(html)? {
        let target = lib::html_unescape(&target);
        let mut absolute = match start.join(&target) {
            Ok(absolute) => absolute,
            Err(error) => {
                links.unresolved.push((target, error.to_string()));
                continue;
            }
        };
        absolute.set_fragment(None);
        if absolute.origin() != start.origin()
            || absolute.as_str().trim_end_matches('/') == start.as_str().trim_end_matches('/')
        {
            continue;
        }
        let address = absolute.to_string();
        if links.entries.iter().any(|(known, _)| *known == address) {
            continue;
        }
        let text = lib::html_unescape(&text.split_whitespace().collect::<Vec<_>>().join(" "));
        let text = if text.is_empty() {
            address.clone()
        } else {
            text
        };
        links.entries.push((address, text));
    }
    Ok(links)
}
