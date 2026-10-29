//! Where each state image lies in the record's primary motion. The motion is
//! decoded once, every frame it holds at its own resolution, and each frame
//! is placed at the timestamp ffprobe reports for it; each state image is
//! scaled to that same resolution, and its place is the frame with the least
//! mean absolute luminance difference. How far a state may lie from that
//! frame is measured from the motion rather than stated: the mean difference
//! between its adjacent frames. A state no further from its best frame than
//! the motion's own frames are from each other is that frame; a still motion,
//! whose frames do not drift, proves only an identical state.

use super::*;
use std::io::Read;
use std::process::Stdio;

/// The motion frame closest to one state image.
pub(super) struct StateMatch {
    pub(super) timestamp_seconds: f64,
    pub(super) mean_abs_diff: f64,
}

/// What one pass over the motion found.
pub(super) struct MotionSearch {
    /// For each state asked about, in that order: its closest frame, or why
    /// it could not be compared.
    pub(super) matches: Vec<Result<StateMatch, String>>,
    /// The mean difference between adjacent frames; nothing when the motion
    /// holds fewer than two, since one frame has no neighbour to measure.
    pub(super) drift: Option<f64>,
    /// How many frames the motion holds, every one of them compared.
    pub(super) frames: usize,
}

/// The mean absolute difference of two equally sized luminance frames, per
/// pixel: the one distance states and frames are compared by.
fn mean_abs_diff(left: &[u8], right: &[u8]) -> f64 {
    let total: u64 = left
        .iter()
        .zip(right)
        .map(|(a, b)| u64::from(a.abs_diff(*b)))
        .sum();
    total as f64 / left.len() as f64
}

/// The motion's frame size and the presentation time of every frame, as
/// ffprobe reads them from the first video stream.
fn geometry(motion: &Path) -> Result<((usize, usize), Vec<f64>)> {
    let shown = motion.to_string_lossy();
    let raw = sh(&[
        "ffprobe",
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height:frame=best_effort_timestamp_time",
        "-of",
        "json",
        &shown,
    ])?;
    let report: Value = serde_json::from_str(&raw)
        .with_context(|| format!("ffprobe's frame report on {shown} is not JSON"))?;
    let stream = report.pointer("/streams/0").with_context(|| format!("ffprobe found no video stream in {shown}"))?;
    let side = |name: &str| {
        stream
            .get(name)
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .with_context(|| format!("ffprobe states no {name} for the video stream of {shown}: {stream}"))
    };
    let size = (side("width")?, side("height")?);
    let frames = report.get("frames").and_then(Value::as_array).with_context(|| format!("ffprobe listed no frames of {shown}"))?;
    let times = frames
        .iter()
        .enumerate()
        .map(|(index, frame)| {
            frame
                .get("best_effort_timestamp_time")
                .and_then(Value::as_str)
                .and_then(|time| time.parse::<f64>().ok())
                .with_context(|| format!("frame {index} of {shown} has no presentation time: {frame}"))
        })
        .collect::<Result<Vec<f64>>>()?;
    Ok((size, times))
}

/// The luminance of the image at `state`, scaled to `width` x `height`.
fn state_luminance(state: &Path, (width, height): (usize, usize)) -> Result<Vec<u8>> {
    let shown = state.to_string_lossy();
    let scale = format!("scale={width}:{height}");
    let pixels = run_bytes(&[
        "ffmpeg", "-v", "error", "-i", &shown, "-frames:v", "1", "-vf", &scale, "-pix_fmt", "gray", "-f",
        "rawvideo", "-",
    ])?;
    if pixels.len() != width * height {
        anyhow::bail!(
            "ffmpeg decoded {} luminance bytes of {shown}, not the {} of a {width}x{height} frame",
            pixels.len(),
            width * height
        );
    }
    Ok(pixels)
}

/// Compare every frame of `motion` with every image of `states`, once.
pub(super) fn search(motion: &Path, states: &[PathBuf]) -> Result<MotionSearch> {
    let shown = motion.to_string_lossy().to_string();
    let (size, times) = geometry(motion)?;
    let targets: Vec<Result<Vec<u8>, String>> = states
        .iter()
        .map(|state| state_luminance(state, size).map_err(|error| format!("{error:#}")))
        .collect();
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-i", &shown, "-map", "0:v:0", "-fps_mode", "passthrough"])
        .args(["-pix_fmt", "gray", "-f", "rawvideo", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("ffmpeg could not start to decode {shown}"))?;
    let mut stderr = child.stderr.take().context("ffmpeg's standard error was not piped")?;
    let stderr_reader = std::thread::spawn(move || {
        let mut text = String::new();
        stderr.read_to_string(&mut text).map(|_| text)
    });
    let mut stdout = child.stdout.take().context("ffmpeg's standard output was not piped")?;
    let waiting = stado_wait::begin(
        stado_wait::Kind::Process,
        format!("ffmpeg decoding every frame of {shown}"),
        format!("pid {}", child.id()),
    );
    let frame_len = (size.0 * size.1) as u64;
    let mut best: Vec<Option<(usize, f64)>> = targets.iter().map(|_| None).collect();
    let mut drifts: Vec<f64> = Vec::new();
    let mut previous: Option<Vec<u8>> = None;
    let mut frame: Vec<u8> = Vec::new();
    let mut frames = 0;
    let read = loop {
        frame.clear();
        match (&mut stdout).take(frame_len).read_to_end(&mut frame) {
            Ok(read) if read as u64 == frame_len => {}
            Ok(read) if frame.is_empty() => break Ok(read),
            Ok(read) => break Err(format!("the stream ended inside frame {frames} after {read} of its {frame_len} bytes")),
            Err(error) => break Err(format!("frame {frames} could not be read: {error}")),
        }
        for (target, found) in targets.iter().zip(best.iter_mut()) {
            if let Ok(target) = target {
                let distance = mean_abs_diff(&frame, target);
                if found.is_none_or(|(_, closest)| distance < closest) {
                    *found = Some((frames, distance));
                }
            }
        }
        if let Some(earlier) = &previous {
            drifts.push(mean_abs_diff(earlier, &frame));
        }
        frames += 1;
        frame = previous.replace(std::mem::take(&mut frame)).unwrap_or_default();
    };
    // Closing the pipe ends a decoder this read stopped listening to.
    drop(stdout);
    let status = waiting.settle(child.wait()).with_context(|| format!("waiting for ffmpeg decoding {shown} failed"))?;
    let complaint = match stderr_reader.join() {
        Ok(Ok(text)) => text.trim().to_string(),
        Ok(Err(error)) => format!("its standard error could not be read: {error}"),
        Err(_) => "its standard error reader panicked".to_string(),
    };
    if !status.success() {
        anyhow::bail!("ffmpeg could not decode {shown} ({status}): {complaint}");
    }
    if let Err(cut) = read {
        anyhow::bail!("ffmpeg's frames of {shown} could not be read whole: {cut}");
    }
    if frames != times.len() {
        anyhow::bail!("ffmpeg decoded {frames} frames of {shown}, but ffprobe lists {} presentation times", times.len());
    }
    let matches = targets
        .into_iter()
        .zip(best)
        .map(|(target, found)| {
            target?;
            let (index, distance) = found.ok_or_else(|| format!("{shown} holds no frame to compare with"))?;
            Ok(StateMatch { timestamp_seconds: times[index], mean_abs_diff: distance })
        })
        .collect();
    let drift = (!drifts.is_empty()).then(|| drifts.iter().sum::<f64>() / drifts.len() as f64);
    Ok(MotionSearch { matches, drift, frames })
}
