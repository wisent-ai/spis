use super::*;

pub(crate) fn validate_corpus(
    corpus_dir: &Path,
    required_uri: Option<&str>,
    origin: CorpusOrigin,
) -> Result<AttemptCorpus> {
    let mut observed = std::fs::read_dir(corpus_dir)
        .with_context(|| format!("list documentation corpus {}", corpus_dir.display()))?
        .map(|entry| entry.map(|value| value.file_name().to_string_lossy().to_string()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    observed.sort();
    let mut expected = CORPUS_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    expected.sort();
    if observed != expected {
        bail!(
            "documentation corpus {} does not contain the exact retrieval artifact set",
            corpus_dir.display()
        );
    }

    let state_path = corpus_dir.join("state.json");
    let report_path = corpus_dir.join("docs-retrieval-run.json");
    let state = read_json(&state_path)?;
    let report = read_json(&report_path)?;
    if state.get("schema").and_then(Value::as_str) != Some("wisent.docs-crawl-state.v3") {
        bail!("documentation corpus uses an unsupported durable state schema");
    }
    if report.get("schema").and_then(Value::as_str) != Some("wisent.docs-retrieval-run.v2") {
        bail!("documentation corpus uses an unsupported retrieval report schema");
    }
    for field in [
        "run_id",
        "record",
        "record_key",
        "attempt_id",
        "source_revision",
        "source_input_sha256",
        "source_url",
        "effective_source_url",
        "completed_at",
    ] {
        matching_string(&state, &report, field)?;
    }
    let state_attempt = state
        .get("attempt")
        .and_then(Value::as_u64)
        .context("durable state has no immutable attempt")?;
    if report.get("attempt").and_then(Value::as_u64) != Some(state_attempt) {
        bail!("durable state and retrieval report disagree on attempt");
    }
    let report_sha256 = state
        .get("report_sha256")
        .and_then(Value::as_str)
        .context("completed durable state has no report_sha256")?;
    exact_lower_hex(report_sha256, "report_sha256")?;
    if hash_file(&report_path)?.0 != report_sha256 {
        bail!("retrieval report differs from the digest in durable state");
    }
    validate_manifest_coordinates(&report, required_uri)?;
    validate_current_definition(&report)?;
    validate_outcomes(corpus_dir, &state, &report)?;
    let inventory_downloaded_bytes = state
        .get("inventory_downloaded_bytes")
        .and_then(Value::as_u64)
        .context("durable state has no inventory_downloaded_bytes")?;
    if inventory_downloaded_bytes > 64 * 1024 * 1024
        || report
            .pointer("/retrieval/inventory_downloaded_bytes")
            .and_then(Value::as_u64)
            != Some(inventory_downloaded_bytes)
    {
        bail!("retrieval inventory byte accounting is inconsistent");
    }
    let slug = report
        .get("record")
        .and_then(Value::as_str)
        .context("retrieval report has no record")?
        .to_string();
    let completed_at = report
        .get("completed_at")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("retrieval report has no completion timestamp")?
        .to_string();
    validate_completion_timestamp(&completed_at)?;
    let attempt_id = report
        .get("attempt_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("retrieval report has no attempt_id")?
        .to_string();
    let retrieval_status = report
        .get("retrieval_status")
        .and_then(Value::as_str)
        .context("retrieval report has no retrieval_status")?
        .to_string();
    if !matches!(
        retrieval_status.as_str(),
        "retrieval_complete" | "retrieval_partial" | "retrieval_no_text" | "retrieval_empty"
    ) {
        bail!("retrieval report has an unsupported retrieval_status");
    }
    Ok(AttemptCorpus {
        slug,
        corpus_dir: corpus_dir.to_path_buf(),
        origin,
        completed_at,
        attempt: state_attempt,
        attempt_id,
        retrieval_status,
        state,
        report,
    })
}

/// Every corpus under `directory`. Symbolic links are never followed, so the
/// walk cannot loop and needs no depth or directory count chosen here.
pub(crate) fn visit_corpora(
    directory: &Path,
    corpora: &mut Vec<AttemptCorpus>,
    origin: CorpusOrigin,
) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(directory)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Ok(());
    }
    // A corpus is identified by its content, not by its directory name. The
    // durable layout writes the four artifacts straight into the attempt root
    // (`native_attempt_root`), while `import_artifact` stages them under a
    // directory literally named `corpus`; only a content test sees both.
    if CORPUS_FILES
        .iter()
        .all(|name| directory.join(name).is_file())
    {
        corpora.push(validate_corpus(directory, None, origin)?);
        return Ok(());
    }
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            visit_corpora(&entry.path(), corpora, origin)?;
        }
    }
    Ok(())
}

pub(crate) fn selected_corpora() -> Result<HashMap<String, AttemptCorpus>> {
    let mut candidates = Vec::new();
    visit_corpora(&crawl_root()?, &mut candidates, CorpusOrigin::Local)?;
    visit_corpora(&imports_root()?, &mut candidates, CorpusOrigin::Imported)?;
    let mut selected = HashMap::<String, AttemptCorpus>::new();
    for candidate in candidates {
        let replace = selected.get(&candidate.slug).is_none_or(|current| {
            (
                &candidate.completed_at,
                candidate.attempt,
                &candidate.attempt_id,
            ) > (&current.completed_at, current.attempt, &current.attempt_id)
        });
        if replace {
            selected.insert(candidate.slug.clone(), candidate);
        }
    }
    Ok(selected)
}
