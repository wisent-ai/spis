use super::*;

pub(crate) fn inventory_sha256(
    targets: &[CrawlTarget],
    diagnostics: &[CrawlDiagnostic],
    robots: &RobotsSnapshot,
    downloaded_bytes: u64,
    capacity: Option<CorpusCapacity>,
) -> Result<String> {
    inventory_digest(
        serde_json::to_value(targets)?,
        serde_json::to_value(diagnostics)?,
        serde_json::to_value(robots)?,
        downloaded_bytes,
        capacity
            .map(|value| -> Result<Value> { Ok(serde_json::to_value(value)?) })
            .transpose()?,
    )
}

pub(crate) fn validate_state(
    state: &DurableState,
    manifest: &super::crawl::RuntimeManifest,
    source_url: &str,
) -> Result<()> {
    if state.schema != "wisent.docs-crawl-state.v3" {
        bail!(
            "durable documentation state uses unsupported schema {}",
            state.schema
        );
    }
    let (attempt, attempt_id) = manifest_attempt(manifest)?;
    if state.run_id != manifest.run_id
        || state.source_revision != manifest.source_revision
        || state.source_input_sha256 != manifest.source_input_sha256
        || state.record_key != manifest.record_key
        || state.record != manifest.record
        || state.attempt != attempt
        || state.attempt_id != attempt_id
        || state.source_url != source_url
    {
        bail!(
            "durable documentation state identity does not match runtime manifest run_id/source_revision/source_input_sha256/record_key/record/attempt/attempt_id/source"
        );
    }
    let policy = UrlPolicy::new(source_url)?;
    let effective = policy.canonical(
        &state.effective_source_url,
        None,
        "durable effective documentation source URL",
    )?;
    if effective.as_str() != state.effective_source_url {
        bail!("durable effective documentation source URL is not canonical");
    }
    if state.started_at.is_empty() {
        bail!("durable documentation state has no started_at");
    }
    if !state.inventory_complete {
        if state.inventory_sha256.is_some()
            || !state.inventory_diagnostics.is_empty()
            || state.inventory_downloaded_bytes != 0
            || state.robots.is_some()
            || !state.targets.is_empty()
            || !state.outcomes.is_empty()
            || state.committed_bytes != 0
            || state.completed_at.is_some()
            || state.report_sha256.is_some()
        {
            bail!("incomplete documentation inventory has committed crawl data");
        }
        return Ok(());
    }
    let robots = state
        .robots
        .as_ref()
        .context("complete documentation inventory has no persisted robots policy")?;
    if state.inventory_diagnostics.len() > MAX_INVENTORY_DIAGNOSTICS + 1
        || robots.directives.len() > MAX_ROBOTS_RULES
    {
        bail!("durable documentation inventory exceeds persisted diagnostic/rule bounds");
    }
    if state.inventory_downloaded_bytes > MAX_TOTAL_INVENTORY_BYTES {
        bail!(
            "durable inventory download counter exceeds the {MAX_TOTAL_INVENTORY_BYTES}-byte limit"
        );
    }
    let expected_inventory_sha256 = inventory_sha256(
        &state.targets,
        &state.inventory_diagnostics,
        robots,
        state.inventory_downloaded_bytes,
        state.corpus_capacity,
    )?;
    if state.inventory_sha256.as_deref() != Some(expected_inventory_sha256.as_str()) {
        bail!("durable documentation target inventory digest does not match its contents");
    }
    if state.targets.is_empty() || state.targets.len() > MAX_TARGETS {
        bail!("durable documentation target inventory has an invalid target count");
    }
    if !state
        .targets
        .iter()
        .any(|target| target.url == policy.source_url.as_str())
    {
        bail!("durable documentation target inventory does not include canonical source_url");
    }
    let mut seen_urls = HashSet::new();
    let mut previous_url: Option<&str> = None;
    for (sequence, target) in state.targets.iter().enumerate() {
        let canonical = policy.canonical(&target.url, None, "durable documentation target")?;
        if canonical.as_str() != target.url
            || target.sequence != sequence
            || target.key != lib::sha256_hex(target.url.as_bytes())
            || previous_url.is_some_and(|previous| previous >= target.url.as_str())
            || !seen_urls.insert(target.url.as_str())
        {
            bail!("durable documentation target inventory is not canonical and ordered");
        }
        previous_url = Some(&target.url);
    }
    let mut missing = false;
    let mut committed_end = 0u64;
    let mut downloaded_bytes = state.inventory_downloaded_bytes;
    for target in &state.targets {
        match state.outcomes.get(&target.key) {
            Some(outcome) => {
                if missing {
                    bail!("durable documentation outcomes are not a contiguous target prefix");
                }
                if outcome.sequence != target.sequence || outcome.url != target.url {
                    bail!(
                        "durable documentation outcome {} does not match its target",
                        target.key
                    );
                }
                match (
                    outcome.record_sha256.as_deref(),
                    outcome.corpus_start,
                    outcome.corpus_end,
                ) {
                    (Some(digest), Some(start), Some(end)) => {
                        exact_lower_hex(digest, 64, "durable page record_sha256")?;
                        if start != committed_end || end <= start {
                            bail!(
                                "durable documentation outcome {} has a non-contiguous corpus range",
                                target.key
                            );
                        }
                        committed_end = end;
                    }
                    (None, None, None) => {}
                    _ => bail!(
                        "durable documentation outcome {} has an incomplete corpus range",
                        target.key
                    ),
                }
                let resolved = policy.canonical(
                    &outcome.resolved_url,
                    None,
                    "durable resolved documentation page URL",
                )?;
                if resolved.as_str() != outcome.resolved_url {
                    bail!(
                        "durable documentation outcome {} has a noncanonical resolved URL",
                        target.key
                    );
                }
                if outcome.record_sha256.is_some() != outcome.text_bytes.is_some() {
                    bail!(
                        "durable documentation outcome {} has inconsistent text metadata",
                        target.key
                    );
                }
                downloaded_bytes = downloaded_bytes
                    .checked_add(outcome.downloaded_bytes)
                    .context("durable download byte counter overflow")?;
            }
            None => missing = true,
        }
    }
    if state.outcomes.len() > state.targets.len()
        || state
            .outcomes
            .keys()
            .any(|key| !state.targets.iter().any(|target| &target.key == key))
    {
        bail!("durable documentation state contains outcomes outside its target inventory");
    }
    if committed_end != state.committed_bytes {
        bail!(
            "durable documentation state commits {} bytes but outcome ranges end at {}",
            state.committed_bytes,
            committed_end
        );
    }
    if downloaded_bytes > MAX_TOTAL_DOWNLOAD_BYTES {
        bail!(
            "durable download byte counter exceeds the {MAX_TOTAL_DOWNLOAD_BYTES}-byte limit"
        );
    }
    let has_completed_at = state.completed_at.is_some();
    let has_report_sha256 = state.report_sha256.is_some();
    if has_completed_at != has_report_sha256
        || (has_completed_at && state.outcomes.len() != state.targets.len())
    {
        bail!("durable documentation completion marker and report digest are inconsistent");
    }
    if let Some(digest) = &state.report_sha256 {
        exact_lower_hex(digest, 64, "durable crawl report_sha256")?;
    }
    Ok(())
}

pub(crate) fn hash_reader(mut reader: impl Read) -> Result<(String, u64)> {
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        total += read as u64;
    }
    Ok((hex::encode(hasher.finalize()), total))
}

pub(crate) fn reconcile_corpus(layout: &WorkLayout, state: &DurableState) -> Result<()> {
    std::fs::create_dir_all(&layout.corpus).with_context(|| {
        format!(
            "create durable documentation corpus {}",
            layout.corpus.display()
        )
    })?;
    let mut file = open_regular_file(
        &layout.pages,
        true,
        true,
        false,
        true,
        "durable documentation pages",
    )?;
    let actual = file.metadata()?.len();
    if actual < state.committed_bytes {
        bail!(
            "durable documentation corpus is truncated: checkpoint commits {} bytes but stream has {} bytes",
            state.committed_bytes,
            actual
        );
    }
    if actual > state.committed_bytes {
        file.set_len(state.committed_bytes).with_context(|| {
            format!(
                "truncate uncommitted documentation corpus tail {}",
                layout.pages.display()
            )
        })?;
        file.sync_all().with_context(|| {
            format!(
                "fsync recovered documentation corpus {}",
                layout.pages.display()
            )
        })?;
        File::open(&layout.corpus)?.sync_all()?;
    }
    file.seek(SeekFrom::Start(0))?;
    let (digest, length) = hash_reader(file.take(state.committed_bytes))?;
    if length != state.committed_bytes || digest != state.committed_sha256 {
        bail!(
            "durable documentation corpus digest differs from the checkpoint at {} bytes",
            state.committed_bytes
        );
    }
    Ok(())
}
