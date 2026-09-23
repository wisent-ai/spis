use super::*;

pub(crate) fn validate_manifest_coordinates(report: &Value, required_uri: Option<&str>) -> Result<String> {
    let manifest = report
        .get("runtime_manifest")
        .context("retrieval report has no runtime_manifest")?;
    for field in [
        "run_id",
        "record",
        "record_key",
        "attempt_id",
        "source_revision",
        "source_input_sha256",
    ] {
        let report_value = report
            .get(field)
            .and_then(Value::as_str)
            .with_context(|| format!("retrieval report has no {field}"))?;
        let manifest_value = manifest
            .get(field)
            .and_then(Value::as_str)
            .with_context(|| format!("runtime manifest has no {field}"))?;
        if report_value != manifest_value {
            bail!("retrieval report and runtime manifest disagree on {field}");
        }
    }
    let run_id = report["run_id"].as_str().unwrap();
    let catalog = manifest
        .get("catalog")
        .and_then(Value::as_str)
        .context("runtime manifest has no catalog")?;
    let record = report["record"].as_str().unwrap();
    let record_key = report["record_key"].as_str().unwrap();
    let attempt = report
        .get("attempt")
        .and_then(Value::as_u64)
        .filter(|attempt| *attempt > 0)
        .context("retrieval report has no positive attempt")?;
    if manifest.get("attempt").and_then(Value::as_u64) != Some(attempt) {
        bail!("retrieval report and runtime manifest disagree on attempt");
    }
    let attempt_id = report["attempt_id"].as_str().unwrap();
    for (value, label) in [
        (run_id, "run_id"),
        (catalog, "catalog"),
        (record, "record"),
        (attempt_id, "attempt_id"),
    ] {
        safe_component(value, label)?;
    }
    exact_lower_hex(record_key, "record_key")?;
    let base = crate::crawl_attempt_base_uri(
        run_id,
        catalog,
        record,
        record_key,
        attempt,
        attempt_id,
    );
    let artifact_uri = manifest
        .get("artifact_uri")
        .and_then(Value::as_str)
        .context("runtime manifest has no artifact_uri")?;
    let output_uri = manifest
        .get("output_uri")
        .and_then(Value::as_str)
        .context("runtime manifest has no output_uri")?;
    if artifact_uri != format!("{base}/artifacts.tar.gz")
        || output_uri != format!("{base}/worker-output.log")
    {
        bail!("runtime manifest does not use canonical immutable attempt URIs");
    }
    if required_uri.is_some_and(|required| required != artifact_uri) {
        bail!("imported retrieval artifact does not identify the requested Stado URI");
    }
    Ok(artifact_uri.to_string())
}

pub(crate) fn validate_current_definition(report: &Value) -> Result<()> {
    let slug = report["record"].as_str().unwrap();
    let structure_path = engine_root().join(format!("{slug}.json"));
    let structure_bytes = std::fs::read(&structure_path).with_context(|| {
        format!(
            "current committed documentation definition for {slug} is unavailable at {}",
            structure_path.display()
        )
    })?;
    let structure_sha256 = lib::sha256_hex(&structure_bytes);
    let reported_structure = report
        .get("structure_sha256")
        .and_then(Value::as_str)
        .context("retrieval report has no structure_sha256")?;
    let manifest_structure = report
        .pointer("/runtime_manifest/docs_structure_sha256")
        .and_then(Value::as_str)
        .context("runtime manifest has no docs_structure_sha256")?;
    if structure_sha256 != reported_structure || structure_sha256 != manifest_structure {
        bail!("retrieval corpus is stale relative to the current committed documentation definition");
    }
    let structure: Value = serde_json::from_slice(&structure_bytes)?;
    let declared_source = report
        .get("declared_source_url")
        .or_else(|| report.get("source_url"))
        .and_then(Value::as_str)
        .context("retrieval report has no declared source URL")?;
    if structure.get("source_url").and_then(Value::as_str) != Some(declared_source)
        || report
            .pointer("/runtime_manifest/runtime_product/declared_identifier")
            .and_then(Value::as_str)
            != Some(declared_source)
    {
        bail!("retrieval corpus source URL differs from the current committed definition");
    }
    let definition_path = engine_root().join("full-text-manifest.json");
    let definition_sha256 = lib::sha256_hex(&std::fs::read(&definition_path)?);
    if report.get("definition_sha256").and_then(Value::as_str)
        != Some(definition_sha256.as_str())
    {
        bail!("retrieval corpus is stale relative to the current crawl definition");
    }
    Ok(())
}

pub(crate) fn validate_journal(corpus_dir: &Path, state: &Value) -> Result<()> {
    let path = corpus_dir.join("outcomes.jsonl");
    let mut journal = open_regular_read(&path, "outcome journal")?;
    if journal.metadata()?.len() > MAX_OUTCOME_JOURNAL_BYTES {
        bail!("outcome journal exceeds its durable byte limit");
    }
    let mut bytes = Vec::new();
    journal.read_to_end(&mut bytes)?;
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        bail!("completed outcome journal has an incomplete trailing record");
    }
    let state_outcomes = state
        .get("outcomes")
        .and_then(Value::as_object)
        .context("durable state has no outcomes object")?;
    let mut reconstructed = serde_json::Map::new();
    let mut committed_bytes = 0u64;
    let mut committed_sha256 = lib::sha256_hex(&[]);
    for line in bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let batch: Value = serde_json::from_slice(line)?;
        let entries = batch
            .get("outcomes")
            .and_then(Value::as_array)
            .context("outcome journal batch has no outcomes")?;
        let first = batch
            .get("first_sequence")
            .and_then(Value::as_u64)
            .context("outcome journal batch has no first_sequence")? as usize;
        let last = batch
            .get("last_sequence")
            .and_then(Value::as_u64)
            .context("outcome journal batch has no last_sequence")? as usize;
        if batch.get("schema").and_then(Value::as_str) != Some("wisent.docs-outcome-batch.v1")
            || entries.is_empty()
            || entries.len() > 32
            || first != reconstructed.len()
            || last + 1 != first.saturating_add(entries.len())
        {
            bail!("outcome journal is not canonical and contiguous");
        }
        for entry in entries {
            let key = entry
                .get("key")
                .and_then(Value::as_str)
                .context("outcome journal entry has no key")?;
            let outcome = entry
                .get("outcome")
                .context("outcome journal entry has no outcome")?
                .clone();
            if reconstructed.insert(key.to_string(), outcome).is_some() {
                bail!("outcome journal repeats a target key");
            }
        }
        committed_bytes = batch
            .get("committed_bytes")
            .and_then(Value::as_u64)
            .context("outcome journal batch has no committed_bytes")?;
        committed_sha256 = batch
            .get("committed_sha256")
            .and_then(Value::as_str)
            .context("outcome journal batch has no committed_sha256")?
            .to_string();
        exact_lower_hex(&committed_sha256, "journal committed_sha256")?;
    }
    if &reconstructed != state_outcomes
        || state.get("committed_bytes").and_then(Value::as_u64) != Some(committed_bytes)
        || state.get("committed_sha256").and_then(Value::as_str)
            != Some(committed_sha256.as_str())
    {
        bail!("outcome journal does not reconstruct the completed durable state");
    }
    Ok(())
}
