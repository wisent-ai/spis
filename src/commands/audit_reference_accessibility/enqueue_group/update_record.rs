use super::*;

pub(crate) fn update_record(reference: &Reference, summary: &Map<String, Value>) -> Result<(String, String)> {
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
    let current_url = canonical_url(
        document.get(current_field).unwrap_or(&Value::Null),
        &rid,
        current_field,
    )?;
    if current_url != reference.source_url {
        bail!(
            "{rid}: {current_field}: changed from {:?} to {current_url:?} while audit ran",
            reference.source_url
        );
    }

    let accessibility_missing = document
        .get("accessibility")
        .map(Value::is_null)
        .unwrap_or(true);
    let mut accessibility: Map<String, Value> = if accessibility_missing {
        Map::new()
    } else {
        document
            .get("accessibility")
            .and_then(Value::as_object)
            .cloned()
            .ok_or_else(|| anyhow!("{rid}: accessibility: expected an object"))?
    };

    let observations_value = accessibility
        .get("observations")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));
    let observations_list = observations_value.as_array().ok_or_else(|| {
        anyhow!("{rid}: accessibility.observations: expected an array of strings")
    })?;
    if observations_list.iter().any(|item| !item.is_string()) {
        bail!("{rid}: accessibility.observations: expected an array of strings");
    }
    let unknowns_value = accessibility
        .get("unknowns")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));
    let unknowns_list = unknowns_value
        .as_array()
        .ok_or_else(|| anyhow!("{rid}: accessibility.unknowns: expected an array of strings"))?;
    if unknowns_list.iter().any(|item| !item.is_string()) {
        bail!("{rid}: accessibility.unknowns: expected an array of strings");
    }

    let mut observations: Vec<Value> = observations_list
        .iter()
        .filter(|item| !item.as_str().unwrap_or("").starts_with("[axe-core]"))
        .cloned()
        .collect();
    for observation in axe_observations(summary) {
        observations.push(Value::String(observation));
    }
    accessibility.insert("observations".into(), Value::Array(observations));
    accessibility.insert("unknowns".into(), unknowns_value);
    accessibility.insert("measured".into(), Value::Bool(true));

    let get_str = |key: &str| -> String {
        summary
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let get_num = |key: &str| -> Value { summary.get(key).cloned().unwrap_or(Value::Null) };
    let measurement = json!({
        "tool": "axe-core",
        "version": get_str("axe_version"),
        "captured_at": get_str("captured_at"),
        "renderer": get_str("renderer"),
        "weles_version": get_str("weles_version"),
        "source_url": get_str("source_url"),
        "viewport": summary.get("viewport").cloned().unwrap_or(Value::Null),
        "raw_path": "media/accessibility/axe.json",
        "summary_path": "media/accessibility/axe-summary.json",
        "raw_bytes": get_num("bytes"),
        "raw_sha256": get_num("sha256"),
        "violation_count": get_num("violation_count"),
        "passes_count": get_num("passes_count"),
        "incomplete_count": get_num("incomplete_count"),
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
    if let Some(site_slug) = row.get("site_slug").and_then(Value::as_str) {
        if site_slug != reference.slug {
            bail!(
                "{rid}: status.site_slug: expected {:?}, got {site_slug:?}",
                reference.slug
            );
        }
    }
    if let Some(prefix) = row.get("artifact_prefix").and_then(Value::as_str) {
        let expected = action
            .get("artifact_prefix")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if prefix != expected {
            bail!("{rid}: status.artifact_prefix: expected {expected:?}, got {prefix:?}");
        }
    }
    let keys = artifact_keys(row);
    let raw_key = named_artifact(&keys, "axe.json", reference)?;
    let summary_key = named_artifact(&keys, "axe-summary.json", reference)?;
    let stage = staging_root()
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
    let output = Command::new(exe)
        .args([
            "verify-reference-evidence",
            "--catalog",
            catalog,
            "--apply",
            "--no-state-match",
        ])
        .output()
        .context("spawn spis verify-reference-evidence")?;
    if !output.status.success() {
        let combined = if output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };
        let lines: Vec<&str> = combined
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        let detail = if lines.is_empty() {
            format!("exit {}", output.status.code().unwrap_or(-1))
        } else {
            lines
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        };
        bail!("spis verify-reference-evidence --catalog {catalog}: {detail}");
    }
    Ok(())
}
