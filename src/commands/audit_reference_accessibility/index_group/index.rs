use super::*;

pub(crate) const INDEX: &str = "accessibility-audit-index.json";

pub(crate) const PLAN_SCHEMA: &str = "wisent.weles-capture-plan.v1";

pub(crate) const INDEX_SCHEMA: &str = "wisent.accessibility-audit-index.v1";

pub(crate) const ACTION: &str = "generic_accessibility_audit";

pub(crate) const NAMESPACE: &str = "stado://weles-captures/";

pub(crate) const DEFAULT_TARGET: &str = "charless-mac-mini";

pub(crate) const DEFAULT_CATALOGS: &[&str] = &[
    "web-app-examples",
    "dashboard-console-examples",
    "documentation-site-examples",
    "design-system-examples",
    "onboarding-auth-examples",
];

pub(crate) const ACTION_KEYS: &[&str] = &[
    "batch",
    "site_slug",
    "source_url",
    "viewport",
    "artifact_prefix",
];

pub(crate) const PLAN_KEYS: &[&str] = &["schema", "batch", "target", "captures"];

pub(crate) const SUMMARY_FIELDS: &[&str] = &[
    "source_url",
    "viewport",
    "captured_at",
    "renderer",
    "weles_version",
    "axe_version",
    "violation_count",
    "violations",
    "passes_count",
    "incomplete_count",
    "bytes",
    "sha256",
];

/// Deviation from the Python original, which used ~/.stado/work: generated
/// working files stay under ~/.spis/work per harness policy.
pub(crate) fn work_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".spis").join("work")
}

pub(crate) fn plan_dir() -> PathBuf {
    work_root().join("accessibility-audit-plans")
}

pub(crate) fn staging_root() -> PathBuf {
    work_root().join("accessibility-audits")
}

// ---------------------------------------------------------------------------
// Small utilities

pub(crate) fn which(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

pub(crate) fn now() -> String {
    crate::now_iso_utc()
}

pub(crate) fn default_batch() -> String {
    // Python: strftime("accessibility-%Y%m%dt%H%M%Sz")
    let iso = now();
    format!(
        "accessibility-{}{}{}t{}{}{}z",
        &iso[0..4],
        &iso[5..7],
        &iso[8..10],
        &iso[11..13],
        &iso[14..16],
        &iso[17..19]
    )
}

pub(crate) fn pid() -> u32 {
    std::process::id()
}

/// Lexical normalization that works for not-yet-existing paths (Path::resolve
/// equivalent for our purposes).
pub(crate) fn lex_norm(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub(crate) fn is_relative_to(path: &Path, base: &Path) -> bool {
    path.starts_with(base)
}

pub(crate) fn basename_of(key: &str) -> String {
    key.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string()
}

pub(crate) fn looks_like_slug(value: &str) -> bool {
    // fullmatch r"[a-z0-9][a-z0-9._-]{0,80}"
    let mut chars = value.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    value.chars().count() <= 81
        && value.chars().skip(1).all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-'
        })
}

pub(crate) fn is_hex64_lower(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// ---------------------------------------------------------------------------
// Strict JSON parsing with duplicate-key detection

pub(crate) struct StrictParser<'a> {
    pub(crate) text: &'a [u8],
    pub(crate) pos: usize,
    pub(crate) depth: usize,
}
