use super::*;

pub(crate) fn revision() -> Result<String> { super::crawl::build_revision() }

pub(crate) fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        bail!("{flag} must contain only letters, digits, '-' or '_'");
    }
    Ok(())
}

pub(crate) fn attempt_root(
    base: &Path,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<PathBuf> {
    super::crawl::native_attempt_root(base, manifest)
}

pub(crate) struct DesktopSubmission<'a> {
    pub(crate) host: &'a str,
    pub(crate) catalog: &'a str,
    pub(crate) record: &'a str,
    pub(crate) max_states: usize,
    pub(crate) max_depth: usize,
    pub(crate) manifest: &'a super::crawl::RuntimeManifest,
}

pub(crate) fn submit_worker(request: DesktopSubmission<'_>) -> Result<()> {
    safe_job_value(request.host, "--host")?;
    safe_job_value(request.catalog, "catalog")?;
    safe_job_value(request.record, "--record")?;
    let _attempt_binding = attempt_root(Path::new("."), request.manifest)?;
    if revision()? != request.manifest.source_revision {
        bail!("desktop coordinator revision does not match immutable runtime manifest");
    }
    let encoded = request.manifest.encoded()?;
    let artifact = request.manifest.artifact_uri.clone();
    let output_uri = request.manifest.output_uri.clone();
    // The absolute path this host executes cargo at, never the bare name.
    // Every worker in this repository is `cargo run --release`, and the job's
    // shell is a non-login `/bin/sh` that reads no profile, so a bare name
    // resolves to nothing however the host installs Rust -- the defect that
    // cost job-545551889f9e88be30daa81f sixteen minutes of a claimed slot in
    // the documentation engine, still open in this one.
    let cargo = super::crawl::resolved_worker_program(request.host)?;
    let command = format!(
        "{cargo} run --release -- crawl-desktop {} --worker --record {} --max-states {} --max-depth {} --artifact-uri {} --runtime-manifest-base64 '{}'",
        request.catalog,
        request.record,
        request.max_states,
        request.max_depth,
        artifact,
        encoded,
    );
    let arguments = vec![
        "submit".to_string(),
        command,
        "--run-id".to_string(),
        request.manifest.stado_run_id.clone(),
        "--pinned-host".to_string(),
        request.host.to_string(),
        "--exclusive".to_string(),
        "--repo".to_string(),
        REPOSITORY.to_string(),
        "--repo-ref".to_string(),
        request.manifest.source_revision.clone(),
        "--repo-workdir".to_string(),
        super::crawl::STADO_REPO_WORKDIR.to_string(),
        "--repo-extras".to_string(),
        String::new(),
        "--output-uri".to_string(),
        output_uri.clone(),
    ];
    let mut stado = super::crawl::stado_command();
    stado.args(arguments);
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "submit desktop crawl through Stado",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "Stado refused desktop crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    super::crawl::print_submission(
        request.catalog,
        "desktop",
        request.host,
        Some(&artifact),
        &output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

pub(crate) fn worker_report(
    manifest: &super::crawl::RuntimeManifest,
    artifact: Option<Value>,
    failure: Option<Value>,
) -> Result<Value> {
    let value = serde_json::to_value(manifest)?;
    let attempt = value
        .get("attempt")
        .and_then(Value::as_u64)
        .context("desktop runtime manifest has no attempt for worker report")?;
    let attempt_id = value
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("desktop runtime manifest has no attempt_id for worker report")?;
    let bindings_file_sha256 = value
        .get("bindings_file_sha256")
        .and_then(Value::as_str)
        .context("desktop runtime manifest has no bindings_file_sha256")?;
    let bindings_sha256 = value
        .get("bindings_sha256")
        .and_then(Value::as_str)
        .context("desktop runtime manifest has no bindings_sha256")?;
    let execution_identity = value
        .get("execution_identity")
        .filter(|identity| identity.is_object())
        .context("desktop runtime manifest has no typed execution_identity")?
        .clone();
    if let Some(artifact) = artifact.as_ref() {
        if artifact.get("uri").and_then(Value::as_str) != Some(manifest.artifact_uri.as_str()) {
            bail!("published desktop artifact URI does not match the immutable runtime manifest");
        }
    }
    Ok(json!({
        "schema": "wisent.native-worker-report.v1",
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "record_key": manifest.record_key,
        "attempt": attempt,
        "attempt_id": attempt_id,
        "engine": manifest.engine,
        "state": if failure.is_some() { "failed" } else { "artifact_published" },
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "reference_sha256": manifest.reference_sha256,
        "bindings_file_sha256": bindings_file_sha256,
        "bindings_sha256": bindings_sha256,
        "execution_identity": execution_identity,
        "artifact": artifact,
        "failure": failure,
    }))
}
