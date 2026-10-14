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
            "workload",
            "run",
            "weles-capture",
            "--target",
            target,
            "--plan",
            plan_arg.as_str(),
            "--json",
        ],
        true,
    )?;
    const WHAT: &str = "weles-capture";
    let payload = payload.ok_or_else(|| anyhow!("{WHAT}: Stado answered no JSON"))?;
    let rows = action_rows(&payload, WHAT)?;
    let captures = plan
        .get("captures")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{WHAT}: the plan holds no `captures` list"))?;
    if rows.len() != captures.len() {
        bail!(
            "weles-capture enqueued {} actions for a plan of {}; \
             refusing to attribute artifacts to records on a mismatched list",
            rows.len(),
            captures.len()
        );
    }
    let returned_action = answer_text(&payload, "action", WHAT)?;
    if returned_action != ACTION {
        bail!("{WHAT}: action: expected {ACTION}, got {returned_action}");
    }
    let returned_batch = answer_text(&payload, "batch", WHAT)?.to_string();
    let planned_batch = plan
        .get("batch")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{WHAT}: the plan names no batch"))?;
    if returned_batch != planned_batch {
        bail!("{WHAT}: batch: expected {planned_batch:?}, got {returned_batch:?}");
    }
    let mut ids: Vec<String> = Vec::new();
    for (position, (row, capture)) in rows.iter().zip(captures.iter()).enumerate() {
        let what = format!("{WHAT} action {}", position + 1);
        let identifier = action_id(row, &what)?;
        for field in ["site_slug", "artifact_prefix"] {
            let returned = answer_text(row, field, &what)?;
            let planned = answer_text(capture, field, &format!("{what} in the plan"))?;
            if returned != planned {
                bail!("{what}: {field}: expected {planned:?}, got {returned:?}");
            }
        }
        ids.push(identifier);
    }
    let unique = ids.iter().cloned().collect::<HashSet<_>>();
    if unique.len() != ids.len() {
        bail!("{WHAT}: action_id: duplicate ids prevent record attribution");
    }
    Ok((returned_batch, ids))
}

/// The state Stado recorded for one capture run.
pub(crate) fn state_of(row: &Value) -> Result<String> {
    answer_text(row, "state", "weles-capture status").map(str::to_string)
}

/// One read of Stado's batch record after the synchronous enqueue returned.
pub(crate) fn read_states(
    target: &str,
    batch: &str,
    expected_ids: &HashSet<String>,
    log: &dyn Fn(&str),
) -> Result<std::collections::HashMap<String, Value>> {
    let receipt = format!("weles-capture:{batch}");
    let (payload, _) = stado(
        &[
            "workload",
            "status",
            receipt.as_str(),
            "--target",
            target,
            "--json",
        ],
        true,
    )?;
    const WHAT: &str = "weles-capture status";
    let payload = payload.ok_or_else(|| anyhow!("{WHAT}: Stado answered no JSON"))?;
    let returned_action = answer_text(&payload, "action", WHAT)?;
    if returned_action != ACTION {
        bail!("{WHAT}: action: expected {ACTION}, got {returned_action}");
    }
    let rows = action_rows(&payload, WHAT)?;
    let mut latest: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for row in &rows {
        let identifier = action_id(row, WHAT)?;
        if expected_ids.contains(&identifier) {
            latest.insert(identifier, row.clone());
        }
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in latest.values() {
        *counts.entry(state_of(row)?).or_default() += 1;
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
    Ok(latest)
}

/// The storage keys Stado lists under one capture's prefix: its `artifacts`.
pub(crate) fn artifact_keys(row: &Value) -> Result<Vec<String>> {
    const WHAT: &str = "weles-capture status";
    row.get("artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{WHAT}: artifacts: expected a list"))?
        .iter()
        .enumerate()
        .map(|(position, key)| {
            key.as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("{WHAT}: artifacts[{position}]: expected a string"))
        })
        .collect()
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

pub(crate) fn count_field(
    summary: &Map<String, Value>,
    field: &str,
    reference: &Reference,
) -> Result<u64> {
    let value = require_summary_field(summary, field, reference)?;
    value.as_u64().ok_or_else(|| {
        anyhow!(
            "{}: axe-summary.json.{field}: expected a non-negative integer",
            reference.id()
        )
    })
}
