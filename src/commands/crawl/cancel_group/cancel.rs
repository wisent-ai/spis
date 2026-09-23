use super::*;

pub(crate) fn cancel(rest: &[String]) -> Result<()> {
    let mut run_id = None;
    let mut selected_record = None;
    let mut reason = None;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--run" => {
                index += 1;
                run_id = Some(rest.get(index).context("--run needs a value")?.clone());
            }
            "--record" => {
                index += 1;
                selected_record = Some(rest.get(index).context("--record needs a value")?.clone());
            }
            "--reason" => {
                index += 1;
                reason = Some(rest.get(index).context("--reason needs a value")?.clone());
            }
            value => bail!("unknown argument: {value}"),
        }
        index += 1;
    }
    let run_id = run_id.context("--run is required")?;
    let reason = reason
        .filter(|value| !value.trim().is_empty() && value.len() <= 1024)
        .context("--reason must be nonempty and at most 1024 bytes")?;
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
    for catalog in snapshot
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let catalog_name = catalog
            .get("catalog")
            .and_then(Value::as_str)
            .unwrap_or_default();
        for record in catalog
            .get("records")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let record_name = record
                .get("record")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if selected_record.as_deref().is_none_or(|wanted| {
                record_name == wanted
                    || record_name.split_once('-').map(|(_, tail)| tail) == Some(wanted)
            }) {
                targets.push((catalog_name.to_string(), record_name.to_string()));
            }
        }
    }
    if targets.is_empty() {
        bail!("no crawl record matches the cancellation selection");
    }
    if selected_record.is_some() && targets.len() != 1 {
        bail!("--record must resolve to exactly one retained crawl record");
    }
    for (catalog_name, record_name) in targets {
        let _record_guard =
            RecordMutationGuard::acquire(&run_id, &catalog_name, &record_name)?;
        let current = load(Some(&run_id))?;
        let record = current
            .get("catalogs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|catalog| {
                catalog.get("catalog").and_then(Value::as_str) == Some(catalog_name.as_str())
            })
            .and_then(|catalog| catalog.get("records"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|record| {
                record.get("record").and_then(Value::as_str) == Some(record_name.as_str())
            })
            .cloned()
            .context("crawl record disappeared during cancellation")?;
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
        let record_key = manifest
            .as_ref()
            .and_then(|value| value.get("record_key"))
            .cloned()
            .unwrap_or(Value::Null);
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
            if matches!(
                entry.get("state").and_then(Value::as_str),
                Some("planned" | "preflighting" | "preflight_passed" | "unavailable")
            ) {
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
            Some(job_id) => match machine_status(job_id) {
                Ok(job) => json!({"state": machine_state(&job), "job": job}),
                Err(error) => json!({
                    "state": if error.not_found { "not_found" } else { "lookup_failed" },
                    "diagnostic": error.diagnostic,
                }),
            },
            None => json!({"state": "not_submitted"}),
        };
        let before_state = status_before
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("lookup_failed");
        let (action, status_after) = if let Some(job_id) = job_id {
            if matches!(before_state, "queued" | "running") {
                let output = stado_command()
                    .args(["machine", "cancel", job_id])
                    .output()
                    .context("cancel Stado machine job")?;
                if !output.status.success() {
                    (
                        json!({
                            "state": "cancel_failed",
                            "stderr": String::from_utf8_lossy(&output.stderr).trim(),
                        }),
                        status_before.clone(),
                    )
                } else {
                    let after = machine_status(job_id)
                        .map(|job| json!({"state": machine_state(&job), "job": job}))
                        .unwrap_or_else(|error| {
                            json!({
                                "state": if error.not_found { "not_found" } else { "lookup_failed" },
                                "diagnostic": error.diagnostic,
                            })
                        });
                    (json!({"state": "cancel_dispatched"}), after)
                }
            } else if terminal_machine_state(before_state) {
                (json!({"state": "already_terminal"}), status_before.clone())
            } else {
                (json!({"state": "not_cancellable"}), status_before.clone())
            }
        } else {
            (json!({"state": "cancelled_before_submission"}), status_before.clone())
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
        let final_state = target
            .pointer("/cancel/status_after/state")
            .and_then(Value::as_str)
            .unwrap_or("lookup_failed");
        if matches!(final_state, "cancelled" | "canceled")
            || target.pointer("/cancel/stado_action/state").and_then(Value::as_str)
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
