use super::*;

pub(crate) fn cancel(rest: &[String]) -> Result<()> {
    let mut run_id = None;
    let mut selected_record = None;
    let mut reason = None;
    let mut arguments = rest.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--run" => {
                run_id = Some(
                    crate::commands::required(arguments.next(), "--run needs a value")?.clone(),
                );
            }
            "--record" => {
                selected_record = Some(
                    crate::commands::required(arguments.next(), "--record needs a value")?.clone(),
                );
            }
            "--reason" => {
                reason = Some(
                    crate::commands::required(arguments.next(), "--reason needs a value")?.clone(),
                );
            }
            value => return Err(crate::commands::usage(format!("unknown argument: {value}"))),
        }
    }
    let run_id = run_id.context("--run is required")?;
    let reason = reason
        .filter(|value| !value.trim().is_empty())
        .context("--reason must name why the crawl is cancelled")?;
    {
        let _guard = RunMutationGuard::acquire(&run_id)?;
        let mut run = load(Some(&run_id))?;
        let before = run.clone();
        migrate_legacy_catalog_jobs(&mut run);
        if run != before {
            persist(&mut run)?;
        }
    }
    let snapshot = load(Some(&run_id))?;
    let mut targets = Vec::new();
    for catalog in run_catalogs(&snapshot)? {
        let catalog_name = catalog.name;
        for record in catalog.records {
            let record_name = record_name(catalog_name, record)?;
            if selected_matches(record_name, selected_record.as_deref()) {
                targets.push((catalog_name.to_string(), record_name.to_string()));
            }
        }
    }
    if targets.is_empty() {
        bail!("no crawl record matches the cancellation selection");
    }
    if selected_record.is_some() && !matches!(targets.as_slice(), [_]) {
        bail!("--record must resolve to exactly one retained crawl record");
    }
    for (catalog_name, record_name) in targets {
        let _record_guard = RecordMutationGuard::acquire(&run_id, &catalog_name, &record_name)?;
        let current = load(Some(&run_id))?;
        let record = run_record(&current, &catalog_name, &record_name)
            .context("crawl record disappeared during cancellation")?
            .clone();
        let manifest = record
            .get("manifest")
            .filter(|value| value.is_object())
            .cloned();
        let attempt_id = manifest
            .as_ref()
            .and_then(|value| value.get("attempt_id"))
            .or_else(|| record.get("attempt_id"))
            .and_then(Value::as_str)
            .context("crawl record has no attempt id to cancel")?
            .to_string();
        let attempt = manifest
            .as_ref()
            .and_then(|value| value.get("attempt"))
            .and_then(Value::as_u64)
            .or_else(|| record.get("attempt").and_then(Value::as_u64))
            .context("crawl record has no attempt number to cancel")?;
        let job_id = record
            .get("stado_job_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let stado_run_id = manifest
            .as_ref()
            .and_then(|value| value.get("stado_run_id"))
            .and_then(Value::as_str)
            .map(str::to_string);
        // A manifest that names no record_key leaves it null in the intent.
        let record_key = manifest
            .as_ref()
            .and_then(|value| value.get("record_key"))
            .cloned();
        let intent = json!({
            "schema": "wisent.crawl-cancel-intent.v1",
            "run_id": run_id,
            "catalog": catalog_name,
            "record": record_name,
            "record_key": record_key,
            "attempt": attempt,
            "attempt_id": attempt_id,
            "stado_run_id": stado_run_id,
            "stado_job_id": job_id,
            "reason": reason,
        });
        // Persist the durable local intent BEFORE any external effect. Every
        // submission path in `continue_record` refuses to submit, and refuses to
        // leave a submitted job running, once this object exists — so a crash
        // between here and the Stado cancellation can never resurrect the record.
        mutate_record(&run_id, &catalog_name, &record_name, |entry| {
            if !entry.get("cancel_intent").is_some_and(Value::is_object) {
                entry["cancel_intent"] = intent.clone();
            }
            if State::of(&entry["state"])?.before_submission() {
                entry["state"] = json!("cancelled");
                entry["diagnostic"] = json!({
                    "code": "cancelled_before_submission",
                    "message": "durable cancel intent recorded before any submission",
                });
            }
            Ok(())
        })?;
        let intent = record_snapshot(&run_id, &catalog_name, &record_name)?
            .get("cancel_intent")
            .cloned()
            .context("durable cancel intent disappeared after persistence")?;
        // The canonical attempt coordinate is the artifact URI's parent; a record
        // with no manifest has no attempt to cancel and was rejected above.
        let base_uri = manifest
            .as_ref()
            .and_then(|value| value.get("artifact_uri"))
            .and_then(Value::as_str)
            .and_then(|uri| uri.rsplit_once('/').map(|(parent, _)| parent.to_string()))
            .context("crawl record has no canonical attempt artifact coordinate")?;
        let intent_uri = format!("{base_uri}/cancel-intent.json");
        let intent_sha256 = publish_cancel_intent(&intent_uri, &intent)?;
        let job_id = job_id.as_deref();
        let status_before = match job_id {
            Some(job_id) => job_status(job_id),
            None => json!({"state": "not_submitted"}),
        };
        let (action, status_after) = if let Some(job_id) = job_id {
            // Stado's job record says whether the job has ended
            // (`terminal`); a job it holds that has not ended is cancelled.
            if status_before
                .get("job")
                .is_some_and(|job| !machine_terminal(job))
            {
                let output = command_output(
                    stado_command().args(["machine", "cancel", job_id]),
                    "cancel Stado machine job",
                )?;
                if !output.status.success() {
                    (
                        json!({
                            "state": "cancel_failed",
                            "stderr": String::from_utf8_lossy(&output.stderr).trim(),
                        }),
                        status_before.clone(),
                    )
                } else {
                    (json!({"state": "cancel_dispatched"}), job_status(job_id))
                }
            } else if status_before.get("job").is_some_and(machine_terminal) {
                (json!({"state": "already_terminal"}), status_before.clone())
            } else {
                (json!({"state": "not_cancellable"}), status_before.clone())
            }
        } else {
            (
                json!({"state": "cancelled_before_submission"}),
                status_before.clone(),
            )
        };
        let result = json!({
            "schema": "wisent.crawl-cancel-result.v1",
            "intent_uri": intent_uri,
            "intent_sha256": intent_sha256,
            "reason": reason,
            "status_before": status_before,
            "stado_action": action,
            "status_after": status_after,
            // The coordinator holds only secret *references*; the bearer is
            // injected by Stado into the pinned worker. Cancelling the Stado job
            // therefore terminates the process that owns the Weles task, and that
            // is the authoritative boundary rather than a coordinator-side API call.
            "weles_action": match record.get("weles_task_id").and_then(Value::as_str) {
                Some(task_id) => json!({
                    "state": "stado_job_cancellation_is_authoritative",
                    "weles_task_id": task_id,
                    "diagnostic": "the inner Weles task is owned by the cancelled Stado worker; the coordinator holds no admission bearer",
                }),
                None => json!({"state": "no_retained_task_id"}),
            },
        });
        let _guard = RunMutationGuard::acquire(&run_id)?;
        let mut run = load(Some(&run_id))?;
        let target = run
            .get_mut("catalogs")
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
            .find(|catalog| {
                catalog.get("catalog").and_then(Value::as_str) == Some(catalog_name.as_str())
            })
            .and_then(|catalog| catalog.get_mut("records"))
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
            .find(|record| {
                record.get("record").and_then(Value::as_str) == Some(record_name.as_str())
            })
            .context("crawl record disappeared before cancellation result persistence")?;
        target["cancel"] = result;
        let cancelled = target
            .pointer("/cancel/status_after/state")
            .and_then(Value::as_str)
            == Some("cancelled");
        if cancelled
            || target
                .pointer("/cancel/stado_action/state")
                .and_then(Value::as_str)
                == Some("cancelled_before_submission")
        {
            target["state"] = json!("cancelled");
        }
        if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
            for entry in entries {
                aggregate_catalog_entry(entry);
            }
        }
        update_run_state(&mut run);
        persist(&mut run)?;
    }
    let run = load(Some(&run_id))?;
    print_operation("cancel", &run, selected_record.as_deref())
}

/// What `stado machine status` says of `job_id`: its state and record, or,
/// when the lookup fails, whether Stado said the job does not exist and the
/// lookup's own diagnostic.
fn job_status(job_id: &str) -> Value {
    match machine_status(job_id) {
        Ok(job) => json!({"state": machine_state(&job), "job": job}),
        Err(error) => json!({
            "state": if error.not_found { "not_found" } else { "lookup_failed" },
            "diagnostic": error.diagnostic,
        }),
    }
}
