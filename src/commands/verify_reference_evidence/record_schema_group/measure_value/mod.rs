use super::*;

mod drift;
mod motion;
mod states;

/// The record as measured, apart from the date it carries: what decides
/// whether a measurement changed anything.
fn without_date(record: &Value) -> Value {
    let mut record = record.clone();
    if let Some(object) = record.as_object_mut() {
        object.remove("measured_at");
    }
    record
}

pub(crate) fn measure_value(
    data: &mut Value,
    base: &Path,
    locate_states: bool,
) -> Result<Vec<String>> {
    let as_found = without_date(data);
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
    let primary_motion =
        motion::measure_motion(data, base, &requirements, &provenance_context, &mut gaps)?;

    // ---- states ----
    states::measure_states(
        data,
        base,
        locate_states,
        &requirements,
        &provenance_context,
        primary_motion,
        &mut gaps,
    )?;

    // ---- journey ----
    // An absent journey, or one without steps, has fewer steps than any floor.
    let journey = data.get("journey").cloned();
    let steps_len = journey
        .as_ref()
        .and_then(|journey| journey.get("steps"))
        .and_then(Value::as_array)
        .map(Vec::len);
    if steps_len.is_none_or(|len| len < requirements.min_journey_steps) {
        gaps.push(format!(
            "journey exposes fewer than {} observed steps for {} profile",
            requirements.min_journey_steps, requirements.profile
        ));
    }
    // The journey's fields are the contract's declared shape; its steps are
    // counted against the profile's floor above, not merely required.
    for key in reference_contract::field_names::<reference_contract::Journey>()
        .into_iter()
        .filter(|key| key != "steps")
    {
        if !truthy(journey.as_ref().and_then(|journey| journey.get(&key))) {
            gaps.push(format!("journey missing {key}"));
        }
    }

    // ---- interactions ----
    // A record without an interactions list maps no interactions.
    let interactions = match data.get("interactions").and_then(Value::as_array) {
        Some(interactions) => interactions.clone(),
        None => Vec::new(),
    };
    if interactions.len() < requirements.min_interactions {
        gaps.push(format!(
            "fewer than {} mapped interactions for {} profile",
            requirements.min_interactions, requirements.profile
        ));
    }
    let interaction_fields = reference_contract::field_names::<reference_contract::Interaction>();
    for (index, item) in interactions.iter().enumerate() {
        let missing: Vec<&str> = interaction_fields
            .iter()
            .map(String::as_str)
            .filter(|f| !truthy(item.get(f)))
            .collect();
        if !missing.is_empty() {
            // Named by its place in the record, which it always has; its
            // name may itself be one of the missing fields.
            gaps.push(format!(
                "interactions[{index}] missing {}",
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
            let mut missing_reported = false;
            for item in entries.iter_mut() {
                let Some(obj) = item.as_object_mut() else {
                    continue;
                };
                let older: Vec<String> = obj.keys().cloned().collect();
                for key in older {
                    if let Some(canonical) = reference_contract::canonical_analysis_field(&key) {
                        if let Some(older_value) = obj.remove(&key) {
                            obj.entry(canonical).or_insert(older_value);
                        }
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
                obj.insert("timing_class".into(), json!(canonical_timing));
                if declared_timing.is_some() && canonical_timing.is_none() {
                    gaps.push(format!(
                        "motion analysis timing class {:?} is not a timing class the contract declares and was removed",
                        declared_timing.unwrap()
                    ));
                }
                let required =
                    reference_contract::field_names::<reference_contract::MotionAnalysis>();
                for key in &required {
                    obj.entry(key.clone()).or_insert(Value::Null);
                }
                let known: BTreeSet<String> = required
                    .iter()
                    .cloned()
                    .chain(reference_contract::field_names::<
                        reference_contract::MotionAnalysisExtras,
                    >())
                    .collect();
                // A field the contract does not declare is removed from the
                // record, and the gap names it with the value it held.
                let unknown: Vec<String> = obj
                    .keys()
                    .filter(|k| !known.contains(k.as_str()))
                    .cloned()
                    .collect();
                for field in unknown {
                    if let Some(value) = obj.remove(&field) {
                        gaps.push(format!(
                            "motion analysis field {field:?} is not in the reference contract and was removed (it held {value})"
                        ));
                    }
                }
                let missing: Vec<&str> = required
                    .iter()
                    .map(String::as_str)
                    .filter(|f| !truthy(obj.get(*f)))
                    .collect();
                if !missing.is_empty() && !missing_reported {
                    gaps.push(format!("motion analysis missing {}", missing.join(", ")));
                    missing_reported = true;
                }
            }
            *analysis = if was_array {
                Value::Array(entries)
            } else {
                // A single analysis was read into exactly one entry above.
                match entries.into_iter().next() {
                    Some(entry) => entry,
                    None => unreachable!("a single motion analysis became one entry"),
                }
            };
        }
    }

    // ---- accessibility ----
    // An absent accessibility section, or one without observations, has
    // fewer observations than any floor and was never measured.
    let access = data.get("accessibility").cloned();
    let observations = access
        .as_ref()
        .and_then(|access| access.get("observations"))
        .and_then(Value::as_array)
        .map(Vec::len);
    if observations.is_none_or(|count| count < requirements.min_accessibility_observations) {
        gaps.push(format!(
            "fewer than {} accessibility observations for {} profile",
            requirements.min_accessibility_observations, requirements.profile
        ));
    }
    if !truthy(access.as_ref().and_then(|access| access.get("measured"))) {
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
    }
    // A measurement that changed the record is dated today; one that found it
    // exactly as it was keeps the date it was last measured on, so measuring
    // again does not rewrite every record.
    let changed = without_date(data) != as_found;
    if let Some(obj) = data.as_object_mut() {
        if changed || !obj.contains_key("measured_at") {
            obj.insert("measured_at".into(), json!(crate::today_utc()));
        }
    }

    Ok(gaps)
}
