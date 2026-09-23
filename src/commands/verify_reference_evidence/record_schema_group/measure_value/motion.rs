use super::*;

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
        let declared_kind = obj
            .get("media_kind")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let canonical = canonical_motion_kind(declared_kind.as_deref());
        obj.insert(
            "declared_media_kind".into(),
            declared_kind
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
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
            probe
                .sha256
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        obj.insert(
            "bytes".into(),
            probe.bytes.map(|b| json!(b)).unwrap_or(Value::Null),
        );
        obj.insert(
            "width".into(),
            probe.width.map(|w| json!(w)).unwrap_or(Value::Null),
        );
        obj.insert(
            "height".into(),
            probe.height.map(|h| json!(h)).unwrap_or(Value::Null),
        );
        obj.insert(
            "duration_seconds".into(),
            probe
                .duration_seconds
                .map(|d| json!(d))
                .unwrap_or(Value::Null),
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
        if probe.duration_seconds.unwrap_or(0.0) < requirements.min_motion_seconds && !is_still {
            gaps.push(format!(
                "motion shorter than {} for {} profile: {local_path} ({})",
                fmt_g(requirements.min_motion_seconds),
                requirements.profile,
                fmt_opt_num(probe.duration_seconds)
            ));
        }
        let provenance = provenance_class(
            &Value::Object(obj.clone()),
            &provenance_context,
        );
        obj.insert("provenance_class".into(), json!(provenance));
        if provenance == "unverified-source-media" {
            gaps.push(format!(
                "motion provenance unverified: {local_path} has no typed verified crawl or owner observation"
            ));
        }
        if !is_still
            && provenance != "unverified-source-media"
            && primary_motion.is_none()
        {
            primary_motion = Some(local);
        }
    }
    Ok(primary_motion)
}
