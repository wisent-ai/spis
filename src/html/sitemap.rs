//! A sitemap (https://www.sitemaps.org/protocol.html) read by the same parser
//! as every page: a sitemap index lists child sitemaps, a URL set lists pages
//! with their last modification. Each address and date is the element's text
//! with its character and entity references decoded; a search between
//! literal tags read `&amp;` and `&#38;` differently and missed a tag written
//! with attributes or another letter case.

use scraper::{ElementRef, Html, Selector};

/// The CSS selector `selector`, written in this file and known to parse.
fn selector(selector: &str) -> Selector {
    Selector::parse(selector).expect("the sitemap selectors in this file are CSS selectors")
}

/// The trimmed text of the first element under `parent` that `child` selects.
fn child_text(parent: ElementRef<'_>, child: &Selector) -> Option<String> {
    parent
        .select(child)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_string())
        .filter(|text| !text.is_empty())
}

/// The child sitemaps of a sitemap index, or the pages (with their last
/// modification when stated) of a URL set.
pub fn parse_sitemap(body: &[u8]) -> (Vec<String>, Vec<(String, Option<String>)>) {
    let document = Html::parse_document(&String::from_utf8_lossy(body));
    let location = selector("loc");
    let modified = selector("lastmod");
    let children: Vec<String> = document
        .select(&selector("sitemapindex sitemap"))
        .filter_map(|entry| child_text(entry, &location))
        .collect();
    let pages = document
        .select(&selector("urlset url"))
        .filter_map(|entry| {
            child_text(entry, &location).map(|page| (page, child_text(entry, &modified)))
        })
        .collect();
    (children, pages)
}
