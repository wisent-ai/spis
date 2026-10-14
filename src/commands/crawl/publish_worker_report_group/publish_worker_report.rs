use super::*;

/// Publish the worker's typed terminal report independently of the queue
/// agent's best-effort stdout retention.
///
/// The queue receipt commits `output_uri` before execution, but a local agent
/// can finish the workload after failing to persist its captured stdout. The
/// worker already owns the crawl namespace bearer because it publishes the
/// attempt archive there, so it also writes and reads back the one report line
/// the importer treats as the terminal receipt. Stdout remains useful for
/// humans and queue diagnostics, but is no longer the only copy of proof.
pub(crate) fn publish_worker_report(manifest: &RuntimeManifest, report: &Value) -> Result<()> {
    if !manifest
        .output_uri
        .starts_with(&format!("{}/", crate::CRAWL_ATTEMPT_ROOT))
        || !manifest.output_uri.ends_with("/worker-output.log")
    {
        bail!("worker report URI is outside the canonical Spis attempt coordinates");
    }
    let directory = crate::commands::docs_corpus::work_area("worker-reports")?;
    std::fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    }
    let source = directory.join(format!("{}.log", manifest.attempt_id));
    let bytes = (serde_json::to_string(report)? + "\n").into_bytes();
    write_immutable_file(&source, &bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o600))?;
    }
    let output = command_output(
        crawl_storage_command()
            .args([
                "storage",
                "put",
                "--if-absent",
                "--content-type",
                "application/x-ndjson",
                &manifest.output_uri,
            ])
            .arg(&source),
        "publish worker report",
    )?;
    if !output.status.success() {
        bail!(
            "stado storage put refused the worker report: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let readback = directory.join(format!(
        ".{}.{}.readback",
        manifest.attempt_id,
        std::process::id()
    ));
    crate::discard(&readback)?;
    let output = crate::discarding(
        command_output(
            crawl_storage_command()
                .args(["storage", "get", &manifest.output_uri])
                .arg(&readback),
            "read back worker report",
        ),
        &readback,
    )?;
    if !output.status.success() {
        return crate::discarding(
            Err(anyhow::anyhow!(
                "stado storage get refused the published worker report: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )),
            &readback,
        );
    }
    let observed = std::fs::read(&readback);
    crate::discard(&readback)?;
    if observed? != bytes {
        bail!("published worker report read-back differs from the typed report");
    }
    Ok(())
}

#[derive(Debug)]
pub(crate) struct RecordLockBusy {
    pub(crate) run_id: String,
    pub(crate) catalog: String,
    pub(crate) record: String,
}

impl std::fmt::Display for RecordLockBusy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}/{}/{} is already being mutated",
            self.run_id, self.catalog, self.record
        )
    }
}

impl std::error::Error for RecordLockBusy {}

/// Held for as long as one record of a crawl run is mutated; dropping it
/// closes the lock file, which releases the lock.
pub(crate) struct RecordMutationGuard {
    _held: File,
}

impl RecordMutationGuard {
    pub(crate) fn acquire(run_id: &str, catalog: &str, record: &str) -> Result<Self> {
        safe_component(run_id, "run id")?;
        safe_component(catalog, "catalog")?;
        safe_component(record, "record")?;
        let directory = run_root()?.join(run_id).join("record-locks").join(catalog);
        std::fs::create_dir_all(&directory)?;
        let Some(held) = crate::exclusive(&directory.join(format!("{record}.lock")))? else {
            return Err(RecordLockBusy {
                run_id: run_id.to_string(),
                catalog: catalog.to_string(),
                record: record.to_string(),
            }
            .into());
        };
        Ok(Self { _held: held })
    }
}

/// Held for as long as a crawl run is mutated; dropping it closes the lock
/// file, which releases the lock.
pub(crate) struct RunMutationGuard {
    _held: File,
}

impl RunMutationGuard {
    pub(crate) fn acquire(run_id: &str) -> Result<Self> {
        safe_component(run_id, "run id")?;
        let directory = run_root()?.join(run_id);
        std::fs::create_dir_all(&directory)?;
        let Some(held) = crate::exclusive(&directory.join(".mutation.lock"))? else {
            bail!("crawl run {run_id} is already being mutated by another process");
        };
        Ok(Self { _held: held })
    }
}

pub(crate) fn sync_attempt_history(run: &mut Value) {
    let Some(catalogs) = run.get_mut("catalogs").and_then(Value::as_array_mut) else {
        return;
    };
    for record in catalogs
        .iter_mut()
        .filter_map(|catalog| catalog.get_mut("records").and_then(Value::as_array_mut))
        .flatten()
    {
        let attempt_id = record
            .get("manifest")
            .and_then(|manifest| manifest.get("attempt_id"))
            .and_then(Value::as_str)
            .or_else(|| record.get("attempt_id").and_then(Value::as_str))
            .map(str::to_string)
            .or_else(|| {
                record
                    .get("stado_job_id")
                    .and_then(Value::as_str)
                    .map(|job| format!("legacy-job-{job}"))
            });
        let Some(attempt_id) = attempt_id else {
            continue;
        };
        let mut snapshot = record.clone();
        snapshot
            .as_object_mut()
            .map(|object| object.remove("attempts"));
        snapshot["attempt_id"] = json!(attempt_id);
        let attempts = record
            .as_object_mut()
            .expect("crawl record must be an object")
            .entry("attempts")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .expect("crawl record attempts must be an array");
        if let Some(existing) = attempts.iter_mut().find(|attempt| {
            attempt.get("attempt_id").and_then(Value::as_str) == Some(attempt_id.as_str())
        }) {
            *existing = snapshot;
        } else {
            attempts.push(snapshot);
        }
    }
}

pub(crate) fn persist(run: &mut Value) -> Result<()> {
    let run_id = run
        .get("run_id")
        .and_then(Value::as_str)
        .context("run has no run_id")?;
    let path = run_path(run_id)?;
    let parent = path.parent().context("crawl run path has no parent")?;
    std::fs::create_dir_all(parent)?;
    let Some(lock) = crate::exclusive(&parent.join(".run.json.lock"))? else {
        bail!("crawl run {run_id} is already being persisted");
    };
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = parent.join(format!(".run.json.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<Value> {
        let expected = run
            .get("mutation_revision")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if path.is_file() {
            let current: Value = crate::read_json(path.to_str().context("run path is not UTF-8")?)?;
            // The revision on disk and the one this run was read at are
            // compared as each states it: a run written before revisions
            // existed states none on either side and still matches, and one
            // side stating none while the other states one is a change.
            let stated =
                |document: &Value| document.get("mutation_revision").and_then(Value::as_u64);
            if stated(&current) != stated(run) {
                bail!(
                    "crawl run {run_id} changed concurrently: expected revision {:?}, found {:?}",
                    stated(run),
                    stated(&current)
                );
            }
        } else if expected != 0 {
            bail!("crawl run {run_id} disappeared before revision {expected} could be persisted");
        }
        let mut staged = run.clone();
        sync_attempt_history(&mut staged);
        staged["mutation_revision"] = json!(expected + 1);
        staged["updated_at"] = json!(crate::now_iso_utc());
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        output.write_all((serde_json::to_string_pretty(&staged)? + "\n").as_bytes())?;
        output.sync_all()?;
        std::fs::rename(&temporary, &path)?;
        File::open(parent)?.sync_all()?;
        Ok(staged)
    })();
    drop(lock);
    *run = crate::discarding(result, &temporary)?;
    Ok(())
}
