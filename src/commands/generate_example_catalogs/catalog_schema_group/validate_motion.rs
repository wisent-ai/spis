use super::*;

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

pub(crate) fn validate_motion(
    record: &Value,
    record_path: &str,
    reference_dir: &Path,
    verified_provenance: &VerifiedProvenanceSet,
) -> Result<Vec<String>> {
    let requirements = super::reference_contract::completeness_requirements(reference_dir);
    let motion = record.get("motion");
    let Some(motion) = motion.and_then(Value::as_array) else {
        bail!("{record_path}: motion must be a list");
    };
    if motion.is_empty() {
        if EvidenceStatus::of(record) == Some(EvidenceStatus::Partial) {
            return Ok(Vec::new());
        }
        bail!("{record_path}: complete evidence needs at least one motion asset");
    }

    let mut classes: Vec<String> = Vec::new();
    for (position, item) in motion.iter().enumerate() {
        let context = format!("{record_path}: motion {}", position + 1);
        require_nonempty(item, &field_names::<MotionEntry>(), &context)?;
        let media_kind = required_text(item, "media_kind", &context)?;
        if reference_contract::named::<MotionKind>(media_kind).is_none() {
            bail!("{context}: media kind '{media_kind}' is not a canonical motion kind");
        }
        let provenance_class = required_text(item, "provenance_class", &context)?;
        let Some(declared) = reference_contract::named::<Provenance>(provenance_class) else {
            bail!("{context}: unknown provenance class '{provenance_class}'");
        };
        let derived = verified_provenance.provenance_class(item);
        if provenance_class != derived {
            bail!(
                "{context}: provenance class '{provenance_class}' contradicts typed provenance '{derived}'"
            );
        }
        if declared == Provenance::UnverifiedSourceMedia
            && EvidenceStatus::of(record) == Some(EvidenceStatus::Complete)
        {
            bail!("{context}: complete evidence requires verified typed motion provenance");
        }
        if !py_truthy(item.get("measured")) {
            bail!("{context}: asset was never measured; run verify-reference-evidence.py");
        }
        let source_url = required_text(item, "source_url", &context)?;
        if !url_ok(source_url) {
            bail!("{context}: invalid source URL");
        }
        let local_path = required_text(item, "local_path", &context)?;
        let motion_path = resolve_evidence_path(reference_dir, local_path, &context)?;
        if !Extension::of(&motion_path).is_some_and(Extension::holds_motion) {
            bail!("{context}: unsupported motion format");
        }
        let duration = item.get("duration_seconds").and_then(Value::as_f64);
        let duration_ok = matches!(duration, Some(d) if d >= requirements.min_motion_seconds);
        if !duration_ok {
            bail!(
                "{context}: measured duration {} is below the {} profile floor {}",
                python_repr(item.get("duration_seconds")),
                requirements.profile,
                requirements.min_motion_seconds
            );
        }
        if Extension::of(&motion_path) != Some(Extension::Cast) {
            require_nonempty(item, &field_names::<Dimensions>(), &context)?;
        }
        validate_file_metadata(&motion_path, item, &context)?;
        classes.push(provenance_class.to_string());
    }

    let Some(entries) = record.get("motion_provenance").and_then(Value::as_array) else {
        bail!("{record_path}: motion_provenance must be a list");
    };
    let declared: BTreeSet<String> = entries
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let measured: BTreeSet<String> = classes.iter().cloned().collect();
    if measured != declared {
        bail!("{record_path}: motion_provenance does not match the motion entries");
    }
    Ok(classes)
}

pub(crate) fn validate_states(
    record: &Value,
    record_path: &str,
    reference_dir: &Path,
    verified_provenance: &VerifiedProvenanceSet,
) -> Result<()> {
    let requirements = super::reference_contract::completeness_requirements(reference_dir);
    let Some(states) = record.get("states").and_then(Value::as_array) else {
        bail!("{record_path}: states must be a list");
    };
    if states.len() < requirements.min_states
        && EvidenceStatus::of(record) == Some(EvidenceStatus::Complete)
    {
        bail!(
            "{record_path}: complete {} evidence needs at least {} local states",
            requirements.profile,
            requirements.min_states
        );
    }
    for (position, item) in states.iter().enumerate() {
        let context = format!("{record_path}: state {}", position + 1);
        require_nonempty(item, &field_names::<StateEntry>(), &context)?;
        let local_path = required_text(item, "local_path", &context)?;
        let state_path = resolve_evidence_path(reference_dir, local_path, &context)?;
        if !Extension::of(&state_path).is_some_and(Extension::holds_state) {
            bail!("{context}: unsupported state-image format");
        }
        validate_file_metadata(&state_path, item, &context)?;
        if EvidenceStatus::of(record) == Some(EvidenceStatus::Complete) {
            if let Err(reason) = verified_provenance.supports_value(item) {
                bail!("{context}: semantic state name lacks independently verified observation provenance: {reason}");
            }
            if !py_truthy(item.get("source_motion_path"))
                || !item.get("source_match").is_some_and(Value::is_object)
            {
                bail!("{context}: state has no recomputed relationship to verified motion");
            }
        }
    }
    Ok(())
}
