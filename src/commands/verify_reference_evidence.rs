//! `spis verify-reference-evidence` — measure the stored reference evidence and
//! rewrite every record to what the files prove.
//!
//! Reads every `<catalog>/references/*/reference.json`, probes the real media
//! with ffprobe (or the asciinema cast header), verifies bytes and SHA-256,
//! derives the media kind and provenance class from observable facts, locates
//! each state frame inside its motion source by pixel comparison, and recomputes
//! `evidence_status` from the measured floor.
//!
//! Nothing here invents evidence. A field that cannot be measured is null and
//! named in `evidence_gaps`; the record is not called complete.
//!
//! Ported 1:1 from verify-reference-evidence.py + reference_contract.py.

use crate as lib;
use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

// ---------------------------------------------------------------- contract --
// From reference_contract.py. Change these rules and this tool changes with it.

const RECORD_SCHEMA: &str = "wisent.full-product-reference.v2";
const INDEX_SCHEMA: &str = "wisent.full-reference-catalog.v2";
const STILL_KIND: &str = "still-image";

/// Declared media kind (any historical spelling) -> canonical kind.
fn canonical_motion_kind(declared: Option<&str>) -> Option<String> {
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
fn container_kind(name: &str) -> Option<&'static str> {
    Some(match name {
        "mov,mp4,m4a,3gp,3g2,mj2" => "video-mp4",
        "matroska,webm" => "video-webm",
        "gif" => "animated-gif",
        "webp_pipe" | "webp" => "animated-webp",
        "image2" | "png_pipe" | "mjpeg" => STILL_KIND,
        _ => return None,
    })
}

const MIN_MOTION_SECONDS: f64 = 0.2;
const MIN_MOTION_FRAMES: i64 = 2;
const MIN_STATES: usize = 3;
const MIN_JOURNEY_STEPS: usize = 5;
const MIN_INTERACTIONS: usize = 8;
const MIN_ACCESSIBILITY_OBSERVATIONS: usize = 3;
/// Mean abs difference, 0-255, for a proven frame match.
const STATE_MATCH_MAX_DIFF: f64 = 12.0;

const INTERACTION_FIELDS: &[&str] = &[
    "name",
    "trigger",
    "response",
    "feedback",
    "cancellation",
    "failure",
    "recovery",
    "evidence",
];

const MOTION_ANALYSIS_FIELDS: &[&str] = &[
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
const MOTION_ANALYSIS_ALIASES: &[(&str, &str)] = &[
    ("interruption_reversal", "interruption_or_reversal"),
    ("interruption_and_reversal", "interruption_or_reversal"),
    (
        "reduced_motion_or_nonanimated_equivalent",
        "reduced_motion_equivalent",
    ),
];

/// Extra keys a motion analysis may carry beyond the required eight.
const MOTION_ANALYSIS_OPTIONAL: &[&str] = &["source_title", "evidence", "timing_description"];

const TIMING_CLASSES: &[&str] = &[
    "instant",
    "sub-second",
    "one-to-three-seconds",
    "multi-second",
    "continuous",
];

const TIMING_CLASS_ALIASES: &[(&str, &str)] = &[
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

fn canonical_timing_class(value: Option<&str>) -> Option<String> {
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

// Case-insensitive pattern tables. The Python originals were regexes with word
// boundaries; plain lowercase-substring matching preserves the classifications
// on this corpus without pulling in a regex engine.
const LOCAL_RUN_PATTERNS: &[&str] = &[
    "pseudo-terminal",
    "pseudoterminal",
    "pty",
    "asciinema",
    "terminal cast",
    "real executable",
    "stdout/stderr",
    "real installed product recorded",
    "isolated temporary working directory",
    "local run of the installed product",
];

const LOCAL_BROWSER_PATTERNS: &[&str] =
    &["weles", "patched chromium", "browser recording", "screencast"];

const UPSTREAM_PATTERNS: &[&str] = &[
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
];

const UPSTREAM_MEDIA_WORDS: &[&str] = &[
    "media", "asset", "recording", "stream", "preview", "trailer", "tour", "download",
];

/// r"official[- ][\w -]*(media|asset|recording|stream|preview|trailer|tour|download)"
fn official_media_phrase(text: &str) -> bool {
    let lower = text.to_lowercase();
    let mut from = 0;
    while let Some(pos) = lower[from..].find("official") {
        let start = from + pos;
        // Require the non-word boundary before "official".
        let boundary_ok = start == 0
            || !lower[..start]
                .chars()
                .next_back()
                .map(|c| c.is_ascii_alphanumeric())
                .unwrap_or(false);
        let window_end = (start + 48).min(lower.len());
        let window = &lower[start..window_end];
        if boundary_ok
            && UPSTREAM_MEDIA_WORDS
                .iter()
                .any(|w| window.contains(&format!(" {w}")) || window.contains(&format!("-{w}")))
        {
            return true;
        }
        from = start + "official".len();
    }
    false
}

/// Derive the provenance class from the recorded capture method. The order
/// matters: a locally driven run wins over any wording about the source.
fn classify_provenance(capture_method: Option<&str>, media_kind: Option<&str>) -> &'static str {
    let text = capture_method.unwrap_or("");
    let hay = text.to_lowercase();
    if LOCAL_RUN_PATTERNS.iter().any(|p| hay.contains(p)) {
        return "local-product-run";
    }
    if LOCAL_BROWSER_PATTERNS.iter().any(|p| hay.contains(p)) {
        return "local-browser-recording";
    }
    if UPSTREAM_PATTERNS.iter().any(|p| hay.contains(p)) || official_media_phrase(text) {
        return "upstream-owner-media";
    }
    if media_kind == Some("terminal-cast") {
        return "local-product-run";
    }
    "unclassified"
}

// ------------------------------------------------------------------ probes --

#[derive(Debug, Clone, Default)]
struct Probe {
    exists: bool,
    bytes: Option<u64>,
    sha256: Option<String>,
    kind: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
    duration_seconds: Option<f64>,
    frame_count: Option<i64>,
    error: Option<String>,
}

fn digest(path: &Path) -> Result<(u64, String)> {
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
fn probe_cast(path: &Path) -> Result<Probe> {
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
        frame_count: if frames > 0 { Some(frames as i64) } else { None },
        error: None,
    })
}

fn sh(args: &[&str]) -> Result<String> {
    let out = Command::new(args[0]).args(&args[1..]).output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stderr = stderr.trim();
        let stderr = &stderr[..stderr.len().min(300)];
        anyhow::bail!("{} failed: {stderr}", args[0]);
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn probe_media(path: &Path) -> Result<Probe> {
    if !path.exists() {
        return Ok(Probe {
            exists: false,
            error: Some("file missing".into()),
            ..Default::default()
        });
    }
    if path.extension().and_then(|e| e.to_str()) == Some("cast") {
        return probe_cast(path);
    }

    let (size, sha) = digest(path)?;
    let path_str = path.to_string_lossy().to_string();
    let raw = match sh(&[
        "ffprobe",
        "-v",
        "error",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
        "-count_frames",
        &path_str,
    ]) {
        Ok(raw) => raw,
        Err(e) => {
            return Ok(Probe {
                exists: true,
                bytes: Some(size),
                sha256: Some(sha),
                error: Some(format!("{e:#}")),
                ..Default::default()
            })
        }
    };

    let data: Value = serde_json::from_str(&raw)?;
    let stream = data["streams"]
        .as_array()
        .and_then(|a| {
            a.iter()
                .find(|s| s.get("codec_type").and_then(|v| v.as_str()) == Some("video"))
        })
        .cloned();
    let Some(stream) = stream else {
        return Ok(Probe {
            exists: true,
            bytes: Some(size),
            sha256: Some(sha),
            error: Some("no video stream".into()),
            ..Default::default()
        });
    };
    let fmt = data.get("format").cloned().unwrap_or(Value::Null);

    let frames = stream
        .get("nb_read_frames")
        .or_else(|| stream.get("nb_frames"))
        .and_then(|v| match v {
            Value::String(s) => s.parse::<i64>().ok(),
            Value::Number(n) => n.as_i64(),
            _ => None,
        });

    let duration = stream
        .get("duration")
        .or_else(|| fmt.get("duration"))
        .and_then(|v| match v {
            Value::String(s) => s.parse::<f64>().ok(),
            Value::Number(n) => n.as_f64(),
            _ => None,
        })
        .map(round3);
    let mut duration = duration;
    if duration.is_none() {
        if let (Some(frames), Some(rate)) = (frames, avg_frame_rate(&stream)) {
            if rate != 0.0 {
                duration = Some(round3(frames as f64 / rate));
            }
        }
    }

    let mut kind = fmt
        .get("format_name")
        .and_then(|v| v.as_str())
        .and_then(container_kind)
        .map(str::to_string);
    let suffix = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if kind.as_deref().map(|k| k == STILL_KIND).unwrap_or(true) && suffix == "webp" {
        kind = Some(
            if frames.unwrap_or(1) > 1 {
                "animated-webp"
            } else {
                STILL_KIND
            }
            .to_string(),
        );
    }
    if let Some(frames) = frames {
        if frames < MIN_MOTION_FRAMES {
            kind = Some(STILL_KIND.into());
        }
    }

    Ok(Probe {
        exists: true,
        bytes: Some(size),
        sha256: Some(sha),
        kind,
        width: stream.get("width").and_then(|v| v.as_i64()),
        height: stream.get("height").and_then(|v| v.as_i64()),
        duration_seconds: duration,
        frame_count: frames,
        error: None,
    })
}

/// avg_frame_rate "num/den" -> f64 (None for "0/0" or unparseable).
fn avg_frame_rate(stream: &Value) -> Option<f64> {
    let s = stream.get("avg_frame_rate").and_then(|v| v.as_str())?;
    let (num, den) = s.split_once('/')?;
    let num: f64 = num.parse().ok()?;
    let den: f64 = den.parse().ok()?;
    if den == 0.0 {
        return None;
    }
    Some(num / den)
}

// --------------------------------------------------- frame matching (16x16) --

const SAMPLE_FPS: f64 = 2.0;
const SIG_BYTES: usize = 256;

type Signature = Vec<u8>;

fn raw_signatures(args: &[&str]) -> Vec<Signature> {
    let out = Command::new(args[0]).args(&args[1..]).output();
    let Ok(out) = out else {
        return vec![];
    };
    if !out.status.success() {
        return vec![];
    }
    let data = out.stdout;
    (0..)
        .map(|i| i * SIG_BYTES)
        .take_while(|off| off + SIG_BYTES <= data.len())
        .map(|off| data[off..off + SIG_BYTES].to_vec())
        .collect()
}

fn motion_signatures(path: &Path) -> Vec<Signature> {
    let path_str = path.to_string_lossy().to_string();
    raw_signatures(&[
        "ffmpeg",
        "-v",
        "error",
        "-i",
        &path_str,
        "-vf",
        "fps=2,scale=16:16,format=gray",
        "-f",
        "rawvideo",
        "-",
    ])
}

fn still_signature(path: &Path) -> Option<Signature> {
    let path_str = path.to_string_lossy().to_string();
    raw_signatures(&[
        "ffmpeg",
        "-v",
        "error",
        "-i",
        &path_str,
        "-frames:v",
        "1",
        "-vf",
        "scale=16:16,format=gray",
        "-f",
        "rawvideo",
        "-",
    ])
    .into_iter()
    .next()
}

/// Find the timestamp in `motion` whose frame is closest to `state`.
///
/// Deterministic: the motion is decoded once at two frames per second into
/// 16x16 grayscale signatures, the state image is reduced the same way, and the
/// two are compared by mean absolute difference.
fn locate_state_in_motion(state: &Path, motion_frames: &[Signature]) -> Result<Option<Value>> {
    if !state.exists() || state.extension().and_then(|e| e.to_str()) == Some("cast") {
        return Ok(None);
    }
    let Some(target) = still_signature(state) else {
        return Ok(None);
    };
    if motion_frames.is_empty() {
        return Ok(None);
    }
    let mut best: Option<(f64, f64)> = None;
    for (index, sig) in motion_frames.iter().enumerate() {
        let dist = sig
            .iter()
            .zip(target.iter())
            .map(|(a, b)| (*a as i64 - *b as i64).unsigned_abs())
            .sum::<u64>() as f64
            / sig.len() as f64;
        if best.map_or(true, |(_, bd)| dist < bd) {
            best = Some((index as f64 / SAMPLE_FPS, dist));
        }
    }
    let Some((timestamp, diff)) = best else {
        return Ok(None);
    };
    Ok(Some(json!({
        "timestamp_seconds": round3(timestamp),
        "mean_abs_diff": round4(diff),
        "sampled_frames": motion_frames.len(),
    })))
}

// ------------------------------------------------------------- measurement --

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
    }
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

fn round4(x: f64) -> f64 {
    (x * 10000.0).round() / 10000.0
}

/// Python's `{x:g}` formatting (trailing zeros stripped).
fn fmt_g(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e15 {
        return format!("{}", x as i64);
    }
    let mut s = format!("{x}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

/// Python's `f"{value}"` rendering of an optional number inside a message.
fn fmt_opt_num(v: Option<f64>) -> String {
    match v {
        Some(x) => fmt_g(x),
        None => "None".to_string(),
    }
}

fn entry_local_path(entry: &Map<String, Value>) -> String {
    entry.get("local_path").and_then(|v| v.as_str()).unwrap_or("").to_string()
}

/// Frozen measurement date, kept byte-identical with the Python tool.
const TODAY: &str = "2026-08-19";

fn measure_value(data: &mut Value, base: &Path, locate_states: bool) -> Result<Vec<String>> {
    let mut gaps: Vec<String> = Vec::new();

    if let Some(obj) = data.as_object_mut() {
        obj.insert("schema".into(), json!(RECORD_SCHEMA));
    }

    // ---- motion ----
    // First non-still measured motion becomes the anchor for state matching.
    let mut primary_motion: Option<PathBuf> = None;
    let motion_len = data
        .get("motion")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    for idx in 0..motion_len {
        let entry = data
            .get_mut("motion")
            .and_then(|v| v.as_array_mut())
            .and_then(|a| a.get_mut(idx))
            .context("motion entry vanished")?;
        let Some(obj) = entry.as_object_mut() else {
            continue;
        };
        let local_path = entry_local_path(obj);
        let local = base.join(&local_path);
        let probe = probe_media(&local)?;
        let declared_kind = obj.get("media_kind").and_then(|v| v.as_str()).map(str::to_string);
        let canonical = canonical_motion_kind(declared_kind.as_deref());
        obj.insert(
            "declared_media_kind".into(),
            declared_kind.clone().map(Value::String).unwrap_or(Value::Null),
        );
        if !probe.exists {
            obj.insert(
                "media_kind".into(),
                json!(canonical.clone().unwrap_or_else(|| "missing".into())),
            );
            obj.insert("measured".into(), json!(false));
            gaps.push(format!("motion file missing: {local_path}"));
            continue;
        }
        let kind = probe
            .kind
            .clone()
            .or(canonical)
            .unwrap_or_else(|| "unknown".into());
        let is_still = kind == STILL_KIND;
        if truthy(obj.get("sha256"))
            && probe.sha256.as_deref() != obj.get("sha256").and_then(|v| v.as_str())
        {
            gaps.push(format!("motion sha256 mismatch: {local_path}"));
        }
        obj.insert(
            "sha256".into(),
            probe.sha256.clone().map(Value::String).unwrap_or(Value::Null),
        );
        obj.insert("bytes".into(), probe.bytes.map(|b| json!(b)).unwrap_or(Value::Null));
        obj.insert("width".into(), probe.width.map(|w| json!(w)).unwrap_or(Value::Null));
        obj.insert("height".into(), probe.height.map(|h| json!(h)).unwrap_or(Value::Null));
        obj.insert(
            "duration_seconds".into(),
            probe.duration_seconds.map(|d| json!(d)).unwrap_or(Value::Null),
        );
        obj.insert(
            "frame_count".into(),
            probe.frame_count.map(|f| json!(f)).unwrap_or(Value::Null),
        );
        obj.insert(
            "measurement_method".into(),
            json!(if kind == "terminal-cast" {
                "asciinema-v2 header and event stream"
            } else {
                "ffprobe -count_frames"
            }),
        );
        if let Some(err) = &probe.error {
            gaps.push(format!("motion probe error ({local_path}): {err}"));
        }
        if is_still {
            gaps.push(format!("motion asset is a still image: {local_path}"));
        }
        if probe.duration_seconds.unwrap_or(0.0) < MIN_MOTION_SECONDS && !is_still {
            gaps.push(format!(
                "motion shorter than {}: {local_path} ({})",
                fmt_g(MIN_MOTION_SECONDS),
                fmt_opt_num(probe.duration_seconds)
            ));
        }
        let capture_method = obj
            .get("capture_method")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let provenance = classify_provenance(capture_method.as_deref(), Some(kind.as_str()));
        obj.insert("provenance_class".into(), json!(provenance));
        if provenance == "unclassified" {
            gaps.push(format!("motion provenance unclassified: {local_path}"));
        }
        if !is_still && primary_motion.is_none() {
            primary_motion = Some(local);
        }
    }

    // Lazy decode of the primary motion for the state-frame search (the Python
    // tool memoized this globally; per-record is equivalent because every state
    // of a record is matched against the same primary motion).
    let mut motion_frames_cache: Option<Vec<Signature>> = None;

    // ---- states ----
    let states_len = data
        .get("states")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    for idx in 0..states_len {
        let entry = data
            .get_mut("states")
            .and_then(|v| v.as_array_mut())
            .and_then(|a| a.get_mut(idx))
            .context("state entry vanished")?;
        let Some(obj) = entry.as_object_mut() else {
            continue;
        };
        let local_path = entry_local_path(obj);
        let local = base.join(&local_path);
        let probe = probe_media(&local)?;
        if !probe.exists {
            gaps.push(format!("state file missing: {local_path}"));
            continue;
        }
        if truthy(obj.get("sha256"))
            && probe.sha256.as_deref() != obj.get("sha256").and_then(|v| v.as_str())
        {
            gaps.push(format!("state sha256 mismatch: {local_path}"));
        }
        obj.insert(
            "sha256".into(),
            probe.sha256.clone().map(Value::String).unwrap_or(Value::Null),
        );
        obj.insert("bytes".into(), probe.bytes.map(|b| json!(b)).unwrap_or(Value::Null));
        obj.insert("width".into(), probe.width.map(|w| json!(w)).unwrap_or(Value::Null));
        obj.insert("height".into(), probe.height.map(|h| json!(h)).unwrap_or(Value::Null));

        // v1 records already used `name` and `source_motion_path`. A failed v2
        // migration introduced parallel `state_name` / `source_relationship`
        // fields and then called every original record unnamed. Clean cutover:
        // retain one vocabulary and consume the aliases if an observer wrote one.
        if let Some(alias_name) = obj.remove("state_name") {
            if truthy(Some(&alias_name)) {
                obj.insert("name".into(), alias_name);
            }
        }
        if !truthy(obj.get("name")) {
            gaps.push(format!("state unnamed: {local_path}"));
        }
        if let Some(alias_relationship) = obj.remove("source_relationship") {
            obj.insert("source_motion_path".into(), alias_relationship);
        }

        if locate_states {
            let has_primary = primary_motion.is_some();
            if has_primary && motion_frames_cache.is_none() {
                motion_frames_cache = Some(motion_signatures(primary_motion.as_ref().unwrap()));
            }
            if let Some(frames) = motion_frames_cache.as_ref() {
                if let Some(m) = locate_state_in_motion(&local, frames)? {
                    let rel = local
                        .strip_prefix(base)
                        .unwrap_or(&local)
                        .to_string_lossy()
                        .to_string();
                    let diff = m["mean_abs_diff"].as_f64().unwrap_or(f64::INFINITY);
                    let ts = m["timestamp_seconds"].as_f64().unwrap_or(0.0);
                    let mut source_match = m.clone();
                    if let Some(sm) = source_match.as_object_mut() {
                        sm.insert("motion_path".into(), json!(rel));
                        sm.insert(
                            "method".into(),
                            json!("16x16 grayscale mean-absolute-difference frame search"),
                        );
                    }
                    obj.insert("source_match".into(), source_match);
                    if !truthy(obj.get("source_motion_path")) && diff <= STATE_MATCH_MAX_DIFF {
                        obj.insert(
                            "source_motion_path".into(),
                            json!(format!(
                                "frame of {rel} at {}s (mean abs diff {}/255)",
                                fmt_g(ts),
                                fmt_g(diff)
                            )),
                        );
                    }
                }
            }
        }
        if !truthy(obj.get("source_motion_path")) {
            gaps.push(format!("state source relationship unproven: {local_path}"));
        }
    }

    let states_count = data
        .get("states")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if states_count < MIN_STATES {
        gaps.push(format!("fewer than {MIN_STATES} states"));
    }

    // ---- journey ----
    let journey = data.get("journey").cloned().unwrap_or(Value::Null);
    let steps_len = journey
        .get("steps")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if steps_len < MIN_JOURNEY_STEPS {
        gaps.push(format!(
            "journey exposes fewer than {MIN_JOURNEY_STEPS} observed steps"
        ));
    }
    for key in [
        "actor",
        "goal",
        "prerequisites",
        "failure_route",
        "recovery_route",
        "completion_evidence",
    ] {
        if !truthy(journey.get(key)) {
            gaps.push(format!("journey missing {key}"));
        }
    }

    // ---- interactions ----
    let interactions = data
        .get("interactions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if interactions.len() < MIN_INTERACTIONS {
        gaps.push(format!("fewer than {MIN_INTERACTIONS} mapped interactions"));
    }
    for item in &interactions {
        let missing: Vec<&str> = INTERACTION_FIELDS
            .iter()
            .copied()
            .filter(|f| !truthy(item.get(f)))
            .collect();
        if !missing.is_empty() {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            gaps.push(format!("interaction '{name}' missing {}", missing.join(", ")));
            break;
        }
    }

    // ---- motion analysis ----
    match data.get_mut("motion_analysis") {
        None | Some(Value::Null) => gaps.push("motion analysis absent".to_string()),
        Some(analysis) => {
            let was_array = analysis.is_array();
            let mut entries: Vec<Value> = match analysis.take() {
                Value::Array(a) => a,
                other => vec![other],
            };
            'items: for item in entries.iter_mut() {
                let Some(obj) = item.as_object_mut() else {
                    continue;
                };
                for (alias, canonical) in MOTION_ANALYSIS_ALIASES {
                    if let Some(alias_value) = obj.remove(*alias) {
                        obj.entry(canonical.to_string())
                            .or_insert(alias_value);
                    }
                }
                let declared_timing = obj
                    .get("timing_class")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let canonical_timing = canonical_timing_class(declared_timing.as_deref());
                if let (Some(declared), Some(_)) = (&declared_timing, &canonical_timing) {
                    if declared.as_str() != canonical_timing.as_deref().unwrap() {
                        obj.entry("timing_description".to_string())
                            .or_insert(json!(declared));
                    }
                }
                obj.insert(
                    "timing_class".into(),
                    canonical_timing
                        .as_ref()
                        .map(|c| json!(c))
                        .unwrap_or(Value::Null),
                );
                if declared_timing.is_some() && canonical_timing.is_none() {
                    gaps.push(format!(
                        "motion analysis timing class unrecognized: {}",
                        declared_timing.unwrap()
                    ));
                }
                for key in MOTION_ANALYSIS_FIELDS {
                    obj.entry(key.to_string()).or_insert(Value::Null);
                }
                let known: BTreeSet<&str> = MOTION_ANALYSIS_FIELDS
                    .iter()
                    .chain(MOTION_ANALYSIS_OPTIONAL.iter())
                    .copied()
                    .collect();
                let mut unknown: Vec<String> = obj
                    .keys()
                    .filter(|k| !known.contains(k.as_str()))
                    .cloned()
                    .collect();
                if !unknown.is_empty() {
                    unknown.sort();
                    gaps.push(format!(
                        "motion analysis carries unknown fields {}",
                        unknown.join(", ")
                    ));
                }
                let missing: Vec<&str> = MOTION_ANALYSIS_FIELDS
                    .iter()
                    .copied()
                    .filter(|f| !truthy(obj.get(*f)))
                    .collect();
                if !missing.is_empty() {
                    gaps.push(format!("motion analysis missing {}", missing.join(", ")));
                    break 'items;
                }
            }
            *analysis = if was_array {
                Value::Array(entries)
            } else {
                entries.into_iter().next().unwrap_or(Value::Null)
            };
        }
    }

    // ---- accessibility ----
    let access = data.get("accessibility").cloned().unwrap_or(Value::Null);
    let observations = access
        .get("observations")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if observations < MIN_ACCESSIBILITY_OBSERVATIONS {
        gaps.push("fewer than three accessibility observations".to_string());
    }
    if !truthy(access.get("measured")) {
        gaps.push("accessibility never measured against the product".to_string());
    }

    // ---- provenance rollup ----
    let mut classes: BTreeSet<String> = BTreeSet::new();
    if let Some(entries) = data.get("motion").and_then(|v| v.as_array()) {
        for e in entries {
            if truthy(e.get("measured")) {
                if let Some(p) = e.get("provenance_class").and_then(|v| v.as_str()) {
                    classes.insert(p.to_string());
                }
            }
        }
    }
    if classes.is_empty() {
        gaps.push("no measured motion evidence".to_string());
    }
    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "motion_provenance".into(),
            Value::Array(classes.into_iter().map(Value::String).collect()),
        );
        obj.insert(
            "evidence_gaps".into(),
            Value::Array(gaps.iter().cloned().map(Value::String).collect()),
        );
        obj.insert(
            "evidence_status".into(),
            json!(if gaps.is_empty() { "complete" } else { "partial" }),
        );
        obj.insert("measured_at".into(), json!(TODAY));
    }

    Ok(gaps)
}

fn catalogs(selected: Option<&str>) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(".")
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| {
            let p = e.path();
            p.strip_prefix(".").unwrap_or(p.as_path()).to_path_buf()
        })
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .map(|n| n.to_string_lossy().ends_with("-examples"))
                    .unwrap_or(false)
                && p.join("references").is_dir()
        })
        .collect();
    found.sort();
    if let Some(sel) = selected {
        found.retain(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy() == sel)
                .unwrap_or(false)
        });
    }
    found
}

fn records_in(catalog: &Path) -> Vec<PathBuf> {
    let mut records: Vec<PathBuf> = std::fs::read_dir(catalog.join("references"))
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("reference.json"))
        .filter(|p| p.is_file())
        .collect();
    records.sort();
    records
}

fn gap_key(gap: &str) -> String {
    match gap.find([':', '(']) {
        Some(pos) => gap[..pos].trim().to_string(),
        None => gap.trim().to_string(),
    }
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut apply = false;
    let mut no_state_match = false;
    let mut jobs: usize = 8;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--catalog" => {
                i += 1;
                catalog = Some(rest.get(i).context("--catalog needs a value")?.clone());
            }
            "--apply" => apply = true,
            "--no-state-match" => no_state_match = true,
            "--jobs" => {
                i += 1;
                jobs = rest
                    .get(i)
                    .context("--jobs needs a value")?
                    .parse()
                    .context("--jobs expects an integer")?;
            }
            other => anyhow::bail!("unknown argument: {other}"),
        }
        i += 1;
    }
    let locate_states = !no_state_match;

    if !apply {
        println!("dry run: records are measured and reported, nothing is written");
    }


    let mut total = 0usize;
    let mut complete = 0usize;
    let mut gap_counter: Vec<(String, usize)> = Vec::new();

    for cat in catalogs(catalog.as_deref()) {
        let records = records_in(&cat);
        let results: parking_lot::Mutex<Vec<(PathBuf, Vec<String>)>> =
            parking_lot::Mutex::new(Vec::new());
        let next = AtomicUsize::new(0);
        let workers = jobs.max(1).min(records.len().max(1));
        std::thread::scope(|s| {
            for _ in 0..workers {
                s.spawn(|| loop {
                    let idx = next.fetch_add(1, Ordering::SeqCst);
                    if idx >= records.len() {
                        break;
                    }
                    let path = &records[idx];
                    let outcome = if apply {
                        measure_apply(path, locate_states)
                    } else {
                        measure_dry(path, locate_states)
                    };
                    match outcome {
                        Ok(gaps) => results.lock().push((path.clone(), gaps)),
                        Err(e) => {
                            eprintln!("verify-reference-evidence: {}: {e:#}", path.display())
                        }
                    }
                });
            }
        });
        let mut results = results.into_inner();
        results.sort_by(|a, b| a.0.cmp(&b.0));

        let cat_complete = results.iter().filter(|(_, gaps)| gaps.is_empty()).count();
        total += results.len();
        complete += cat_complete;
        for (_, gaps) in &results {
            for gap in gaps {
                let key = gap_key(gap);
                if let Some(slot) = gap_counter.iter_mut().find(|(k, _)| *k == key) {
                    slot.1 += 1;
                } else {
                    gap_counter.push((key, 1));
                }
            }
        }
        println!(
            "{}: {cat_complete}/{} complete",
            cat.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            results.len()
        );

        let index = cat.join("references.json");
        if apply && index.exists() {
            let mut payload: Value = lib::read_json(index.to_str().context("non-UTF8 path")?)?;
            let by_path: HashMap<String, &Vec<String>> = results
                .iter()
                .map(|(path, gaps)| {
                    (
                        format!(
                            "references/{}/reference.json",
                            path.parent()
                                .and_then(|p| p.file_name())
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default()
                        ),
                        gaps,
                    )
                })
                .collect();
            if let Some(refs) = payload.get_mut("references").and_then(|v| v.as_array_mut()) {
                for r in refs.iter_mut() {
                    let Some(key) = r.get("path").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(gaps) = by_path.get(key) else {
                        continue;
                    };
                    if let Some(obj) = r.as_object_mut() {
                        obj.insert(
                            "evidence_status".into(),
                            json!(if gaps.is_empty() { "complete" } else { "partial" }),
                        );
                        obj.insert("evidence_gap_count".into(), json!(gaps.len()));
                    }
                }
            }
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("schema".into(), json!(INDEX_SCHEMA));
                obj.insert("measured_at".into(), json!(TODAY));
                obj.insert("complete_count".into(), json!(cat_complete));
                obj.insert("partial_count".into(), json!(results.len() - cat_complete));
            }
            std::fs::write(&index, serde_json::to_string_pretty(&payload)? + "\n")?;
        }
    }

    println!(
        "\nmeasured {total} records, {complete} complete, {} partial",
        total - complete
    );
    let mut ranked = gap_counter;
    ranked.sort_by(|a, b| b.1.cmp(&a.1)); // stable: ties keep first-seen order
    for (key, count) in ranked {
        println!("{count:5}  {key}");
    }
    Ok(())
}

/// Apply mode: measure in place and rewrite the record file (always rewritten,
/// mirroring the Python tool).
fn measure_apply(path: &Path, locate_states: bool) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut data: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let base = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let gaps = measure_value(&mut data, &base, locate_states)?;
    std::fs::write(path, serde_json::to_string_pretty(&data)? + "\n")?;
    Ok(gaps)
}

/// Dry run: measure an in-memory copy of the record; nothing is written.
fn measure_dry(path: &Path, locate_states: bool) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut data: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let base = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let gaps = measure_value(&mut data, &base, locate_states)?;
    Ok(gaps)
}
