use super::*;

pub(crate) fn validate_outcomes(corpus_dir: &Path, state: &Value, report: &Value) -> Result<()> {
    let targets = state
        .get("targets")
        .and_then(Value::as_array)
        .context("durable state has no targets")?;
    let inventory_downloaded_bytes = state
        .get("inventory_downloaded_bytes")
        .and_then(Value::as_u64)
        .context("durable state has no inventory_downloaded_bytes")?;
    // The same derivation the producer hashes, called rather than copied.
    let capacity = state
        .get("corpus_capacity")
        .filter(|value| !value.is_null())
        .cloned();
    let inventory_sha256 = super::crawl_docs::inventory_digest(
        Value::Array(targets.clone()),
        state
            .get("inventory_diagnostics")
            .context("durable state has no inventory diagnostics")?
            .clone(),
        state
            .get("robots")
            .context("durable state has no robots policy")?
            .clone(),
        inventory_downloaded_bytes,
        capacity.clone(),
    )?;
    if state.get("inventory_sha256").and_then(Value::as_str)
        != Some(inventory_sha256.as_str())
    {
        bail!("durable inventory digest differs from its exact contents");
    }
    let outcomes = state
        .get("outcomes")
        .and_then(Value::as_object)
        .context("durable state has no outcomes")?;
    if targets.is_empty() || targets.len() > MAX_PAGE_RECORDS || outcomes.len() != targets.len() {
        bail!("durable target and outcome counts are invalid");
    }
    let declared = url::Url::parse(
        state
            .get("source_url")
            .and_then(Value::as_str)
            .context("durable state has no source_url")?,
    )?;
    let origin = declared.origin();
    let effective = url::Url::parse(
        state
            .get("effective_source_url")
            .and_then(Value::as_str)
            .context("durable state has no effective_source_url")?,
    )?;
    if effective.origin() != origin || effective.as_str() != state["effective_source_url"].as_str().unwrap() {
        bail!("durable effective source URL is noncanonical or cross-origin");
    }
    let mut committed_end = 0u64;
    let mut downloaded_bytes = 0u64;
    let mut retrieved_count = 0usize;
    let mut ok_count = 0usize;
    let mut text_page_count = 0usize;
    let mut outcome_diagnostic_count = 0usize;
    for (sequence, target) in targets.iter().enumerate() {
        let url = target
            .get("url")
            .and_then(Value::as_str)
            .context("durable target has no URL")?;
        let parsed = url::Url::parse(url)?;
        let key = target
            .get("key")
            .and_then(Value::as_str)
            .context("durable target has no key")?;
        if parsed.as_str() != url
            || parsed.origin() != origin
            || target.get("sequence").and_then(Value::as_u64) != Some(sequence as u64)
            || key != lib::sha256_hex(url.as_bytes())
        {
            bail!("durable target inventory is not canonical and ordered");
        }
        let outcome = outcomes
            .get(key)
            .context("durable state is missing a target outcome")?;
        let resolved = url::Url::parse(
            outcome
                .get("resolved_url")
                .and_then(Value::as_str)
                .context("durable outcome has no resolved_url")?,
        )?;
        if outcome.get("sequence").and_then(Value::as_u64) != Some(sequence as u64)
            || outcome.get("url").and_then(Value::as_str) != Some(url)
            || resolved.origin() != origin
            || resolved.as_str() != outcome["resolved_url"].as_str().unwrap()
        {
            bail!("durable outcome identity is not canonical");
        }
        downloaded_bytes = downloaded_bytes
            .checked_add(outcome.get("downloaded_bytes").and_then(Value::as_u64).unwrap_or(0))
            .context("durable download byte counter overflow")?;
        let digest = outcome.get("record_sha256").and_then(Value::as_str);
        let start = outcome.get("corpus_start").and_then(Value::as_u64);
        let end = outcome.get("corpus_end").and_then(Value::as_u64);
        match (digest, start, end) {
            (Some(digest), Some(start), Some(end)) => {
                exact_lower_hex(digest, "page record_sha256")?;
                if start != committed_end || end <= start {
                    bail!("durable page ranges are not contiguous");
                }
                committed_end = end;
                retrieved_count += 1;
            }
            (None, None, None) => {}
            _ => bail!("durable page outcome has incomplete corpus range metadata"),
        }
        let text_bytes = outcome.get("text_bytes").and_then(Value::as_u64);
        if digest.is_some() != text_bytes.is_some() {
            bail!("durable page text metadata is inconsistent");
        }
        if text_bytes.unwrap_or(0) > 0 {
            text_page_count += 1;
        }
        if outcome.get("status").and_then(Value::as_u64) == Some(200) {
            ok_count += 1;
        }
        if !outcome.get("diagnostic").unwrap_or(&Value::Null).is_null() {
            outcome_diagnostic_count += 1;
        }
    }
    if committed_end != state["committed_bytes"].as_u64().unwrap_or(u64::MAX)
        || downloaded_bytes > MAX_TOTAL_PAGE_DOWNLOAD_BYTES
    {
        bail!("durable corpus byte counters are inconsistent or exceed their limits");
    }
    let inventory_downloaded_bytes = state
        .get("inventory_downloaded_bytes")
        .and_then(Value::as_u64)
        .context("durable state has no inventory_downloaded_bytes")?;
    if inventory_downloaded_bytes > MAX_TOTAL_INVENTORY_BYTES {
        bail!("durable inventory download counter exceeds its limit");
    }
    let total_downloaded_bytes = inventory_downloaded_bytes
        .checked_add(downloaded_bytes)
        .filter(|bytes| *bytes <= MAX_TOTAL_DOWNLOAD_BYTES)
        .context("durable total download counter exceeds its limit")?;
    if report.get("inventory_downloaded_bytes").and_then(Value::as_u64)
        != Some(inventory_downloaded_bytes)
        || report.get("page_downloaded_bytes").and_then(Value::as_u64)
            != Some(downloaded_bytes)
        || report.get("downloaded_bytes").and_then(Value::as_u64)
            != Some(total_downloaded_bytes)
        || report
            .pointer("/retrieval/inventory_downloaded_bytes")
            .and_then(Value::as_u64)
            != Some(inventory_downloaded_bytes)
        || report
            .pointer("/retrieval/page_downloaded_bytes")
            .and_then(Value::as_u64)
            != Some(downloaded_bytes)
        || report.pointer("/retrieval/downloaded_bytes").and_then(Value::as_u64)
            != Some(total_downloaded_bytes)
    {
        bail!("retrieval report download byte counters differ from durable state");
    }
    let (pages_sha256, pages_bytes) = hash_file(&corpus_dir.join("pages.jsonl.gz"))?;
    let expected_sha256 = state
        .get("committed_sha256")
        .and_then(Value::as_str)
        .context("durable state has no committed_sha256")?;
    if report.get("pages_sha256").and_then(Value::as_str) != Some(pages_sha256.as_str())
        || report.get("pages_bytes").and_then(Value::as_u64) != Some(pages_bytes)
        || report.pointer("/retrieval/pages_sha256").and_then(Value::as_str)
            != Some(pages_sha256.as_str())
        || report.pointer("/retrieval/pages_bytes").and_then(Value::as_u64) != Some(pages_bytes)
    {
        bail!("retrieval report page digest or length differs from the exact corpus");
    }
    exact_lower_hex(expected_sha256, "state committed_sha256")?;
    if pages_bytes != committed_end || pages_sha256 != expected_sha256 {
        bail!("documentation gzip stream differs from the durable length or SHA-256");
    }
    let inventory_diagnostics = state
        .get("inventory_diagnostics")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    // One derivation, two callers. This side used to re-implement the rule
    // and the two implementations disagreed, which is what made an over-bound
    // site both unretrievable and unreportable: the producer wrote
    // `retrieval_complete` and this refused it with "counts or completion
    // status differ from durable outcomes", a sentence about arithmetic for a
    // fact about capacity.
    let declared_outside = capacity
        .as_ref()
        .and_then(|value| value.get("pages_outside_corpus"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let declared_exact = capacity
        .as_ref()
        .and_then(|value| value.get("exact"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let expected_status = super::crawl_docs::retrieval_status(&super::crawl_docs::RetrievalCounts {
        target_count: targets.len(),
        retrieved_count,
        ok_count,
        text_page_count,
        diagnostic_count: inventory_diagnostics + outcome_diagnostic_count,
        pages_outside_corpus: declared_outside,
    });
    if report.get("retrieval_status").and_then(Value::as_str) != Some(expected_status)
        || report.pointer("/retrieval/target_count").and_then(Value::as_u64)
            != Some(targets.len() as u64)
        || report.pointer("/retrieval/outcome_count").and_then(Value::as_u64)
            != Some(outcomes.len() as u64)
        || report.pointer("/retrieval/retrieved_count").and_then(Value::as_u64)
            != Some(retrieved_count as u64)
        || report.pointer("/retrieval/text_page_count").and_then(Value::as_u64)
            != Some(text_page_count as u64)
        || report.pointer("/retrieval/page_downloaded_bytes").and_then(Value::as_u64)
            != Some(downloaded_bytes)
        // The quantity is held to the durable inventory exactly as every
        // other count is. A report free to name its own remainder is a
        // report that can under-state what a record is missing.
        || report.pointer("/retrieval/pages_outside_corpus").and_then(Value::as_u64)
            != Some(declared_outside)
        || report
            .pointer("/retrieval/pages_outside_corpus_exact")
            .and_then(Value::as_bool)
            != Some(declared_exact)
        || report.pointer("/retrieval/corpus_bound").and_then(Value::as_u64)
            != Some(MAX_PAGE_RECORDS as u64)
    {
        bail!("retrieval report counts or completion status differ from durable outcomes");
    }
    validate_journal(corpus_dir, state)
}
