use super::*;

/// A state whose image exists, waiting to be placed in the primary motion.
struct Present {
    position: usize,
    local: PathBuf,
    local_path: String,
    /// Where in the gaps this state's unproven relationship belongs: right
    /// after the state's own other gaps, as the record lists its states.
    gap_at: usize,
}

/// Measure every state entry in place and, when asked, locate each one as a frame of the primary motion.
pub(super) fn measure_states(
    data: &mut Value,
    base: &Path,
    locate_states: bool,
    requirements: &crate::commands::reference_contract::CompletenessRequirements,
    provenance_context: &ProvenanceContext,
    primary_motion: Option<motion::PrimaryMotion>,
    gaps: &mut Vec<String>,
) -> Result<()> {
    // ---- states ----
    // A record without a states list holds no states.
    let positions: Vec<usize> = match data.get("states").and_then(Value::as_array) {
        Some(states) => states.iter().enumerate().map(|(index, _)| index).collect(),
        None => Vec::new(),
    };
    let states_count = positions.len();
    // States are named for people in the order the record lists them, by
    // their catalog ordinal (`reference_contract::ordinals`).
    let mut present: Vec<Present> = Vec::new();
    for (idx, ordinal) in positions.into_iter().zip(crate::commands::reference_contract::ordinals()) {
        let entry = state_entry(data, idx)?;
        let semantic_name_verified = observation_supported(entry, provenance_context);
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
        if crate::commands::reference_contract::stated(obj.get("sha256"))
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
        if !crate::commands::reference_contract::stated(obj.get("name")) {
            obj.insert("name".into(), json!(generic_state_name(ordinal)));
        }
        if !semantic_name_verified {
            gaps.push(format!(
                "state semantic name unverified: {local_path}; pixels and digest prove only an observed frame"
            ));
        }
        present.push(Present { position: idx, local, local_path, gap_at: gaps.len() });
    }

    let located = locate(data, base, locate_states, primary_motion.as_ref(), &present)?;
    // Inserted from the last state back, so each earlier place still holds.
    for (state, unproven) in present.iter().zip(located).rev() {
        if let Some(reason) = unproven {
            gaps.insert(
                state.gap_at,
                format!("state source relationship unproven: {}; {reason}", state.local_path),
            );
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

fn state_entry(data: &mut Value, idx: usize) -> Result<&mut Value> {
    data.get_mut("states")
        .and_then(|v| v.as_array_mut())
        .and_then(|a| a.get_mut(idx))
        .context("state entry vanished")
}

/// Place every present state in the primary motion, writing the proven ones'
/// `source_match` and `source_motion_path`; for each state, why it is not
/// proven, or nothing when it is.
fn locate(
    data: &mut Value,
    base: &Path,
    locate_states: bool,
    primary_motion: Option<&motion::PrimaryMotion>,
    present: &[Present],
) -> Result<Vec<Option<String>>> {
    let unproven = |reason: String| -> Vec<Option<String>> { present.iter().map(|_| Some(reason.clone())).collect() };
    let Some(motion) = primary_motion.filter(|_| locate_states) else {
        return Ok(unproven("no threshold match against independently verified motion".to_string()));
    };
    // A motion outside the record's directory is named by its full path.
    let shown = match motion.path.strip_prefix(base) {
        Ok(relative) => relative,
        Err(_outside) => motion.path.as_path(),
    };
    let rel = shown.to_string_lossy().to_string();
    if motion.terminal_cast() {
        return Ok(unproven(format!(
            "the primary motion {rel} is declared a terminal cast, whose events are text, not frames a state image can be found in"
        )));
    }
    let images: Vec<PathBuf> = present.iter().map(|state| state.local.clone()).collect();
    let search = match frame_search::search(&motion.path, &images) {
        Ok(search) => search,
        Err(error) => return Ok(unproven(format!("the primary motion {rel} could not be searched: {error:#}"))),
    };
    let mut reasons = Vec::new();
    for (state, matched) in present.iter().zip(search.matches) {
        let matched = match matched {
            Ok(matched) => matched,
            Err(cause) => {
                reasons.push(Some(format!("it could not be compared with {rel}: {cause}")));
                continue;
            }
        };
        let Some(tolerance) = search.drift.filter(|tolerance| matched.mean_abs_diff <= *tolerance) else {
            reasons.push(Some(format!(
                "its closest frame of {rel}, at {}s, differs by {}/255 per pixel, more than the motion's own adjacent frames ({})",
                matched.timestamp_seconds,
                matched.mean_abs_diff,
                match search.drift {
                    Some(drift) => format!("{drift}/255"),
                    None => format!("{rel} holds {} frame(s), so no drift between frames was measured", search.frames),
                }
            )));
            continue;
        };
        let source_match = json!({
            "timestamp_seconds": matched.timestamp_seconds,
            "mean_abs_diff": matched.mean_abs_diff,
            "compared_frames": search.frames,
            "motion_path": rel,
            "method": "full-resolution grayscale mean-absolute-difference search over every frame",
            "max_mean_abs_diff": tolerance,
            "tolerance": "mean difference between the motion's adjacent frames",
        });
        let obj = state_entry(data, state.position)?
            .as_object_mut()
            .context("state entry is no longer an object")?;
        obj.insert("source_match".into(), source_match);
        obj.insert(
            "source_motion_path".into(),
            json!(format!(
                "frame of {rel} at {}s (mean abs diff {}/255)",
                matched.timestamp_seconds, matched.mean_abs_diff
            )),
        );
        reasons.push(None);
    }
    Ok(reasons)
}
