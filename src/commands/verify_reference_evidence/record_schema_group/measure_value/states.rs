use super::*;
use std::num::NonZeroUsize;

/// Measure every state entry in place and, when asked, locate each one as a frame of the primary motion.
pub(super) fn measure_states(
    data: &mut Value,
    base: &Path,
    locate_states: bool,
    requirements: &crate::commands::reference_contract::CompletenessRequirements,
    provenance_context: &ProvenanceContext,
    primary_motion: Option<PathBuf>,
    gaps: &mut Vec<String>,
) -> Result<()> {
    // Lazy decode of the primary motion for the state-frame search (the Python
    // tool memoized this globally; per-record is equivalent because every state
    // of a record is matched against the same primary motion).
    let mut motion_frames_cache: Option<Vec<Signature>> = None;

    // ---- states ----
    // A record without a states list holds no states.
    let positions: Vec<usize> = match data.get("states").and_then(Value::as_array) {
        Some(states) => states.iter().enumerate().map(|(index, _)| index).collect(),
        None => Vec::new(),
    };
    let states_count = positions.len();
    // States are named for people from one, in the order the record lists them.
    for (idx, ordinal) in positions.into_iter().zip(NonZeroUsize::MIN.get()..) {
        let entry = data
            .get_mut("states")
            .and_then(|v| v.as_array_mut())
            .and_then(|a| a.get_mut(idx))
            .context("state entry vanished")?;
        let semantic_name_verified = observation_supported(entry, &provenance_context);
        let Some(obj) = entry.as_object_mut() else {
            continue;
        };
        let Some(local_path) = entry_local_path(obj) else {
            gaps.push(format!("states[{idx}] names no local_path"));
            continue;
        };
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
        // A measurement the probe did not produce is recorded as null.
        obj.insert("sha256".into(), json!(probe.sha256));
        obj.insert("bytes".into(), json!(probe.bytes));
        obj.insert("width".into(), json!(probe.width));
        obj.insert("height".into(), json!(probe.height));

        obj.remove("state_name");
        obj.remove("source_relationship");
        obj.remove("source_motion_path");
        obj.remove("source_match");
        if !truthy(obj.get("name")) {
            obj.insert("name".into(), json!(generic_state_name(ordinal)));
        }
        if !semantic_name_verified {
            gaps.push(format!(
                "state semantic name unverified: {local_path}; pixels and digest prove only an observed frame"
            ));
        }

        if locate_states {
            if primary_motion.is_some() && motion_frames_cache.is_none() {
                motion_frames_cache = Some(motion_signatures(primary_motion.as_ref().unwrap()));
            }
            if let (Some(frames), Some(motion)) =
                (motion_frames_cache.as_ref(), primary_motion.as_ref())
            {
                if let Some(matched) = locate_state_in_motion(&local, frames)? {
                    let diff = round4(matched.mean_abs_diff);
                    let tolerance = super::drift::frame_drift(frames);
                    if tolerance.is_some_and(|tolerance| diff <= tolerance) {
                        // A motion outside the record's directory is named by
                        // its full path.
                        let shown = match motion.strip_prefix(base) {
                            Ok(relative) => relative,
                            Err(_outside) => motion.as_path(),
                        };
                        let rel = shown.to_string_lossy().to_string();
                        let ts = round3(matched.timestamp_seconds);
                        let mut source_match = matched.record();
                        if let Some(details) = source_match.as_object_mut() {
                            details.insert("motion_path".into(), json!(rel));
                            details.insert(
                                "method".into(),
                                json!("16x16 grayscale mean-absolute-difference frame search"),
                            );
                            details.insert("max_mean_abs_diff".into(), json!(tolerance));
                            details.insert(
                                "tolerance".into(),
                                json!(
                                    "mean difference between the motion's adjacent sampled frames"
                                ),
                            );
                        }
                        obj.insert("source_match".into(), source_match);
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
            gaps.push(format!(
                "state source relationship unproven: {local_path}; no threshold match against independently verified motion"
            ));
        }
    }

    if states_count < requirements.min_states {
        gaps.push(format!(
            "fewer than {} states for {} profile",
            requirements.min_states, requirements.profile
        ));
    }
    Ok(())
}
