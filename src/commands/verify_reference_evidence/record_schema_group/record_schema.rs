use super::*;

// ---------------------------------------------------------------- contract --
// The reference-evidence vocabulary, its schemas and its completeness floors
// are the ones `reference_contract` declares; this tool keeps no copy of them.

pub(crate) use crate::commands::reference_contract::{
    canonical_motion_kind, canonical_timing_class, container_kind, INDEX_SCHEMA, MIN_MOTION_FRAMES,
    RECORDS_PER_CATALOG, RECORD_SCHEMA, STILL_KIND,
};

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
        // A damaged event line is damaged evidence: skipped, the cast would
        // verify with fewer frames and a shorter duration than recorded.
        let event: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(error) => {
                return Ok(Probe {
                    exists: true,
                    bytes: Some(size),
                    sha256: Some(sha),
                    kind: Some("terminal-cast".into()),
                    error: Some(format!("cast event is not JSON ({error}): {line}")),
                    ..Default::default()
                })
            }
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
