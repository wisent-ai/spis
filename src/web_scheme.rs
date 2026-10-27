//! Whether an address is a web address, read from its scheme. HTTP defines
//! exactly two URI schemes for its resources, `http` and `https`
//! (https://www.rfc-editor.org/rfc/rfc9110#section-4.2); every Spis reader
//! asks this type instead of spelling them again.

/// The scheme of a web address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebScheme {
    /// Plain HTTP.
    Http,
    /// HTTP over TLS.
    Https,
}

impl WebScheme {
    /// Every web scheme.
    pub const ALL: &'static [WebScheme] = &[WebScheme::Http, WebScheme::Https];

    /// The scheme's name as a parsed URL writes it (lowercase).
    pub fn name(self) -> &'static str {
        match self {
            WebScheme::Http => "http",
            WebScheme::Https => "https",
        }
    }

    /// The web scheme `url` is addressed by; none when it is not a web address.
    pub fn of(url: &url::Url) -> Option<WebScheme> {
        Self::named(url.scheme())
    }

    /// The web scheme called exactly `scheme` (lowercase, as a parsed URL
    /// writes it); none when no web scheme is called that.
    pub fn named(scheme: &str) -> Option<WebScheme> {
        Self::ALL.iter().copied().find(|web| web.name() == scheme)
    }

    /// The address is reached over TLS.
    pub fn encrypted(self) -> bool {
        self == WebScheme::Https
    }
}
