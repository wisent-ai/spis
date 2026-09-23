use super::*;

pub(crate) fn probe_media(path: &Path) -> Result<Probe> {
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
pub(crate) fn avg_frame_rate(stream: &Value) -> Option<f64> {
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

pub(crate) const SAMPLE_FPS: f64 = 2.0;

pub(crate) const SIG_BYTES: usize = 256;

pub(crate) type Signature = Vec<u8>;

pub(crate) fn raw_signatures(args: &[&str]) -> Vec<Signature> {
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

pub(crate) fn motion_signatures(path: &Path) -> Vec<Signature> {
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

pub(crate) fn still_signature(path: &Path) -> Option<Signature> {
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
pub(crate) fn locate_state_in_motion(state: &Path, motion_frames: &[Signature]) -> Result<Option<Value>> {
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

pub(crate) fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
    }
}

pub(crate) fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub(crate) fn round4(x: f64) -> f64 {
    (x * 10000.0).round() / 10000.0
}

/// Python's `{x:g}` formatting (trailing zeros stripped).
pub(crate) fn fmt_g(x: f64) -> String {
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
pub(crate) fn fmt_opt_num(v: Option<f64>) -> String {
    match v {
        Some(x) => fmt_g(x),
        None => "None".to_string(),
    }
}
