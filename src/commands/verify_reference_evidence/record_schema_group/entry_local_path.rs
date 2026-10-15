use super::*;
use std::num::NonZeroUsize;

/// The file an evidence entry names, or nothing when it names none; the caller
/// reports such an entry rather than probing the record's directory.
pub(crate) fn entry_local_path(entry: &Map<String, Value>) -> Option<String> {
    entry
        .get("local_path")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
        .map(str::to_string)
}

/// The name a state carries when nothing verified names it: its place in the
/// record, counted from one.
pub(crate) fn generic_state_name(ordinal: usize) -> String {
    format!("Observed state {ordinal}")
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

    // A record without an interactions list has none to verify.
    let interactions = match data.get("interactions").and_then(Value::as_array) {
        Some(interactions) => interactions.clone(),
        None => Vec::new(),
    };
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

    let mut states = match data.get("states").and_then(Value::as_array) {
        Some(states) => states.clone(),
        None => Vec::new(),
    };
    for ((index, state), ordinal) in states.iter_mut().enumerate().zip(NonZeroUsize::MIN.get()..) {
        if observation_supported(state, context) {
            continue;
        }
        let name_field = format!("states[{index}].name");
        let generic_name = generic_state_name(ordinal);
        let existing_name = state.get("name").cloned();
        let name_preserved = notes
            .iter()
            .any(|note| note.get("field").and_then(Value::as_str) == Some(&name_field));
        if !name_preserved
            && existing_name
                .as_ref()
                .and_then(Value::as_str)
                .is_some_and(|name| name != generic_name)
        {
            notes.push(unverified_note(
                name_field,
                "semantic state name is not proven by pixels or digest and lacks independently verified observation provenance",
                json!(existing_name),
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

    let journey = data.get("journey").cloned();
    let journey_supported = journey.as_ref().is_some_and(|journey| {
        journey.is_object()
            && observation_supported(journey, context)
            && journey
                .get("steps")
                .and_then(Value::as_array)
                .is_some_and(|steps| {
                    !steps.is_empty()
                        && steps
                            .iter()
                            .all(|step| observation_supported(step, context))
                })
    });
    let verified_journey = match journey {
        Some(journey) if journey_supported => journey,
        Some(Value::Null) | None => Value::Null,
        Some(journey) => {
            notes.push(unverified_note(
                "journey".to_string(),
                "canonical journey and every step require typed provenance linked to verified raw evidence",
                journey,
            ));
            Value::Null
        }
    };

    let was_analysis_array = data.get("motion_analysis").is_some_and(Value::is_array);
    let analysis_items = match data.get("motion_analysis").cloned() {
        Some(Value::Array(items)) => items,
        Some(Value::Null) | None => Vec::new(),
        Some(item) => vec![item],
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
        // A single analysis was read into exactly one entry above.
        match verified_analysis.into_iter().next() {
            Some(analysis) => analysis,
            None => unreachable!("a non-empty list has a first entry"),
        }
    };

    // A record without an accessibility section starts from the contract's
    // empty one: not measured, no observations, no unknowns.
    let mut accessibility = match data.get("accessibility").filter(|value| value.is_object()) {
        Some(accessibility) => accessibility.clone(),
        None => json!({"measured": false, "observations": [], "unknowns": []}),
    };
    let observations = match accessibility.get("observations").and_then(Value::as_array) {
        Some(observations) => observations.clone(),
        None => Vec::new(),
    };
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
    if !accessibility.get("unknowns").is_some_and(Value::is_array) {
        accessibility["unknowns"] = json!([]);
    }

    let migrated_fields: BTreeSet<&str> = notes
        .iter()
        .filter(|note| note.get("schema").and_then(Value::as_str) == Some(UNVERIFIED_NOTE_SCHEMA))
        .filter_map(|note| note.get("field").and_then(Value::as_str))
        .collect();
    if migrated_fields
        .iter()
        .any(|field| field.starts_with("interactions["))
    {
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
