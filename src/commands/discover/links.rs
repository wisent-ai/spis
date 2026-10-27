//! The same-origin page links of a fetched start page. The page's anchors are
//! read by Spis's one HTML reader (`spis::html`), and every href is resolved
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

/// Every `<a href>` of `html` with the text inside it, in document order, as
/// the HTML parser reads them: the href unescaped, and an anchor the page
/// left open closed where the standard closes it.
fn anchors(html: &str) -> Result<Vec<(String, String)>> {
    let mut found = Vec::new();
    for anchor in lib::html::elements(html, "a[href]")? {
        let Some(target) = anchor.attribute("href") else {
            bail!("discover: the anchor selector a[href] matched an anchor without an href");
        };
        found.push((target.to_string(), anchor.text.clone()));
    }
    Ok(found)
}

/// Every page `start` links to on its own origin, other than itself.
pub(crate) fn page_links(start: &url::Url, html: &str) -> Result<PageLinks> {
    let mut links = PageLinks::default();
    for (target, text) in anchors(html)? {
        // The parser has already unescaped the href and the anchor's text.
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
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let text = if text.is_empty() {
            address.clone()
        } else {
            text
        };
        links.entries.push((address, text));
    }
    Ok(links)
}
