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
/// not yet terminal, or mid-submission with a job that may already exist.
pub(crate) fn record_occupies_host(record: &Value) -> bool {
    matches!(
        record.get("state").and_then(Value::as_str),
        Some("preflight_passed" | "submitting" | "queued" | "running")
    )
}

pub(crate) fn continue_start(run_id: &str) -> Result<Value> {
    let catalogs = load(Some(run_id))?
        .get("catalogs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for catalog_entry in catalogs {
        let catalog = catalog_entry
            .get("catalog")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let engine = catalog_entry
            .get("engine")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let host = catalog_entry
            .get("host")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if host.is_empty() {
            continue;
        }
        let service_identity: Option<RuntimeServiceIdentity> = catalog_entry
            .get("records")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find_map(|record| record.pointer("/manifest/service_identity"))
            .filter(|value| value.is_object())
            .and_then(|value| serde_json::from_value(value.clone()).ok());
        let host_report = ensure_host_preflight(
            run_id,
            &catalog,
            &engine,
            &host,
            service_identity.as_ref(),
        )?;
        let records = catalog_entry
            .get("records")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let window = host_record_window(&host_report, &records);
        let mut occupied = records.iter().filter(|record| record_occupies_host(record)).count();
        for record in records {
            let Some(record_name) = record.get("record").and_then(Value::as_str) else {
                continue;
            };
            let occupies = record_occupies_host(&record);
            if !occupies && occupied >= window {
                continue;
            }
            if !occupies {
                occupied += 1;
            }
            if let Err(error) = continue_record(
                run_id,
                &catalog,
                &host,
                &host_report,
                record_name,
            ) {
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
