use super::*;

// ---------------------------------------------------------------- contract --
// From reference_contract.py. Change these rules and this tool changes with it.

pub(crate) const RECORD_SCHEMA: &str = "wisent.full-product-reference.v2";

pub(crate) const INDEX_SCHEMA: &str = "wisent.full-reference-catalog.v2";

pub(crate) const STILL_KIND: &str = "still-image";

pub(crate) const RECORDS_PER_CATALOG: usize = 50;

pub(crate) const KNOWN_CATALOGS: &[&str] = &[
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

/// Declared media kind (any historical spelling) -> canonical kind.
pub(crate) fn canonical_motion_kind(declared: Option<&str>) -> Option<String> {
    let d = declared?.trim().to_lowercase();
    let canonical = match d.as_str() {
        "mp4" | "video/mp4" | "h264" | "video-mp4" => "video-mp4",
        "webm" | "video/webm" | "video-webm" => "video-webm",
        "gif" | "image/gif" | "animated-gif" => "animated-gif",
        "webp" | "image/webp" | "animated-webp" => "animated-webp",
        "cast" | "asciinema-v2-terminal-cast" | "terminal-cast" => "terminal-cast",
        _ => return None,
    };
    Some(canonical.to_string())
}

/// ffprobe container name -> canonical kind.
pub(crate) fn container_kind(name: &str) -> Option<&'static str> {
    Some(match name {
        "mov,mp4,m4a,3gp,3g2,mj2" => "video-mp4",
        "matroska,webm" => "video-webm",
        "gif" => "animated-gif",
        "webp_pipe" | "webp" => "animated-webp",
        "image2" | "png_pipe" | "mjpeg" => STILL_KIND,
        _ => return None,
    })
}

pub(crate) const MIN_MOTION_FRAMES: i64 = 2;

/// Mean abs difference, 0-255, for a proven frame match.
pub(crate) const STATE_MATCH_MAX_DIFF: f64 = 12.0;

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

/// The corpus spelled two of these fields three different ways. Records are
/// rewritten to the canonical spelling; aliases normalize old records.
pub(crate) const MOTION_ANALYSIS_ALIASES: &[(&str, &str)] = &[
    ("interruption_reversal", "interruption_or_reversal"),
    ("interruption_and_reversal", "interruption_or_reversal"),
    (
        "reduced_motion_or_nonanimated_equivalent",
        "reduced_motion_equivalent",
    ),
];

/// Extra keys a motion analysis may carry beyond the required eight.
pub(crate) const MOTION_ANALYSIS_OPTIONAL: &[&str] =
    &["source_title", "evidence", "timing_description", "provenance"];

pub(crate) const TIMING_CLASSES: &[&str] = &[
    "instant",
    "sub-second",
    "one-to-three-seconds",
    "multi-second",
    "continuous",
];

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

pub(crate) fn canonical_timing_class(value: Option<&str>) -> Option<String> {
    let value = value?;
    let normalized = value.trim().to_lowercase();
    if TIMING_CLASSES.contains(&normalized.as_str()) {
        return Some(normalized);
    }
    TIMING_CLASS_ALIASES
        .iter()
        .find(|(alias, _)| *alias == normalized)
        .map(|(_, canon)| canon.to_string())
}

pub(crate) const UNVERIFIED_NOTE_SCHEMA: &str = "wisent.unverified-source-note.v1";

/// Per-record cache of receipts and artifact contexts that were reverified by the
/// pinned official Weles client. Each observation must still carry its own typed
/// link to an exact artifact member; record-level presence never supports a value.
#[derive(Clone, Default)]
pub(crate) struct ProvenanceContext {
    pub(crate) verified: VerifiedProvenanceSet,
}

impl ProvenanceContext {
    pub(crate) fn from_record(record: &Value, base: &Path) -> Self {
        Self {
            verified: VerifiedProvenanceSet::verify_record(record, base),
        }
    }

    pub(crate) fn failures(&self) -> &[String] {
        self.verified.failures()
    }
}

pub(crate) fn observation_supported(value: &Value, context: &ProvenanceContext) -> bool {
    context.verified.supports_value(value)
}

pub(crate) fn provenance_class(value: &Value, context: &ProvenanceContext) -> &'static str {
    context.verified.provenance_class(value)
}

// ------------------------------------------------------------------ probes --

#[derive(Debug, Clone, Default)]
pub(crate) struct Probe {
    pub(crate) exists: bool,
    pub(crate) bytes: Option<u64>,
    pub(crate) sha256: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) width: Option<i64>,
    pub(crate) height: Option<i64>,
    pub(crate) duration_seconds: Option<f64>,
    pub(crate) frame_count: Option<i64>,
    pub(crate) error: Option<String>,
}

pub(crate) fn digest(path: &Path) -> Result<(u64, String)> {
    use sha2::Digest;
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut h = sha2::Sha256::new();
    let mut size = 0u64;
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        h.update(&buf[..n]);
    }
    Ok((
        size,
        h.finalize().iter().map(|b| format!("{b:02x}")).collect(),
    ))
}

/// asciinema v2: JSON header line, then [time, kind, data] events.
pub(crate) fn probe_cast(path: &Path) -> Result<Probe> {
    use std::io::BufRead;
    let (size, sha) = digest(path)?;
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut first = String::new();
    reader.read_line(&mut first)?;
    let header: Value = match serde_json::from_str(first.trim()) {
        Ok(v) => v,
        Err(_) => {
            return Ok(Probe {
                exists: true,
                bytes: Some(size),
                sha256: Some(sha),
                kind: Some("terminal-cast".into()),
                error: Some("unreadable cast header".into()),
                ..Default::default()
            })
        }
    };
    let mut last_time = 0.0f64;
    let mut frames = 0usize;
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let event: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(arr) = event.as_array() {
            if arr.is_empty() {
                continue;
            }
            frames += 1;
            if let Some(t) = arr[0].as_f64() {
                last_time = last_time.max(t);
            }
        }
    }
    Ok(Probe {
        exists: true,
        bytes: Some(size),
        sha256: Some(sha),
        kind: Some("terminal-cast".into()),
        width: header.get("width").and_then(|v| v.as_i64()),
        height: header.get("height").and_then(|v| v.as_i64()),
        duration_seconds: if last_time != 0.0 {
            Some(round3(last_time))
        } else {
            None
        },
        frame_count: if frames > 0 {
            Some(frames as i64)
        } else {
            None
        },
        error: None,
    })
}

pub(crate) fn sh(args: &[&str]) -> Result<String> {
    let out = Command::new(args[0]).args(&args[1..]).output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stderr = stderr.trim();
        let stderr = &stderr[..stderr.len().min(300)];
        anyhow::bail!("{} failed: {stderr}", args[0]);
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
