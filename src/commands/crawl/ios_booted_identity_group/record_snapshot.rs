use super::*;

/// One catalog entry of a crawl run, read by its name and records list.
pub(crate) struct RunCatalog<'a> {
    pub(crate) name: &'a str,
    pub(crate) entry: &'a Value,
    pub(crate) records: &'a Vec<Value>,
}

/// The catalogs of a crawl run. A run without its catalogs list, or an entry
/// without its name or records list, is refused: read as empty, a selection
/// over the run matched nothing and said only that nothing matched.
pub(crate) fn run_catalogs(run: &Value) -> Result<Vec<RunCatalog<'_>>> {
    let run_id = &run["run_id"];
    run.get("catalogs")
        .and_then(Value::as_array)
        .with_context(|| format!("run {run_id} records no catalogs list"))?
        .iter()
        .map(|entry| {
            let name = entry
                .get("catalog")
                .and_then(Value::as_str)
                .with_context(|| {
                    format!("run {run_id}: a catalog entry names no catalog: {entry}")
                })?;
            let records = entry
                .get("records")
                .and_then(Value::as_array)
                .with_context(|| format!("run {run_id}: catalog {name} records no records list"))?;
            Ok(RunCatalog {
                name,
                entry,
                records,
            })
        })
        .collect()
}

/// The name a crawl record is kept under; a record without one is refused.
pub(crate) fn record_name<'a>(catalog: &str, record: &'a Value) -> Result<&'a str> {
    record
        .get("record")
        .and_then(Value::as_str)
        .with_context(|| format!("{catalog}: a crawl record names no record: {record}"))
}

/// One record of a crawl run, by catalog and record name.
pub(crate) fn run_record<'a>(run: &'a Value, catalog: &str, record: &str) -> Result<&'a Value> {
    let found = run_catalogs(run)?
        .into_iter()
        .find(|entry| entry.name == catalog)
        .with_context(|| format!("{catalog}: crawl catalog disappeared"))?;
    found
        .records
        .iter()
        .find(|entry| entry.get("record").and_then(Value::as_str) == Some(record))
        .with_context(|| format!("{catalog}/{record}: crawl record disappeared"))
}

pub(crate) fn record_snapshot(run_id: &str, catalog: &str, record: &str) -> Result<Value> {
    let run = load(Some(run_id))?;
    Ok(run_record(&run, catalog, record)?.clone())
}

pub(crate) fn mutate_record<F>(run_id: &str, catalog: &str, record: &str, mutation: F) -> Result<()>
where
    F: FnOnce(&mut Value) -> Result<()>,
{
    let _guard = RunMutationGuard::acquire(run_id)?;
    let mut run = load(Some(run_id))?;
    // Positions found through the one run reader, so a damaged run is refused
    // with the same sentence every reader gives.
    let (catalog_index, record_index) = {
        let catalogs = run_catalogs(&run)?;
        let catalog_index = catalogs
            .iter()
            .position(|entry| entry.name == catalog)
            .with_context(|| format!("{catalog}: crawl catalog disappeared"))?;
        let record_index = catalogs[catalog_index]
            .records
            .iter()
            .position(|entry| entry.get("record").and_then(Value::as_str) == Some(record))
            .with_context(|| format!("{catalog}/{record}: crawl record disappeared"))?;
        (catalog_index, record_index)
    };
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
