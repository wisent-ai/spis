use super::*;

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
        let semantic_name_verified = observation_supported(entry, &provenance_context);
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

        obj.remove("state_name");
        obj.remove("source_relationship");
        obj.remove("source_motion_path");
        obj.remove("source_match");
        if !truthy(obj.get("name")) {
            obj.insert("name".into(), json!(format!("Observed state {}", idx + 1)));
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
                    let diff = matched["mean_abs_diff"]
                        .as_f64()
                        .unwrap_or(f64::INFINITY);
                    if diff <= STATE_MATCH_MAX_DIFF {
                        let rel = motion
                            .strip_prefix(base)
                            .unwrap_or(motion)
                            .to_string_lossy()
                            .to_string();
                        let ts = matched["timestamp_seconds"].as_f64().unwrap_or(0.0);
                        let mut source_match = matched;
                        if let Some(details) = source_match.as_object_mut() {
                            details.insert("motion_path".into(), json!(rel));
                            details.insert(
                                "method".into(),
                                json!("16x16 grayscale mean-absolute-difference frame search"),
                            );
                            details.insert(
                                "max_mean_abs_diff".into(),
                                json!(STATE_MATCH_MAX_DIFF),
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

    let states_count = data
        .get("states")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if states_count < requirements.min_states {
        gaps.push(format!("fewer than {} states for {} profile", requirements.min_states, requirements.profile));
    }
    Ok(())
}
