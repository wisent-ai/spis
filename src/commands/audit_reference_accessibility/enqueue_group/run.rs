use super::*;

// ---------------------------------------------------------------------------

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalogs_arg: Vec<String> = Vec::new();
    let mut records: Option<String> = None;
    let mut batch_arg: Option<String> = None;
    let mut target = DEFAULT_TARGET.to_string();
    let mut plan_arg: Option<String> = None;
    let mut dry_run = false;
    let mut poll_seconds: u64 = 15;
    let mut timeout_minutes: u64 = 120;

    let mut i = 0usize;
    while i < rest.len() {
        match rest[i].as_str() {
            "--catalog" => {
                i += 1;
                catalogs_arg.push(rest.get(i).context("--catalog needs a value")?.clone());
            }
            "--records" => {
                i += 1;
                records = Some(rest.get(i).context("--records needs a value")?.clone());
            }
            "--batch" => {
                i += 1;
                batch_arg = Some(rest.get(i).context("--batch needs a value")?.clone());
            }
            "--target" => {
                i += 1;
                target = rest.get(i).context("--target needs a value")?.clone();
            }
            "--plan" => {
                i += 1;
                plan_arg = Some(rest.get(i).context("--plan needs a value")?.clone());
            }
            "--dry-run" => dry_run = true,
            "--poll-seconds" => {
                i += 1;
                poll_seconds = rest
                    .get(i)
                    .context("--poll-seconds needs a value")?
                    .parse()?;
            }
            "--timeout-minutes" => {
                i += 1;
                timeout_minutes = rest
                    .get(i)
                    .context("--timeout-minutes needs a value")?
                    .parse()?;
            }
            "--help" | "-h" => {
                println!("usage: spis audit-reference-accessibility [--catalog NAME]... [--records SEL] [--batch ID] [--target HOST] [--plan PATH] [--dry-run] [--poll-seconds N] [--timeout-minutes N]");
                return Ok(());
            }
            other => bail!("unknown argument: {other}"),
        }
        i += 1;
    }

    let log = |line: &str| {
        eprintln!("{line}");
        use std::io::Write;
        let _ = std::io::stderr().flush();
    };

    // ---- planning phase: failures exit 2 --------------------------------
    let plan: Map<String, Value>;
    let plan_path: PathBuf;
    let references: Vec<Reference>;
    let batch: String;
    {
        let outcome: Result<(Map<String, Value>, PathBuf, Vec<Reference>, String)> = (|| {
            if poll_seconds < 1 {
                bail!("--poll-seconds: must be at least 1");
            }
            if timeout_minutes < 1 {
                bail!("--timeout-minutes: must be at least 1");
            }
            let sources: Vec<String> = if catalogs_arg.is_empty() {
                DEFAULT_CATALOGS.iter().map(|s| s.to_string()).collect()
            } else {
                catalogs_arg.clone()
            };
            let mut catalogs: Vec<String> = Vec::new();
            for value in &sources {
                let normalized = normalize_catalog(value)?;
                if !catalogs.contains(&normalized) {
                    catalogs.push(normalized);
                }
            }
            let selection = parse_record_selection(records.as_ref())?;
            let refs = load_references(&catalogs, selection.as_ref())?;
            if refs.is_empty() {
                bail!("selection: no records selected");
            }
            let batch = batch_arg.clone().unwrap_or_else(default_batch);
            let captures: Vec<Value> = refs
                .iter()
                .map(|r| Value::Object(r.action(&batch)))
                .collect();
            let plan = serde_json::json!({
                "schema": PLAN_SCHEMA,
                "batch": batch,
                "target": target,
                "captures": captures,
            });
            let validated = validate_plan(&plan, &target, &refs)?;
            let plan_path = plan_path_for(plan_arg.as_ref(), &batch)?;
            atomic_json(&plan_path, &Value::Object(validated.clone()))?;
            let text = std::fs::read_to_string(&plan_path)
                .with_context(|| "plan: unreadable after write")?;
            let parsed = strict_json(&text, "plan")?;
            let validated_again = validate_plan(&parsed, &target, &refs)?;
            Ok((validated_again, plan_path, refs, batch))
        })();
        match outcome {
            Ok(value) => {
                plan = value.0;
                plan_path = value.1;
                references = value.2;
                batch = value.3;
            }
            Err(e) => {
                eprintln!("error: {e:#}");
                std::process::exit(2);
            }
        }
    }

    if dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&Value::Object(plan.clone()))?
        );
        println!(
            "dry run: planned={} complete=0 failed=0 pending={}; no host was contacted; plan={}",
            references.len(),
            references.len(),
            plan_path.display()
        );
        return Ok(());
    }

    let captures = plan
        .get("captures")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut rows: Vec<Value> = references
        .iter()
        .zip(captures.iter())
        .map(|(reference, capture)| initial_row(reference, capture.as_object().unwrap()))
        .collect();
    let mut verifier_errors: Vec<String> = Vec::new();

    // ---- enqueue + poll: failures mark every row and exit 2 -------------
    let (ids, states): (Vec<String>, std::collections::HashMap<String, Value>) = {
        let outcome: Result<(
            String,
            Vec<String>,
            std::collections::HashMap<String, Value>,
        )> = (|| {
            let (enqueued_batch, ids) = enqueue(&target, &plan_path, &plan)?;
            log(&format!(
                "batch {enqueued_batch}: {} {ACTION} actions enqueued",
                ids.len()
            ));
            let id_set: HashSet<String> = ids.iter().cloned().collect();
            let states = poll(
                &target,
                &enqueued_batch,
                &id_set,
                poll_seconds,
                timeout_minutes * 60,
                &log,
            )?;
            Ok((enqueued_batch, ids, states))
        })();
        match outcome {
            Ok((_, ids, states)) => (ids, states),
            Err(e) => {
                let reason = format!("{e:#}");
                for row in rows.iter_mut() {
                    if let Some(obj) = row.as_object_mut() {
                        obj.insert("reason".into(), Value::String(reason.clone()));
                    }
                }
                let payload = write_index(&rows, &batch, &target, &plan_path, &verifier_errors)?;
                let totals = payload.get("totals").cloned().unwrap_or(Value::Null);
                log(&format!("error: {reason}"));
                log(&format!("{INDEX}: {totals}"));
                std::process::exit(2);
            }
        }
    };

    // ---- per-reference retrieve -----------------------------------------
    let mut completed_catalogs: Vec<String> = Vec::new();
    for position in 0..references.len() {
        let reference = &references[position];
        let action = captures[position].as_object().cloned().unwrap_or_default();
        let identifier = &ids[position];
        let Some(state_row) = states.get(identifier) else {
            if let Some(obj) = rows[position].as_object_mut() {
                obj.insert(
                    "reason".into(),
                    Value::String(format!(
                        "action {identifier} had not reported a state when polling stopped"
                    )),
                );
            }
            continue;
        };
        let state = state_of(state_row);
        if state != "done" {
            let exact_error = match pick(state_row, &["error", "message", "reason"]) {
                Some(error) => {
                    let text = error.to_string();
                    let trimmed = text.trim_matches('"').to_string();
                    if trimmed.is_empty() {
                        format!("action ended in state {state}")
                    } else {
                        trimmed
                    }
                }
                None => format!("action ended in state {state}"),
            };
            if let Some(obj) = rows[position].as_object_mut() {
                obj.insert("status".into(), Value::String("failed".into()));
                obj.insert("reason".into(), Value::String(exact_error.clone()));
            }
            log(&format!(
                "FAILED {}: action.error: {exact_error}",
                reference.id()
            ));
            continue;
        }
        match retrieve(reference, &action, state_row, &batch) {
            Ok(result_row) => {
                rows[position] = result_row;
                if !completed_catalogs.contains(&reference.catalog) {
                    completed_catalogs.push(reference.catalog.clone());
                }
                let raw_path = rows[position]
                    .get("raw_path")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                log(&format!("COMPLETE {}: {raw_path}", reference.id()));
            }
            Err(e) => {
                let reason = format!("{e:#}");
                if let Some(obj) = rows[position].as_object_mut() {
                    obj.insert("status".into(), Value::String("failed".into()));
                    obj.insert("reason".into(), Value::String(reason.clone()));
                }
                log(&format!("REFUSED {reason}"));
            }
        }
    }

    completed_catalogs.sort();
    for catalog in &completed_catalogs {
        match run_verifier(catalog) {
            Ok(()) => log(&format!("verified {catalog} with --apply --no-state-match")),
            Err(e) => {
                verifier_errors.push(format!("{e:#}"));
                log(&format!("error: {e:#}"));
            }
        }
    }

    let payload = write_index(&rows, &batch, &target, &plan_path, &verifier_errors)?;
    let totals = payload.get("totals").cloned().unwrap_or(Value::Null);
    let total = |key: &str| totals.get(key).and_then(Value::as_u64).unwrap_or(0);
    log(&format!(
        "{INDEX}: planned={} complete={} failed={} pending={}",
        total("planned"),
        total("complete"),
        total("failed"),
        total("pending")
    ));
    if total("failed") == 0 && total("pending") == 0 && verifier_errors.is_empty() {
        Ok(())
    } else {
        std::process::exit(3);
    }
}
