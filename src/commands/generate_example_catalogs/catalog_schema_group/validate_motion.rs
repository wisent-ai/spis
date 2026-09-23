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
        if evidence_status_of(record) == Some("partial") {
            return Ok(Vec::new());
        }
        bail!("{record_path}: complete evidence needs at least one motion asset");
    }

    let mut classes: Vec<String> = Vec::new();
    for (position, item) in motion.iter().enumerate() {
        let context = format!("{record_path}: motion {}", position + 1);
        require_nonempty(
            item,
            &[
                "local_path",
                "source_url",
                "media_kind",
                "bytes",
                "sha256",
                "capture_method",
                "provenance_class",
            ],
            &context,
        )?;
        let media_kind = item.get("media_kind").and_then(Value::as_str).unwrap_or("");
        if !CANONICAL_MOTION_KINDS.contains(&media_kind) {
            bail!(
                "{context}: media kind '{}' is not in the canonical vocabulary {:?}",
                media_kind,
                CANONICAL_MOTION_KINDS
            );
        }
        let provenance_class = item
            .get("provenance_class")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !PROVENANCE_CLASSES.contains(&provenance_class) {
            bail!("{context}: unknown provenance class '{provenance_class}'");
        }
        let derived = verified_provenance.provenance_class(item);
        if provenance_class != derived {
            bail!(
                "{context}: provenance class '{provenance_class}' contradicts typed provenance '{derived}'"
            );
        }
        if provenance_class == "unverified-source-media"
            && evidence_status_of(record) == Some("complete")
        {
            bail!("{context}: complete evidence requires verified typed motion provenance");
        }
        if !py_truthy(item.get("measured")) {
            bail!("{context}: asset was never measured; run verify-reference-evidence.py");
        }
        let source_url = item.get("source_url").and_then(Value::as_str).unwrap_or("");
        if !url_ok(source_url) {
            bail!("{context}: invalid source URL");
        }
        let local_path = item.get("local_path").and_then(Value::as_str).unwrap_or("");
        let motion_path = resolve_evidence_path(reference_dir, local_path, &context)?;
        if !has_suffix(&motion_path, MOTION_SUFFIXES) {
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
        if !has_suffix(&motion_path, &[".cast"]) {
            require_nonempty(item, &["width", "height"], &context)?;
        }
        validate_file_metadata(&motion_path, item, &context)?;
        classes.push(provenance_class.to_string());
    }

    let declared: BTreeSet<String> = record
        .get("motion_provenance")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
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
    if states.len() < requirements.min_states && evidence_status_of(record) == Some("complete") {
        bail!("{record_path}: complete {} evidence needs at least {} local states", requirements.profile, requirements.min_states);
    }
    for (position, item) in states.iter().enumerate() {
        let context = format!("{record_path}: state {}", position + 1);
        require_nonempty(
            item,
            &["local_path", "width", "height", "bytes", "sha256"],
            &context,
        )?;
        let local_path = item.get("local_path").and_then(Value::as_str).unwrap_or("");
        let state_path = resolve_evidence_path(reference_dir, local_path, &context)?;
        if !has_suffix(&state_path, STATE_SUFFIXES) {
            bail!("{context}: unsupported state-image format");
        }
        validate_file_metadata(&state_path, item, &context)?;
        if evidence_status_of(record) == Some("complete") {
            if !verified_provenance.supports_value(item) {
                bail!("{context}: semantic state name lacks independently verified observation provenance");
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
