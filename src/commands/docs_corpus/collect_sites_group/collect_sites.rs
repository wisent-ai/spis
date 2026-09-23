use super::*;

pub(crate) fn collect_sites() -> Result<Vec<SiteInfo>> {
    let selected = selected_corpora()?;
    let mut out = Vec::new();
    for entry in std::fs::read_dir(engine_root())? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json")
            || path.file_name().and_then(|name| name.to_str()) == Some("full-text-manifest.json")
        {
            continue;
        }
        let meta = read_json(&path)?;
        let slug = path
            .file_stem()
            .and_then(|value| value.to_str())
            .context("documentation structure file has a non-UTF-8 name")?
            .to_string();
        let corpus = selected.get(&slug);
        let outcomes = corpus
            .and_then(|attempt| attempt.state.get("outcomes"))
            .and_then(Value::as_object);
        let seen = outcomes.map_or(0, serde_json::Map::len);
        let cumulative_ok = outcomes.map_or(0, |values| {
            values
                .values()
                .filter(|outcome| outcome.get("text_bytes").and_then(Value::as_u64).unwrap_or(0) > 0)
                .count()
        });
        let target_count = corpus
            .and_then(|attempt| attempt.report.pointer("/retrieval/target_count"))
            .and_then(Value::as_i64)
            .or_else(|| meta.get("inventory_url_count").and_then(Value::as_i64))
            .unwrap_or(0);
        out.push(SiteInfo {
            slug,
            name: meta.get("name").and_then(Value::as_str).unwrap_or_default().to_string(),
            category: meta
                .get("category")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            source_url: meta
                .get("source_url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            inventory_url_count: target_count,
            seen,
            cumulative_ok,
            noise: seen.saturating_sub(cumulative_ok),
            retrieval_status: corpus.map(|attempt| attempt.retrieval_status.clone()),
            // Read off the durable state, which is the copy the inventory
            // digest covers, rather than off the report, which is derived
            // from it.
            pages_outside_corpus: corpus
                .and_then(|attempt| {
                    attempt
                        .state
                        .pointer("/corpus_capacity/pages_outside_corpus")
                        .and_then(Value::as_u64)
                })
                .unwrap_or(0),
            pages_outside_corpus_exact: corpus
                .and_then(|attempt| {
                    attempt
                        .state
                        .pointer("/corpus_capacity/exact")
                        .and_then(Value::as_bool)
                })
                .unwrap_or(true),
            attempt: corpus.map(|attempt| attempt.attempt),
            attempt_id: corpus.map(|attempt| attempt.attempt_id.clone()),
            corpus_dir: corpus.map(|attempt| attempt.corpus_dir.clone()),
        });
    }
    out.sort_by(|left, right| right.inventory_url_count.cmp(&left.inventory_url_count));
    Ok(out)
}

pub(crate) fn open_jsonl(
    corpus_dir: Option<&Path>,
) -> Result<Option<std::io::BufReader<flate2::read::MultiGzDecoder<File>>>> {
    let Some(corpus_dir) = corpus_dir else {
        return Ok(None);
    };
    let path = corpus_dir.join("pages.jsonl.gz");
    let file = open_regular_read(&path, "documentation pages")?;
    Ok(Some(std::io::BufReader::new(
        flate2::read::MultiGzDecoder::new(file),
    )))
}

pub(crate) fn read_corpus_record(
    reader: &mut impl BufRead,
    decompressed_bytes: &mut u64,
    record_count: &mut usize,
) -> Result<Option<Value>> {
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf().context("decompress documentation corpus")?;
        if available.is_empty() {
            if line.is_empty() {
                return Ok(None);
            }
            bail!("documentation corpus ends with an incomplete JSON record");
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(available.len(), |position| position + 1);
        let content = newline.map_or(consumed, |position| position);
        if line.len().saturating_add(content) > MAX_PAGE_RECORD_BYTES {
            bail!("documentation corpus record exceeds its decompressed byte limit");
        }
        *decompressed_bytes = decompressed_bytes
            .checked_add(consumed as u64)
            .filter(|bytes| *bytes <= MAX_DECOMPRESSED_CORPUS_BYTES)
            .context("documentation corpus exceeds its decompressed byte limit")?;
        line.extend_from_slice(&available[..content]);
        reader.consume(consumed);
        if newline.is_some() {
            *record_count = record_count
                .checked_add(1)
                .filter(|count| *count <= MAX_PAGE_RECORDS)
                .context("documentation corpus exceeds its record-count limit")?;
            let record = serde_json::from_slice(&line)
                .context("parse bounded documentation corpus record")?;
            return Ok(Some(record));
        }
    }
}

pub(crate) fn scan_site(
    site: &SiteInfo,
    query: &str,
    limit: usize,
    hits: &mut Vec<Value>,
) -> Result<usize> {
    let Some(mut reader) = open_jsonl(site.corpus_dir.as_deref())? else {
        return Ok(0);
    };
    let query_lower = query.to_lowercase();
    let mut scanned = 0usize;
    let mut decompressed_bytes = 0u64;
    let mut record_count = 0usize;
    while let Some(record) =
        read_corpus_record(&mut reader, &mut decompressed_bytes, &mut record_count)?
    {
        scanned += 1;
        let text = record.get("text").and_then(Value::as_str).unwrap_or("");
        let title = record.get("title").and_then(Value::as_str).unwrap_or("");
        let url = record.get("url").and_then(Value::as_str).unwrap_or("");
        let text_lower = text.to_lowercase();
        if !text_lower.contains(&query_lower)
            && !title.to_lowercase().contains(&query_lower)
            && !url.to_lowercase().contains(&query_lower)
        {
            continue;
        }
        let snippet = text_lower.find(&query_lower).map(|byte_position| {
            let start = text_lower[..byte_position]
                .char_indices()
                .rev()
                .nth(60)
                .map(|(offset, _)| offset)
                .unwrap_or(0);
            let requested_end = byte_position
                .saturating_add(query_lower.len())
                .saturating_add(120);
            let end = text_lower
                .char_indices()
                .find_map(|(offset, _)| (offset >= requested_end).then_some(offset))
                .unwrap_or(text_lower.len());
            text_lower[start..end].replace('\n', " ")
        });
        hits.push(json!({
            "slug": site.slug,
            "site": site.name,
            "url": url,
            "title": (!title.is_empty()).then_some(title),
            "snippet": snippet,
            "attempt": site.attempt,
            "attempt_id": site.attempt_id,
        }));
        if hits.len() >= limit {
            break;
        }
    }
    Ok(scanned)
}

pub(crate) fn archive_member_name(path: &Path) -> Result<String> {
    let components = path
        .components()
        .filter_map(|component| match component {
            Component::CurDir => None,
            Component::Normal(value) => Some(Ok(value.to_string_lossy().to_string())),
            _ => Some(Err(anyhow::anyhow!("retrieval archive contains an unsafe path"))),
        })
        .collect::<Result<Vec<_>>>()?;
    match components.as_slice() {
        [name] => Ok(name.clone()),
        _ => bail!("retrieval archive member is outside its exact corpus directory"),
    }
}

pub(crate) fn extract_corpus_archive(archive_path: &Path, corpus_dir: &Path) -> Result<()> {
    std::fs::create_dir(corpus_dir)?;
    let decoder = flate2::read::GzDecoder::new(open_regular_read(
        archive_path,
        "documentation archive",
    )?);
    let mut archive = tar::Archive::new(decoder);
    let expected = CORPUS_FILES.iter().copied().collect::<HashSet<_>>();
    let mut observed = HashSet::<String>::new();
    let mut total = 0u64;
    for entry in archive.entries().context("read retrieval archive entries")? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            bail!("retrieval archive contains a non-regular member");
        }
        let name = archive_member_name(&entry.path()?)?;
        if !expected.contains(name.as_str()) || !observed.insert(name.clone()) {
            bail!("retrieval archive contains an unexpected or duplicate member {name}");
        }
        total = total
            .checked_add(entry.size())
            .filter(|bytes| *bytes <= MAX_IMPORTED_CORPUS_BYTES)
            .context("retrieval archive exceeds the extracted corpus byte limit")?;
        let destination = corpus_dir.join(&name);
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&destination)?;
        std::io::copy(&mut entry, &mut output)?;
        output.flush()?;
        output.sync_all()?;
    }
    if observed.len() != expected.len() {
        bail!("retrieval archive does not contain the exact corpus artifact set");
    }
    File::open(corpus_dir)?.sync_all()?;
    Ok(())
}

pub(crate) fn validate_installed_import(
    destination: &Path,
    uri: &str,
    expected_archive_sha256: &str,
    expected_archive_bytes: u64,
) -> Result<AttemptCorpus> {
    let archive_path = destination.join("artifact.tar.gz");
    let (archive_sha256, archive_bytes) = hash_file(&archive_path)?;
    if archive_sha256 != expected_archive_sha256 || archive_bytes != expected_archive_bytes {
        bail!("installed immutable documentation artifact differs from the expected digest or length");
    }
    validate_corpus(
        &destination.join("corpus"),
        Some(uri),
        CorpusOrigin::Imported,
    )
}
