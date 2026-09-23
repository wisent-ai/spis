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
    let mut corpus_bytes = 0u64;
    for name in CORPUS_FILES {
        corpus_bytes = corpus_bytes
            .checked_add(hash_file(&corpus.corpus_dir.join(name))?.1)
            .context("imported corpus byte count overflow")?;
    }
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

pub(crate) fn import_verified_worker_report(receipt: &Value, source_archive: Option<&Path>) -> Result<Value> {
    let (uri, expected_sha256) = validate_docs_worker_report(receipt)?;
    let expected_bytes = receipt
        .pointer("/artifact/bytes")
        .and_then(Value::as_u64)
        .context("validated attempt receipt artifact has no bytes")?;
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

pub fn run(rest: &[String]) -> Result<()> {
    let mut sub = "";
    let mut query = String::new();
    let mut site: Option<String> = None;
    let mut url_filter = String::new();
    let mut attempt_receipt: Option<PathBuf> = None;
    let mut limit = 20usize;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "status" | "search" | "show" | "import" => sub = rest[index].as_str(),
            "--query" => {
                index += 1;
                query = rest.get(index).context("--query needs a value")?.clone();
            }
            "--site" => {
                index += 1;
                site = Some(rest.get(index).context("--site needs a value")?.clone());
            }
            "--url" => {
                index += 1;
                url_filter = rest.get(index).context("--url needs a value")?.clone();
            }
            "--attempt-receipt" => {
                index += 1;
                attempt_receipt = Some(PathBuf::from(
                    rest.get(index)
                        .context("--attempt-receipt needs a value")?,
                ));
            }
            "--limit" => {
                index += 1;
                limit = rest.get(index).context("--limit needs a value")?.parse()?;
            }
            other => bail!("unknown argument: {other}"),
        }
        index += 1;
    }
    if sub.is_empty() {
        bail!("usage: spis docs-corpus status | search --query T [--site S] [--limit N] | show --site S --url U | import --attempt-receipt FILE");
    }
    if limit == 0 || limit > 10_000 {
        bail!("--limit must be between 1 and 10000");
    }

    match sub {
        "status" => {
            let sites = collect_sites()?;
            let output = sites
                .iter()
                .map(|site| {
                    json!({
                        "slug": site.slug,
                        "name": site.name,
                        "category": site.category,
                        "source_url": site.source_url,
                        "inventory_url_count": site.inventory_url_count,
                        "seen": site.seen,
                        "cumulative_ok": site.cumulative_ok,
                        "noise": site.noise,
                        // `done` stays exactly "is this record complete", and
                        // an over-capacity record is not: it is as retrieved
                        // as this contract can make it, with a stated number
                        // of pages that no attempt of this record can hold.
                        "done": site.retrieval_status.as_deref() == Some("retrieval_complete"),
                        "retrieval_status": site.retrieval_status,
                        "corpus_bound": MAX_PAGE_RECORDS,
                        "pages_outside_corpus": site.pages_outside_corpus,
                        "pages_outside_corpus_exact": site.pages_outside_corpus_exact,
                        "attempt": site.attempt,
                        "attempt_id": site.attempt_id,
                    })
                })
                .collect::<Vec<_>>();
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
        "search" => {
            if query.is_empty() {
                bail!("--query required");
            }
            let sites = collect_sites()?;
            let mut hits = Vec::new();
            let mut scanned = 0usize;
            for candidate in &sites {
                if site.as_deref().is_some_and(|slug| slug != candidate.slug) {
                    continue;
                }
                scanned += scan_site(candidate, &query, limit, &mut hits)?;
                if hits.len() >= limit {
                    break;
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"hits": hits, "scanned": scanned, "limit": limit})
                )?
            );
            Ok(())
        }
        "show" => {
            let slug = site.context("show needs --site <slug>")?;
            if url_filter.is_empty() {
                bail!("show needs --url <url>");
            }
            let selected = selected_corpora()?;
            let corpus = selected
                .get(&slug)
                .with_context(|| format!("no completed v2 retrieval corpus for {slug}"))?;
            let mut reader = open_jsonl(Some(&corpus.corpus_dir))?
                .context("selected retrieval corpus has no page stream")?;
            let mut decompressed_bytes = 0u64;
            let mut record_count = 0usize;
            while let Some(record) =
                read_corpus_record(&mut reader, &mut decompressed_bytes, &mut record_count)?
            {
                if record.get("url").and_then(Value::as_str) == Some(url_filter.as_str()) {
                    println!("{record}");
                    return Ok(());
                }
            }
            bail!("url not found in the completed {slug} retrieval corpus")
        }
        "import" => {
            let receipt_path =
                attempt_receipt.context("import needs --attempt-receipt <file>")?;
            let (receipt, _, _) = read_attempt_receipt(&receipt_path)?;
            let mut imported = import_worker_report(&receipt)?;
            imported["attempt_receipt"] = json!(receipt_path);
            println!("{}", serde_json::to_string_pretty(&imported)?);
            Ok(())
        }
        _ => unreachable!(),
    }
}
