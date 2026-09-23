use super::*;

/// The most records of one catalog that may occupy a host at the same time,
/// whatever its disk says.
///
/// Measured on `charless-mac-mini` on 2026-09-06: ten records in flight took
/// the disk from 18.4 GiB to 0.1 GiB in fifty minutes, and three took it from
/// 11 GiB to 2.1 GiB in fifteen. The fleet's disk gate cannot prevent either -
/// it stops new claims below the watermark and has no say over the growth of
/// claims already running - and `--exclusive` prevents it only by demanding an
/// idle machine, which on a host that also carries release and qualification
/// work means the family never starts at all.
pub(crate) const HOST_RECORD_WINDOW: usize = 3;

/// What one record in flight costs the host at its peak, in GiB.
///
/// A record holds its whole crawl on the host until the attempt artifact is
/// published, and the object store keeps a same-disk backup twin of what it
/// then stores. Measured on `charless-mac-mini` on 2026-09-05 and 2026-09-06:
/// MDN, the largest site in the catalog, moved the disk from 17.7 to 13.8 GiB
/// while it ran and gave it back on import; three records together moved it
/// from 11 to 2.1 GiB. That is 3.9 GiB for the largest and about 3 GiB each
/// for a mixed three, so four is the largest record rounded up rather than the
/// average - the average is what filled the disk twice.
pub(crate) const RECORD_PEAK_GIB: f64 = 4.0;

/// How many records may be submitted against the free space this host just
/// reported, keeping one record's worth of it unspent.
///
/// A window that ignores the disk is a guess, and both guesses were wrong: one
/// starved the family behind foreign work, ten and then three killed the
/// machine. The host answers `df -h` in its own preflight, so the number of
/// slots is arithmetic rather than a constant, and an unreadable answer yields
/// no slots at all - the family waits and says why, instead of finding out by
/// filling a production disk.
///
/// The reserved slot is the difference between planning to use the disk and
/// planning to use all of it. With 12 GiB free and a 6 GiB peak the naive
/// division admits two records whose combined peak is exactly the whole
/// volume; this admits one and leaves the other 6 GiB for the host's own work,
/// which on `charless-mac-mini` is a release store, a queue, and everyone
/// else's jobs.
pub(crate) fn host_record_window(host_report: &Value) -> usize {
    let Some(free_gib) = observed_free_gib(host_report) else {
        return 0;
    };
    let affordable = (free_gib / RECORD_PEAK_GIB).floor() - 1.0;
    (affordable.max(0.0) as usize).min(HOST_RECORD_WINDOW)
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
        let window = host_record_window(&host_report);
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
