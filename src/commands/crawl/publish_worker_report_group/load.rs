use super::*;

pub(crate) fn load(run_id: Option<&str>) -> Result<Value> {
    migrate_run_state(run_id)?;
    let selected = match run_id {
        Some(value) => {
            safe_component(value, "run id")?;
            value.to_string()
        }
        None => {
            let root = run_root()?;
            let mut ids: Vec<String> = std::fs::read_dir(&root)
                .with_context(|| format!("no crawl runs exist under {}", root.display()))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().join("run.json").is_file())
                .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
                .filter(|id| safe_component(id, "run id").is_ok())
                .collect();
            ids.sort();
            ids.pop().context("no persisted crawl run exists")?
        }
    };
    let path = run_path(&selected)?;
    crate::read_json(path.to_str().context("run path is not UTF-8")?)
}

pub(crate) const STADO_SUBMISSION_RECEIPT_SCHEMA: &str = "stado.submission-receipt.v3";

pub(crate) const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

/// The checkout subdirectory every crawl worker runs from. It is no longer a
/// receipt field, so it is asserted where it is actually declared: the
/// `--repo-workdir` each engine passes to `stado submit`.
pub(crate) const STADO_REPO_WORKDIR: &str = "spis";

/// Where the executor Stado resolved for a job is reported. Retained whole:
/// the placement a crawl actually ran on is evidence, and refusing to read it
/// would be refusing the receipt.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StadoResolvedExecutor {
    pub(crate) provider: String,
    pub(crate) machine_type: String,
    pub(crate) gpu_type: String,
    pub(crate) platform_os: String,
    pub(crate) architecture: String,
}

/// One job of a `stado.submission-receipt.v3` document, field for field as
/// `stado-rs/src/cli/submit.rs` serialises it.
///
/// This side denies unknown fields on purpose, so it has to name every field
/// the producer emits and no field it does not. Three that were declared here
/// never existed in the receipt — `repo`, `executor` and `state` at job level —
/// and three that do exist were missing: `command`, `job_key` and
/// `resolved_executor`. The result was that a receipt proving a successful
/// submission read as a contract violation while the job was already queued on
/// the host.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StadoSubmissionJob {
    pub(crate) command_index: u64,
    /// The exact command line Stado accepted. Verified below against
    /// `command_digest`, so it is proof rather than decoration.
    pub(crate) command: String,
    pub(crate) command_digest: String,
    /// Stado derives `job_id` from this key, which is checked below.
    pub(crate) job_key: String,
    pub(crate) job_id: String,
    pub(crate) output_uri: String,
    pub(crate) pinned_host: String,
    pub(crate) resolved_executor: StadoResolvedExecutor,
    pub(crate) repo_ref: String,
    pub(crate) submission_request_digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StadoSubmissionReceipt {
    pub(crate) schema: String,
    pub(crate) run_id: String,
    pub(crate) source_revision: String,
    pub(crate) request_digest: String,
    pub(crate) source_digest: String,
    pub(crate) input_digest: String,
    pub(crate) repo: String,
    pub(crate) repo_ref: String,
    pub(crate) jobs: Vec<StadoSubmissionJob>,
}

/// Whether the pinned host a Stado receipt echoes is the placement host.
///
/// Stado pins a job to a registry host and then reports the pin in its
/// CONSUMER spelling: `charless-mac-mini` was submitted and the receipt came
/// back `local-charless-mac-mini.local`, which is the name that host's own
/// capacity beacon publishes. A literal comparison therefore refused a
/// submission that had already been accepted — measured on run
/// `docs-50-222852`, where thirty-seven records carried a real `job_id` and an
/// `output_uri` inside their own attempt prefix and were still recorded
/// `submission_failed`, so the queue ran work no record admitted owning.
///
/// The registry name is the identity; the consumer spelling is a rendering of
/// it. Both are accepted, nothing else is, and the comparison stays
/// case-insensitive because the beacon capitalizes the hostname the way macOS
/// reports it.
pub(crate) fn pinned_host_is(pinned: &str, host: &str) -> bool {
    let pinned = pinned.trim();
    let host = host.trim();
    if pinned.eq_ignore_ascii_case(host) {
        return true;
    }
    pinned
        .strip_prefix("local-")
        .and_then(|rest| rest.strip_suffix(".local"))
        .is_some_and(|rest| rest.eq_ignore_ascii_case(host))
}

pub(crate) fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Accept only an exact typed Stado v3 submission receipt for one job.
///
/// Every digest must be lowercase SHA-256, the repository/ref/workdir/pinned host
/// must equal the immutable attempt the caller is submitting, the single job must
/// map to command index 0 with the receipt's own command and request digests, and
/// its output URI must be the canonical attempt output coordinate. Unknown fields
/// are refused so a v2 or extended receipt can never be mistaken for proof.
pub(crate) fn compact_submission(catalog: &str, engine: &str, host: &str, artifact_uri: Option<&str>, output_uri: &str, stado_stdout: &str) -> Result<Value> {
    let raw = stado_stdout
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .find(|value| {
            value.get("schema").and_then(Value::as_str)
                == Some(STADO_SUBMISSION_RECEIPT_SCHEMA)
        })
        .context("Stado accepted the command but returned no exact typed v3 submission receipt")?;
    let receipt: StadoSubmissionReceipt = serde_json::from_value(raw.clone())
        .context("Stado submission receipt does not match the exact typed v3 contract")?;
    let expected_revision = build_revision()?;
    for (label, digest) in [
        ("request_digest", receipt.request_digest.as_str()),
        ("source_digest", receipt.source_digest.as_str()),
        ("input_digest", receipt.input_digest.as_str()),
    ] {
        if !is_lower_sha256(digest) {
            bail!("Stado submission receipt {label} is not a lowercase SHA-256 digest");
        }
    }
    if receipt.schema != STADO_SUBMISSION_RECEIPT_SCHEMA
        || receipt.run_id.trim().is_empty()
        || receipt.repo != REPOSITORY
        || receipt.repo_ref != expected_revision
        || receipt.source_revision != expected_revision
    {
        bail!(
            "Stado submission receipt run/repository/ref does not bind this exact attempt"
        );
    }
    if receipt.jobs.len() != 1 {
        bail!("per-record crawl submission must map to exactly one Stado job");
    }
    let job = &receipt.jobs[0];
    if job.job_id.trim().is_empty() || safe_component(&job.job_id, "stado job id").is_err() {
        bail!("Stado submission receipt has no portable exact job id");
    }
    // The pinned host and the repository ref are per job in the emitted
    // receipt, not per receipt, so they are compared where they live.
    if job.command_index != 0
        || !is_lower_sha256(&job.command_digest)
        || !is_lower_sha256(&job.job_key)
        || job.submission_request_digest != receipt.request_digest
        || job.repo_ref != receipt.repo_ref
        || !pinned_host_is(&job.pinned_host, host)
        || job.output_uri != output_uri
    {
        // No assertion on `resolved_executor`: a job pinned to a registry host
        // needs no provider resolution, and Stado reports every one of its
        // fields empty for exactly that case — measured on this attempt. It is
        // retained evidence of the placement, not a precondition.
        bail!(
            "Stado receipt job mapping, digests, host or output URI does not match the submitted attempt"
        );
    }
    // Two derivations Stado performs and this side re-performs, so the receipt
    // is proof and not a claim: the accepted command line hashes to its own
    // command digest, and the job id is the first 24 hex of the job key.
    if crate::sha256_hex(job.command.as_bytes()) != job.command_digest {
        bail!("Stado receipt job command does not hash to its own command digest");
    }
    if job.job_key.len() < 24 || job.job_id != format!("job-{}", &job.job_key[..24]) {
        bail!("Stado receipt job id is not derived from its own job key");
    }
    Ok(json!({
        "schema": SUBMISSION_SCHEMA,
        "catalog": catalog,
        "engine": engine,
        "host": host,
        "stado_job_id": job.job_id,
        "stado_run_id": receipt.run_id,
        "artifact_uri": artifact_uri,
        "output_uri": output_uri,
        "state": "queued",
        // The placement a crawl actually ran on, lifted out of the receipt so
        // a reader of the compact line does not have to parse the whole one.
        "stado_executor": {
            "provider": job.resolved_executor.provider,
            "machine_type": job.resolved_executor.machine_type,
            "gpu_type": job.resolved_executor.gpu_type,
            "platform_os": job.resolved_executor.platform_os,
            "architecture": job.resolved_executor.architecture,
        },
        "stado_job_key": job.job_key,
        "stado_receipt": raw,
    }))
}

/// Surface-specific coordinators call this after Stado accepts a job. The final
/// compact line is stable machine input while the preceding Stado text remains
/// useful to a person invoking the low-level engine directly.
pub fn print_submission(catalog: &str, engine: &str, host: &str, artifact_uri: Option<&str>, output_uri: &str, stado_stdout: &str) -> Result<()> {
    print!("{stado_stdout}");
    let report = compact_submission(catalog, engine, host, artifact_uri, output_uri, stado_stdout)?;
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

pub(crate) fn parse_submission(stdout: &[u8]) -> Result<Value> {
    let text = String::from_utf8_lossy(stdout);
    text.lines()
        .rev()
        .find_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .filter(|value| value.get("schema").and_then(Value::as_str) == Some(SUBMISSION_SCHEMA))
        .context("crawler returned no machine-readable submission")
}

pub(crate) fn engine_command(manifest: &RuntimeManifest, host: &str) -> Result<Vec<String>> {
    let execution = manifest
        .execution_identity
        .as_ref()
        .context("cannot build worker command without execution identity")?;
    if execution.host != host {
        bail!("worker command host differs from the bound execution identity");
    }
    if manifest.engine == "web"
        && manifest
            .service_identity
            .as_ref()
            .is_none_or(|service| service.active_host != host)
    {
        bail!("web worker command host differs from the bound Weles service placement");
    }
    let catalog = manifest.catalog.as_str();
    let engine = manifest.engine.as_str();
    let mut args = match engine {
        "mobile" => vec!["crawl-mobile".into(), catalog.into(), "--host".into(), host.into()],
        "desktop" => vec!["crawl-desktop".into(), catalog.into(), "--host".into(), host.into()],
        "web" => vec!["crawl-web".into(), catalog.into(), "--host".into(), host.into()],
        "tui" => vec!["crawl-tui".into(), "--host".into(), host.into()],
        "cli" => vec!["crawl-cli".into(), "--host".into(), host.into()],
        "docs" => vec!["crawl-docs".into(), "--host".into(), host.into()],
        _ => bail!("unknown crawler engine {engine}"),
    };
    args.push("--record".into());
    args.push(manifest.record.clone());
    args.push("--runtime-manifest-base64".into());
    args.push(manifest.encoded()?);
    Ok(args)
}
