use super::*;

/// The meta names that state a page's own image (Open Graph's `property`,
/// Twitter's `name`).
const IMAGE_META: &[&str] = &[
    "og:image",
    "og:image:secure_url",
    "twitter:image",
    "twitter:image:src",
];
/// The elements that name images in their attributes: each element's
/// selector, the origin of an image it names in one attribute, and the
/// origin of one it lists in a source set.
const IMAGE_ELEMENTS: &[(&str, &str, &str)] = &[
    ("img", "img", "img-srcset"),
    ("source", "source", "source-srcset"),
];
/// The attributes an image element names one image in, eager or lazy.
const SINGLE_SOURCES: &[&str] = &["src", "data-src", "data-lazy-src", "data-original"];
/// The attributes an image element lists images in, each with a width or
/// density after it.
const SOURCE_SETS: &[&str] = &["srcset", "data-srcset"];
/// A CSS `url(...)` naming an absolute http(s) address, quoted, quoted by an
/// HTML entity, or bare.
const CSS_URL: &str = r#"(?i)url\(\s*(?:&quot;|['"])?(https?://[^)'"\s]+)"#;
/// An http(s) address written in a page's text, up to whitespace, a quote or
/// a tag bracket.
const ADDRESS: &str = r#"https?://[^\s'"<>]+"#;
/// The file extensions that make an address written in a page's text an
/// image's.
const IMAGE_EXTENSIONS: &[&str] = &[".png", ".jpg", ".jpeg", ".webp"];

/// List `url` as the next candidate: its order is its position in the listing.
fn push(out: &mut Vec<Candidate>, url: &str, hint: String, origin: &'static str) {
    let order = out.len();
    out.push(Candidate {
        url: url.to_string(),
        hint,
        order,
        origin,
    });
}

/// Every image the page's tags name, read by Spis's one HTML reader
/// (`spis::html`): the images a `meta` states for the page, each source of an
/// `img` or `source` with the words describing it (its alt text, title and
/// class) as the hint, and a `link rel=image_src`. An attribute a tag does
/// not carry names no candidate.
pub(crate) fn extract_tag_candidates(html: &str, out: &mut Vec<Candidate>) -> Result<()> {
    for element in crate::html::elements(html, "meta[content]")? {
        let key = match (element.attribute("property"), element.attribute("name")) {
            (Some(named), _) | (None, Some(named)) => named.to_lowercase(),
            (None, None) => continue,
        };
        if let (true, Some(content)) = (
            IMAGE_META.contains(&key.as_str()),
            element.attribute("content"),
        ) {
            push(out, content, key, "meta");
        }
    }
    for (selector, single, listed) in IMAGE_ELEMENTS {
        for element in crate::html::elements(html, selector)? {
            let described = [
                element.attribute("alt"),
                element.attribute("title"),
                element.attribute("class"),
            ];
            let hint = described
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ");
            let hint = hint.trim();
            for url in SINGLE_SOURCES
                .iter()
                .filter_map(|name| element.attribute(name))
            {
                push(out, url, hint.to_string(), single);
            }
            for set in SOURCE_SETS
                .iter()
                .filter_map(|name| element.attribute(name))
            {
                // Each entry is a URL, then its width or density.
                for url in set
                    .split(',')
                    .filter_map(|entry| entry.split_whitespace().next())
                {
                    push(out, url, hint.to_string(), listed);
                }
            }
        }
    }
    // `~=` matches one of the space-separated link types `rel` holds.
    for element in crate::html::elements(html, "link[rel~=image_src][href]")? {
        let Some(href) = element.attribute("href") else {
            bail!("the selector link[rel~=image_src][href] matched a link without an href");
        };
        let title = element.attribute("title").map(str::to_string);
        push(out, href, title.into_iter().collect(), "link");
    }
    Ok(())
}

/// Every absolute address a CSS `url(...)` in `text` names, as a background
/// image, unescaped as markup writes it.
pub(crate) fn extract_css_candidates(text: &str, out: &mut Vec<Candidate>) -> Result<()> {
    let pattern = regex::Regex::new(CSS_URL).context("the CSS url() pattern does not compile")?;
    for found in pattern.captures_iter(text) {
        let (_, [url]) = found.extract();
        push(
            out,
            &crate::html::decoded(url),
            "css background".to_string(),
            "css",
        );
    }
    Ok(())
}

/// Every http(s) address written in `text` whose path, before its query, ends
/// in an image's file extension, unescaped as markup writes it.
pub(crate) fn extract_embedded_candidates(text: &str, out: &mut Vec<Candidate>) -> Result<()> {
    let pattern = regex::Regex::new(ADDRESS).context("the address pattern does not compile")?;
    for found in pattern.find_iter(text) {
        let run = found.as_str();
        let stem = match run.split_once('?') {
            Some((stem, _)) => stem,
            None => run,
        }
        .to_lowercase();
        if IMAGE_EXTENSIONS
            .iter()
            .any(|extension| stem.ends_with(extension))
        {
            push(
                out,
                &crate::html::decoded(run),
                "embedded image URL".to_string(),
                "embedded",
            );
        }
    }
    Ok(())
}

/// `reference` resolved against `page` by the URL standard, when it names a
/// web address with a host (`crate::WebScheme`); `None` for any other scheme
/// (`data:`, `mailto:`, `javascript:`), which no image is fetched from. A
/// reference that does not resolve is refused naming it and the cause.
pub(crate) fn fetchable(page: &url::Url, reference: &str) -> Result<Option<url::Url>> {
    let resolved = page
        .join(reference)
        .with_context(|| format!("the image reference {reference:?} on {page} does not resolve"))?;
    let web = crate::WebScheme::of(&resolved).is_some()
        && resolved.host_str().is_some_and(|host| !host.is_empty());
    Ok(web.then_some(resolved))
}
