//! Whether an address is a web address, read from its scheme. HTTP defines
//! exactly two URI schemes for its resources, `http` and `https`
//! (https://www.rfc-editor.org/rfc/rfc9110#section-4.2); every Spis reader
//! asks this type instead of spelling them again.

use serde::Deserialize;
use serde_json::Value;

/// The scheme of a web address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebScheme {
    /// Plain HTTP.
    Http,
    /// HTTP over TLS.
    Https,
}

impl WebScheme {
    /// The web scheme `url` is addressed by, or nothing when it is not a
    /// web address.
    pub fn of(url: &url::Url) -> Option<WebScheme> {
        Self::named(url.scheme())
    }

    /// The web scheme called exactly `scheme` (lowercase, as a parsed URL
    /// writes it), or nothing when no web scheme is called that.
    pub fn named(scheme: &str) -> Option<WebScheme> {
        serde_json::from_value(Value::String(scheme.to_string())).ok()
    }

    /// The address is reached over TLS.
    pub fn encrypted(self) -> bool {
        self == WebScheme::Https
    }
}
