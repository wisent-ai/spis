use super::*;

pub(crate) fn entry_local_path(entry: &Map<String, Value>) -> String {
    entry
        .get("local_path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Measurement date recorded by this migration release.
pub(crate) const TODAY: &str = "2026-09-01";

pub(crate) fn unverified_note(field: String, reason: &str, source_value: Value) -> Value {
    json!({
        "schema": UNVERIFIED_NOTE_SCHEMA,
        "field": field,
        "reason": reason,
        "source_value": source_value,
    })
}

pub(crate) fn demote_unsupported_semantics(
    data: &mut Value,
    context: &ProvenanceContext,
) -> Vec<String> {
    let mut notes: Vec<Value> = data
        .get("unverified_source_notes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    let mut migration_gaps = Vec::new();

    let interactions = data
        .get("interactions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut verified_interactions = Vec::new();
    for (index, interaction) in interactions.into_iter().enumerate() {
        if observation_supported(&interaction, context) {
            verified_interactions.push(interaction);
        } else {
            notes.push(unverified_note(
                format!("interactions[{index}]"),
                "canonical interaction lacks typed provenance linked to a verified crawl import or proven owner observation",
                interaction,
            ));
        }
    }

    let mut states = data
        .get("states")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for (index, state) in states.iter_mut().enumerate() {
        if observation_supported(state, context) {
            continue;
        }
        let name_field = format!("states[{index}].name");
        let generic_name = format!("Observed state {}", index + 1);
        let existing_name = state.get("name").cloned().unwrap_or(Value::Null);
        let name_preserved = notes
            .iter()
            .any(|note| note.get("field").and_then(Value::as_str) == Some(&name_field));
        if !name_preserved
            && existing_name
                .as_str()
                .is_some_and(|name| name != generic_name)
        {
            notes.push(unverified_note(
                name_field,
                "semantic state name is not proven by pixels or digest and lacks independently verified observation provenance",
                existing_name,
            ));
        }
        let relationship_field = format!("states[{index}].source_relationship");
        let relationship_preserved = notes
            .iter()
            .any(|note| note.get("field").and_then(Value::as_str) == Some(&relationship_field));
        let existing_relationship = json!({
            "source_motion_path": state.get("source_motion_path"),
            "source_match": state.get("source_match"),
        });
        if !relationship_preserved
            && existing_relationship
                .as_object()
                .is_some_and(|object| object.values().any(|value| !value.is_null()))
        {
            notes.push(unverified_note(
                relationship_field,
                "stored state relationship was not independently recomputed against verified motion",
                existing_relationship,
            ));
        }
        if let Some(object) = state.as_object_mut() {
            object.insert("name".into(), json!(generic_name));
            object.remove("source_motion_path");
            object.remove("source_match");
            object.remove("source_relationship");
        }
    }

    let journey = data.get("journey").cloned().unwrap_or(Value::Null);
    let journey_supported = journey.is_object()
        && observation_supported(&journey, context)
        && journey
            .get("steps")
            .and_then(Value::as_array)
            .is_some_and(|steps| {
                !steps.is_empty()
                    && steps
                        .iter()
                        .all(|step| observation_supported(step, context))
            });
    let verified_journey = if journey_supported {
        journey
    } else {
        if !journey.is_null() {
            notes.push(unverified_note(
                "journey".to_string(),
                "canonical journey and every step require typed provenance linked to verified raw evidence",
                journey,
            ));
        }
        Value::Null
    };

    let motion_analysis = data
        .get("motion_analysis")
        .cloned()
        .unwrap_or(Value::Null);
    let was_analysis_array = motion_analysis.is_array();
    let analysis_items = match motion_analysis {
        Value::Array(items) => items,
        Value::Null => Vec::new(),
        item => vec![item],
    };
    let mut verified_analysis = Vec::new();
    for (index, analysis) in analysis_items.into_iter().enumerate() {
        if observation_supported(&analysis, context) {
            verified_analysis.push(analysis);
        } else {
            notes.push(unverified_note(
                format!("motion_analysis[{index}]"),
                "canonical motion analysis lacks typed provenance linked to verified raw evidence",
                analysis,
            ));
        }
    }
    let verified_analysis = if verified_analysis.is_empty() {
        Value::Null
    } else if was_analysis_array {
        Value::Array(verified_analysis)
    } else {
        verified_analysis.into_iter().next().unwrap_or(Value::Null)
    };

    let mut accessibility = data
        .get("accessibility")
        .cloned()
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({"measured": false, "observations": [], "unknowns": []}));
    let observations = accessibility
        .get("observations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut verified_accessibility = Vec::new();
    for (index, observation) in observations.into_iter().enumerate() {
        let statement_present = observation
            .get("observation")
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty());
        if statement_present && observation_supported(&observation, context) {
            verified_accessibility.push(observation);
        } else {
            notes.push(unverified_note(
                format!("accessibility.observations[{index}]"),
                "canonical accessibility observation must be a typed observation with verified provenance",
                observation,
            ));
        }
    }
    accessibility["measured"] = json!(!verified_accessibility.is_empty());
    accessibility["observations"] = Value::Array(verified_accessibility);
    if !accessibility
        .get("unknowns")
        .is_some_and(Value::is_array)
    {
        accessibility["unknowns"] = json!([]);
    }

    let migrated_fields: BTreeSet<&str> = notes
        .iter()
        .filter(|note| note.get("schema").and_then(Value::as_str) == Some(UNVERIFIED_NOTE_SCHEMA))
        .filter_map(|note| note.get("field").and_then(Value::as_str))
        .collect();
    if migrated_fields.iter().any(|field| field.starts_with("interactions[")) {
        migration_gaps.push(
            "unsupported interaction semantics preserved as unverified source notes; verified per-observation provenance absent".to_string(),
        );
    }
    if migrated_fields
        .iter()
        .any(|field| field.starts_with("states[") && field.ends_with(".name"))
    {
        migration_gaps.push(
            "semantic state names preserved as unverified source notes; pixels and digests do not prove labels".to_string(),
        );
    }
    if migrated_fields
        .iter()
        .any(|field| field.starts_with("states[") && field.ends_with(".source_relationship"))
    {
        migration_gaps.push(
            "stored state-to-motion relationships demoted; only a recomputed threshold match against independently verified motion may count".to_string(),
        );
    }
    if migrated_fields.contains("journey") {
        migration_gaps.push(
            "unsupported journey semantics preserved as unverified source notes; journey and step provenance absent".to_string(),
        );
    }
    if migrated_fields
        .iter()
        .any(|field| field.starts_with("motion_analysis["))
    {
        migration_gaps.push(
            "unsupported motion analysis preserved as unverified source notes; verified observation provenance absent".to_string(),
        );
    }
    if migrated_fields
        .iter()
        .any(|field| field.starts_with("accessibility.observations["))
    {
        migration_gaps.push(
            "unsupported accessibility observations preserved as unverified source notes; typed statement provenance absent".to_string(),
        );
    }

    if let Some(object) = data.as_object_mut() {
        object.insert("states".into(), Value::Array(states));
        object.insert("interactions".into(), Value::Array(verified_interactions));
        object.insert("journey".into(), verified_journey);
        object.insert("motion_analysis".into(), verified_analysis);
        object.insert("accessibility".into(), accessibility);
        object.insert("unverified_source_notes".into(), Value::Array(notes));
    }
    migration_gaps
}
