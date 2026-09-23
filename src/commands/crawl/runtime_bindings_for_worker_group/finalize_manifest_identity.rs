use super::*;

pub(crate) fn finalize_manifest_identity(manifest: &mut RuntimeManifest, reference_bytes: &[u8]) -> Result<()> {
    manifest.reference_sha256 = crate::sha256_hex(reference_bytes);
    let input_identity = json!({
        "reference_sha256": manifest.reference_sha256,
        "runtime_product": manifest.runtime_product,
        "account": manifest.account,
        "constraints": manifest.constraints,
        "delivery": manifest.delivery,
        "bindings_sha256": manifest.bindings_sha256,
        "bindings_file_sha256": manifest.bindings_file_sha256,
        "prepared_proof": manifest.prepared_proof,
        "execution_identity": manifest.execution_identity,
        "resource_lease": manifest.resource_lease,
        "docs_structure_sha256": manifest.docs_structure_sha256,
        "service_identity": manifest.service_identity,
    });
    manifest.source_input_sha256 =
        crate::sha256_hex(&serde_json::to_vec(&input_identity)?);
    manifest.catalog_key = crate::sha256_hex(
        format!(
            "{}\0{}\0{}",
            manifest.source_revision, manifest.run_id, manifest.catalog
        )
        .as_bytes(),
    );
    manifest.record_key = crate::sha256_hex(
        format!(
            "{}\0{}\0{}",
            manifest.catalog_key, manifest.record, manifest.source_input_sha256
        )
        .as_bytes(),
    );
    manifest.attempt_id = format!(
        "attempt-{}-{}",
        manifest.attempt,
        &crate::sha256_hex(
            format!(
                "{}\0{}\0{}",
                manifest.record_key, manifest.attempt, manifest.execution_identity.as_ref().map(|value| value.host.as_str()).unwrap_or("")
            )
            .as_bytes()
        )[..16]
    );
    manifest.correlation_id = format!("spis-{}-{}", &manifest.record_key[..24], manifest.attempt);
    manifest.stado_run_id = format!("{}-{}", manifest.correlation_id, manifest.attempt_id);
    let base_uri = crate::crawl_attempt_base_uri(
        &manifest.run_id,
        &manifest.catalog,
        &manifest.record,
        &manifest.record_key,
        manifest.attempt,
        &manifest.attempt_id,
    );
    manifest.artifact_uri = format!("{base_uri}/artifacts.tar.gz");
    manifest.output_uri = format!("{base_uri}/worker-output.log");
    Ok(())
}

pub(crate) fn native_attempt_root(
    base: &Path,
    manifest: &RuntimeManifest,
) -> Result<PathBuf> {
    for (name, component) in [
        ("run_id", manifest.run_id.as_str()),
        ("catalog", manifest.catalog.as_str()),
        ("record", manifest.record.as_str()),
        ("record_key", manifest.record_key.as_str()),
        ("attempt_id", manifest.attempt_id.as_str()),
    ] {
        safe_component(component, name)?;
    }
    if manifest.attempt == 0 {
        bail!("runtime manifest attempt must be a nonzero u32");
    }
    let coordinate = crate::crawl_attempt_base_uri(
        &manifest.run_id,
        &manifest.catalog,
        &manifest.record,
        &manifest.record_key,
        manifest.attempt,
        &manifest.attempt_id,
    );
    let artifact_uri = format!("{coordinate}/artifacts.tar.gz");
    let output_uri = format!("{coordinate}/worker-output.log");
    if manifest.artifact_uri != artifact_uri || manifest.output_uri != output_uri {
        bail!("runtime manifest artifact/output URIs are not the canonical attempt coordinates");
    }
    Ok(base
        .join(&manifest.run_id)
        .join(&manifest.catalog)
        .join(&manifest.record)
        .join(&manifest.record_key)
        .join("attempts")
        .join(manifest.attempt.to_string())
        .join(&manifest.attempt_id))
}

/// One explicit typed attempt entry for a record that cannot be planned.
///
/// An unconfigured native binding, an unreadable reference or an unbound Weles
/// placement is still an attempted record: it keeps a deterministic attempt id so
/// `sync_attempt_history` retains exactly one durable diagnostic instead of
/// letting the record disappear from the run.
pub(crate) fn unavailable_record(
    run_id: &str,
    catalog: &str,
    slug: &str,
    code: &str,
    message: String,
    detail: Value,
) -> Value {
    let attempt_id = format!(
        "unattempted-{}",
        &crate::sha256_hex(format!("{run_id}\0{catalog}\0{slug}\0{code}").as_bytes())[..16]
    );
    json!({
        "record": slug,
        "state": "unavailable",
        "attempt": 1,
        "attempt_id": attempt_id,
        "manifest": Value::Null,
        "command": Value::Null,
        "stado_job_id": Value::Null,
        "artifact_uri": Value::Null,
        "output_uri": Value::Null,
        "submission_receipt": Value::Null,
        "preflight": Value::Null,
        "diagnostic": {
            "code": code,
            "retryable": true,
            "message": message,
            "detail": detail,
        },
        "attempts": [],
    })
}
