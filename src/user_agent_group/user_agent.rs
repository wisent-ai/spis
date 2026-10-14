use super::*;

pub const USER_AGENT: &str = concat!(
    "Spis/",
    env!("CARGO_PKG_VERSION"),
    " (evidence-grade interface corpus; +https://spis.wisent.com/docs)"
);

/// Every object a crawl publishes lives under one namespace and one of its two
/// named roots.
///
/// Stado authorizes object traffic per namespace *prefix*
/// (`object_api.namespaces`, `ObjectPrefixPolicy`), and a prefix only matches
/// as a prefix when it ends in `/`; a key that begins with a per-run or
/// per-digest identifier can therefore be granted by nothing narrower than the
/// empty prefix, which is the whole namespace. Every namespace Stado declares
/// grants named first segments instead, so the crawl tree carries two: one
/// attempt root and one input root. They share a namespace deliberately — the
/// caller sends exactly one bearer per request and the service compares it
/// against the credential item of the namespace being addressed, so a second
/// namespace would mean a second bearer for the same coordinator run.
pub const CRAWL_NAMESPACE: &str = "stado://spis-crawls";

/// Immutable attempt trees: `{CRAWL_ATTEMPT_ROOT}/{run_id}/...`.
pub const CRAWL_ATTEMPT_ROOT: &str = "stado://spis-crawls/runs";

/// Immutable coordinator inputs, addressed by content digest. Today that is
/// the generated runtime-bindings document every worker re-downloads.
pub const CRAWL_INPUT_ROOT: &str = "stado://spis-crawls/inputs/runtime-bindings";

/// The immutable coordinate of one record inside one run.
pub fn crawl_record_base_uri(
    run_id: &str,
    catalog: &str,
    record: &str,
    record_key: &str,
) -> String {
    format!("{CRAWL_ATTEMPT_ROOT}/{run_id}/{catalog}/{record}/{record_key}")
}

/// The immutable coordinate of one attempt of one record. Every producer and
/// every verifier of an attempt URI derives it here, so a change of shape can
/// never leave one side of a digest comparison spelling it the old way.
pub fn crawl_attempt_base_uri(
    run_id: &str,
    catalog: &str,
    record: &str,
    record_key: &str,
    // The attempt number is a u32 in the runtime manifest, a u64 in the
    // durable state and a serde integer in a receipt. It renders identically
    // in all three, and accepting them by display keeps one derivation
    // instead of a cast at every call site.
    attempt: impl std::fmt::Display,
    attempt_id: &str,
) -> String {
    format!(
        "{}/attempts/{attempt}/{attempt_id}",
        crawl_record_base_uri(run_id, catalog, record, record_key)
    )
}

pub(crate) fn between<'a>(hay: &'a str, open: &str, close: &str) -> Option<String> {
    let s = hay.find(open)? + open.len();
    let e = hay[s..].find(close)? + s;
    Some(html_unescape(hay[s..e].trim()))
}
