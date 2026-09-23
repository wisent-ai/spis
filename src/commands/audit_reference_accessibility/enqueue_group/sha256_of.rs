use super::*;

pub(crate) fn sha256_of(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(crate::sha256_hex(&bytes))
}

pub(crate) fn validate_artifacts(
    reference: &Reference,
    action: &Map<String, Value>,
    raw_path: &Path,
    summary_path: &Path,
) -> Result<Map<String, Value>> {
    let rid = reference.id();
    let summary_text = std::fs::read_to_string(summary_path)
        .map_err(|e| anyhow!("{rid}: axe-summary.json: not UTF-8/readable: {e}"))?;
    let summary_doc = strict_json(&summary_text, &format!("{rid}: axe-summary.json"))?;
    let summary = summary_doc
        .as_object()
        .ok_or_else(|| anyhow!("{rid}: axe-summary.json: expected an object"))?
        .clone();
    for field in SUMMARY_FIELDS {
        require_summary_field(&summary, field, reference)?;
    }
    if summary.get("source_url") != action.get("source_url") {
        bail!(
            "{rid}: axe-summary.json.source_url: expected {}, got {}",
            action.get("source_url").unwrap_or(&Value::Null),
            summary.get("source_url").unwrap_or(&Value::Null)
        );
    }
    if summary.get("viewport") != action.get("viewport") {
        bail!(
            "{rid}: axe-summary.json.viewport: expected {}, got {}",
            action.get("viewport").unwrap_or(&Value::Null),
            summary.get("viewport").unwrap_or(&Value::Null)
        );
    }
    for field in ["captured_at", "renderer", "weles_version", "axe_version"] {
        nonempty_text(
            require_summary_field(&summary, field, reference)?,
            field,
            reference,
        )?;
    }
    let raw_size = std::fs::metadata(raw_path)
        .map_err(|e| anyhow!("{rid}: staged axe.json: {e}"))?
        .len();
    let raw_hash = sha256_of(raw_path)?;
    let expected_size = count_field(&summary, "bytes", reference)?;
    if raw_size != expected_size {
        bail!(
            "{rid}: axe-summary.json.bytes: downloaded axe.json has {raw_size} bytes, summary records {expected_size}"
        );
    }
    let expected_hash = require_summary_field(&summary, "sha256", reference)?
        .as_str()
        .filter(|h| is_hex64_lower(h))
        .ok_or_else(|| {
            anyhow!("{rid}: axe-summary.json.sha256: expected 64 lowercase hex characters")
        })?;
    if raw_hash != expected_hash {
        bail!(
            "{rid}: axe-summary.json.sha256: downloaded axe.json hashes to {raw_hash}, summary records {expected_hash}"
        );
    }
    let violation_count = count_field(&summary, "violation_count", reference)?;
    let passes_count = count_field(&summary, "passes_count", reference)?;
    let incomplete_count = count_field(&summary, "incomplete_count", reference)?;
    let violations = require_summary_field(&summary, "violations", reference)?
        .as_array()
        .ok_or_else(|| anyhow!("{rid}: axe-summary.json.violations: expected an array"))?;
    if violations.len() as u64 != violation_count {
        bail!(
            "{rid}: axe-summary.json.violation_count: records {violation_count}, but violations has {} entries",
            violations.len()
        );
    }
    for (position, violation) in violations.iter().enumerate() {
        let field = format!("violations[{position}]");
        let violation = violation
            .as_object()
            .ok_or_else(|| anyhow!("{rid}: axe-summary.json.{field}: expected an object"))?;
        let id_ok = violation
            .get("id")
            .and_then(Value::as_str)
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        if !id_ok {
            bail!("{rid}: axe-summary.json.{field}.id: expected a non-empty string");
        }
        if let Some(impact) = violation.get("impact") {
            if !(impact.is_null() || impact.is_string()) {
                bail!("{rid}: axe-summary.json.{field}.impact: expected a string or null");
            }
        }
        if !violation.get("help").map(Value::is_string).unwrap_or(false) {
            bail!("{rid}: axe-summary.json.{field}.help: expected a string");
        }
        let nodes_ok = violation
            .get("node_count")
            .and_then(Value::as_u64)
            .is_some();
        if !nodes_ok {
            bail!("{rid}: axe-summary.json.{field}.node_count: expected a non-negative integer");
        }
    }
    let raw_text = std::fs::read_to_string(raw_path)
        .map_err(|e| anyhow!("{rid}: axe.json: not UTF-8/readable: {e}"))?;
    let raw = strict_json(&raw_text, &format!("{rid}: axe.json"))?;
    if !raw.is_object() {
        bail!("{rid}: axe.json: expected an object");
    }
    for (field, count) in [
        ("violations", violation_count),
        ("passes", passes_count),
        ("incomplete", incomplete_count),
    ] {
        let length = raw
            .get(field)
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("{rid}: axe.json.{field}: expected an array"))?
            .len() as u64;
        if length != count {
            bail!("{rid}: axe.json.{field}: has {length} entries, summary records {count}");
        }
    }
    let mut summary = summary;
    summary.insert("bytes".to_string(), Value::Number(raw_size.into()));
    summary.insert("sha256".to_string(), Value::String(raw_hash));
    Ok(summary)
}

pub(crate) fn install_artifacts(
    reference: &Reference,
    raw_stage: &Path,
    summary_stage: &Path,
) -> Result<(PathBuf, PathBuf)> {
    let directory = reference
        .path
        .parent()
        .context("reference path has no parent")?
        .join("media")
        .join("accessibility");
    std::fs::create_dir_all(&directory)?;
    let raw_path = directory.join("axe.json");
    let summary_path = directory.join("axe-summary.json");
    let raw_part = directory.join(format!(".axe.json.{}.part", pid()));
    let summary_part = directory.join(format!(".axe-summary.json.{}.part", pid()));
    std::fs::copy(raw_stage, &raw_part)?;
    std::fs::copy(summary_stage, &summary_part)?;
    std::fs::rename(raw_part, &raw_path)?;
    std::fs::rename(summary_part, &summary_path)?;
    Ok((raw_path, summary_path))
}

pub(crate) fn axe_observations(summary: &Map<String, Value>) -> Vec<String> {
    let version = summary
        .get("axe_version")
        .and_then(Value::as_str)
        .unwrap_or("");
    let viewport = summary.get("viewport").cloned().unwrap_or(Value::Null);
    let width = viewport
        .get("width")
        .map(|v| v.to_string())
        .unwrap_or_default();
    let height = viewport
        .get("height")
        .map(|v| v.to_string())
        .unwrap_or_default();
    let dsf = viewport
        .get("device_scale_factor")
        .map(|v| v.to_string())
        .unwrap_or_default();
    let captured_at = summary
        .get("captured_at")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut observations = vec![format!(
        "[axe-core] axe-core {version} reported {} violation rules, {} passing rules, and {} incomplete rules against the live product at {width}x{height}@{dsf} on {captured_at}.",
        summary.get("violation_count").map(|v| v.to_string()).unwrap_or_default(),
        summary.get("passes_count").map(|v| v.to_string()).unwrap_or_default(),
        summary.get("incomplete_count").map(|v| v.to_string()).unwrap_or_default(),
    )];
    if let Some(violations) = summary.get("violations").and_then(Value::as_array) {
        for violation in violations {
            let Some(violation) = violation.as_object() else {
                continue;
            };
            let rule = violation
                .get("id")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("unnamed-rule");
            let impact_text = match violation.get("impact") {
                Some(Value::Null) | None => "impact not reported".to_string(),
                Some(other) => other.to_string().trim_matches('"').to_string(),
            };
            let nodes = violation
                .get("node_count")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0".to_string());
            let help_text = violation
                .get("help")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let suffix = if help_text.is_empty() {
                String::new()
            } else {
                format!(": {help_text}")
            };
            observations.push(format!(
                "[axe-core] Rule {rule} ({impact_text}) affected {nodes} nodes{suffix}."
            ));
        }
    }
    observations
}
