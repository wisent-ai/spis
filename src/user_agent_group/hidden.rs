//! The HTML elements whose content a browser never renders as text: the raw
//! text elements, the inert template, the no-script fallback and embedded SVG
//! (https://html.spec.whatwg.org/multipage/syntax.html#elements-2 and
//! https://html.spec.whatwg.org/multipage/rendering.html#hidden-elements).

use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum HiddenElement {
    Script,
    Style,
    Template,
    Noscript,
    Svg,
}

impl HiddenElement {
    /// The hidden element a lowercase tag name names, or nothing.
    pub(crate) fn named(tag: &str) -> Option<HiddenElement> {
        serde_json::from_value(Value::String(tag.to_string())).ok()
    }
}
