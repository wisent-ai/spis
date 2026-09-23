use super::*;

pub(crate) fn validate_behaviour(
    record: &Value,
    record_path: &str,
    reference_dir: &Path,
    verified_provenance: &VerifiedProvenanceSet,
) -> Result<()> {
    let requirements = super::reference_contract::completeness_requirements(reference_dir);
    let Some(interactions) = record.get("interactions").and_then(Value::as_array) else {
        bail!("{record_path}: interactions must be a list");
    };
    if interactions.len() < requirements.min_interactions && evidence_status_of(record) == Some("complete") {
        bail!(
            "{record_path}: complete {} evidence needs at least {} observed interactions",
            requirements.profile,
            requirements.min_interactions
        );
    }
    for (position, item) in interactions.iter().enumerate() {
        require_nonempty(
            item,
            INTERACTION_FIELDS,
            &format!("{record_path}: interaction {}", position + 1),
        )?;
        if !verified_provenance.supports_value(item) {
            bail!(
                "{record_path}: interaction {} lacks verified typed provenance",
                position + 1
            );
        }
    }

    let journey_value = record.get("journey");
    if py_truthy(journey_value) {
        let journey = journey_value.expect("truthy");
        require_nonempty(journey, JOURNEY_FIELDS, &format!("{record_path}: journey"))?;
        if !verified_provenance.supports_value(journey) {
            bail!("{record_path}: journey lacks verified typed provenance");
        }
        let steps_ok = journey
            .get("steps")
            .and_then(Value::as_array)
            .map(|steps| steps.len() >= requirements.min_journey_steps)
            .unwrap_or(false);
        if !steps_ok {
            bail!("{record_path}: {} journey needs at least {} observed steps", requirements.profile, requirements.min_journey_steps);
        }
        for (position, step) in journey["steps"].as_array().unwrap().iter().enumerate() {
            require_nonempty(
                step,
                JOURNEY_STEP_FIELDS,
                &format!("{record_path}: journey step {}", position + 1),
            )?;
            if !verified_provenance.supports_value(step) {
                bail!(
                    "{record_path}: journey step {} lacks verified typed provenance",
                    position + 1
                );
            }
            let index = step.get("index").and_then(Value::as_i64);
            if index != Some(position as i64 + 1) {
                bail!("{record_path}: journey step order is invalid");
            }
        }
    } else if evidence_status_of(record) == Some("complete") {
        bail!("{record_path}: complete evidence needs a journey");
    }

    match record.get("motion_analysis") {
        Some(analysis) if !analysis.is_null() => {
            let entries: Vec<Value> = match analysis {
                Value::Array(items) => items.clone(),
                other => vec![other.clone()],
            };
            for (position, item) in entries.iter().enumerate() {
                let allowed: std::collections::HashSet<&str> = MOTION_ANALYSIS_FIELDS
                    .iter()
                    .chain(MOTION_ANALYSIS_OPTIONAL.iter())
                    .copied()
                    .collect();
                let obj = match item.as_object() {
                    Some(obj) => obj,
                    None => bail!(
                        "{record_path}: motion analysis {} is malformed",
                        position + 1
                    ),
                };
                let unknown: Vec<String> = obj
                    .keys()
                    .filter(|key| !allowed.contains(key.as_str()))
                    .cloned()
                    .collect();
                if !unknown.is_empty() {
                    bail!(
                        "{record_path}: motion analysis {} has unknown fields {:?}",
                        position + 1,
                        unknown
                    );
                }
                let missing: Vec<&str> = MOTION_ANALYSIS_FIELDS
                    .iter()
                    .filter(|field| !obj.contains_key(**field))
                    .copied()
                    .collect();
                if !missing.is_empty() {
                    bail!(
                        "{record_path}: motion analysis {} omits {:?}",
                        position + 1,
                        missing
                    );
                }
                if !verified_provenance.supports_value(item) {
                    bail!(
                        "{record_path}: motion analysis {} lacks verified typed provenance",
                        position + 1
                    );
                }
                if let Some(timing) = item.get("timing_class") {
                    if !timing.is_null() {
                        let timing = timing.as_str().unwrap_or("");
                        if !TIMING_CLASSES.contains(&timing) {
                            bail!(
                                "{record_path}: timing class '{timing}' is not one of {TIMING_CLASSES:?}"
                            );
                        }
                    }
                }
            }
        }
        _ => {
            if evidence_status_of(record) == Some("complete") {
                bail!("{record_path}: complete evidence needs motion_analysis");
            }
        }
    }

    let accessibility = record.get("accessibility").unwrap_or(&Value::Null);
    let observations = accessibility.get("observations");
    let unknowns = accessibility.get("unknowns");
    if !observations.map(Value::is_array).unwrap_or(false)
        || !unknowns.map(Value::is_array).unwrap_or(false)
    {
        bail!("{record_path}: accessibility observations and unknowns are required");
    }
    for (position, observation) in observations
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        if !nonempty_observation(observation)
            || !verified_provenance.supports_value(observation)
        {
            bail!(
                "{record_path}: accessibility observation {} lacks a typed statement or verified provenance",
                position + 1
            );
        }
    }
    if evidence_status_of(record) == Some("complete") {
        let count = observations.and_then(Value::as_array).map(Vec::len).unwrap_or(0);
        if count < requirements.min_accessibility_observations {
            bail!("{record_path}: complete {} evidence needs at least {} accessibility observations", requirements.profile, requirements.min_accessibility_observations);
        }
        if !py_truthy(accessibility.get("measured")) {
            bail!("{record_path}: complete {} evidence must be measured against the product", requirements.profile);
        }
    }
    Ok(())
}
