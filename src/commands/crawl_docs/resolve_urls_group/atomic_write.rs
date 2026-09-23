use super::*;

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("durable file path has no parent")?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create durable directory {}", parent.display()))?;
    regular_file_exists(path, "durable checkpoint")?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .context("durable filename is not UTF-8")?;
    // Stage outside the directory being written. `corpus_summary` and
    // `audit_attempt_tree` both require the attempt root to hold exactly the four
    // run artifacts, so a temp file orphaned there by SIGKILL/OOM would brick
    // every later resume of the attempt. The parent already carries the crawl
    // lock, the archive lock, the published archive and the read-back file, so it
    // is the established staging location and is on the same filesystem, which
    // keeps the `rename` below atomic.
    let staging_parent = parent
        .parent()
        .context("durable file path has no staging parent")?;
    let owner = parent
        .file_name()
        .and_then(|value| value.to_str())
        .context("durable file path has no UTF-8 owning directory")?;
    std::fs::create_dir_all(staging_parent).with_context(|| {
        format!(
            "create durable staging directory {}",
            staging_parent.display()
        )
    })?;
    let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    let temporary = staging_parent.join(temporary_name(owner, name, sequence));
    let result = (|| -> Result<()> {
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&temporary)
            .with_context(|| format!("create atomic checkpoint {}", temporary.display()))?;
        output
            .write_all(bytes)
            .with_context(|| format!("write atomic checkpoint {}", temporary.display()))?;
        output
            .flush()
            .with_context(|| format!("flush atomic checkpoint {}", temporary.display()))?;
        output
            .sync_all()
            .with_context(|| format!("fsync atomic checkpoint {}", temporary.display()))?;
        std::fs::rename(&temporary, path).with_context(|| {
            format!(
                "replace durable checkpoint {} with {}",
                temporary.display(),
                path.display()
            )
        })?;
        File::open(parent)
            .with_context(|| format!("open checkpoint parent {}", parent.display()))?
            .sync_all()
            .with_context(|| format!("fsync checkpoint parent {}", parent.display()))?;
        Ok(())
    })();
    if result.is_err() && temporary.exists() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

pub(crate) fn state_bytes(state: &DurableState) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(state)?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn checkpoint_state(path: &Path, state: &DurableState) -> Result<()> {
    atomic_write(path, &state_bytes(state)?)
        .with_context(|| format!("checkpoint documentation crawl state {}", path.display()))
}

pub(crate) fn read_state(path: &Path) -> Result<DurableState> {
    let mut file = open_regular_file(path, true, false, false, false, "durable state")?;
    if file.metadata()?.len() > MAX_STATE_BYTES {
        bail!("durable documentation state exceeds its byte limit");
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse durable documentation state {}", path.display()))
}

pub(crate) fn reset_outcome_journal(layout: &WorkLayout) -> Result<()> {
    let journal = open_regular_file(
        &layout.journal,
        false,
        true,
        false,
        true,
        "outcome journal",
    )?;
    journal.set_len(0)?;
    journal
        .sync_all()
        .with_context(|| format!("fsync reset outcome journal {}", layout.journal.display()))?;
    File::open(&layout.corpus)?
        .sync_all()
        .context("fsync corpus directory after resetting outcome journal")
}

pub(crate) fn replay_outcome_journal(layout: &WorkLayout, state: &mut DurableState) -> Result<()> {
    if !regular_file_exists(&layout.journal, "outcome journal")? {
        reset_outcome_journal(layout)?;
    }
    let mut journal_file =
        open_regular_file(&layout.journal, true, false, false, false, "outcome journal")?;
    let journal_length = journal_file.metadata()?.len();
    if journal_length > MAX_JOURNAL_BYTES {
        bail!(
            "documentation outcome journal exceeds the {MAX_JOURNAL_BYTES}-byte limit"
        );
    }
    let mut bytes = Vec::with_capacity(journal_length as usize);
    journal_file.read_to_end(&mut bytes)?;
    let complete_length = bytes
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    if complete_length != bytes.len() {
        bytes.truncate(complete_length);
        let journal = open_regular_file(
            &layout.journal,
            false,
            true,
            false,
            false,
            "outcome journal",
        )?;
        journal.set_len(complete_length as u64)?;
        journal.sync_all()?;
        File::open(&layout.corpus)?.sync_all()?;
    }

    let persisted_outcomes = state.outcomes.clone();
    let persisted_bytes = state.committed_bytes;
    let persisted_sha256 = state.committed_sha256.clone();
    let persisted_effective_source_url = state.effective_source_url.clone();
    let canonical_declared_source_url = Url::parse(&state.source_url)?.to_string();
    state.outcomes.clear();
    state.committed_bytes = 0;
    state.committed_sha256 = lib::sha256_hex(&[]);
    state.effective_source_url = canonical_declared_source_url.clone();
    for (line_index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        if line.is_empty() {
            continue;
        }
        let batch: OutcomeJournalBatch = serde_json::from_slice(line).with_context(|| {
            format!("parse outcome journal line {}", line_index + 1)
        })?;
        if batch.schema != "wisent.docs-outcome-batch.v1"
            || batch.outcomes.is_empty()
            || batch.outcomes.len() > WRITER_BATCH_SIZE
            || batch.first_sequence != state.outcomes.len()
            || batch.last_sequence + 1
                != batch.first_sequence.saturating_add(batch.outcomes.len())
        {
            bail!("outcome journal line {} is not canonical", line_index + 1);
        }
        for entry in batch.outcomes {
            let expected_sequence = state.outcomes.len();
            let target = state.targets.get(expected_sequence).with_context(|| {
                format!(
                    "outcome journal line {} exceeds target inventory",
                    line_index + 1
                )
            })?;
            if target.url == canonical_declared_source_url {
                state.effective_source_url = entry.outcome.resolved_url.clone();
            }
            if entry.key != target.key
                || entry.outcome.sequence != expected_sequence
                || entry.outcome.url != target.url
                || state.outcomes.insert(entry.key, entry.outcome).is_some()
            {
                bail!(
                    "outcome journal line {} does not match ordered target {}",
                    line_index + 1,
                    target.url
                );
            }
        }
        exact_lower_hex(
            &batch.committed_sha256,
            64,
            "outcome journal committed_sha256",
        )?;
        state.committed_bytes = batch.committed_bytes;
        state.committed_sha256 = batch.committed_sha256;
    }
    if !persisted_outcomes.is_empty() || persisted_bytes != 0 {
        if serde_json::to_value(&persisted_outcomes)? != serde_json::to_value(&state.outcomes)?
            || persisted_bytes != state.committed_bytes
            || persisted_sha256 != state.committed_sha256
            || persisted_effective_source_url != state.effective_source_url
        {
            bail!("durable state progress differs from its append-only outcome journal");
        }
    }
    Ok(())
}

pub(crate) fn fresh_state(
    manifest: &super::crawl::RuntimeManifest,
    policy: &UrlPolicy,
    started_at: String,
) -> Result<DurableState> {
    let (attempt, attempt_id) = manifest_attempt(manifest)?;
    Ok(DurableState {
        schema: "wisent.docs-crawl-state.v3".into(),
        run_id: manifest.run_id.clone(),
        source_revision: manifest.source_revision.clone(),
        source_input_sha256: manifest.source_input_sha256.clone(),
        record_key: manifest.record_key.clone(),
        record: manifest.record.clone(),
        attempt,
        attempt_id,
        source_url: policy.declared_source_url.clone(),
        effective_source_url: policy.source_url.as_str().to_string(),
        started_at,
        inventory_complete: false,
        inventory_sha256: None,
        inventory_diagnostics: Vec::new(),
        corpus_capacity: None,
        inventory_downloaded_bytes: 0,
        robots: None,
        targets: Vec::new(),
        outcomes: BTreeMap::new(),
        committed_bytes: 0,
        committed_sha256: lib::sha256_hex(&[]),
        completed_at: None,
        report_sha256: None,
    })
}

/// The one derivation of a record's inventory digest, called by the producer
/// through [`inventory_sha256`] and by `docs_corpus`'s validator directly.
///
/// One function for the same reason [`retrieval_status`] is one: the digest
/// is a contract between two modules, and a second implementation of it is a
/// second opinion about whether a corpus is authentic.
///
/// `capacity` is inserted only when the corpus bound actually excluded
/// pages, so every corpus written before that member existed hashes to
/// exactly the digest it already carries and stays valid. `serde_json` runs
/// with `preserve_order`, so the insertion order below is part of the
/// contract.
pub(crate) fn inventory_digest(
    targets: Value,
    diagnostics: Value,
    robots: Value,
    downloaded_bytes: u64,
    capacity: Option<Value>,
) -> Result<String> {
    let mut descriptor = serde_json::Map::new();
    descriptor.insert("targets".into(), targets);
    descriptor.insert("diagnostics".into(), diagnostics);
    descriptor.insert("robots".into(), robots);
    descriptor.insert("downloaded_bytes".into(), json!(downloaded_bytes));
    if let Some(capacity) = capacity {
        descriptor.insert("corpus_capacity".into(), capacity);
    }
    Ok(lib::sha256_hex(&serde_json::to_vec(&Value::Object(
        descriptor,
    ))?))
}
