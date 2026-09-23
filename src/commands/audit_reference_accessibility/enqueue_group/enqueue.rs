use super::*;

// ---------------------------------------------------------------------------
// Enqueue / poll / retrieve

pub(crate) fn enqueue(
    target: &str,
    plan_path: &Path,
    plan: &Map<String, Value>,
) -> Result<(String, Vec<String>)> {
    let plan_arg = plan_path.to_string_lossy().to_string();
    let (payload, _) = stado(
        &[
            "host",
            "weles-capture",
            target,
            "--plan",
            plan_arg.as_str(),
            "--json",
        ],
        true,
    )?;
    let payload = payload.unwrap_or(Value::Null);
    let rows = action_rows(&payload, "weles-capture")?;
    let captures = plan
        .get("captures")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if rows.len() != captures.len() {
        bail!(
            "weles-capture enqueued {} actions for a plan of {}; \
             refusing to attribute artifacts to records on a mismatched list",
            rows.len(),
            captures.len()
        );
    }
    if let Some(returned_action) = payload.get("action") {
        if returned_action.as_str() != Some(ACTION) {
            bail!("weles-capture: action: expected {ACTION}, got {returned_action}");
        }
    }
    let returned_batch = pick(&payload, &["batch", "batch_id", "id"])
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            plan.get("batch")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        })
        .trim_matches('"')
        .to_string();
    let planned_batch = plan
        .get("batch")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if returned_batch != planned_batch {
        bail!("weles-capture: batch: expected {planned_batch:?}, got {returned_batch:?}");
    }
    let mut ids: Vec<String> = Vec::new();
    for (position, (row, capture)) in rows.iter().zip(captures.iter()).enumerate() {
        let position = position + 1;
        let Some(identifier) = action_id(row) else {
            bail!("weles-capture action {position}: action_id: missing");
        };
        let identifier_trimmed = identifier.trim_matches('"').to_string();
        if let Some(site_slug) = row.get("site_slug").and_then(Value::as_str) {
            if let Some(expected_slug) = capture.get("site_slug").and_then(Value::as_str) {
                if site_slug != expected_slug {
                    bail!(
                        "weles-capture action {position}: site_slug: expected {expected_slug:?}, got {site_slug:?}"
                    );
                }
            }
        }
        if let Some(prefix) = row.get("artifact_prefix").and_then(Value::as_str) {
            if let Some(expected_prefix) = capture.get("artifact_prefix").and_then(Value::as_str) {
                if prefix != expected_prefix {
                    bail!(
                        "weles-capture action {position}: artifact_prefix: expected {expected_prefix:?}, got {prefix:?}"
                    );
                }
            }
        }
        ids.push(identifier_trimmed);
    }
    let unique = ids.iter().cloned().collect::<HashSet<_>>();
    if unique.len() != ids.len() {
        bail!("weles-capture: action_id: duplicate ids prevent record attribution");
    }
    Ok((returned_batch, ids))
}

pub(crate) fn state_of(row: &Value) -> String {
    pick(row, &["state", "status"])
        .map(|v| v.to_string().trim_matches('"').to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

pub(crate) fn poll(
    target: &str,
    batch: &str,
    expected_ids: &HashSet<String>,
    interval: u64,
    timeout_seconds: u64,
    log: &dyn Fn(&str),
) -> Result<std::collections::HashMap<String, Value>> {
    let terminal: HashSet<&str> = [
        "done",
        "failed",
        "error",
        "cancelled",
        "canceled",
        "skipped",
    ]
    .into_iter()
    .collect();
    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    let mut latest: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    loop {
        let (payload, _) = stado(
            &[
                "host",
                "weles-capture-status",
                target,
                "--batch",
                batch,
                "--json",
            ],
            true,
        )?;
        let payload = payload.unwrap_or(Value::Null);
        if let Some(returned_action) = payload.get("action") {
            if returned_action.as_str() != Some(ACTION) {
                bail!("weles-capture-status: action: expected {ACTION}, got {returned_action}");
            }
        }
        let rows = action_rows(&payload, "weles-capture-status")?;
        latest.clear();
        for row in &rows {
            if let Some(identifier) = action_id(row) {
                let identifier = identifier.trim_matches('"').to_string();
                if expected_ids.contains(&identifier) {
                    latest.insert(identifier, row.clone());
                }
            }
        }
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for row in latest.values() {
            *counts.entry(state_of(row)).or_insert(0) += 1;
        }
        let counts_text = counts
            .iter()
            .map(|(state, count)| format!("{state}={count}"))
            .collect::<Vec<_>>()
            .join(", ");
        log(&format!(
            "  {counts_text} ({}/{})",
            latest.len(),
            expected_ids.len()
        ));
        let all_terminal = latest.len() == expected_ids.len()
            && latest
                .values()
                .all(|row| terminal.contains(state_of(row).as_str()));
        if all_terminal {
            return Ok(latest);
        }
        if Instant::now() > deadline {
            log(&format!(
                "  timed out after {timeout_seconds}s; unresolved actions remain pending"
            ));
            return Ok(latest);
        }
        std::thread::sleep(Duration::from_secs(interval));
    }
}

pub(crate) fn artifact_keys(row: &Value) -> Vec<String> {
    let raw = match pick(row, &["artifacts", "artefacts", "objects", "keys"]) {
        Some(value) => value.clone(),
        None => return Vec::new(),
    };
    let mut result: Vec<String> = Vec::new();
    match raw {
        Value::String(item) => result.push(item),
        Value::Array(items) => {
            for item in items {
                match item {
                    Value::String(text) => result.push(text),
                    Value::Object(map) => {
                        let holder = Value::Object(map);
                        if let Some(value) = pick(
                            &holder,
                            &[
                                "key", "uri", "url", "artifact", "artefact", "object", "path",
                            ],
                        ) {
                            result.push(value.to_string().trim_matches('"').to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    result
}

pub(crate) fn named_artifact(keys: &[String], name: &str, reference: &Reference) -> Result<String> {
    let matches: Vec<&String> = keys.iter().filter(|key| basename_of(key) == name).collect();
    if matches.len() != 1 {
        bail!(
            "{}: artifacts.{name}: expected exactly one storage object, got {}",
            reference.id(),
            matches.len()
        );
    }
    Ok(matches[0].clone())
}

pub(crate) fn fetch_artifact(key: &str, destination: &Path) -> Result<()> {
    destination
        .parent()
        .map(std::fs::create_dir_all)
        .transpose()?;
    if destination.exists() {
        std::fs::remove_file(destination)?;
    }
    let dest = destination.to_string_lossy().to_string();
    stado(&["storage", "get", key, dest.as_str()], false)?;
    if !destination.is_file() {
        bail!(
            "stado storage get {key}: nothing was written to {}",
            destination.display()
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Artifact validation

pub(crate) fn require_summary_field<'a>(
    summary: &'a Map<String, Value>,
    field: &str,
    reference: &Reference,
) -> Result<&'a Value> {
    summary.get(field).ok_or_else(|| {
        anyhow!(
            "{}: axe-summary.json.{field}: field is missing",
            reference.id()
        )
    })
}

pub(crate) fn nonempty_text(value: &Value, field: &str, reference: &Reference) -> Result<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            anyhow!(
                "{}: axe-summary.json.{field}: expected a non-empty string",
                reference.id()
            )
        })
}

pub(crate) fn count_field(summary: &Map<String, Value>, field: &str, reference: &Reference) -> Result<u64> {
    let value = require_summary_field(summary, field, reference)?;
    value.as_u64().ok_or_else(|| {
        anyhow!(
            "{}: axe-summary.json.{field}: expected a non-negative integer",
            reference.id()
        )
    })
}
