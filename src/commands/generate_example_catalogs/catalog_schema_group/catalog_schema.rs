use super::*;

// ---------------------------------------------------------------------------
// reference_contract vocabulary
// ---------------------------------------------------------------------------

pub(crate) const CATALOG_SCHEMA: &str = "wisent.example-catalog.v2";

pub(crate) const RECORD_SCHEMA: &str = "wisent.full-product-reference.v2";

pub(crate) const INDEX_SCHEMA: &str = "wisent.full-reference-catalog.v2";

/// Canonical motion-kind vocabulary (`CANONICAL_MOTION_KINDS`, sorted).
pub(crate) const CANONICAL_MOTION_KINDS: &[&str] = &[
    "animated-gif",
    "animated-webp",
    "terminal-cast",
    "video-mp4",
    "video-webm",
];

pub(crate) const MOTION_SUFFIXES: &[&str] = &[".gif", ".webp", ".mp4", ".webm", ".cast"];

pub(crate) const STATE_SUFFIXES: &[&str] = &[".png", ".webp", ".jpg", ".jpeg"];

pub(crate) const PROVENANCE_CLASSES: &[&str] = &[
    "local-product-run",
    "weles-signed-browser-evidence",
    "upstream-owner-media",
    "unverified-source-media",
];

pub(crate) const LOCAL_PROVENANCE: &[&str] = &["local-product-run"];

pub(crate) const RECORDS_PER_CATALOG: usize = 50;

pub(crate) const INTERACTION_FIELDS: &[&str] = &[
    "name",
    "trigger",
    "response",
    "feedback",
    "cancellation",
    "failure",
    "recovery",
    "evidence",
];

pub(crate) const MOTION_ANALYSIS_FIELDS: &[&str] = &[
    "trigger",
    "start_state",
    "end_state",
    "continuity",
    "timing_class",
    "interruption_or_reversal",
    "feedback",
    "reduced_motion_equivalent",
];

pub(crate) const MOTION_ANALYSIS_OPTIONAL: &[&str] =
    &["source_title", "evidence", "timing_description", "provenance"];

pub(crate) const TIMING_CLASSES: &[&str] = &[
    "continuous",
    "instant",
    "multi-second",
    "one-to-three-seconds",
    "sub-second",
];

pub(crate) const EVIDENCE_STATUSES: &[&str] = &["complete", "partial"];

pub(crate) const JOURNEY_FIELDS: &[&str] = &[
    "actor",
    "goal",
    "prerequisites",
    "steps",
    "failure_route",
    "recovery_route",
    "completion_evidence",
];

pub(crate) const JOURNEY_STEP_FIELDS: &[&str] = &[
    "index",
    "user_action",
    "system_response",
    "state",
    "evidence",
];

/// Exact public corpus contract. Extra or missing families are refused.
pub(crate) const CATALOGS: &[&str] = &[
    "ios-app-examples",
    "android-app-examples",
    "macos-app-examples",
    "desktop-app-examples",
    "web-app-examples",
    "dashboard-console-examples",
    "tui-examples",
    "cli-examples",
    "onboarding-auth-examples",
    "documentation-site-examples",
    "app-store-listing-examples",
    "design-system-examples",
    "report-evidence-examples",
    "pricing-page-examples",
    "landing-page-examples",
];

pub(crate) fn provenance_label(name: &str) -> &'static str {
    match name {
        "local-product-run" => "verified product run here",
        "weles-signed-browser-evidence" => "Weles-signed browser evidence",
        "upstream-owner-media" => "verified product-owner media",
        "unverified-source-media" => "unverified source media",
        _ => "invalid provenance",
    }
}

// ---------------------------------------------------------------------------
// Small helpers mirroring Python semantics
// ---------------------------------------------------------------------------

/// Truthiness of a JSON value under Python rules (None/false/0/""/[]/{}) are falsey).
pub(crate) fn py_truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

pub(crate) fn nonempty_observation(value: &Value) -> bool {
    value
        .get("observation")
        .and_then(Value::as_str)
        .is_some_and(|text| !text.trim().is_empty())
}

pub(crate) fn is_null_or_empty_string_or_list(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) => s.is_empty(),
        Some(Value::Array(a)) => a.is_empty(),
        _ => false,
    }
}

pub(crate) fn require_nonempty(record: &Value, fields: &[&str], context: &str) -> Result<()> {
    let missing: Vec<&str> = fields
        .iter()
        .filter(|field| is_null_or_empty_string_or_list(record.get(**field)))
        .copied()
        .collect();
    if !missing.is_empty() {
        bail!("{context}: missing {:?}", missing);
    }
    Ok(())
}

pub(crate) fn python_repr(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "None".into(),
        Some(Value::Bool(true)) => "True".into(),
        Some(Value::Bool(false)) => "False".into(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::String(s)) => format!("'{s}'"),
        other => format!("{}", other.map(|v| v.to_string()).unwrap_or_default()),
    }
}

/// urlparse-based http(s) URL validity: scheme must be http/https, netloc non-empty.
pub(crate) fn url_ok(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if !(scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")) {
        return false;
    }
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    // Strip userinfo; netloc itself must be non-empty.
    let host = authority.rsplit('@').next().unwrap_or("");
    !authority.is_empty() && !host.is_empty()
}

/// Resolve `relative` under `base`, refuse escapes and non-files. Lexical
/// normalization stands in for pathlib.resolve(); symlinks inside a catalog
/// pointing outside are not followed here.
pub(crate) fn resolve_evidence_path(base: &Path, relative: &str, context: &str) -> Result<PathBuf> {
    let joined = base.join(relative);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    bail!("{context}: unavailable local evidence {relative:?}");
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    if !normalized.starts_with(base) || !normalized.is_file() {
        bail!("{context}: unavailable local evidence {relative:?}");
    }
    Ok(normalized)
}

pub(crate) fn validate_file_metadata(path: &Path, record: &Value, context: &str) -> Result<()> {
    let payload = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let expected_bytes = record.get("bytes").and_then(Value::as_u64);
    if Some(payload.len() as u64) != expected_bytes {
        bail!("{context}: byte count differs from the file");
    }
    let expected_sha = record.get("sha256").and_then(Value::as_str);
    if Some(lib::sha256_hex(&payload).as_str()) != expected_sha {
        bail!("{context}: SHA-256 differs from the file");
    }
    Ok(())
}

pub(crate) fn has_suffix(path: &Path, suffixes: &[&str]) -> bool {
    path.extension()
        .map(|ext| {
            let dotted = format!(".{}", ext.to_string_lossy().to_lowercase());
            suffixes.contains(&dotted.as_str())
        })
        .unwrap_or(false)
}

pub(crate) fn evidence_status_of(record: &Value) -> Option<&str> {
    record.get("evidence_status").and_then(Value::as_str)
}
