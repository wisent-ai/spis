use super::*;

/// Extract one crawl attempt archive into an empty staging directory.
///
/// Only ordinary files and directories are accepted. Absolute paths, `..`
/// components, symlinks, hard links, devices, duplicate members and archives
/// beyond the entry/byte bounds are refused, and every member is created with
/// `create_new` so a pre-existing path can never be followed or overwritten.
pub(crate) fn extract_attempt_archive(archive: &Path, destination: &Path) -> Result<Vec<String>> {
    if destination.exists() {
        std::fs::remove_dir_all(destination)?;
    }
    std::fs::create_dir_all(destination)?;
    let mut tar = tar::Archive::new(GzDecoder::new(File::open(archive)?));
    let mut entries = 0_usize;
    let mut total = 0_u64;
    let mut extracted = Vec::new();
    for member in tar.entries()? {
        let mut member = member?;
        let kind = member.header().entry_type();
        let relative = member.path()?.into_owned();
        if relative
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            bail!(
                "crawl attempt archive contains the unsafe path {}",
                relative.display()
            );
        }
        let target = destination.join(&relative);
        if kind.is_dir() {
            std::fs::create_dir_all(&target)?;
            continue;
        }
        if !kind.is_file() {
            bail!(
                "crawl attempt archive member {} is not a regular file",
                relative.display()
            );
        }
        entries += 1;
        if entries > MAX_EXTRACTED_ENTRIES {
            bail!("crawl attempt archive exceeds the {MAX_EXTRACTED_ENTRIES}-entry bound");
        }
        let size = member.header().size()?;
        total = total
            .checked_add(size)
            .filter(|value| *value <= MAX_EXTRACTED_BYTES)
            .context("crawl attempt archive exceeds the extracted byte bound")?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .with_context(|| format!("extract crawl member {}", relative.display()))?;
        let written = std::io::copy(&mut member.by_ref().take(size), &mut file)?;
        if written != size {
            bail!(
                "crawl attempt archive member {} is truncated",
                relative.display()
            );
        }
        file.sync_all()?;
        extracted.push(
            relative
                .to_str()
                .context("crawl attempt archive member name is not UTF-8")?
                .to_string(),
        );
    }
    extracted.sort();
    Ok(extracted)
}

pub(crate) fn fsync_tree(root: &Path) -> Result<()> {
    let mut directories = vec![root.to_path_buf()];
    let mut index = 0;
    while index < directories.len() {
        let directory = directories[index].clone();
        index += 1;
        for entry in std::fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)?;
            if metadata.is_dir() {
                directories.push(path);
            } else if metadata.is_file() {
                File::open(&path)?.sync_all()?;
            }
        }
    }
    for directory in directories.iter().rev() {
        File::open(directory)?.sync_all()?;
    }
    Ok(())
}

/// Atomically install a fully staged, fsynced tree over `destination`.
///
/// The staged tree is durable before the rename, the previous tree is moved
/// aside rather than deleted in place, and the parent directory is fsynced, so a
/// crash always leaves either the complete previous tree or the complete new
/// one — never a half-copied record.
pub(crate) fn install_staged_tree(staged: &Path, destination: &Path) -> Result<()> {
    let parent = destination
        .parent()
        .context("import destination has no parent")?;
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .context("import destination has no UTF-8 name")?;
    std::fs::create_dir_all(parent)?;
    fsync_tree(staged)?;
    let superseded = parent.join(format!(".{name}.superseded"));
    if superseded.exists() {
        std::fs::remove_dir_all(&superseded)?;
    }
    if destination.exists() {
        std::fs::rename(destination, &superseded)?;
    }
    std::fs::rename(staged, destination)?;
    File::open(parent)?.sync_all()?;
    if superseded.exists() {
        std::fs::remove_dir_all(&superseded)?;
    }
    Ok(())
}

pub(crate) fn worker_report_schema(engine: &str) -> &'static str {
    match engine {
        "web" => "wisent.web-worker-report.v1",
        "docs" => "wisent.docs-worker-report.v1",
        _ => "wisent.native-worker-report.v1",
    }
}

/// Read the exact typed worker report out of one attempt's retained output log.
///
/// The importer accepts only the engine's declared report schema on its own
/// line. There is no `command_output.log` fallback and no heuristic scan, so a
/// worker that failed to print its typed report is an import failure rather than
/// an invitation to guess.
pub(crate) fn retained_worker_report(engine: &str, output_log: &Path) -> Result<Value> {
    let metadata = std::fs::symlink_metadata(output_log)
        .with_context(|| format!("read retained worker output {}", output_log.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!(
            "retained worker output {} is not a regular file",
            output_log.display()
        );
    }
    if metadata.len() > MAX_WORKER_OUTPUT_BYTES {
        bail!(
            "retained worker output {} exceeds the {MAX_WORKER_OUTPUT_BYTES}-byte bound",
            output_log.display()
        );
    }
    let bytes = std::fs::read(output_log)?;
    let text = String::from_utf8_lossy(&bytes);
    let schema = worker_report_schema(engine);
    text.lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .find(|value| value.get("schema").and_then(Value::as_str) == Some(schema))
        .with_context(|| format!("retained worker output carries no {schema} report"))
}

/// Prove the worker report describes exactly this attempt.
///
/// Every identity field must equal the immutable manifest, the artifact URI must
/// be the canonical attempt coordinate, and the retained Stado submission
/// receipt must bind the same job and source revision.
///
/// What this document is NOT: content-addressed. Unlike the evidence manifest, the
/// provenance document and every retained artifact, the report has no digest committed
/// anywhere before it is read — `retained_worker_report` takes the last matching line of
/// the worker's own output log, and the artifact digest it declares is proved against the
/// bytes in durable storage rather than against an independently signed value. What the
/// report carries is therefore trusted only as far as something else re-proves it: the
/// checks below bind it to the immutable attempt and to the retained submission receipt,
/// the archive is re-hashed, and every field of the attempt envelope that matters is
/// re-compared against the SIGNED receipt claims by
/// `weles_provenance::verify_attempt_binding` at record-verification time. Nothing here
/// may be read as proof of a fact that no signature or digest covers.
pub(crate) fn verify_worker_report(
    report: &Value,
    manifest: &RuntimeManifest,
    entry: &Value,
    receipt: &Value,
    // `artifact_published` for the accepted attempt, `failed` for a non-success attempt
    // whose signed failure proof is being imported. Everything else this function proves is
    // identical for both, so neither path gets its own weaker identity rules.
    expected_state: &str,
) -> Result<Value> {
    let expected_strings = [
        ("run_id", manifest.run_id.as_str()),
        ("catalog", manifest.catalog.as_str()),
        ("record", manifest.record.as_str()),
        ("record_key", manifest.record_key.as_str()),
        ("attempt_id", manifest.attempt_id.as_str()),
        ("engine", manifest.engine.as_str()),
        ("source_revision", manifest.source_revision.as_str()),
        ("source_input_sha256", manifest.source_input_sha256.as_str()),
        ("reference_sha256", manifest.reference_sha256.as_str()),
        (
            "bindings_file_sha256",
            manifest.bindings_file_sha256.as_str(),
        ),
        ("bindings_sha256", manifest.bindings_sha256.as_str()),
    ];
    for (field, expected) in expected_strings {
        let observed = report.get(field).and_then(Value::as_str);
        if observed != Some(expected) {
            bail!(
                "worker report {field} is {observed:?} but the immutable attempt declares {expected:?}"
            );
        }
    }
    if report.get("attempt").and_then(Value::as_u64) != Some(u64::from(manifest.attempt)) {
        bail!("worker report attempt differs from the immutable attempt");
    }
    if report.get("state").and_then(Value::as_str) != Some(expected_state) {
        bail!(
            "worker report state is {:?}, not the {expected_state} state this import requires",
            report.get("state")
        );
    }
    let artifact = report
        .get("artifact")
        .filter(|value| value.is_object())
        .cloned()
        .context("worker report has no typed published artifact")?;
    if artifact.get("uri").and_then(Value::as_str) != Some(manifest.artifact_uri.as_str()) {
        bail!("worker report artifact URI is not the canonical attempt coordinate");
    }
    let sha256 = artifact
        .get("sha256")
        .and_then(Value::as_str)
        .filter(|value| is_lower_sha256(value))
        .context("worker report artifact has no lowercase SHA-256 digest")?;
    let bytes = artifact
        .get("bytes")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .context("worker report artifact has no positive byte count")?;
    let job_id = entry
        .get("stado_job_id")
        .and_then(Value::as_str)
        .context("imported record retains no Stado job id")?;
    if receipt.get("stado_job_id").and_then(Value::as_str) != Some(job_id) {
        bail!("retained submission receipt names a different Stado job");
    }
    if receipt
        .pointer("/stado_receipt/source_revision")
        .and_then(Value::as_str)
        != Some(manifest.source_revision.as_str())
    {
        bail!("retained submission receipt does not bind the attempt source revision");
    }
    if receipt
        .pointer("/stado_receipt/jobs/0/output_uri")
        .and_then(Value::as_str)
        != Some(manifest.output_uri.as_str())
    {
        bail!("retained submission receipt output URI is not the canonical attempt coordinate");
    }
    Ok(json!({"sha256": sha256, "bytes": bytes, "artifact": artifact}))
}
