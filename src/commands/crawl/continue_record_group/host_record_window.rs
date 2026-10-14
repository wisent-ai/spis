use super::*;

/// The most bytes one record of this catalog has held on its host, measured:
/// the attempt tree its worker audited plus the archive published from it,
/// read off every record already imported. `None` until one has been.
fn measured_record_peak_bytes(records: &[Value]) -> Option<u64> {
    records
        .iter()
        .filter_map(|record| {
            let import = record.get("import")?;
            let tree = import.get("tree_bytes")?.as_u64()?;
            let archive = import.get("artifact_bytes")?.as_u64()?;
            tree.checked_add(archive)
        })
        .max()
}

/// How many records may be submitted against the free space this host just
/// reported, keeping one record's worth of it unspent.
///
/// The host answers `df -h` in its own preflight and the catalog's own
/// imported records say what one record costs, so the number of slots is
/// arithmetic on two measurements rather than a constant. An unreadable
/// answer yields no slots at all - the family waits and says why, instead of
/// finding out by filling a production disk. Before any record of the
/// catalog has been imported there is no measurement, so one record runs
/// alone and its import supplies the size every later window uses.
///
/// The reserved slot is the difference between planning to use the disk and
/// planning to use all of it: with free space for exactly two records this
/// admits one and leaves the other's worth for the host's own work.
pub(crate) fn host_record_window(host_report: &Value, records: &[Value]) -> usize {
    let Some(free_gib) = observed_free_gib(host_report) else {
        return 0;
    };
    let Some(peak_bytes) = measured_record_peak_bytes(records) else {
        return usize::from(free_gib > 0.0);
    };
    // GiB is 2^30 bytes, the unit `df -h` reports in.
    let peak_gib = peak_bytes as f64 / f64::from(1u32 << 30);
    let affordable = (free_gib / peak_gib).floor() - 1.0;
    affordable.max(0.0) as usize
}

/// Whether this record is holding a slot on the host right now: submitted and
/// not yet terminal, or mid-submission with a job that may already exist. A
/// record whose state is not a declared one is refused naming it.
pub(crate) fn record_occupies_host(record: &Value) -> Result<bool> {
    Ok(State::of(&record["state"])?.occupies_host())
}

pub(crate) fn continue_start(run_id: &str) -> Result<Value> {
    let catalogs = load(Some(run_id))?
        .get("catalogs")
        .and_then(Value::as_array)
        .cloned()
        .with_context(|| format!("run {run_id} records no catalogs list"))?;
    for catalog_entry in catalogs {
        let named = |field: &str| {
            catalog_entry
                .get(field)
                .and_then(Value::as_str)
                .map(str::to_string)
                .with_context(|| {
                    format!("run {run_id}: a catalog entry names no {field}: {catalog_entry}")
                })
        };
        let catalog = named("catalog")?;
        let engine = named("engine")?;
        // An entry `start` could place on no host records host null and has
        // nothing to continue; any other entry names its host. Every entry
        // carries its records list, read once here.
        let host = match catalog_entry.get("host") {
            Some(Value::Null) => continue,
            _ => named("host")?,
        };
        let records = catalog_entry
            .get("records")
            .and_then(Value::as_array)
            .cloned()
            .with_context(|| format!("run {run_id}: catalog {catalog} records no records list"))?;
        // A recorded identity that does not parse is refused by name: read as
        // "no identity", the host would be preflighted without the release
        // check the record asked for.
        let service_identity: Option<RuntimeServiceIdentity> = records
            .iter()
            .find_map(|record| record.pointer("/manifest/service_identity"))
            .filter(|value| value.is_object())
            .map(|value| {
                serde_json::from_value(value.clone()).with_context(|| {
                    format!("host {host}: a record's manifest service_identity is not a runtime service identity")
                })
            })
            .transpose()?;
        let host_report =
            ensure_host_preflight(run_id, &catalog, &engine, &host, service_identity.as_ref())?;
        let window = host_record_window(&host_report, &records);
        let occupancy = records
            .iter()
            .map(record_occupies_host)
            .collect::<Result<Vec<bool>>>()?;
        let mut occupied = occupancy.iter().filter(|occupies| **occupies).count();
        for (record, occupies) in records.into_iter().zip(occupancy) {
            let Some(record_name) = record.get("record").and_then(Value::as_str) else {
                continue;
            };
            if !occupies && occupied >= window {
                continue;
            }
            if !occupies {
                occupied += 1;
            }
            if let Err(error) = continue_record(run_id, &catalog, &host, &host_report, record_name)
            {
                if error.downcast_ref::<RecordLockBusy>().is_some() {
                    continue;
                }
                mark_record_failure(
                    run_id,
                    &catalog,
                    record_name,
                    "submission_failed",
                    "record_coordinator_failed",
                    format!("{error:#}"),
                )?;
            }
        }
    }
    load(Some(run_id))
}
