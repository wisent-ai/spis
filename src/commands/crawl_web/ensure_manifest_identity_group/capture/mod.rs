use super::*;

mod completed;
mod product_url;
mod submission;

pub(crate) fn capture(
    manifest: &super::crawl::RuntimeManifest,
    wait_seconds: u64,
    attempt_root: &Path,
    private: &PrivateBridge,
    collected: &mut Collected,
) -> Outcome<()> {
    let catalog = manifest.catalog.as_str();
    let record = manifest.record.as_str();
    let identity = service_identity(manifest)?;
    confirm_service_release(&identity)?;
    let binding = attempt_binding(manifest, &identity)?;
    let base = attempt_base(&binding);

    let (product_url, parsed, origin) = product_url::checked_product_url(manifest)?;

    let reference_path = format!("{catalog}/references/{record}/reference.json");
    let reference: Value = crate::read_json(&reference_path)?;
    // The verifier re-derives the receipt origin from the record's own product_url, so an
    // attempt whose manifest URL differs from the committed record can never verify.
    let reference_url = reference
        .get("product_url")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            WorkerFailure::new(
                "web_reference_product_url_absent",
                "the committed record declares no product_url",
            )
        })?;
    ensure(
        url::Url::parse(reference_url).ok().as_ref() == Some(&parsed),
        "web_reference_product_url_mismatch",
        "the committed record product_url differs from the runtime manifest product URL",
    )?;
    let name = reference.get("name").and_then(Value::as_str).unwrap_or_default();
    let goal = reference
        .pointer("/journey/goal")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let objective = objective(catalog, name, goal)?;
    let constraints = task_constraints(&manifest.constraints)?;

    ensure(
        manifest.account.mode == "anonymous-read-only-probe"
            && manifest.account.credential_refs.is_empty(),
        "weles_anonymous_probe_required",
        "web crawls submit an anonymous read-only probe with no credential references",
    )?;

    // A pure function of the immutable attempt: a resubmitted identical attempt is exactly
    // idempotent at Weles, and never creates a second task.
    let idempotency_key = format!(
        "spis-{}",
        crate::sha256_hex(
            format!(
                "{}\0{}\0{}\0{}\0{}\0{}\0{}",
                manifest.source_revision,
                manifest.run_id,
                manifest.catalog,
                manifest.record,
                manifest.record_key,
                manifest.attempt,
                manifest.attempt_id,
            )
            .as_bytes()
        )
    );

    let organization_id = std::env::var("WISENT_ORGANIZATION_ID").map_err(|_| {
        WorkerFailure::new(
            "weles_delivery_env_absent",
            "WISENT_ORGANIZATION_ID was not delivered to this worker",
        )
    })?;
    let bearer = std::env::var("WELES_TOKEN").map_err(|_| {
        WorkerFailure::new(
            "weles_delivery_env_absent",
            "WELES_TOKEN was not delivered to this worker",
        )
    })?;
    ensure(
        !organization_id.trim().is_empty() && !bearer.trim().is_empty(),
        "weles_delivery_env_absent",
        "the delivered Weles organization and bearer must both be nonempty",
    )?;
    private.write_config(&identity.endpoint, &bearer, &organization_id)?;

    let identity_value = serde_json::to_value(&identity)?;
    let input = weles::WelesOfficialTaskInput {
        product_url: product_url.clone(),
        objective: objective.clone(),
        constraints: constraints.clone(),
        spis_binding: binding.clone(),
    };
    let justification = format!(
        "Spis anonymous browser-evidence capture for {catalog}/{record} attempt {} ({})",
        manifest.attempt, manifest.attempt_id
    );
    // The bridge normalizes `request` to exactly these six fields and injects `schema` and
    // `organizationId` itself before it computes the canonical request digest.
    let submit_command = json!({
        "schema": weles::BRIDGE_COMMAND_SCHEMA,
        "operation": "submit",
        "serviceIdentity": identity_value,
        "request": {
            "origin": origin,
            "action": weles::SPIS_WELES_ACTION,
            "input": serde_json::to_value(&input)?,
            "credentialRefs": [],
            "evidencePolicy": "full",
            "justification": justification,
        },
        "idempotencyKey": idempotency_key,
    });
    // The bridge persists the request-bound submission itself, so its destination is the
    // attempt root rather than the merged `weles/` subtree for the same reason
    // `retain_attempt_document` explains: this name is fixed and its bytes are unique to
    // this attempt. `weles/` is created by the content-addressed writes below.
    let submission_path = attempt_root.join("weles-submission.json");
    run_bridge(
        attempt_root,
        private,
        "submit",
        &submit_command,
        Some(&submission_path),
        true,
    )?;
    let submission = read_submission(&submission_path)?;
    collected.submission = Some(submission.clone());
    submission::check_submission(
        &submission,
        &organization_id,
        &origin,
        &idempotency_key,
        &identity,
        &binding,
        &product_url,
        &objective,
        &constraints,
    )?;
    let weles_task_id = submission.task_id.clone();

    let expected_task = json!({
        "taskId": weles_task_id,
        "organizationId": organization_id,
        "origin": origin,
        "action": weles::SPIS_WELES_ACTION,
    });
    let get_command = json!({
        "schema": weles::BRIDGE_COMMAND_SCHEMA,
        "operation": "get",
        "serviceIdentity": identity_value,
        "taskId": weles_task_id,
        "expectedTask": expected_task,
    });
    let deadline = Instant::now() + Duration::from_secs(wait_seconds);
    let status = loop {
        let stdout = run_bridge(attempt_root, private, "get", &get_command, None, true)?;
        let observed: weles::WelesTaskStatus = serde_json::from_slice(&stdout)?;
        if observed.terminal || Instant::now() >= deadline {
            break observed;
        }
        std::thread::sleep(POLL_INTERVAL);
    };
    retain_attempt_document(
        attempt_root,
        "weles-status.json",
        &serde_json::to_value(&status)?,
    )?;
    collected.status = Some(status.clone());
    ensure(
        status.schema == weles::TASK_STATUS_SCHEMA,
        "weles_status_invalid",
        "the retained task status does not declare the typed status schema",
    )?;
    ensure(
        status.task_id == weles_task_id,
        "weles_status_invalid",
        "the retained task status names a different Weles task",
    )?;
    ensure(
        status.request_identity == submission.request_identity,
        "weles_status_invalid",
        "the retained task status request identity differs from the submission",
    )?;
    ensure(
        status.service_identity == identity,
        "weles_status_invalid",
        "the retained task status service identity differs from the runtime directory",
    )?;
    if !status.terminal {
        // The wait budget is spent while the task is still live at Weles. Leaving it
        // running would hold a browser session and a leased worker for an attempt that
        // can no longer publish evidence, so the same bridge that submitted the task
        // cancels it and the typed cancellation is retained with the attempt.
        let cancellation = cancel_task(
            attempt_root,
            private,
            &identity,
            &identity_value,
            &expected_task,
            &submission,
            // A pure function of the immutable attempt, exactly like the key: Weles
            // refuses a second cancellation of the same task under a different reason,
            // so the wait budget is reported in the failure below rather than signed
            // into the cancellation.
            &format!(
                "Spis browser-evidence attempt {} ({}) exhausted its wait budget",
                manifest.attempt, manifest.attempt_id
            ),
            collected,
        )?;
        return Err(WorkerFailure::new(
            "weles_task_not_terminal",
            format!(
                "Weles task {weles_task_id} was still {} after {wait_seconds}s and was cancelled through the official bridge (cancel status {})",
                status.status, cancellation.status
            ),
        ));
    }
    let terminal_outcome = status
        .outcome
        .clone()
        .filter(|outcome| !outcome.trim().is_empty())
        .ok_or_else(|| {
            WorkerFailure::new(
                "weles_status_invalid",
                format!(
                    "Weles task {weles_task_id} reported status={} with no terminal outcome",
                    status.status
                ),
            )
        })?;
    if terminal_outcome != weles::SUCCESSFUL_OUTCOME {
        // A non-success is still a signed, delivered result, so the attempt fails carrying
        // the service's own proof of that failure rather than this worker's status text.
        let summary = format!(
            "Weles task {weles_task_id} finished as status={} outcome={terminal_outcome}",
            status.status
        );
        let provenance = retain_failure_provenance(
            manifest,
            attempt_root,
            private,
            &status,
            &submission,
            &binding,
            &identity,
            &base,
            &organization_id,
            &origin,
            &product_url,
            &weles_task_id,
            &terminal_outcome,
            collected,
        )
        .map_err(|mut failure| {
            failure.message = format!(
                "{summary}; its signed failure provenance could not be retained: {}",
                failure.message
            );
            failure
        })?;
        return Err(WorkerFailure::new(
            "weles_task_not_completed",
            format!(
                "{summary}; the signed failure provenance {} was retained with the attempt",
                provenance.id
            ),
        ));
    }
    completed::retain_completed(
        manifest,
        attempt_root,
        private,
        collected,
        &status,
        &submission,
        identity,
        binding,
        base,
        organization_id,
        origin,
        product_url,
        parsed,
        weles_task_id,
    )
}
