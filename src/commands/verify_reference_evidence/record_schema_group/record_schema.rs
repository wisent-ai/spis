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
    context.verified.supports_value(value).is_ok()
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
    let mut file = std::fs::File::open(path)?;
    let mut h = sha2::Sha256::new();
    let size = std::io::copy(&mut file, &mut h)?;
    Ok((size, hex::encode(h.finalize())))
}

/// asciinema v2: JSON header line, then [time, kind, data] events, each time
/// in seconds from the recording's start.
/// https://docs.asciinema.org/manual/asciicast/v2/
pub(crate) fn probe_cast(path: &Path) -> Result<Probe> {
    use std::io::BufRead;
    let (size, sha) = digest(path)?;
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut first = String::new();
    reader.read_line(&mut first)?;
    let header: Value = match serde_json::from_str(first.trim()) {
        Ok(v) => v,
        Err(error) => {
            return Ok(Probe {
                exists: true,
                bytes: Some(size),
                sha256: Some(sha),
                kind: Some("terminal-cast".into()),
                error: Some(format!("unreadable cast header ({error})")),
                ..Default::default()
            })
        }
    };
    let mut times: Vec<Option<f64>> = Vec::new();
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
            if let Some(time) = arr.first() {
                times.push(time.as_f64());
            }
        }
    }
    let last_time = times.iter().flatten().copied().reduce(f64::max);
    Ok(Probe {
        exists: true,
        bytes: Some(size),
        sha256: Some(sha),
        kind: Some("terminal-cast".into()),
        width: header.get("width").and_then(|v| v.as_i64()),
        height: header.get("height").and_then(|v| v.as_i64()),
        // A cast whose events all sit at its start (time zero) measured no
        // duration: zero is not a normal float, so keeping only a normal
        // latest time drops exactly that case.
        duration_seconds: last_time.filter(|time| time.is_normal()).map(round3),
        frame_count: (!times.is_empty()).then_some(times.len() as i64),
        error: None,
    })
}

/// Runs `args` to its own exit through the announced wait and returns its
/// standard output; a program that fails is refused with its whole standard
/// error, which is the cause it gave.
pub(crate) fn sh(args: &[&str]) -> Result<String> {
    let Some((program, rest)) = args.split_first() else {
        anyhow::bail!("an empty command names no program to run");
    };
    let out = stado_wait::output(Command::new(program).args(rest))
        .with_context(|| format!("`{}` could not start", args.join(" ")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!(
            "`{}` failed with {}: {}",
            args.join(" "),
            out.status,
            stderr.trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
