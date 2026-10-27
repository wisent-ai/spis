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
        // Counted over the attempt's outcomes; a site with no selected corpus
        // has none to count.
        let seen = outcomes.into_iter().flatten().count();
        let cumulative_ok = outcomes
            .into_iter()
            .flat_map(serde_json::Map::values)
            .filter(|outcome| {
                outcome
                    .get("text_bytes")
                    .and_then(Value::as_u64)
                    .and_then(std::num::NonZeroU64::new)
                    .is_some()
            })
            .count();
        let target_count = corpus
            .and_then(|attempt| attempt.report.pointer("/retrieval/target_count"))
            .and_then(Value::as_i64)
            .or_else(|| meta.get("inventory_url_count").and_then(Value::as_i64))
            .with_context(|| {
                format!("documentation site {slug} states no target_count or inventory_url_count")
            })?;
        // A structure file that names no site, category or source is refused
        // with its slug, never listed with an empty one.
        let stated = |field: &str| {
            meta.get(field)
                .and_then(Value::as_str)
                .map(str::to_string)
                .with_context(|| format!("documentation site {slug} states no {field}"))
        };
        let (name, category, source_url) =
            (stated("name")?, stated("category")?, stated("source_url")?);
        out.push(SiteInfo {
            slug,
            name,
            category,
            source_url,
            inventory_url_count: target_count,
            seen,
            cumulative_ok,
            noise: seen.saturating_sub(cumulative_ok),
            retrieval_status: corpus.map(|attempt| attempt.retrieval_status.clone()),
            attempt: corpus.map(|attempt| attempt.attempt),
            attempt_id: corpus.map(|attempt| attempt.attempt_id.clone()),
            corpus_dir: corpus.map(|attempt| attempt.corpus_dir.clone()),
        });
    }
    out.sort_by(|left, right| right.inventory_url_count.cmp(&left.inventory_url_count));
    Ok(out)
}

pub(crate) fn open_jsonl(
    corpus_dir: &Path,
) -> Result<std::io::BufReader<flate2::read::MultiGzDecoder<File>>> {
    let path = corpus_dir.join("pages.jsonl.gz");
    let file = open_regular_read(&path, "documentation pages")?;
    Ok(std::io::BufReader::new(flate2::read::MultiGzDecoder::new(
        file,
    )))
}

/// The next record of a corpus page stream, or `None` at its end. A record is
/// as long as the page it holds and is read to its newline, without a size
/// chosen here; a stream that ends inside a record is refused.
pub(crate) fn read_corpus_record(reader: &mut impl BufRead) -> Result<Option<Value>> {
    let mut line = Vec::new();
    reader
        .read_until(b'\n', &mut line)
        .context("decompress documentation corpus")?;
    if line.is_empty() {
        return Ok(None);
    }
    if line.pop() != Some(b'\n') {
        bail!("documentation corpus ends with an incomplete JSON record");
    }
    serde_json::from_slice(&line)
        .map(Some)
        .context("parse documentation corpus record")
}

/// Appends every record mentioning the already-lowercased query to `hits`,
/// stopping once `hits` holds `limit` when the caller asked for a limit, and
/// answers how many records it read.
pub(crate) fn scan_site(
    site: &SiteInfo,
    corpus_dir: &Path,
    query_lower: &str,
    limit: Option<std::num::NonZeroUsize>,
    hits: &mut Vec<Value>,
) -> Result<usize> {
    let mut reader = open_jsonl(corpus_dir)?;
    let mut failure = None;
    let scanned = std::iter::from_fn(|| {
        if limit.is_some_and(|limit| hits.len() >= limit.get()) {
            return None;
        }
        let read = read_corpus_record(&mut reader).and_then(|record| {
            record
                .map(|record| matching_hit(site, query_lower, &record).map(|hit| hits.extend(hit)))
                .transpose()
        });
        match read {
            Ok(record) => record,
            Err(error) => {
                failure = Some(error);
                None
            }
        }
    })
    .count();
    match failure {
        Some(error) => Err(error),
        None => Ok(scanned),
    }
}

/// The hit a record makes when its text, title or URL mentions the query.
/// Every record names its URL; one that does not is refused rather than
/// listed as a hit nobody can open. A record without text or a title is
/// matched on what it has.
fn matching_hit(site: &SiteInfo, query_lower: &str, record: &Value) -> Result<Option<Value>> {
    let url = record.get("url").and_then(Value::as_str).with_context(|| {
        format!(
            "documentation corpus {} holds a record without its url",
            site.slug
        )
    })?;
    let text = record.get("text").and_then(Value::as_str);
    let title = record
        .get("title")
        .and_then(Value::as_str)
        .filter(|title| !title.is_empty());
    let mentioned = text.is_some_and(|text| text.to_lowercase().contains(query_lower))
        || title.is_some_and(|title| title.to_lowercase().contains(query_lower))
        || url.to_lowercase().contains(query_lower);
    if !mentioned {
        return Ok(None);
    }
    let snippet = text.and_then(|text| paragraph_around(text, query_lower));
    Ok(Some(json!({
        "slug": site.slug,
        "site": site.name,
        "url": url,
        "title": title,
        "snippet": snippet,
        "attempt": site.attempt,
        "attempt_id": site.attempt_id,
    })))
}

/// The line of `text` holding the first mention of the query, as the page
/// wrote it, so a hit shows the passage that matched rather than a width
/// chosen here. A mention that spans a line break has no single line to show.
fn paragraph_around(text: &str, query_lower: &str) -> Option<String> {
    text.lines()
        .find(|line| line.to_lowercase().contains(query_lower))
        .map(str::to_string)
}

pub(crate) fn archive_member_name(path: &Path) -> Result<String> {
    let components = path
        .components()
        .filter_map(|component| match component {
            Component::CurDir => None,
            Component::Normal(value) => Some(Ok(value.to_string_lossy().to_string())),
            _ => Some(Err(anyhow::anyhow!(
                "retrieval archive contains an unsafe path"
            ))),
        })
        .collect::<Result<Vec<_>>>()?;
    match components.as_slice() {
        [name] => Ok(name.clone()),
        _ => bail!("retrieval archive member is outside its exact corpus directory"),
    }
}

pub(crate) fn extract_corpus_archive(archive_path: &Path, corpus_dir: &Path) -> Result<()> {
    std::fs::create_dir(corpus_dir)?;
    // The extracted corpus may take the room its volume reports now, no more.
    let (room_bytes, _) = crate::commands::crawl::volume_room(corpus_dir)?;
    let decoder =
        flate2::read::GzDecoder::new(open_regular_read(archive_path, "documentation archive")?);
    let mut archive = tar::Archive::new(decoder);
    let expected = CORPUS_FILES.iter().copied().collect::<HashSet<_>>();
    let mut observed = HashSet::<String>::new();
    let mut room = room_bytes;
    for entry in archive
        .entries()
        .context("read retrieval archive entries")?
    {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            bail!("retrieval archive contains a non-regular member");
        }
        let name = archive_member_name(&entry.path()?)?;
        if !expected.contains(name.as_str()) || !observed.insert(name.clone()) {
            bail!("retrieval archive contains an unexpected or duplicate member {name}");
        }
        room = room.checked_sub(entry.size()).with_context(|| {
            format!(
                "retrieval archive needs more than the {room_bytes} bytes free on the volume \
                 of {}",
                corpus_dir.display()
            )
        })?;
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
        bail!(
            "installed immutable documentation artifact differs from the expected digest or length"
        );
    }
    validate_corpus(
        &destination.join("corpus"),
        Some(uri),
        CorpusOrigin::Imported,
    )
}
