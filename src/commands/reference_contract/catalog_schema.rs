use super::*;

pub const CATALOG_SCHEMA: &str = "wisent.example-catalog.v2";

pub const RECORD_SCHEMA: &str = "wisent.full-product-reference.v2";

pub const INDEX_SCHEMA: &str = "wisent.full-reference-catalog.v2";

/// Declared media kind (any historical spelling) -> canonical kind.
pub fn canonical_motion_kind(declared: Option<&str>) -> Option<&'static str> {
    let declared = declared?;
    Some(match declared.trim().to_ascii_lowercase().as_str() {
        "mp4" | "video/mp4" | "h264" | "video-mp4" => "video-mp4",
        "webm" | "video/webm" | "video-webm" => "video-webm",
        "gif" | "image/gif" | "animated-gif" => "animated-gif",
        "webp" | "image/webp" | "animated-webp" => "animated-webp",
        "cast" | "asciinema-v2-terminal-cast" | "terminal-cast" => "terminal-cast",
        _ => return None,
    })
}

pub const STILL_KIND: &str = "still-image";

/// ffprobe container name -> canonical kind.
pub fn container_kind(container: &str) -> Option<&'static str> {
    Some(match container {
        "mov,mp4,m4a,3gp,3g2,mj2" => "video-mp4",
        "matroska,webm" => "video-webm",
        "gif" => "animated-gif",
        "webp_pipe" | "webp" => "animated-webp",
        "image2" | "png_pipe" | "mjpeg" => STILL_KIND,
        _ => return None,
    })
}

pub fn is_motion_suffix(suffix: &str) -> bool {
    matches!(
        suffix.to_ascii_lowercase().as_str(),
        ".gif" | ".webp" | ".mp4" | ".webm" | ".cast"
    )
}

pub fn is_state_suffix(suffix: &str) -> bool {
    matches!(
        suffix.to_ascii_lowercase().as_str(),
        ".png" | ".webp" | ".jpg" | ".jpeg"
    )
}

/// How motion was obtained. Cryptographically verified Weles evidence is a distinct
/// class and is never inferred from capture-method prose.
pub const PROVENANCE_CLASSES: &[&str] = &[
    "local-product-run",
    "weles-signed-browser-evidence",
    "upstream-owner-media",
    "unclassified",
];

pub fn is_local_provenance(class: &str) -> bool {
    class == "local-product-run"
}

/// Case-insensitive substring test (the plain-keyword patterns below).
pub(crate) fn ci_contains(hay: &str, needle: &str) -> bool {
    hay.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

/// `\b<word>\b` case-insensitive: occurrence delimited by non-word characters.
pub(crate) fn word_ci(hay: &str, word: &str) -> bool {
    let hay_lc = hay.to_ascii_lowercase();
    let word_lc = word.to_ascii_lowercase();
    let bytes = hay_lc.as_bytes();
    let mut from = 0;
    while let Some(pos) = hay_lc[from..].find(&word_lc) {
        let start = from + pos;
        let end = start + word_lc.len();
        let before_ok = start == 0 || !is_word_byte(bytes[start - 1]);
        let after_ok = end >= bytes.len() || !is_word_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

pub(crate) fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `official[- ][\w -]*(media|asset|recording|stream|preview|trailer|tour|download)`
/// case-insensitive: after `official` plus one separator, a maximal run of
/// word/space/dash characters contains one of the object keywords.
pub(crate) fn official_asset_pattern(text: &str) -> bool {
    const OBJECTS: &[&str] = &[
        "media",
        "asset",
        "recording",
        "stream",
        "preview",
        "trailer",
        "tour",
        "download",
    ];
    let lc = text.to_ascii_lowercase();
    let bytes = lc.as_bytes();
    let mut from = 0;
    while let Some(rel) = lc[from..].find("official") {
        let start = from + rel;
        let mut cursor = start + "official".len();
        match bytes.get(cursor) {
            Some(b'-') | Some(b' ') => cursor += 1,
            _ => {
                from = start + 1;
                continue;
            }
        }
        let span_start = cursor;
        while let Some(&b) = bytes.get(cursor) {
            if b.is_ascii_alphanumeric() || b == b'_' || b == b' ' || b == b'-' {
                cursor += 1;
            } else {
                break;
            }
        }
        let span = &lc[span_start..cursor];
        if OBJECTS.iter().any(|o| span.contains(o)) {
            return true;
        }
        from = start + 1;
    }
    false
}

/// Derive the provenance class from the recorded capture method.
///
/// The order matters: a locally driven run wins over any wording about the source,
/// because a cast we recorded of a product we installed is our observation even when
/// the product's own site is cited as the subject.
pub fn classify_provenance(capture_method: Option<&str>, media_kind: Option<&str>) -> &'static str {
    let text = capture_method.unwrap_or("");
    for pattern in [
        ("word", "pseudo-terminal"),
        ("word", "pseudoterminal"),
        ("word", "pty"),
        ("plain", "asciinema"),
        ("plain", "terminal cast"),
        ("plain", "real executable"),
        ("plain", "stdout/stderr"),
        ("plain", "real installed product recorded"),
        ("plain", "isolated temporary working directory"),
        ("plain", "local run of the installed product"),
    ] {
        let hit = match pattern.0 {
            "word" => word_ci(text, pattern.1),
            _ => ci_contains(text, pattern.1),
        };
        if hit {
            return "local-product-run";
        }
    }
    for pattern in [
        ("word", "weles"),
        ("plain", "patched chromium"),
        ("plain", "browser recording"),
        ("plain", "screencast"),
    ] {
        let hit = match pattern.0 {
            "word" => word_ci(text, pattern.1),
            _ => ci_contains(text, pattern.1),
        };
        if hit {
            return "unclassified";
        }
    }
    for pattern in [
        "yt-dlp",
        "youtube",
        "cobalt",
        "download of",
        "direct download",
        "official-product",
        "product-site",
        "apptrailers",
        "play-games",
        "publisher",
        "video channel",
        "downloaded",
    ] {
        if ci_contains(text, pattern) {
            return "upstream-owner-media";
        }
    }
    if official_asset_pattern(text) {
        return "upstream-owner-media";
    }
    if media_kind == Some("terminal-cast") {
        return "local-product-run";
    }
    "unclassified"
}

pub const MIN_MOTION_SECONDS: f64 = 0.2;

pub const MIN_MOTION_FRAMES: u32 = 2;

pub const MIN_STATES: usize = 3;

pub const MIN_JOURNEY_STEPS: usize = 5;

pub const MIN_INTERACTIONS: usize = 8;

pub const MIN_ACCESSIBILITY_OBSERVATIONS: usize = 3;

/// Mean abs difference, 0-255, for a proven frame match.
pub const STATE_MATCH_MAX_DIFF: f64 = 12.0;

pub const INTERACTION_FIELDS: &[&str] = &[
    "name",
    "trigger",
    "response",
    "feedback",
    "cancellation",
    "failure",
    "recovery",
    "evidence",
];

pub const MOTION_ANALYSIS_FIELDS: &[&str] = &[
    "trigger",
    "start_state",
    "end_state",
    "continuity",
    "timing_class",
    "interruption_or_reversal",
    "feedback",
    "reduced_motion_equivalent",
];

/// The corpus spelled two of these fields three different ways. Records are rewritten
/// to the canonical spelling; the aliases stay here so an old record still normalizes.
pub fn motion_analysis_alias(field: &str) -> Option<&'static str> {
    Some(match field {
        "interruption_reversal" => "interruption_or_reversal",
        "interruption_and_reversal" => "interruption_or_reversal",
        "reduced_motion_or_nonanimated_equivalent" => "reduced_motion_equivalent",
        _ => return None,
    })
}

/// Extra keys a motion analysis may carry beyond the required eight.
pub const MOTION_ANALYSIS_OPTIONAL: &[&str] = &["source_title", "evidence", "timing_description"];

pub fn is_timing_class(value: &str) -> bool {
    matches!(
        value,
        "instant" | "sub-second" | "one-to-three-seconds" | "multi-second" | "continuous"
    )
}

pub(crate) const TIMING_CLASS_ALIASES: &[(&str, &str)] = &[
    (
        "direct-manipulation feedback followed by a short product transition",
        "one-to-three-seconds",
    ),
    (
        "immediate selection feedback followed by short asynchronous settling within the 15-second excerpt.",
        "one-to-three-seconds",
    ),
    (
        "immediate control feedback followed by task-dependent result feedback",
        "one-to-three-seconds",
    ),
    ("extended guided walkthrough", "multi-second"),
    ("extended guided demonstration", "multi-second"),
    ("brief component feedback", "sub-second"),
    ("short guided sequence", "one-to-three-seconds"),
    ("rapid microinteraction", "sub-second"),
];
