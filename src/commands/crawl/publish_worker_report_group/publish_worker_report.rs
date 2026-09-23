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
    let home = std::env::var_os("HOME").context("HOME is required for worker report cache")?;
    let directory = PathBuf::from(home)
        .join(".stado")
        .join("work")
        .join("spis")
        .join("worker-reports");
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
    let output = bounded_command_output(
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
        Duration::from_secs(120),
        4 * 1024 * 1024,
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
    let _ = std::fs::remove_file(&readback);
    let output = bounded_command_output(
        crawl_storage_command()
            .args(["storage", "get", &manifest.output_uri])
            .arg(&readback),
        "read back worker report",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        let _ = std::fs::remove_file(&readback);
        bail!(
            "stado storage get refused the published worker report: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let observed = std::fs::read(&readback);
    let _ = std::fs::remove_file(&readback);
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

pub(crate) struct RecordMutationGuard {
    pub(crate) file: File,
}

impl RecordMutationGuard {
    pub(crate) fn acquire(run_id: &str, catalog: &str, record: &str) -> Result<Self> {
        safe_component(run_id, "run id")?;
        safe_component(catalog, "catalog")?;
        safe_component(record, "record")?;
        let directory = run_root()?
            .join(run_id)
            .join("record-locks")
            .join(catalog);
        std::fs::create_dir_all(&directory)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(directory.join(format!("{record}.lock")))?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(RecordLockBusy {
                run_id: run_id.to_string(),
                catalog: catalog.to_string(),
                record: record.to_string(),
            }
            .into());
        }
        Ok(Self { file })
    }
}

impl Drop for RecordMutationGuard {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

pub(crate) struct RunMutationGuard {
    pub(crate) file: File,
}

impl RunMutationGuard {
    pub(crate) fn acquire(run_id: &str) -> Result<Self> {
        safe_component(run_id, "run id")?;
        let directory = run_root()?.join(run_id);
        std::fs::create_dir_all(&directory)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(directory.join(".mutation.lock"))?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            bail!("crawl run {run_id} is already being mutated by another process");
        }
        Ok(Self { file })
    }
}

impl Drop for RunMutationGuard {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
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
        snapshot.as_object_mut().map(|object| object.remove("attempts"));
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
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(parent.join(".run.json.lock"))?;
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        bail!("crawl run {run_id} is already being persisted");
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = parent.join(format!(
        ".run.json.{}.{}.tmp",
        std::process::id(),
        nonce
    ));
    let result = (|| -> Result<Value> {
        let expected = run
            .get("mutation_revision")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if path.is_file() {
            let current: Value =
                crate::read_json(path.to_str().context("run path is not UTF-8")?)?;
            let actual = current
                .get("mutation_revision")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            if actual != expected {
                bail!(
                    "crawl run {run_id} changed concurrently: expected revision {expected}, found {actual}"
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
    let _ = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) };
    drop(lock);
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    *run = result?;
    Ok(())
}
