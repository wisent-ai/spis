use super::*;

pub(crate) fn import_artifact_from_archive(
    uri: &str,
    expected_archive_sha256: &str,
    expected_archive_bytes: u64,
    source_archive: Option<&Path>,
) -> Result<AttemptCorpus> {
    if !uri.starts_with(&format!("{}/", crate::CRAWL_ATTEMPT_ROOT))
        || !uri.ends_with("/artifacts.tar.gz")
    {
        bail!("--artifact-uri must be an immutable Spis crawl artifact URI");
    }
    exact_lower_hex(expected_archive_sha256, "--archive-sha256")?;
    let root = imports_root()?;
    std::fs::create_dir_all(&root)?;
    let digest = lib::sha256_hex(uri.as_bytes());
    let destination = root.join(&digest);
    if existing_regular_directory(&destination, "installed documentation corpus")? {
        return validate_installed_import(
            &destination,
            uri,
            expected_archive_sha256,
            expected_archive_bytes,
        );
    }
    let staging = super::crawl_docs::staging_directory(&root, "corpus-import-stage")?;
    let archive_path = staging.join("artifact.tar.gz");
    let import_result = (|| -> Result<AttemptCorpus> {
        if let Some(source) = source_archive {
            open_regular_read(source, "already downloaded documentation archive")?;
            std::fs::copy(source, &archive_path)
                .context("stage already downloaded documentation corpus artifact")?;
        } else {
            let mut command = super::crawl::crawl_storage_command();
            command.args(["storage", "get", uri]).arg(&archive_path);
            let output = super::crawl::bounded_command_output(
                &mut command,
                "download immutable documentation corpus artifact",
                std::time::Duration::from_secs(30 * 60),
                super::crawl_docs::STADO_OUTPUT_LIMIT,
            )?;
            if !output.status.success() {
                bail!(
                    "stado storage get refused documentation corpus artifact: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }
        }
        let archive = open_regular_read(&archive_path, "downloaded documentation archive")?;
        let archive_length = archive.metadata()?.len();
        if archive_length > MAX_IMPORTED_ARCHIVE_BYTES {
            bail!("documentation corpus artifact exceeds the import byte limit");
        }
        drop(archive);
        let (archive_sha256, archive_bytes) = hash_file(&archive_path)?;
        if archive_sha256 != expected_archive_sha256
            || archive_bytes != expected_archive_bytes
        {
            bail!("downloaded documentation artifact differs from the expected digest or length");
        }
        let staged_corpus = staging.join("corpus");
        extract_corpus_archive(&archive_path, &staged_corpus)?;
        validate_corpus(&staged_corpus, Some(uri), CorpusOrigin::Imported)?;
        open_regular_read(&archive_path, "downloaded documentation archive")?.sync_all()?;
        File::open(&staging)?.sync_all()?;
        match std::fs::rename(&staging, &destination) {
            Ok(()) => {
                File::open(&root)?.sync_all()?;
                validate_installed_import(
                    &destination,
                    uri,
                    expected_archive_sha256,
                    expected_archive_bytes,
                )
            }
            Err(_error)
                if existing_regular_directory(
                    &destination,
                    "concurrently installed documentation corpus",
                )? =>
            {
                std::fs::remove_dir_all(&staging)?;
                let existing = validate_installed_import(
                    &destination,
                    uri,
                    expected_archive_sha256,
                    expected_archive_bytes,
                )
                .context("concurrent import installed different content")?;
                File::open(&root)?.sync_all()?;
                Ok(existing)
            }
            Err(error) => Err(error).context("atomically install imported documentation corpus"),
        }
    })();
    if import_result.is_err() && staging.exists() {
        std::fs::remove_dir_all(&staging)?;
        File::open(&root)?.sync_all()?;
    }
    import_result
}

/// Validate one typed `wisent.docs-worker-report.v1` document in memory.
///
/// This is the single validation of a documentation worker report, shared by
/// both import paths. `docs-corpus import --attempt-receipt` and
/// `crawl.rs::import_record_attempt` both continue through
/// [`import_worker_report`] so a crawl reported as imported is immediately
/// visible to the product's corpus status, search, and show commands.
/// Returns the artifact URI and its SHA-256.
pub(crate) fn validate_docs_worker_report(receipt: &Value) -> Result<(String, String)> {
    if receipt.get("schema").and_then(Value::as_str) != Some("wisent.docs-worker-report.v1")
        || receipt.get("engine").and_then(Value::as_str) != Some("docs")
        || receipt.get("state").and_then(Value::as_str) != Some("artifact_published")
        || !receipt.get("failure").unwrap_or(&Value::Null).is_null()
    {
        bail!("attempt receipt is not a successful typed documentation worker report");
    }
    let artifact = receipt
        .get("artifact")
        .and_then(Value::as_object)
        .context("attempt receipt has no artifact object")?;
    let uri = artifact
        .get("uri")
        .and_then(Value::as_str)
        .context("attempt receipt artifact has no URI")?
        .to_string();
    let archive_sha256 = artifact
        .get("sha256")
        .and_then(Value::as_str)
        .context("attempt receipt artifact has no SHA-256")?
        .to_string();
    exact_lower_hex(&archive_sha256, "attempt receipt artifact SHA-256")?;
    let archive_bytes = artifact
        .get("bytes")
        .and_then(Value::as_u64)
        .filter(|bytes| *bytes > 0 && *bytes <= MAX_IMPORTED_ARCHIVE_BYTES)
        .context("attempt receipt artifact has no valid byte length")?;
    if artifact.get("media_type").and_then(Value::as_str) != Some("application/gzip") {
        bail!("attempt receipt artifact media_type is not application/gzip");
    }
    let tree_entries = artifact
        .get("tree_entries")
        .and_then(Value::as_u64)
        .context("attempt receipt artifact has no tree_entries")?;
    let tree_bytes = artifact
        .get("tree_bytes")
        .and_then(Value::as_u64)
        .context("attempt receipt artifact has no tree_bytes")?;
    let corpus = receipt
        .get("corpus")
        .and_then(Value::as_object)
        .context("successful attempt receipt has no corpus summary")?;
    let corpus_files = corpus
        .get("files")
        .and_then(Value::as_u64)
        .filter(|files| *files == CORPUS_FILES.len() as u64)
        .context("attempt receipt corpus does not name the exact file count")?;
    let corpus_bytes = corpus
        .get("bytes")
        .and_then(Value::as_u64)
        .filter(|bytes| *bytes > 0 && *bytes <= MAX_IMPORTED_CORPUS_BYTES)
        .context("attempt receipt corpus has no valid byte count")?;
    corpus
        .get("pages")
        .and_then(Value::as_u64)
        .filter(|pages| *pages <= MAX_PAGE_RECORDS as u64)
        .context("attempt receipt corpus has no valid page count")?;
    if tree_entries != corpus_files || tree_bytes != corpus_bytes {
        bail!("attempt receipt artifact tree and corpus summaries disagree");
    }
    let source_revision = receipt
        .get("source_revision")
        .and_then(Value::as_str)
        .context("attempt receipt has no source_revision")?;
    if source_revision.len() != 40
        || !source_revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("attempt receipt source_revision is not a full lowercase commit SHA");
    }
    for field in [
        "record_key",
        "source_input_sha256",
        "reference_sha256",
        "bindings_file_sha256",
        "bindings_sha256",
        "docs_structure_sha256",
    ] {
        exact_lower_hex(
            receipt
                .get(field)
                .and_then(Value::as_str)
                .with_context(|| format!("attempt receipt has no {field}"))?,
            &format!("attempt receipt {field}"),
        )?;
    }
    receipt
        .get("execution_identity")
        .and_then(Value::as_object)
        .context("attempt receipt has no execution_identity object")?;
    let run_id = receipt
        .get("run_id")
        .and_then(Value::as_str)
        .context("attempt receipt has no run_id")?;
    let catalog = receipt
        .get("catalog")
        .and_then(Value::as_str)
        .context("attempt receipt has no catalog")?;
    let record = receipt
        .get("record")
        .and_then(Value::as_str)
        .context("attempt receipt has no record")?;
    let record_key = receipt
        .get("record_key")
        .and_then(Value::as_str)
        .context("attempt receipt has no record_key")?;
    let attempt = receipt
        .get("attempt")
        .and_then(Value::as_u64)
        .filter(|attempt| *attempt > 0)
        .context("attempt receipt has no positive attempt")?;
    let attempt_id = receipt
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("attempt receipt has no attempt_id")?;
    for (value, label) in [
        (run_id, "receipt run_id"),
        (catalog, "receipt catalog"),
        (record, "receipt record"),
        (attempt_id, "receipt attempt_id"),
    ] {
        safe_component(value, label)?;
    }
    let expected_uri = format!(
        "{}/artifacts.tar.gz",
        crate::crawl_attempt_base_uri(run_id, catalog, record, record_key, attempt, attempt_id)
    );
    if uri != expected_uri {
        bail!("attempt receipt artifact URI does not match its immutable coordinates");
    }
    let _ = archive_bytes;
    Ok((uri, archive_sha256))
}

pub(crate) fn read_attempt_receipt(path: &Path) -> Result<(Value, String, String)> {
    let receipt = read_last_worker_report(path)?;
    let (uri, archive_sha256) = validate_docs_worker_report(&receipt)?;
    Ok((receipt, uri, archive_sha256))
}
