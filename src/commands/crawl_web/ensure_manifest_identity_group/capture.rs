use super::*;

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

    // `validate_request_and_evidence_manifest` binds every retained URL to the exact
    // product URL of the current record, so the manifest URL must already be canonical.
    let product_url = manifest.runtime_product.product_url.clone();
    let parsed = url::Url::parse(&product_url).map_err(|_| {
        WorkerFailure::new(
            "web_product_url_invalid",
            "the runtime product URL is not a URL",
        )
    })?;
    ensure(
        matches!(parsed.scheme(), "http" | "https")
            && parsed.username().is_empty()
            && parsed.password().is_none(),
        "web_product_url_invalid",
        "the runtime product URL must be HTTP(S) without credentials",
    )?;
    ensure(
        parsed.as_str() == product_url,
        "web_product_url_invalid",
        "the runtime product URL is not in canonical serialized form",
    )?;
    let origin = parsed.origin().ascii_serialization();
    ensure(
        !origin.is_empty() && origin != "null",
        "web_product_url_invalid",
        "the runtime product URL has an opaque origin",
    )?;
    if let Some(surface) = manifest.runtime_product.surface.as_ref() {
        ensure(
            surface.exact_url == product_url && surface.origin == origin,
            "web_surface_identity_mismatch",
            "the runtime surface identity does not name the exact product URL and origin",
        )?;
        ensure(
            surface
                .allowed_actions
                .iter()
                .any(|action| action == weles::SPIS_WELES_ACTION),
            "web_surface_identity_mismatch",
            "the runtime surface identity does not allow the Spis browser action",
        )?;
    }

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
    ensure(
        submission.schema == weles::SUBMISSION_SCHEMA,
        "weles_submission_invalid",
        "the retained submission does not declare the typed submission schema",
    )?;
    ensure(
        submission.organization_id == organization_id
            && submission.origin == origin
            && submission.action == weles::SPIS_WELES_ACTION,
        "weles_submission_invalid",
        "the retained submission task identity differs from the submitted request",
    )?;
    ensure(
        submission.idempotency_key == idempotency_key,
        "weles_submission_invalid",
        "the retained submission carries a different idempotency key",
    )?;
    ensure(
        submission.service_identity == identity,
        "weles_submission_invalid",
        "the retained submission service identity differs from the runtime directory",
    )?;
    ensure(
        is_sha256_id(&submission.request_digest)
            && submission.request_identity.request_digest == submission.request_digest,
        "weles_submission_invalid",
        "the retained submission request digest is not a bound sha256: identifier",
    )?;
    ensure(
        submission.request_identity.spis_binding == binding
            && submission.request_document.input.spis_binding == binding,
        "weles_submission_invalid",
        "the retained submission does not carry the exact signed Spis binding",
    )?;
    ensure(
        submission.request_document.schema == OFFICIAL_REQUEST_SCHEMA
            && submission.request_document.organization_id == organization_id
            && submission.request_document.origin == origin
            && submission.request_document.action == weles::SPIS_WELES_ACTION,
        "weles_submission_invalid",
        "the retained official request is not the canonical current Weles task",
    )?;
    ensure(
        submission.request_document.credential_refs.is_empty()
            && submission.request_document.evidence_policy == "full",
        "weles_submission_invalid",
        "the retained official request is not an anonymous full-evidence request",
    )?;
    ensure(
        submission.request_document.input.product_url == product_url
            && submission.request_document.input.objective == objective
            && submission.request_document.input.constraints == constraints,
        "weles_submission_invalid",
        "the retained official request input differs from the submitted browser task",
    )?;
    ensure(
        is_portable_component(&submission.task_id),
        "weles_task_id_invalid",
        "the Weles task identifier is not a portable recording component",
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
    let checkpoint = status.receipt_checkpoint.as_ref().ok_or_else(|| {
        WorkerFailure::new(
            "weles_receipt_checkpoint_absent",
            "a completed Weles task must carry a freshly verified receipt checkpoint",
        )
    })?;
    let result_digest = status
        .result_digest
        .clone()
        .filter(|digest| is_sha256_id(digest))
        .ok_or_else(|| {
            WorkerFailure::new(
                "weles_status_invalid",
                "the completed task carries no sha256: result digest",
            )
        })?;
    // The public task-status contract returns the task identity, the request identity,
    // the outcome, the result digest and the receipt, and never a result or artifact
    // reference; the bridge therefore normalizes both to `null`/`[]`. Retained evidence is
    // addressed by this task's own recording prefix below, so nothing here may demand a
    // reference the service never signs — but any reference that IS reported has to
    // belong to exactly this recording.
    let prefix = format!("stado://weles/recordings/{weles_task_id}/");
    ensure(
        status
            .result_ref
            .iter()
            .chain(status.artifact_refs.iter())
            .all(|reference| reference.starts_with(&prefix)),
        "weles_artifact_refs_foreign",
        "a task result or artifact reference is not bound to this exact Weles recording",
    )?;

    let claims = &checkpoint.claims;
    ensure(
        claims.task_id == weles_task_id
            && claims.organization_id == organization_id
            && claims.origin == origin
            && claims.action == weles::SPIS_WELES_ACTION
            && claims.outcome == "completed",
        "weles_receipt_claims_mismatch",
        "the verified receipt claims do not name this completed task",
    )?;
    ensure(
        claims.request_digest == submission.request_digest
            && claims.result_digest == result_digest
            && claims.spis_binding == binding,
        "weles_receipt_claims_mismatch",
        "the verified receipt does not sign this exact request, result and Spis binding",
    )?;

    let stado_job_id = stado_job_id(&weles_task_id)?;

    let recordings = attempt_root.join("recordings").join(&weles_task_id);
    let evidence_manifest_uri = format!("{prefix}evidence-manifest.json");
    let evidence_manifest_path = recordings.join("evidence-manifest.json");
    storage_get(&evidence_manifest_uri, &evidence_manifest_path)?;
    // These exact bytes are the receipt-bound artifact. Re-serializing would change the
    // digest the receipt signed, so they are only ever copied.
    let manifest_bytes = std::fs::read(&evidence_manifest_path)?;
    let artifact_document_sha256 = crate::sha256_hex(&manifest_bytes);
    let artifact_relative = format!("weles/artifacts/{artifact_document_sha256}.json");
    write_exact(&attempt_root.join(&artifact_relative), &manifest_bytes)?;
    ensure(
        claims.evidence_digest == artifact_document_sha256,
        "weles_evidence_digest_mismatch",
        "the verified receipt evidenceDigest is not the retained evidence manifest digest",
    )?;

    let evidence: Value = serde_json::from_slice(&manifest_bytes)?;
    ensure_manifest_identity(
        &evidence,
        EVIDENCE_MANIFEST_SCHEMA,
        "completed",
        &weles_task_id,
        claims,
        &submission.request_digest,
        &result_digest,
        &binding,
        &product_url,
    )?;
    // Only the successful version signs a navigation, so only here are the effective and
    // final URL required and bound to the product origin.
    let effective_url = text(&evidence, "effectiveUrl")?.to_string();
    let final_url = text(&evidence, "finalUrl")?.to_string();
    ensure(
        same_origin(&effective_url, &parsed) && same_origin(&final_url, &parsed),
        "weles_evidence_manifest_invalid",
        "the signed effective/final URLs are not same-origin with the product URL",
    )?;

    let inventory = signed_inventory(&evidence)?;
    let retained_paths =
        retain_signed_inventory(&recordings, &weles_task_id, &prefix, &inventory, true)?;

    let observation_document_sha256 = retain_observation_document(
        attempt_root,
        manifest,
        &weles_task_id,
        &product_url,
        "completed",
        Some((&effective_url, &final_url)),
        &inventory,
        &retained_paths,
    )?;

    let expected_claims = weles::ExpectedReceiptClaims {
        task_id: weles_task_id.clone(),
        organization_id: organization_id.clone(),
        request_digest: submission.request_digest.clone(),
        result_digest: result_digest.clone(),
        spis_binding: binding.clone(),
        origin: origin.clone(),
        action: weles::SPIS_WELES_ACTION.to_string(),
        outcome: "completed".to_string(),
        evidence_digest: artifact_document_sha256.clone(),
    };
    let artifact = weles::RetainedArtifact {
        path: artifact_relative,
        sha256: artifact_document_sha256.clone(),
        bytes: manifest_bytes.len() as u64,
    };
    // The bridge resolves `artifact.path` against its working directory, so the official
    // client re-reads and re-digests exactly the retained bytes.
    let verify_command = json!({
        "schema": weles::BRIDGE_COMMAND_SCHEMA,
        "operation": "verify",
        "receipt": serde_json::to_value(&checkpoint.receipt)?,
        "expectedClaims": serde_json::to_value(&expected_claims)?,
        "artifact": serde_json::to_value(&artifact)?,
    });
    let stdout = run_bridge(attempt_root, private, "verify", &verify_command, None, false)?;
    let fresh: weles::WelesProvenanceDocument = serde_json::from_slice(&stdout)?;
    ensure(
        fresh
            .id
            .strip_prefix("sha256:")
            .is_some_and(is_sha256),
        "weles_provenance_id_invalid",
        "the bridge verification document has no framed sha256: identifier",
    )?;
    let provenance = weles::WelesProvenanceDocument {
        schema: weles::PROVENANCE_DOCUMENT_SCHEMA.to_string(),
        // The framed provenance id is derived by the official bridge from the receipt, the
        // trusted key-set version and the artifact; Rust only re-checks its shape above.
        id: fresh.id.clone(),
        client: checkpoint.client.clone(),
        receipt: checkpoint.receipt.clone(),
        claims: checkpoint.claims.clone(),
        expected_claims,
        artifact,
    };
    ensure(
        fresh == provenance,
        "weles_provenance_mismatch",
        "the fresh official verification differs from the assembled provenance document",
    )?;
    retain_attempt_document(
        attempt_root,
        "weles-provenance.json",
        &serde_json::to_value(&provenance)?,
    )?;
    collected.provenance = Some(provenance);

    let envelope = weles::WelesAttemptEnvelope {
        schema: weles::ATTEMPT_ENVELOPE_SCHEMA.to_string(),
        run_id: manifest.run_id.clone(),
        catalog: manifest.catalog.clone(),
        record: manifest.record.clone(),
        record_key: manifest.record_key.clone(),
        attempt: manifest.attempt,
        attempt_id: manifest.attempt_id.clone(),
        stado_job_id,
        weles_task_id,
        state: "completed".to_string(),
        outcome: Some("completed".to_string()),
        service_identity: identity,
        source_revision: manifest.source_revision.clone(),
        source_input_sha256: manifest.source_input_sha256.clone(),
        reference_sha256: manifest.reference_sha256.clone(),
        spis_binding: binding,
        weles_request_document: submission.request_document.clone(),
        weles_request_digest: submission.request_digest.clone(),
        weles_result_digest: Some(result_digest),
        requested_url: product_url,
        final_url: Some(final_url),
        evidence_inventory: inventory,
        weles_evidence_manifest_uri: evidence_manifest_uri,
        weles_evidence_manifest_sha256: Some(artifact_document_sha256.clone()),
        artifact_document_uri: format!("{base}/weles/artifacts/{artifact_document_sha256}.json"),
        artifact_document_sha256: Some(artifact_document_sha256),
        observation_document_uri: format!(
            "{base}/weles/observations/{observation_document_sha256}.json"
        ),
        observation_document_sha256,
    };
    retain_attempt_document(
        attempt_root,
        "attempt-envelope.json",
        &serde_json::to_value(&envelope)?,
    )?;
    collected.envelope = Some(envelope);
    Ok(())
}
