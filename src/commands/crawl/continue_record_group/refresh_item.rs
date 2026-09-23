use super::*;

pub(crate) fn refresh_item(entry: &mut Value) {
    let Some(job_id) = entry.get("stado_job_id").and_then(Value::as_str).map(str::to_string) else {
        return;
    };
    match machine_status(&job_id) {
        Ok(job) => {
            // The revision binding is proven at submission time:
            // `compact_submission` refuses a receipt whose repo_ref and
            // source_revision are not this exact build, and that receipt is
            // retained with the attempt. `stado machine status` does not report
            // repo_ref at all, so treating its absence as an observation made
            // `observed` the empty string, which never equals the expected
            // revision: every queued record was driven to a terminal
            // runtime_revision_mismatch on its first status refresh. The
            // comparison now happens only when Stado actually reports a
            // revision, and still fails loudly when it reports a different one.
            let expected = entry
                .pointer("/manifest/source_revision")
                .and_then(Value::as_str)
                .map(str::to_string);
            let observed = job
                .get("repo_ref")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            if let (Some(expected), Some(observed)) = (expected, observed) {
                if observed != expected {
                    entry["state"] = json!("failed");
                    entry["diagnostic"] = json!({
                        "code": "runtime_revision_mismatch",
                        "expected": expected,
                        "observed": observed,
                        "stado_job_id": job_id,
                    });
                    entry["job"] = job;
                    return;
                }
            }
            if entry.get("state").and_then(Value::as_str) != Some("imported") {
                entry["state"] = job.get("state").cloned().unwrap_or_else(|| json!("failed"));
            }
            entry["job"] = job;
            entry["lookup_error"] = Value::Null;
            if entry.get("state").and_then(Value::as_str) != Some("partial") {
                entry["diagnostic"] = Value::Null;
                entry["error"] = Value::Null;
            }
        }
        Err(error) if error.not_found => {
            entry["state"] = json!("lost");
            entry["lookup_error"] = error.diagnostic.clone();

            entry["diagnostic"] = json!({
                "code": "stado_job_not_found",
                "message": format!("Stado returned semantic NOT_FOUND for job {job_id}"),
                "stado_job_id": job_id,
            });
            entry["error"] = json!(format!("current Stado lookup confirmed that job {job_id} does not exist"));
            entry["job"] = Value::Null;
        }
        Err(error) => {
            entry["lookup_error"] = error.diagnostic;
        }
    }
}

pub(crate) fn refresh(run: &mut Value) {
    if let Some(catalogs) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in catalogs {
            if let Some(records) = entry.get_mut("records").and_then(Value::as_array_mut) {
                for record in records {
                    refresh_item(record);
                }
                aggregate_catalog_entry(entry);
            } else {
                refresh_item(entry);
            }
        }
    }
    run["updated_at"] = json!(crate::now_iso_utc());
    update_run_state(run);
}

pub(crate) fn migrate_legacy_catalog_jobs(run: &mut Value) {
    let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) else {
        return;
    };
    for entry in entries {
        let has_records = entry
            .get("records")
            .and_then(Value::as_array)
            .is_some_and(|records| !records.is_empty());
        let Some(job_id) = entry.get("job_id").and_then(Value::as_str).map(str::to_string) else {
            continue;
        };
        if has_records {
            continue;
        }
        let suffix = job_id
            .chars()
            .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
            .collect::<String>();
        let record = format!("legacy-job-{suffix}");
        entry["records"] = json!([{
            "record": record,
            "state": entry.get("state").cloned().unwrap_or_else(|| json!("unknown")),
            "stado_job_id": job_id,
            "attempt_id": format!("legacy-attempt-{suffix}"),
            "artifact_uri": entry.get("artifact_uri").cloned().unwrap_or(Value::Null),
            "output_uri": entry.get("output_uri").cloned().unwrap_or(Value::Null),
            "submission_receipt": entry.get("submission_receipt").cloned().unwrap_or(Value::Null),
            "diagnostic": {
                "code": "legacy_catalog_attempt_migrated",
                "message": "legacy catalog-level job retained as one explicit synthetic record attempt"
            },
            "attempts": [],
        }]);
    }
}

pub(crate) fn machine_state(job: &Value) -> &str {
    job.get("state")
        .or_else(|| job.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
}

pub(crate) fn terminal_machine_state(state: &str) -> bool {
    matches!(
        state,
        "completed"
            | "succeeded"
            | "failed"
            | "cancelled"
            | "canceled"
            | "lost"
            | "reaped"
    )
}

pub(crate) fn publish_cancel_intent(uri: &str, intent: &Value) -> Result<String> {
    let bytes = serde_json::to_vec(intent)?;
    let digest = crate::sha256_hex(&bytes);
    let home = std::env::var_os("HOME").context("HOME is required for private Stado work state")?;
    let directory = PathBuf::from(home)
        .join(".stado")
        .join("work")
        .join("spis")
        .join("cancel-intents");
    std::fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    }
    let source = directory.join(format!("{digest}.json"));
    if source.is_file() {
        if std::fs::read(&source)? != bytes {
            bail!("private cancel-intent cache conflicts with its content digest");
        }
    } else {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let temporary = directory.join(format!(".{digest}.{}.{}.tmp", std::process::id(), nonce));
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
        }
        std::fs::rename(&temporary, &source)?;
        File::open(&directory)?.sync_all()?;
    }
    let stored = crawl_storage_command()
        .args(["storage", "put", "--if-absent", "--content-type", "application/json", uri])
        .arg(&source)
        .output()
        .context("persist immutable crawl cancel intent")?;
    if !stored.status.success() {
        bail!(
            "Stado refused immutable cancel intent: {}",
            String::from_utf8_lossy(&stored.stderr).trim()
        );
    }
    let readback = directory.join(format!(".{digest}.{}.readback", std::process::id()));
    let output = crawl_storage_command()
        .args(["storage", "get", uri])
        .arg(&readback)
        .output()
        .context("read back immutable crawl cancel intent")?;
    if !output.status.success() {
        let _ = std::fs::remove_file(&readback);
        bail!(
            "cancel intent read-back failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let recovered = std::fs::read(&readback)?;
    let _ = std::fs::remove_file(&readback);
    if recovered != bytes || crate::sha256_hex(&recovered) != digest {
        bail!("cancel intent read-back differs from the exact requested cancellation");
    }
    Ok(digest)
}
