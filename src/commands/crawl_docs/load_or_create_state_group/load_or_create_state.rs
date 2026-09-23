use super::*;

pub(crate) fn load_or_create_state(
    layout: &WorkLayout,
    manifest: &super::crawl::RuntimeManifest,
    meta: &SiteMeta,
    rules: &SiteRules,
    refresh: bool,
    invocation_started_at: String,
) -> Result<DurableState> {
    std::fs::create_dir_all(&layout.corpus).with_context(|| {
        format!(
            "create durable documentation corpus {}",
            layout.corpus.display()
        )
    })?;
    prune_stale_temporaries(layout)?;
    let policy = UrlPolicy::new(&meta.source_url)?;
    let existing = regular_file_exists(&layout.state, "durable state")?;
    let mut state = if existing {
        let state = read_state(&layout.state)?;
        validate_state(&state, manifest, &meta.source_url)?;
        state
    } else {
        fresh_state(manifest, &policy, invocation_started_at.clone())?
    };
    if refresh {
        refuse_published_refresh(&manifest.artifact_uri)?;
        state = fresh_state(manifest, &policy, invocation_started_at)?;
        checkpoint_state(&layout.state, &state)?;
        reset_outcome_journal(layout)?;
    } else if !existing {
        checkpoint_state(&layout.state, &state)?;
        reset_outcome_journal(layout)?;
    } else if state.inventory_complete {
        replay_outcome_journal(layout, &mut state)?;
        validate_state(&state, manifest, &meta.source_url)?;
    } else {
        reset_outcome_journal(layout)?;
    }
    reconcile_corpus(layout, &state)?;
    if !state.inventory_complete {
        eprintln!(
            "[{}] {} ({}): resolving URL inventory ({})",
            lib::now_iso_utc(),
            manifest.record,
            meta.name,
            meta.inventory_source
        );
        let resolution = resolve_urls(meta, rules, &policy)?;
        let mut resolved = resolution.pages;
        resolved.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
        });
        resolved.dedup_by(|left, right| left.0 == right.0);
        state.targets = resolved
            .into_iter()
            .take(MAX_TARGETS)
            .enumerate()
            .map(|(sequence, (url, lastmod))| CrawlTarget {
                sequence,
                key: lib::sha256_hex(url.as_bytes()),
                url,
                lastmod,
            })
            .collect();
        state.inventory_diagnostics = resolution.diagnostics;
        state.corpus_capacity = resolution.capacity;
        state.robots = Some(resolution.robots);
        state.inventory_downloaded_bytes = resolution.downloaded_bytes;
        state.inventory_sha256 = Some(inventory_sha256(
            &state.targets,
            &state.inventory_diagnostics,
            state
                .robots
                .as_ref()
                .context("resolved documentation inventory has no robots policy")?,
            state.inventory_downloaded_bytes,
            state.corpus_capacity,
        )?);
        state.inventory_complete = true;
        checkpoint_state(&layout.state, &state)?;
        eprintln!(
            "[{}] {}: {} canonical candidate URLs; {} inventory diagnostics",
            lib::now_iso_utc(),
            manifest.record,
            state.targets.len(),
            state.inventory_diagnostics.len()
        );
    }
    validate_state(&state, manifest, &meta.source_url)?;
    Ok(state)
}

pub(crate) fn report_bytes(report: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(report)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Everything the terminal state of one documentation run is decided from.
pub(crate) struct RetrievalCounts {
    pub target_count: usize,
    pub retrieved_count: usize,
    pub ok_count: usize,
    pub text_page_count: usize,
    pub diagnostic_count: usize,
    pub pages_outside_corpus: u64,
}

/// The one derivation of a run's terminal state, called by the producer here
/// and by `docs_corpus`'s validator.
///
/// It is one function because it was two expressions, and they disagreed: the
/// producer read only per-page diagnostics while the validator also counted
/// inventory diagnostics, so an over-bound site whose pages all fetched
/// cleanly was written `retrieval_complete` and then refused on import with
/// "counts or completion status differ from durable outcomes". Two readings of
/// one rule is how a site ends up neither retrieved nor reported.
///
/// `retrieval_over_capacity` outranks `retrieval_partial` deliberately. A
/// partial run can be completed by retrying it; a run over capacity cannot be
/// completed by any number of retries, because the material does not fit the
/// record. They are different states and the operator's decision differs.
pub(crate) fn retrieval_status(counts: &RetrievalCounts) -> &'static str {
    if counts.target_count == 0 || counts.retrieved_count == 0 {
        "retrieval_empty"
    } else if counts.text_page_count == 0 {
        "retrieval_no_text"
    } else if counts.pages_outside_corpus > 0 {
        "retrieval_over_capacity"
    } else if counts.diagnostic_count != 0
        || counts.retrieved_count != counts.target_count
        || counts.ok_count != counts.target_count
        || counts.text_page_count != counts.target_count
    {
        "retrieval_partial"
    } else {
        "retrieval_complete"
    }
}

pub(crate) fn build_report(
    manifest: &super::crawl::RuntimeManifest,
    state: &DurableState,
    structure_sha256: &str,
    completed_at: &str,
) -> Result<Value> {
    let definition_path = source_root()
        .join("documentation-site-examples/content-structure/full-text-manifest.json");
    let definition_hash = lib::sha256_hex(
        &std::fs::read(&definition_path).with_context(|| {
            format!(
                "read documentation crawl definition {}",
                definition_path.display()
            )
        })?,
    );
    let mut diagnostics = state
        .inventory_diagnostics
        .iter()
        .map(serde_json::to_value)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut page_downloaded_bytes = 0u64;
    let mut retrieved_count = 0usize;
    let mut ok_count = 0usize;
    let mut text_page_count = 0usize;
    for target in &state.targets {
        let outcome = state
            .outcomes
            .get(&target.key)
            .with_context(|| format!("missing final outcome for {}", target.url))?;
        if outcome.record_sha256.is_some() {
            retrieved_count += 1;
        }
        if outcome.status.as_u64() == Some(200) {
            ok_count += 1;
        }
        page_downloaded_bytes = page_downloaded_bytes
            .checked_add(outcome.downloaded_bytes)
            .context("documentation report download byte counter overflow")?;
        if outcome.text_bytes.unwrap_or(0) > 0 {
            text_page_count += 1;
        }
        if let Some(diagnostic) = &outcome.diagnostic {
            diagnostics.push(serde_json::to_value(diagnostic)?);
        }
    }
    let capacity = state.corpus_capacity;
    let retrieval_status = retrieval_status(&RetrievalCounts {
        target_count: state.targets.len(),
        retrieved_count,
        ok_count,
        text_page_count,
        diagnostic_count: diagnostics.len(),
        pages_outside_corpus: capacity.map_or(0, |value| value.pages_outside_corpus),
    });
    let records = vec![json!({
        "page_downloaded_bytes": page_downloaded_bytes,
        "record": manifest.record,
        "target_count": state.targets.len(),
        "outcome_count": state.outcomes.len(),
        "retrieved_count": retrieved_count,
        "http_200_count": ok_count,
        "text_page_count": text_page_count,
        "pages_sha256": state.committed_sha256,
        "pages_bytes": state.committed_bytes,
        "retrieval_status": retrieval_status,
        "corpus_bound": MAX_TARGETS,
        "pages_outside_corpus": capacity.map_or(0, |value| value.pages_outside_corpus),
        "pages_outside_corpus_exact": capacity.is_none_or(|value| value.exact),
        "diagnostics": diagnostics,
    })];
    let (attempt, attempt_id) = manifest_attempt(manifest)?;
    Ok(json!({
        "schema": "wisent.docs-retrieval-run.v2",
        "tool": "spis crawl-docs",
        "tool_commit": source_revision()?,
        "run_id": manifest.run_id,
        "record": manifest.record,
        "record_key": manifest.record_key,
        "attempt": attempt,
        "attempt_id": attempt_id,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "source_url": state.source_url,
        "declared_source_url": state.source_url,
        "effective_source_url": state.effective_source_url,
        "runtime_manifest": manifest,
        "runtime_execution_identity": manifest.execution_identity,
        "definition_sha256": definition_hash,
        "structure_sha256": structure_sha256,
        "inventory_sha256": state.inventory_sha256,
        "pages_sha256": state.committed_sha256,
        "pages_bytes": state.committed_bytes,
        "started_at": state.started_at,
        "completed_at": completed_at,
        "retrieval_status": retrieval_status,
            "inventory_downloaded_bytes": state.inventory_downloaded_bytes,
            "page_downloaded_bytes": page_downloaded_bytes,
            "downloaded_bytes": state
                .inventory_downloaded_bytes
                .checked_add(page_downloaded_bytes)
                .context("documentation report total download byte counter overflow")?,
        "retrieval": {
            "records": records,
            "target_count": state.targets.len(),
            "outcome_count": state.outcomes.len(),
            "retrieved_count": retrieved_count,
            "text_page_count": text_page_count,
            "pages_sha256": state.committed_sha256,
            "pages_bytes": state.committed_bytes,
            "inventory_downloaded_bytes": state.inventory_downloaded_bytes,
            "page_downloaded_bytes": page_downloaded_bytes,
            "corpus_bound": MAX_TARGETS,
            "pages_outside_corpus": capacity.map_or(0, |value| value.pages_outside_corpus),
            "pages_outside_corpus_exact": capacity.is_none_or(|value| value.exact),
            "downloaded_bytes": state
                .inventory_downloaded_bytes
                .checked_add(page_downloaded_bytes)
                .context("documentation report total download byte counter overflow")?,
            "diagnostics": diagnostics,
        },
        "limitations": [
            "The documentation retrieval engine measures bounded HTTP retrieval and retained response text only.",
            "No interactive journey, accessibility traversal, or motion variant is part of this engine."
        ],
    }))
}
