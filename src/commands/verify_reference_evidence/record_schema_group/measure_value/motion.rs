use super::*;
use crate::commands::reference_contract::Provenance;

/// Measure every motion entry in place and return the first non-still, verified motion: the
/// anchor the states are matched against.
pub(super) fn measure_motion(
    data: &mut Value,
    base: &Path,
    requirements: &crate::commands::reference_contract::CompletenessRequirements,
    provenance_context: &ProvenanceContext,
    gaps: &mut Vec<String>,
) -> Result<Option<PathBuf>> {
    // First non-still measured motion becomes the anchor for state matching.
    let mut primary_motion: Option<PathBuf> = None;
    // A record without motion has nothing to measure and no anchor.
    let positions: Vec<usize> = match data.get("motion").and_then(Value::as_array) {
        Some(motion) => motion.iter().enumerate().map(|(index, _)| index).collect(),
        None => return Ok(None),
    };
    for idx in positions {
        let entry = data
            .get_mut("motion")
            .and_then(|v| v.as_array_mut())
            .and_then(|a| a.get_mut(idx))
            .context("motion entry vanished")?;
        let Some(obj) = entry.as_object_mut() else {
            continue;
        };
        let Some(local_path) = entry_local_path(obj) else {
            gaps.push(format!("motion[{idx}] names no local_path"));
            continue;
        };
        let local = base.join(&local_path);
        let probe = probe_media(&local)?;
        let declared_kind = obj
            .get("media_kind")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let canonical = canonical_motion_kind(declared_kind.as_deref());
        obj.insert("declared_media_kind".into(), json!(declared_kind));
        if !probe.exists {
            // The file is absent: its kind is what the record declared, or
            // none; "missing" is not a media kind, the gap below says it.
            obj.insert("media_kind".into(), json!(canonical));
            obj.insert("measured".into(), json!(false));
            gaps.push(format!("motion file missing: {local_path}"));
            continue;
        }
        // The probed kind, else the declared one; neither is no kind at all.
        let kind = probe.kind.clone().or(canonical);
        let is_still = kind.as_deref() == Some(STILL_KIND);
        if truthy(obj.get("sha256"))
            && probe.sha256.as_deref() != obj.get("sha256").and_then(|v| v.as_str())
        {
            gaps.push(format!("motion sha256 mismatch: {local_path}"));
        }
        // A measurement the probe did not produce is recorded as null.
        obj.insert("sha256".into(), json!(probe.sha256));
        obj.insert("bytes".into(), json!(probe.bytes));
        obj.insert("width".into(), json!(probe.width));
        obj.insert("height".into(), json!(probe.height));
        obj.insert("duration_seconds".into(), json!(probe.duration_seconds));
        obj.insert("frame_count".into(), json!(probe.frame_count));
        obj.insert(
            "measurement_method".into(),
            json!(if kind.as_deref() == Some("terminal-cast") {
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
        match probe.duration_seconds {
            _ if is_still => {}
            None => gaps.push(format!("motion duration was not measured: {local_path}")),
            Some(duration) if duration < requirements.min_motion_seconds => gaps.push(format!(
                "motion shorter than {} for {} profile: {local_path} ({})",
                fmt_g(requirements.min_motion_seconds),
                requirements.profile,
                fmt_g(duration)
            )),
            Some(_) => {}
        }
        let value = Value::Object(obj.clone());
        obj.insert(
            "provenance_class".into(),
            json!(provenance_class(&value, &provenance_context)),
        );
        let (provenance, unverified_because) = provenance_context.verified.provenance(&value);
        let verified = provenance != Provenance::UnverifiedSourceMedia;
        if let Some(reason) = unverified_because {
            gaps.push(format!(
                "motion provenance unverified: {local_path} has no typed verified crawl or owner observation: {reason}"
            ));
        }
        if !is_still && verified && primary_motion.is_none() {
            primary_motion = Some(local);
        }
    }
    Ok(primary_motion)
}
