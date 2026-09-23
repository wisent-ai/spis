use super::*;

pub(crate) fn finish_run(
    layout: &WorkLayout,
    manifest: &super::crawl::RuntimeManifest,
    mut state: DurableState,
    structure_sha256: &str,
) -> Result<(DurableState, Value)> {
    if let (Some(_), Some(expected_report_sha256)) =
        (&state.completed_at, &state.report_sha256)
    {
        let mut report_file = open_regular_file(
            &layout.report,
            true,
            false,
            false,
            false,
            "completed documentation crawl report",
        )?;
        if report_file.metadata()?.len() > MAX_STATE_BYTES {
            bail!("completed documentation crawl report exceeds its byte limit");
        }
        let mut bytes = Vec::new();
        report_file.read_to_end(&mut bytes)?;
        if lib::sha256_hex(&bytes) != *expected_report_sha256 {
            bail!("completed documentation crawl report differs from its durable digest");
        }
        let report = serde_json::from_slice(&bytes).with_context(|| {
            format!(
                "parse completed documentation crawl report {}",
                layout.report.display()
            )
        })?;
        return Ok((state, report));
    }
    if state.outcomes.len() != state.targets.len() {
        bail!(
            "documentation crawl ended with {} of {} target outcomes durably committed",
            state.outcomes.len(),
            state.targets.len()
        );
    }
    let completed_at = lib::now_iso_utc();
    let report = build_report(manifest, &state, structure_sha256, &completed_at)?;
    let bytes = report_bytes(&report)?;
    atomic_write(&layout.report, &bytes).with_context(|| {
        format!(
            "persist documentation crawl report {}",
            layout.report.display()
        )
    })?;
    state.completed_at = Some(completed_at);
    state.report_sha256 = Some(lib::sha256_hex(&bytes));
    checkpoint_state(&layout.state, &state)?;
    Ok((state, report))
}

pub(crate) fn corpus_summary(layout: &WorkLayout, state: &DurableState) -> Result<Value> {
    let expected = CORPUS_ARTIFACTS;
    let mut observed = std::fs::read_dir(&layout.corpus)?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().to_string()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    observed.sort();
    let mut expected_sorted = expected.to_vec();
    expected_sorted.sort();
    if observed != expected_sorted {
        bail!(
            "documentation corpus contains files outside the exact run artifact set: {}",
            observed.join(", ")
        );
    }
    let mut bytes = 0u64;
    for name in expected {
        let file =
            open_regular_file(&layout.corpus.join(name), true, false, false, false, "corpus artifact")?;
        bytes = bytes
            .checked_add(file.metadata()?.len())
            .context("documentation corpus byte count overflow")?;
    }
    let pages = state
        .outcomes
        .values()
        .filter(|outcome| outcome.record_sha256.is_some())
        .count();
    Ok(json!({
        "files": expected.len(),
        "bytes": bytes,
        "pages": pages,
    }))
}

pub(crate) fn worker_report(
    manifest: &super::crawl::RuntimeManifest,
    state: &str,
    artifact: Option<Value>,
    corpus: Option<Value>,
    failure: Option<(&str, &str)>,
) -> Result<Value> {
    Ok(json!({
        "schema": "wisent.docs-worker-report.v1",
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "record_key": manifest.record_key,
        "attempt": u64::from(manifest.attempt),
        "attempt_id": manifest.attempt_id,
        "engine": "docs",
        "state": state,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "reference_sha256": manifest.reference_sha256,
        "bindings_file_sha256": manifest.bindings_file_sha256,
        "bindings_sha256": manifest.bindings_sha256,
        "docs_structure_sha256": manifest_structure_sha256(manifest)?,
        "execution_identity": serde_json::to_value(&manifest.execution_identity)?,
        "artifact": artifact.unwrap_or(Value::Null),
        "corpus": corpus.unwrap_or(Value::Null),
        "failure": failure
            .map(|(code, message)| json!({"code": code, "message": message}))
            .unwrap_or(Value::Null),
    }))
}

pub(crate) fn crawl_attempt(
    rest: &[String],
    manifest: &super::crawl::RuntimeManifest,
    layout: &WorkLayout,
) -> Result<(DurableState, Value)> {
    let invocation_started_at = lib::now_iso_utc();
    let options = WorkerOptions::parse(rest)?;
    let structure_dir =
        source_root().join("documentation-site-examples/content-structure");
    let mut slugs: Vec<String> = std::fs::read_dir(&structure_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|file| file.ends_with(".json"))
        .map(|file| file.trim_end_matches(".json").to_string())
        .filter(|file| file != "full-text-manifest")
        .collect();
    slugs.sort();
    let chosen: Vec<String> = match &options.site {
        Some(site) => vec![site.clone()],
        None => slugs
            .iter()
            .filter(|slug| !options.exclude.contains(slug))
            .cloned()
            .collect(),
    };
    for slug in &chosen {
        if !slugs.contains(slug) {
            bail!("unknown site: {slug}");
        }
    }
    if chosen.len() != 1 || chosen[0] != manifest.record {
        bail!("documentation worker selection differs from immutable runtime manifest record");
    }

    let (meta, structure_sha256) = validate_worker_source(manifest, &structure_dir)?;
    let rules = site_rules(&manifest.record, &meta, &overrides());
    let state = load_or_create_state(
        layout,
        manifest,
        &meta,
        &rules,
        options.refresh,
        invocation_started_at,
    )?;
    let pending = state
        .targets
        .iter()
        .filter(|target| !state.outcomes.contains_key(&target.key))
        .cloned()
        .collect::<Vec<_>>();
    eprintln!(
        "[{}] queue ready: {} pending URLs for {}; workers={} host-delay={}s",
        lib::now_iso_utc(),
        pending.len(),
        manifest.record,
        options.workers,
        options.host_delay
    );
    let state = if pending.is_empty() {
        state
    } else {
        run_fetch_workers(
            pending,
            options.workers,
            options.host_delay,
            layout,
            state,
        )?
    };
    finish_run(layout, manifest, state, &structure_sha256)
}

pub(crate) fn run_worker(rest: &[String], manifest: &super::crawl::RuntimeManifest) -> Result<()> {
    let layout = work_layout(manifest)?;
    std::fs::create_dir_all(&layout.root)?;
    let _lock = WorkLock::acquire(&layout)?;
    let outcome = crawl_attempt(rest, manifest, &layout)
        .and_then(|(state, _)| corpus_summary(&layout, &state));
    match outcome {
        Ok(corpus) => {
            let artifact =
                super::crawl::publish_attempt_archive(&layout.root, &manifest.artifact_uri)?;
            let report = worker_report(
                manifest,
                "artifact_published",
                Some(artifact),
                Some(corpus),
                None,
            )?;
            super::crawl::publish_worker_report(manifest, &report)?;
            println!("{}", serde_json::to_string(&report)?);
            Ok(())
        }
        Err(error) => {
            let message = format!("{error:#}");
            let failure = json!({
                "schema": "wisent.docs-worker-failure.v1",
                "code": "docs_crawl_failed",
                "message": message,
                "run_id": manifest.run_id,
                "catalog": manifest.catalog,
                "record": manifest.record,
                "attempt": u64::from(manifest.attempt),
                "attempt_id": manifest.attempt_id,
            });
            // Retain the failure diagnostic *beside* the attempt root, never inside
            // it. `corpus_summary` demands the root hold exactly CORPUS_ARTIFACTS,
            // and `atomic_json_write` additionally leaves a permanent
            // `.failure.json.lock` next to its target, so writing this into the root
            // made every later resume of the attempt fail forever. Import would not
            // catch it either: `extract_attempt_archive` runs no member-name allowlist,
            // so a `failure.json` inside the root would simply be installed with the
            // rest of the tree. The name matches the `<attempt_id>.tar.gz` convention
            // already used in this directory.
            match failure_diagnostic_path(&layout) {
                Ok(path) => {
                    if let Err(write_error) = super::crawl::atomic_json_write(&path, &failure) {
                        eprintln!(
                            "documentation worker failure artifact could not be retained: {write_error:#}"
                        );
                    }
                }
                Err(path_error) => eprintln!(
                    "documentation worker failure artifact has no retainable path: {path_error:#}"
                ),
            }
            let artifact =
                super::crawl::publish_attempt_archive(&layout.root, &manifest.artifact_uri);
            let report = worker_report(
                manifest,
                "failed",
                artifact.as_ref().ok().cloned(),
                None,
                Some(("docs_crawl_failed", &message)),
            )?;
            super::crawl::publish_worker_report(manifest, &report)?;
            println!("{}", serde_json::to_string(&report)?);
            match artifact {
                Ok(_) => bail!("documentation worker failed: {message}"),
                Err(publish_error) => bail!(
                    "documentation worker failed: {message}; the attempt archive could not be published either: {publish_error:#}"
                ),
            }
        }
    }
}

pub(crate) const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";
