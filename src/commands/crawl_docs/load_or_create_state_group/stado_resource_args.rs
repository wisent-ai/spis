use super::*;

pub(crate) fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    safe_path_component(value, flag).map_err(|_| {
        anyhow::anyhow!("{flag} contains characters that cannot be submitted to a worker")
    })
}

pub(crate) fn source_revision() -> Result<String> {
    crate::commands::crawl::build_revision()
}

#[derive(Deserialize)]
pub(crate) struct StorageStatReceipt {
    pub(crate) schema: String,
    pub(crate) path: String,
    pub(crate) state: String,
    pub(crate) size: Option<u64>,
}

pub(crate) fn storage_artifact_present(uri: &str, context: &str) -> Result<bool> {
    let mut command = crate::commands::crawl::crawl_storage_command();
    command.args(["storage", "stat", uri, "--json"]);
    let output = crate::commands::crawl::command_output(&mut command, context)?;
    if !output.status.success() {
        bail!(
            "cannot determine whether immutable documentation attempt artifact is published: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let response: StorageStatReceipt =
        serde_json::from_slice(&output.stdout).context("parse Stado storage stat response")?;
    if response.schema != "stado.storage-stat-receipt.v1" || response.path != uri {
        bail!("Stado storage stat receipt has the wrong schema or URI identity");
    }
    match response.state.as_str() {
        "absent" if response.size.is_none() => Ok(false),
        "present" if response.size.is_some() => Ok(true),
        state => bail!(
            "immutable documentation attempt URI {uri} has unsupported or inconsistent storage state {state}"
        ),
    }
}

pub(crate) fn refuse_published_refresh(uri: &str) -> Result<()> {
    if storage_artifact_present(
        uri,
        "check immutable documentation attempt artifact before --refresh",
    )? {
        bail!(
            "--refresh refuses published immutable attempt URI {uri}; create a fresh crawl attempt"
        );
    }
    Ok(())
}

pub(crate) fn shell_quote(value: &str) -> Result<String> {
    if value.contains('\0') {
        bail!("documentation worker argument contains a NUL byte");
    }
    Ok(format!("'{}'", value.replace('\'', "'\\''")))
}

pub(crate) fn submit_worker(
    host: &str,
    record: &str,
    manifest: &super::crawl::RuntimeManifest,
    forwarded: &[String],
) -> Result<()> {
    safe_job_value(host, "--host")?;
    safe_job_value(record, "--record")?;
    if source_revision()? != manifest.source_revision {
        bail!("documentation coordinator revision does not match immutable runtime manifest");
    }
    // Called for its refusal: a manifest without a valid attempt stops here.
    manifest_attempt(manifest)?;
    let mut worker_options = forwarded.to_vec();
    if !worker_options
        .iter()
        .any(|argument| matches!(argument.as_str(), "--site" | "--all"))
    {
        worker_options.extend(["--site".into(), record.to_string()]);
    }
    let parsed = WorkerOptions::parse(&worker_options)?;
    if parsed.all {
        bail!("documentation worker arguments do not select the immutable manifest record");
    }
    if parsed.site.as_deref() != Some(record) {
        bail!("documentation worker arguments do not select the immutable manifest record");
    }
    if parsed.refresh {
        refuse_published_refresh(&manifest.artifact_uri)?;
    }
    let artifact = manifest.artifact_uri.clone();
    let output_uri = manifest.output_uri.clone();
    // The absolute path Stado resolved on this host, never the bare name. The
    // submitted command runs under a non-login `/bin/sh` that reads no
    // profile, so `cargo` alone resolved to nothing and a crawl job died
    // many minutes into a claimed slot
    // with `/bin/sh: cargo: command not found`. Through the shared helper now
    // that all six engines do this, so no engine can drift back to naming it
    // bare while another names the resolved path.
    let cargo = crate::commands::crawl::resolved_worker_program(host)?;
    let mut command_arguments = vec![
        cargo,
        "run".to_string(),
        "--release".to_string(),
        "--".to_string(),
        "crawler".to_string(),
        "run".to_string(),
        "docs".to_string(),
        "--worker".to_string(),
        "--artifact-uri".to_string(),
        artifact.clone(),
        "--runtime-manifest-base64".to_string(),
        manifest.encoded()?,
    ];
    command_arguments.extend(worker_options);
    let (program, arguments) = command_arguments.split_first()
        .context("documentation worker command has no program")?;
    let mut command_words = vec![program.clone()];
    command_words.extend(
        arguments
            .iter()
            .map(|argument| shell_quote(argument))
            .collect::<Result<Vec<_>>>()?,
    );
    let command = command_words.join(" ");
    let mut stado = crate::commands::crawl::stado_command();
    stado.args([
        "submit",
        &command,
        "--run-id",
        &manifest.stado_run_id,
        "--pinned-host",
        host,
        "--repo",
        REPOSITORY,
        "--repo-ref",
        &manifest.source_revision,
        "--repo-workdir",
        crate::commands::crawl::STADO_REPO_WORKDIR,
        "--repo-extras",
        "",
        "--output-uri",
        &output_uri,
    ]);
    let output = crate::commands::crawl::command_output(
        &mut stado,
        "submit documentation crawl through Stado",
    )?;
    if !output.status.success() {
        bail!(
            "Stado refused documentation crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    crate::commands::crawl::print_submission(
        "documentation-site-examples",
        "docs",
        host,
        Some(&artifact),
        &output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut host = None;
    let mut worker = false;
    let mut artifact_uri = None;
    let mut record = None;
    let mut runtime_manifest_base64 = None;
    let mut forwarded = Vec::new();
    let mut arguments = rest.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--host" => {
                host = Some(crate::commands::required(arguments.next(), "--host needs a value")?.clone());
            }
            "--record" => {
                record = Some(crate::commands::required(arguments.next(), "--record needs a value")?.clone());
            }
            "--site" => {
                let value = crate::commands::required(arguments.next(), "--site needs a value")?.clone();
                record = Some(value.clone());
                forwarded.push("--site".into());
                forwarded.push(value);
            }
            "--runtime-manifest-base64" => {
                runtime_manifest_base64 = Some(crate::commands::required(
                    arguments.next(), "--runtime-manifest-base64 needs a value"
                )?.clone());
            }
            "--worker" => worker = true,
            "--artifact-uri" => {
                artifact_uri = Some(crate::commands::required(arguments.next(), "--artifact-uri needs a value")?.clone());
            }
            value => forwarded.push(value.to_string()),
        }
    }
    let record = record.context("--record is required for one exact per-record job")?;
    let manifest = crate::commands::crawl::decode_runtime_manifest(
        runtime_manifest_base64
            .as_deref()
            .context("--runtime-manifest-base64 is required")?,
        "documentation-site-examples",
        "docs",
        Some(&record),
    )?;
    if !worker {
        return submit_worker(
            &host
                .context("--host is required; documentation crawls execute as pinned Stado jobs")?,
            &record,
            &manifest,
            &forwarded,
        );
    }
    if host.is_some() {
        bail!("--host cannot be used with --worker");
    }
    let artifact_uri = artifact_uri.context("--artifact-uri is required in worker mode")?;
    if artifact_uri != manifest.artifact_uri {
        bail!("worker artifact URI does not match immutable runtime manifest");
    }
    run_worker(&forwarded, &manifest)?;
    Ok(())
}
