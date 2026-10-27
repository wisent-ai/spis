//! One reading of an HTML page for every Spis command. The page is parsed by
//! `scraper` (html5ever, the WHATWG HTML parsing algorithm browsers follow),
//! so a tag's attributes and text are what a browser reads: values unescaped,
//! names compared without regard to ASCII case, text inside another
//! attribute's value or a comment never taken for a tag, and an element the
//! page left open closed where the standard closes it.

use anyhow::{anyhow, Result};
use scraper::{Html, Selector};

mod blocks;
mod sitemap;
mod text;

pub use sitemap::parse_sitemap;
pub use text::extract_text;

/// One element of a page: its tag name in lower case, its attributes as the
/// page gave them, and the text inside it.
pub struct Element {
    pub name: String,
    attributes: Vec<(String, String)>,
    pub text: String,
}

impl Element {
    /// The value of attribute `name`, unescaped; `None` when the element does
    /// not carry it. An attribute written without a value has an empty one.
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(attribute, _)| attribute.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// Every element of `html` the CSS selector `selector` matches, in document
/// order. A selector that is no CSS selector is refused naming it and the
/// parser's cause.
pub fn elements(html: &str, selector: &str) -> Result<Vec<Element>> {
    let selected = Selector::parse(selector)
        .map_err(|error| anyhow!("the HTML selector {selector:?} is not a CSS selector: {error}"))?;
    let page = Html::parse_document(html);
    let found = page
        .select(&selected)
        .map(|element| {
            let value = element.value();
            Element {
                name: value.name().to_ascii_lowercase(),
                attributes: value
                    .attrs()
                    .map(|(name, value)| (name.to_string(), value.to_string()))
                    .collect(),
                text: element.text().collect(),
            }
        })
        .collect();
    Ok(found)
}

/// `text`, a run of a page's source holding no tag (an address inside CSS or
/// script), with every character reference decoded as the HTML parser decodes
/// text: named, decimal and hexadecimal alike. A list of six spellings left
/// `&#x26;`, `&sol;` and every other reference in the address.
pub fn decoded(text: &str) -> String {
    Html::parse_fragment(text).root_element().text().collect()
}
