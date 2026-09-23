use super::*;

mod motion;
mod states;

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
    let primary_motion = motion::measure_motion(data, base, &requirements, &provenance_context, &mut gaps)?;

    // ---- states ----
    states::measure_states(data, base, locate_states, &requirements, &provenance_context, primary_motion, &mut gaps)?;

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
