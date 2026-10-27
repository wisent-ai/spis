use super::*;

pub(crate) fn refresh_item(entry: &mut Value) {
    let Some(job_id) = entry
        .get("stado_job_id")
        .and_then(Value::as_str)
        .map(str::to_string)
    else {
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
            // A record not yet imported takes Stado's job state; an
            // unreadable record state is replaced by it, as it is not imported.
            if !matches!(State::of(&entry["state"]), Ok(State::Imported)) {
                match job.get("state") {
                    Some(state) => entry["state"] = state.clone(),
                    // A job that reports no state leaves the record's state
                    // as it was and says so; it had been recorded as failed.
                    None => {
                        entry["diagnostic"] = json!({
                            "code": "stado_job_state_unreported",
                            "message": format!("Stado reported job {job_id} without a state"),
                            "stado_job_id": job_id,
                        });
                        entry["job"] = job;
                        return;
                    }
                }
            }
            entry["job"] = job;
            entry["lookup_error"] = Value::Null;
            if !matches!(State::of(&entry["state"]), Ok(State::Partial)) {
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
            entry["error"] = json!(format!(
                "current Stado lookup confirmed that job {job_id} does not exist"
            ));
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
        let Some(job_id) = entry
            .get("job_id")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
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
            "state": match entry.get("state") {
                Some(state) => state.clone(),
                None => json!(STATE_NOT_REPORTED),
            },
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

/// What a record or job reads as when Stado's answer carries neither `state`
/// nor `status`: named as such, so it is neither cancellable nor terminal
/// and every report says the state was not reported instead of guessing.
pub(crate) const STATE_NOT_REPORTED: &str = "state_not_reported";

pub(crate) fn machine_state(job: &Value) -> &str {
    match job
        .get("state")
        .or_else(|| job.get("status"))
        .and_then(Value::as_str)
    {
        Some(state) => state,
        None => STATE_NOT_REPORTED,
    }
}

/// Whether Stado's job record says the job has ended: its own `terminal`
/// flag, read from `stado machine status`, never Spis's guess from the
/// state's word. A record that does not state it has not ended.
pub(crate) fn machine_terminal(job: &Value) -> bool {
    job.get("terminal").and_then(Value::as_bool) == Some(true)
}

pub(crate) fn publish_cancel_intent(uri: &str, intent: &Value) -> Result<String> {
    let bytes = serde_json::to_vec(intent)?;
    let digest = crate::sha256_hex(&bytes);
    let directory = crate::commands::docs_corpus::work_area("cancel-intents")?;
    std::fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Owner read, write and search only (POSIX S_IRWXU).
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(u32::from(libc::S_IRWXU)))?;
    }
    let source = directory.join(format!("{digest}.json"));
    if source.is_file() {
        if std::fs::read(&source)? != bytes {
            bail!("private cancel-intent cache conflicts with its content digest");
        }
    } else {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let temporary = directory.join(format!(".{digest}.{}.{}.tmp", std::process::id(), nonce));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // Owner read and write only (POSIX S_IRUSR | S_IWUSR).
            std::fs::set_permissions(
                &temporary,
                std::fs::Permissions::from_mode(u32::from(libc::S_IRUSR | libc::S_IWUSR)),
            )?;
        }
        std::fs::rename(&temporary, &source)?;
        File::open(&directory)?.sync_all()?;
    }
    let stored = stado_wait::output(
        crawl_storage_command()
            .args([
                "storage",
                "put",
                "--if-absent",
                "--content-type",
                "application/json",
                uri,
            ])
            .arg(&source),
    )
    .context("persist immutable crawl cancel intent")?;
    if !stored.status.success() {
        bail!(
            "Stado refused immutable cancel intent: {}",
            String::from_utf8_lossy(&stored.stderr).trim()
        );
    }
    let readback = directory.join(format!(".{digest}.{}.readback", std::process::id()));
    let output = stado_wait::output(
        crawl_storage_command()
            .args(["storage", "get", uri])
            .arg(&readback),
    )
    .context("read back immutable crawl cancel intent")?;
    if !output.status.success() {
        return crate::discarding(
            Err(anyhow::anyhow!(
                "cancel intent read-back failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )),
            &readback,
        );
    }
    let recovered = crate::discarding(
        std::fs::read(&readback).map_err(anyhow::Error::from),
        &readback,
    )?;
    crate::discard(&readback)?;
    if recovered != bytes || crate::sha256_hex(&recovered) != digest {
        bail!("cancel intent read-back differs from the exact requested cancellation");
    }
    Ok(digest)
}
