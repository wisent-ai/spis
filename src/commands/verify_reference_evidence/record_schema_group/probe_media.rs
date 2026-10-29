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
        });
    let mut duration = duration;
    if duration.is_none() {
        if let (Some(frames), Some(rate)) = (frames, avg_frame_rate(&stream)) {
            if rate != 0.0 {
                duration = Some(frames as f64 / rate);
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
