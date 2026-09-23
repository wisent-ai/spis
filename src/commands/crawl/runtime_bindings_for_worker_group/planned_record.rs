use super::*;

pub(crate) fn planned_record(
    run_id: &str,
    source_revision: &str,
    catalog: &str,
    engine: &str,
    host: &str,
    record_dir: &Path,
    bindings: &RuntimeBindings,
    service_identity: Option<&RuntimeServiceIdentity>,
) -> Value {
    let slug = record_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let path = record_dir.join("reference.json");
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "reference_read_failed",
                error.to_string(),
                json!({"path": path}),
            );
        }
    };
    let reference: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "reference_invalid",
                error.to_string(),
                json!({"path": path}),
            );
        }
    };
    let binding = match runtime_binding(bindings, catalog, engine, &slug) {
        Ok(binding) => binding,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "runtime_binding_missing_or_invalid",
                error.to_string(),
                Value::Null,
            );
        }
    };
    if binding.constraints.headless != (engine == "web") {
        return unavailable_record(
            run_id,
            catalog,
            &slug,
            "runtime_constraint_mismatch",
            format!("{catalog}/{slug}: headless constraint does not match engine {engine}"),
            json!({"headless": binding.constraints.headless, "engine": engine}),
        );
    }
    let service_identity = match (engine, service_identity) {
        ("web", Some(service)) if service.active_host == host => Some(service.clone()),
        ("web", _) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "weles_service_identity_unbound",
                "authorized weles-admission/browser-evidence/generic_browser_task placement is unavailable".into(),
                json!({"host": host}),
            );
        }
        (_, None) => None,
        (_, Some(_)) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "unexpected_service_identity",
                "non-web execution cannot carry a Weles service identity".into(),
                json!({"engine": engine}),
            );
        }
    };
    let product = match runtime_product(
        catalog,
        engine,
        &slug,
        &reference,
        binding.surface.clone(),
    ) {
        Ok(product) => product,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "runtime_product_unresolved",
                error.to_string(),
                json!({"product_url": reference.get("product_url")}),
            );
        }
    };
    let docs_structure_sha256 = match docs_structure_sha256(catalog, &slug, engine) {
        Ok(value) => value,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "docs_structure_missing_or_invalid",
                error.to_string(),
                Value::Null,
            );
        }
    };
    let bindings_sha256 = crate::sha256_hex(
        &serde_json::to_vec(&binding).expect("typed runtime binding serializes"),
    );
    let account = binding.account;
    let constraints = binding.constraints;
    let prepared_proof = binding.prepared_proof;
    let delivery = binding.delivery;
    let input_identity = json!({
        "reference_sha256": crate::sha256_hex(&bytes),
        "runtime_product": product,
        "account": account,
        "constraints": constraints,
        "prepared_proof": prepared_proof,
        "delivery": delivery,
        "bindings_file_sha256": bindings.sha256,
        "bindings_sha256": bindings_sha256,
        "service_identity": service_identity,
        "docs_structure_sha256": docs_structure_sha256,
    });
    let source_input_sha256 = crate::sha256_hex(
        &serde_json::to_vec(&input_identity).expect("typed runtime input serializes"),
    );
    let catalog_key = crate::sha256_hex(
        format!("{source_revision}\0{run_id}\0{catalog}").as_bytes(),
    );
    let record_key = crate::sha256_hex(
        format!("{catalog_key}\0{slug}\0{source_input_sha256}").as_bytes(),
    );
    let correlation_id = format!("spis-{}", &record_key[..32]);
    let base_uri = crate::crawl_record_base_uri(run_id, catalog, &slug, &record_key);
    let mut manifest = RuntimeManifest {
        schema: RUNTIME_MANIFEST_SCHEMA.into(),
        run_id: run_id.into(),
        catalog: catalog.into(),
        record: slug.clone(),
        engine: engine.into(),
        source_revision: source_revision.into(),
        source_input_sha256,
        reference_sha256: crate::sha256_hex(&bytes),
        catalog_key,
        record_key,
        correlation_id: correlation_id.clone(),
        attempt: 1,
        attempt_id: String::new(),
        stado_run_id: correlation_id,
        artifact_uri: format!("{base_uri}/artifacts.tar.gz"),
        output_uri: format!("{base_uri}/worker-output.log"),
        runtime_product: product,
        account,
        constraints,
        docs_structure_sha256,
        delivery,
        bindings_file_sha256: bindings.sha256.clone(),
        bindings_source: bindings.source.clone(),
        bindings_sha256,
        bindings_uri: bindings.uri.clone(),
        prepared_proof,
        execution_identity: None,
        // Terminal surfaces share the registry host's tmux namespace, PATH and CPU,
        // so they need a host-level exclusivity lease even though they need no device.
        resource_lease: matches!(engine, "desktop" | "mobile" | "cli" | "tui")
            .then(|| format!("stado-exclusive://{host}/{engine}")),
        service_identity,
    };
    if let Err(error) = finalize_manifest_identity(&mut manifest, &bytes) {
        return unavailable_record(
            run_id,
            catalog,
            &slug,
            "runtime_manifest_finalization_failed",
            error.to_string(),
            Value::Null,
        );
    }
    json!({
        "record": slug,
        "state": "planned",
        "manifest": manifest,
        "command": Value::Null,
        "stado_job_id": Value::Null,
        "artifact_uri": manifest.artifact_uri,
        "output_uri": manifest.output_uri,
        "submission_receipt": Value::Null,
        "preflight": Value::Null,
        "diagnostic": Value::Null,
    })
}

pub(crate) fn is_git_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// A canonical Weles API base is exactly `<scheme>://<host>[:port]/api/v1` with
/// no credentials, query or fragment. Both the bridge and the Rust verifier
/// enforce the same shape, so an endpoint that would be rejected downstream must
/// never enter a plan.
pub(crate) fn canonical_api_endpoint(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|endpoint| {
        matches!(endpoint.scheme(), "http" | "https")
            && endpoint.username().is_empty()
            && endpoint.password().is_none()
            && endpoint.path() == "/api/v1"
            && endpoint.query().is_none()
            && endpoint.fragment().is_none()
            && endpoint.as_str() == value
    })
}
