use super::*;

pub(crate) fn record_snapshot(run_id: &str, catalog: &str, record: &str) -> Result<Value> {
    let run = load(Some(run_id))?;
    run.get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| entry.get("catalog").and_then(Value::as_str) == Some(catalog))
        .and_then(|entry| entry.get("records"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| entry.get("record").and_then(Value::as_str) == Some(record))
        .cloned()
        .with_context(|| format!("{catalog}/{record}: crawl record disappeared"))
}

pub(crate) fn mutate_record<F>(run_id: &str, catalog: &str, record: &str, mutation: F) -> Result<()>
where
    F: FnOnce(&mut Value) -> Result<()>,
{
    let _guard = RunMutationGuard::acquire(run_id)?;
    let mut run = load(Some(run_id))?;
    let catalog_index = run
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .position(|entry| entry.get("catalog").and_then(Value::as_str) == Some(catalog))
        .with_context(|| format!("{catalog}: crawl catalog disappeared"))?;
    let record_index = run["catalogs"][catalog_index]
        .get("records")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .position(|entry| entry.get("record").and_then(Value::as_str) == Some(record))
        .with_context(|| format!("{catalog}/{record}: crawl record disappeared"))?;
    mutation(&mut run["catalogs"][catalog_index]["records"][record_index])?;
    aggregate_catalog_entry(&mut run["catalogs"][catalog_index]);
    update_run_state(&mut run);
    persist(&mut run)
}

pub(crate) fn mark_record_failure(
    run_id: &str,
    catalog: &str,
    record: &str,
    state: &str,
    code: &str,
    message: String,
) -> Result<()> {
    mutate_record(run_id, catalog, record, |entry| {
        if entry.get("cancel_intent").is_some_and(Value::is_object) {
            entry["state"] = json!("cancelled");
            entry["diagnostic"] = json!({
                "code": "cancelled_during_submission",
                "message": "durable cancel intent takes precedence over the coordinator result",
                "underlying": {"code": code, "message": message},
            });
        } else {
            entry["state"] = json!(state);
            entry["diagnostic"] = json!({"code": code, "message": message});
        }
        Ok(())
    })
}

pub(crate) fn ensure_host_preflight(
    run_id: &str,
    catalog: &str,
    engine: &str,
    host: &str,
    service_identity: Option<&RuntimeServiceIdentity>,
) -> Result<Value> {
    let snapshot = load(Some(run_id))?;
    let existing = snapshot
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| entry.get("catalog").and_then(Value::as_str) == Some(catalog))
        .and_then(|entry| entry.get("host_preflight"))
        .cloned()
        .context("crawl catalog disappeared before host preflight")?;
    if !existing.is_null() && !host_preflight_is_retryable(&existing) {
        return Ok(existing);
    }
    let observed = host_preflight(catalog, engine, host, service_identity);
    let _guard = RunMutationGuard::acquire(run_id)?;
    let mut run = load(Some(run_id))?;
    let catalog_index = run
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .position(|entry| entry.get("catalog").and_then(Value::as_str) == Some(catalog))
        .context("crawl catalog disappeared while retaining host preflight")?;
    let current = &run["catalogs"][catalog_index]["host_preflight"];
    if current.is_null() || host_preflight_is_retryable(current) {
        run["catalogs"][catalog_index]["host_preflight"] = observed;
        run["catalogs"][catalog_index]["state"] = json!("preflighting");
        persist(&mut run)?;
    }
    Ok(run["catalogs"][catalog_index]["host_preflight"].clone())
}
