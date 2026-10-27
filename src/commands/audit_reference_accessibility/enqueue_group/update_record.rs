use super::*;

pub(crate) fn update_record(
    reference: &Reference,
    summary: &Map<String, Value>,
) -> Result<(String, String)> {
    let rid = reference.id();
    let text = std::fs::read_to_string(&reference.path)
        .with_context(|| format!("{rid}: reference.json: unreadable"))?;
    let document_doc = strict_json(&text, &format!("{rid}: reference.json"))?;
    let mut document = document_doc
        .as_object()
        .ok_or_else(|| anyhow!("{rid}: reference.json: expected an object"))?
        .clone();

    let current_field = if document.get("product_url").map(Value::is_null) == Some(false) {
        "product_url"
    } else {
        "source_url"
    };
    let current_url = match document.get(current_field) {
        Some(value) => canonical_url(value, &rid, current_field)?,
        None => bail!("{rid}: {current_field}: missing; a reference names the page it audits"),
    };
    if current_url != reference.source_url {
        bail!(
            "{rid}: {current_field}: changed from {:?} to {current_url:?} while audit ran",
            reference.source_url
        );
    }

    // A reference not audited before has no accessibility section; any other
    // shape is refused rather than replaced.
    let mut accessibility: Map<String, Value> = match document.get("accessibility") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(section)) => section.clone(),
        Some(_) => bail!("{rid}: accessibility: expected an object"),
    };

    // A list the section does not hold yet starts empty; one it holds must be
    // an array of strings.
    let strings = |key: &str| -> Result<Vec<Value>> {
        match accessibility.get(key) {
            None => Ok(Vec::new()),
            Some(Value::Array(items)) if items.iter().all(Value::is_string) => Ok(items.clone()),
            Some(_) => bail!("{rid}: accessibility.{key}: expected an array of strings"),
        }
    };
    let observations_list = strings("observations")?;
    let unknowns_value = Value::Array(strings("unknowns")?);

    let mut observations: Vec<Value> = observations_list
        .into_iter()
        .filter(|item| !matches!(item, Value::String(text) if text.starts_with("[axe-core]")))
        .collect();
    for observation in axe_observations(summary, reference)? {
        observations.push(Value::String(observation));
    }
    accessibility.insert("observations".into(), Value::Array(observations));
    accessibility.insert("unknowns".into(), unknowns_value);
    accessibility.insert("measured".into(), Value::Bool(true));

    // The measurement states what axe-core measured; a summary missing one of
    // these is refused, never recorded as a measured audit with blank fields.
    let get_num = |key: &str| -> Result<Value> {
        summary
            .get(key)
            .filter(|value| !value.is_null())
            .cloned()
            .with_context(|| format!("{rid}: axe-summary.json: no \"{key}\""))
    };
    let get_str = |key: &str| -> Result<Value> {
        let value = get_num(key)?;
        value
            .as_str()
            .with_context(|| format!("{rid}: axe-summary.json: \"{key}\" is not text"))?;
        Ok(value)
    };
    let measurement = json!({
        "tool": "axe-core",
        "version": get_str("axe_version")?,
        "captured_at": get_str("captured_at")?,
        "renderer": get_str("renderer")?,
        "weles_version": get_str("weles_version")?,
        "source_url": get_str("source_url")?,
        "viewport": get_num("viewport")?,
        "raw_path": "media/accessibility/axe.json",
        "summary_path": "media/accessibility/axe-summary.json",
        "raw_bytes": get_num("bytes")?,
        "raw_sha256": get_num("sha256")?,
        "violation_count": get_num("violation_count")?,
        "passes_count": get_num("passes_count")?,
        "incomplete_count": get_num("incomplete_count")?,
    });
    accessibility.insert("measurement".into(), measurement);
    document.insert("accessibility".into(), Value::Object(accessibility));
    atomic_json(&reference.path, &Value::Object(document))?;
    Ok((
        "media/accessibility/axe.json".to_string(),
        "media/accessibility/axe-summary.json".to_string(),
    ))
}

pub(crate) fn retrieve(
    reference: &Reference,
    action: &Map<String, Value>,
    row: &Value,
    batch: &str,
) -> Result<Value> {
    let rid = reference.id();
    let what = format!("{rid}: status");
    let site_slug = answer_text(row, "site_slug", &what)?;
    if site_slug != reference.slug {
        bail!(
            "{rid}: status.site_slug: expected {:?}, got {site_slug:?}",
            reference.slug
        );
    }
    let prefix = answer_text(row, "artifact_prefix", &what)?;
    let expected = action
        .get("artifact_prefix")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{rid}: the plan's capture names no artifact_prefix"))?;
    if prefix != expected {
        bail!("{rid}: status.artifact_prefix: expected {expected:?}, got {prefix:?}");
    }
    let keys = artifact_keys(row)?;
    let raw_key = named_artifact(&keys, "axe.json", reference)?;
    let summary_key = named_artifact(&keys, "axe-summary.json", reference)?;
    let stage = work_root()
        .join("staging")
        .join(batch)
        .join(&reference.catalog)
        .join(&reference.slug);
    let raw_stage = stage.join("axe.json");
    let summary_stage = stage.join("axe-summary.json");
    fetch_artifact(&summary_key, &summary_stage)
        .map_err(|e| anyhow!("{rid}: staging.axe-summary.json: {e:#}"))?;
    fetch_artifact(&raw_key, &raw_stage).map_err(|e| anyhow!("{rid}: staging.axe.json: {e:#}"))?;
    let summary = validate_artifacts(reference, action, &raw_stage, &summary_stage)
        .map_err(|e| anyhow!("{e:#}"))?;
    let (raw_path, summary_path) = install_artifacts(reference, &raw_stage, &summary_stage)
        .map_err(|e| anyhow!("{rid}: media/accessibility: {e}"))?;
    let (raw_relative, summary_relative) =
        update_record(reference, &summary).map_err(|e| anyhow!("{rid}: reference.json: {e:#}"))?;
    let repo_relative = |path: &Path| -> String { path.to_string_lossy().to_string() };
    Ok(json!({
        "id": rid,
        "catalog": reference.catalog,
        "index": reference.index,
        "name": reference.name,
        "site_slug": reference.slug,
        "source_url": reference.source_url,
        "status": "complete",
        "reason": Value::Null,
        "raw_path": repo_relative(&raw_path),
        "summary_path": repo_relative(&summary_path),
        "record_raw_path": raw_relative,
        "record_summary_path": summary_relative,
        "raw_storage_key": raw_key,
        "summary_storage_key": summary_key,
    }))
}

pub(crate) fn initial_row(reference: &Reference, action: &Map<String, Value>) -> Value {
    json!({
        "id": reference.id(),
        "catalog": reference.catalog,
        "index": reference.index,
        "name": reference.name,
        "site_slug": reference.slug,
        "source_url": reference.source_url,
        "status": "pending",
        "artifact_prefix": action.get("artifact_prefix").cloned().unwrap_or(Value::Null),
        "raw_path": Value::Null,
        "summary_path": Value::Null,
        "reason": "not dispatched",
    })
}

pub(crate) fn write_index(
    rows: &[Value],
    batch: &str,
    target: &str,
    plan_path: &Path,
    verifier_errors: &[String],
) -> Result<Value> {
    let count = |status: &str| {
        rows.iter()
            .filter(|row| row.get("status").and_then(Value::as_str) == Some(status))
            .count()
    };
    let totals = json!({
        "planned": rows.len(),
        "complete": count("complete"),
        "failed": count("failed"),
        "pending": count("pending"),
    });
    let payload = json!({
        "schema": INDEX_SCHEMA,
        "generated_at": now(),
        "batch": batch,
        "target": target,
        "plan": plan_path.to_string_lossy(),
        "totals": totals,
        "records": rows,
        "verifier_errors": verifier_errors,
    });
    atomic_json(Path::new(INDEX), &payload)?;
    Ok(payload)
}

pub(crate) fn run_verifier(catalog: &str) -> Result<()> {
    let exe = std::env::current_exe().context("locate the spis executable")?;
    // The verifier runs through stado_wait, which says on stderr what is
    // waited for and how it ended.
    let output = stado_wait::output(Command::new(exe).args([
        "verify-reference-evidence",
        "--catalog",
        catalog,
        "--apply",
        "--no-state-match",
    ]))
    .context("run spis verify-reference-evidence")?;
    if !output.status.success() {
        // The verifier's refusal is reported whole: its status as the
        // process ended (a signal included) and both streams, never the
        // first lines of one or an exit code it did not return.
        bail!(
            "spis verify-reference-evidence --catalog {catalog} ended {}; stderr: {}; stdout: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }
    Ok(())
}
