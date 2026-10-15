use super::*;

/// The families Brama may sort a discovered page into: the answer vocabulary
/// of `brama_rank`'s judgement, never matched against page text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PageFamily {
    Pricing,
    Docs,
    Signup,
    About,
    Product,
    Blog,
    Other,
}

impl PageFamily {
    /// Every family, in the order the question lists them.
    pub(crate) const ALL: &'static [PageFamily] = &[
        PageFamily::Pricing,
        PageFamily::Docs,
        PageFamily::Signup,
        PageFamily::About,
        PageFamily::Product,
        PageFamily::Blog,
        PageFamily::Other,
    ];

    /// The names the question offers, as Brama must answer them.
    pub(crate) fn names() -> Result<Vec<serde_json::Value>> {
        PageFamily::ALL
            .iter()
            .map(|family| {
                serde_json::to_value(family)
                    .with_context(|| format!("discover: page family {family:?} has no name"))
            })
            .collect()
    }
}
