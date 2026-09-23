use super::*;

// What filled 2.5 GiB of the production Mac's remaining disk on 2026-09-04 was
// not the corpus: it was ten simultaneous `cargo build` passes, one per record,
// each writing its own 5 GiB `target/` inside its own immutable checkout. That
// is now one shared target directory per revision, and a finished record's own
// retained corpus is tens of megabytes - MDN, the largest site in the catalog,
// imported 36 MB from 12,248 pages. So the host's declared disk gate is the
// right control for what remains, and it already is: below its low watermark
// the agent claims nothing.
//
// `--exclusive` is not that control. It means "claim the whole GPU: start only
// while idle and admit no other job", and on a host that also carries release
// qualification and Probierz work it means the family waits for an idle
// machine. On 2026-09-05 forty-nine records sat queued from 18:30 to 19:25
// behind foreign jobs, with 17 GiB free and six cores idle, because every one
// of them demanded the machine to itself.
pub(crate) const STADO_RESOURCE_ARGS: [&str; 0] = [];

pub(crate) fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    safe_path_component(value, flag)
        .map_err(|_| anyhow::anyhow!("{flag} contains characters that cannot be submitted to a worker"))
}

pub(crate) fn source_revision() -> Result<String> {
    super::crawl::build_revision()
}

#[derive(Deserialize)]
pub(crate) struct StorageStatReceipt {
    pub(crate) schema: String,
    pub(crate) path: String,
    pub(crate) state: String,
    pub(crate) size: Option<u64>,
}

pub(crate) fn storage_artifact_present(uri: &str, context: &str) -> Result<bool> {
    let mut command = super::crawl::crawl_storage_command();
    command.args(["storage", "stat", uri, "--json"]);
    let output = super::crawl::bounded_command_output(
        &mut command,
        context,
        Duration::from_secs(60),
        STADO_OUTPUT_LIMIT,
    )?;
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
    let _ = manifest_attempt(manifest)?;
    let mut worker_options = forwarded.to_vec();
    if !worker_options
        .iter()
        .any(|argument| matches!(argument.as_str(), "--site" | "--all"))
    {
        worker_options.insert(0, record.to_string());
        worker_options.insert(0, "--site".into());
    }
    let parsed = WorkerOptions::parse(&worker_options)?;
    if parsed.all || parsed.site.as_deref() != Some(record) {
        bail!("documentation worker arguments do not select the immutable manifest record");
    }
    if parsed.refresh {
        refuse_published_refresh(&manifest.artifact_uri)?;
    }
    let artifact = manifest.artifact_uri.clone();
    let output_uri = manifest.output_uri.clone();
    // The absolute path Stado resolved on this host, never the bare name. The
    // submitted command runs under a non-login `/bin/sh` that reads no
    // profile, so `cargo` alone resolved to nothing and
    // job-545551889f9e88be30daa81f died sixteen minutes into a claimed slot
    // with `/bin/sh: cargo: command not found`. Through the shared helper now
    // that all six engines do this, so no engine can drift back to naming it
    // bare while another names the resolved path.
    let cargo = super::crawl::resolved_worker_program(host)?;
    let mut command_arguments = vec![
        cargo,
        "run".to_string(),
        "--release".to_string(),
        "--".to_string(),
        "crawl-docs".to_string(),
        "--worker".to_string(),
        "--artifact-uri".to_string(),
        artifact.clone(),
        "--runtime-manifest-base64".to_string(),
        manifest.encoded()?,
    ];
    command_arguments.extend(worker_options);
    let mut command_words = vec![command_arguments[0].clone()];
    command_words.extend(
        command_arguments[1..]
            .iter()
            .map(|argument| shell_quote(argument))
            .collect::<Result<Vec<_>>>()?,
    );
    let command = command_words.join(" ");
    let mut stado = super::crawl::stado_command();
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
        super::crawl::STADO_REPO_WORKDIR,
        "--repo-extras",
        "",
        "--output-uri",
        &output_uri,
    ]);
    stado.args(STADO_RESOURCE_ARGS);
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "submit documentation crawl through Stado",
        STADO_COMMAND_TIMEOUT,
        STADO_OUTPUT_LIMIT,
    )?;
    if !output.status.success() {
        bail!(
            "Stado refused documentation crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    super::crawl::print_submission(
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
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--site" => {
                i += 1;
                let value = rest.get(i).context("--site needs a value")?.clone();
                record = Some(value.clone());
                forwarded.push("--site".into());
                forwarded.push(value);
            }
            "--runtime-manifest-base64" => {
                i += 1;
                runtime_manifest_base64 =
                    Some(rest.get(i).context("--runtime-manifest-base64 needs a value")?.clone());
            }
            "--worker" => worker = true,
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            value => forwarded.push(value.to_string()),
        }
        i += 1;
    }
    let record = record.context("--record is required for one exact per-record job")?;
    let manifest = super::crawl::decode_runtime_manifest(
        runtime_manifest_base64.as_deref().context("--runtime-manifest-base64 is required")?,
        "documentation-site-examples",
        "docs",
        Some(&record),
    )?;
    if !worker {
        return submit_worker(
            &host.context("--host is required; documentation crawls execute as pinned Stado jobs")?,
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
