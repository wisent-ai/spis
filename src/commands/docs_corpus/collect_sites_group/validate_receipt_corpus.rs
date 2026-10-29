use super::*;

pub(crate) fn validate_receipt_corpus(receipt: &Value, corpus: &AttemptCorpus) -> Result<()> {
    for field in [
        "run_id",
        "record",
        "record_key",
        "attempt_id",
        "source_revision",
        "source_input_sha256",
    ] {
        if receipt.get(field) != corpus.report.get(field) {
            bail!("attempt receipt and retrieval corpus disagree on {field}");
        }
    }
    for field in [
        "reference_sha256",
        "bindings_file_sha256",
        "bindings_sha256",
        "docs_structure_sha256",
    ] {
        if receipt.get(field) != corpus.report.pointer(&format!("/runtime_manifest/{field}")) {
            bail!("attempt receipt and runtime manifest disagree on {field}");
        }
    }
    if receipt.get("attempt") != corpus.report.get("attempt")
        || receipt.get("catalog") != corpus.report.pointer("/runtime_manifest/catalog")
        || receipt.get("execution_identity") != corpus.report.get("runtime_execution_identity")
        || receipt.get("docs_structure_sha256") != corpus.report.get("structure_sha256")
    {
        bail!("attempt receipt and retrieval corpus identity differ");
    }
    // The sibling `artifact.tar.gz` only exists for corpora that `import_artifact`
    // installed. A locally crawled corpus has no archive next to it, so refuse it
    // here rather than reporting a missing-file error from `hash_file`.
    if corpus.origin != CorpusOrigin::Imported {
        bail!(
            "attempt receipt validation requires an imported documentation corpus; {} was crawled locally and has no immutable artifact archive",
            corpus.corpus_dir.display()
        );
    }
    let archive_path = corpus
        .corpus_dir
        .parent()
        .context("imported corpus has no immutable archive parent")?
        .join("artifact.tar.gz");
    let (archive_sha256, archive_bytes) = hash_file(&archive_path)?;
    let artifact = receipt["artifact"]
        .as_object()
        .context("validated attempt receipt has no artifact")?;
    if artifact.get("sha256").and_then(Value::as_str) != Some(archive_sha256.as_str())
        || artifact.get("bytes").and_then(Value::as_u64) != Some(archive_bytes)
    {
        bail!("attempt receipt archive digest or byte length differs from imported artifact");
    }
    // Each file's length is a u64; their sum is taken wide so it cannot wrap,
    // then refused if it does not fit the receipt's field.
    let corpus_bytes = CORPUS_FILES
        .iter()
        .map(|name| -> Result<u128> { Ok(u128::from(hash_file(&corpus.corpus_dir.join(name))?.1)) })
        .sum::<Result<u128>>()?;
    let corpus_bytes =
        u64::try_from(corpus_bytes).context("imported corpus byte count overflow")?;
    let expected_pages = corpus
        .report
        .pointer("/retrieval/retrieved_count")
        .and_then(Value::as_u64)
        .context("retrieval report has no retrieved_count")?;
    // The exact file count is already proven against `CORPUS_FILES` by
    // `validate_docs_worker_report`, which every path into this function runs first and
    // which compares the same constant, so it is not restated here. Bytes and pages are
    // a separate rule: they are compared with the corpus that was actually materialised.
    if receipt.pointer("/corpus/bytes").and_then(Value::as_u64) != Some(corpus_bytes)
        || receipt.pointer("/corpus/pages").and_then(Value::as_u64) != Some(expected_pages)
    {
        bail!("attempt receipt corpus summary differs from imported corpus");
    }
    Ok(())
}

pub(crate) fn import_verified_worker_report(
    receipt: &Value,
    source_archive: Option<&Path>,
) -> Result<Value> {
    let (uri, expected_sha256, expected_bytes) = validate_docs_worker_report(receipt)?;
    let attempt =
        import_artifact_from_archive(&uri, &expected_sha256, expected_bytes, source_archive)?;
    validate_receipt_corpus(receipt, &attempt)?;
    Ok(json!({
        "artifact_uri": uri,
        "archive_sha256": expected_sha256,
        "record": attempt.slug,
        "attempt": attempt.attempt,
        "attempt_id": attempt.attempt_id,
        "completed_at": attempt.completed_at,
        "retrieval_status": attempt.retrieval_status,
        "corpus_dir": attempt.corpus_dir,
    }))
}

pub(crate) fn import_worker_report(receipt: &Value) -> Result<Value> {
    import_verified_worker_report(receipt, None)
}

pub(crate) fn import_worker_report_from_archive(
    receipt: &Value,
    source_archive: &Path,
) -> Result<Value> {
    import_verified_worker_report(receipt, Some(source_archive))
}
