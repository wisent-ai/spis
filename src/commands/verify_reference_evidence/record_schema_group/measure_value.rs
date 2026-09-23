use super::*;

pub(crate) fn measure_value(data: &mut Value, base: &Path, locate_states: bool) -> Result<Vec<String>> {
    let requirements = super::reference_contract::completeness_requirements(base);
    let provenance_context = ProvenanceContext::from_record(data, base);
    let mut gaps: Vec<String> = provenance_context
        .failures()
        .iter()
        .map(|failure| format!("Weles provenance verification failed: {failure}"))
        .collect();
    gaps.extend(demote_unsupported_semantics(data, &provenance_context));

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

    // ---- journey ----
    let journey = data.get("journey").cloned().unwrap_or(Value::Null);
    let steps_len = journey
        .get("steps")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if steps_len < requirements.min_journey_steps {
        gaps.push(format!(
            "journey exposes fewer than {} observed steps for {} profile",
            requirements.min_journey_steps,
            requirements.profile
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
    if interactions.len() < requirements.min_interactions {
        gaps.push(format!("fewer than {} mapped interactions for {} profile", requirements.min_interactions, requirements.profile));
    }
    for item in &interactions {
        let missing: Vec<&str> = INTERACTION_FIELDS
            .iter()
            .copied()
            .filter(|f| !truthy(item.get(f)))
            .collect();
        if !missing.is_empty() {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            gaps.push(format!(
                "interaction '{name}' missing {}",
                missing.join(", ")
            ));
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
                        obj.entry(canonical.to_string()).or_insert(alias_value);
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
    if observations < requirements.min_accessibility_observations {
        gaps.push(format!(
            "fewer than {} accessibility observations for {} profile",
            requirements.min_accessibility_observations,
            requirements.profile
        ));
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
            json!(if gaps.is_empty() {
                "complete"
            } else {
                "partial"
            }),
        );
        obj.insert("measured_at".into(), json!(TODAY));
    }

    Ok(gaps)
}
