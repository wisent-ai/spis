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
    driver_url: Option<&str>,
) -> Value {
    let Some(slug) = record_dir.file_name().and_then(|value| value.to_str()) else {
        let named = record_dir.display().to_string();
        return unavailable_record(
            run_id,
            catalog,
            &named,
            "record_directory_unnamed",
            format!("record directory {named} has no UTF-8 name to use as its slug"),
            json!({"path": record_dir}),
        );
    };
    let slug = slug.to_string();
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
    let product = match runtime_product(catalog, engine, &slug, &reference, binding.surface.clone())
    {
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
    let bindings_sha256 =
        crate::sha256_hex(&serde_json::to_vec(&binding).expect("typed runtime binding serializes"));
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
    let catalog_key =
        crate::sha256_hex(format!("{source_revision}\0{run_id}\0{catalog}").as_bytes());
    let record_key =
        crate::sha256_hex(format!("{catalog_key}\0{slug}\0{source_input_sha256}").as_bytes());
    let base_uri = crate::crawl_record_base_uri(run_id, catalog, &slug, &record_key);
    let declared_engine = match Engine::of(engine) {
        Ok(declared) => declared,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "undeclared_crawl_engine",
                format!("{error:#}"),
                json!({"engine": engine}),
            );
        }
    };
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
        // finalize_manifest_identity derives the correlation and Stado run ids.
        correlation_id: String::new(),
        attempt: crate::commands::crawl::FIRST_ATTEMPT,
        attempt_id: String::new(),
        stado_run_id: String::new(),
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
        // Engines that run the product on the host share its tmux namespace,
        // PATH and CPU, so they need a host-level exclusivity lease even when
        // they need no device.
        resource_lease: declared_engine
            .runs_on_host()
            .then(|| format!("stado-exclusive://{host}/{engine}")),
        service_identity,
        driver_url: driver_url.map(str::to_string),
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
        "state": State::Planned.name(),
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

/// Git's own object name for the empty tree, a SHA-1 object name as Git
/// writes every one: https://git-scm.com/docs/hash-function-transition
const GIT_EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

pub(crate) fn is_git_revision(value: &str) -> bool {
    value.len() == GIT_EMPTY_TREE.len()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
