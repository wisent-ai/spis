use super::*;

pub(crate) fn run_fetch_workers(
    pending: Vec<CrawlTarget>,
    workers: usize,
    host_delay: f64,
    layout: &WorkLayout,
    state: DurableState,
) -> Result<DurableState> {
    let expected_sequences = pending
        .iter()
        .map(|target| target.sequence)
        .collect::<Vec<_>>();
    let policy = UrlPolicy::new(&state.source_url)?;
    // Rebuild the matcher once per run from the persisted snapshot. A snapshot
    // that will not compile aborts the run rather than degrading to allow.
    let robots = CompiledRobots::compile(
        state
            .robots
            .as_ref()
            .context("durable documentation inventory has no robots policy")?,
    )?;
    let page_downloaded_bytes = state
        .outcomes
        .values()
        .try_fold(0u64, |total, outcome| {
            total.checked_add(outcome.downloaded_bytes)
        })
        .context("durable page download byte counter overflow")?;
    let downloaded_bytes = state
        .inventory_downloaded_bytes
        .checked_add(page_downloaded_bytes)
        .context("durable total download byte counter overflow")?;
    if downloaded_bytes > MAX_TOTAL_DOWNLOAD_BYTES {
        bail!(
            "durable download byte counter exceeds the {MAX_TOTAL_DOWNLOAD_BYTES}-byte limit"
        );
    }
    let (writer, receiver) = mpsc::channel::<WriterMessage>();
    let cancelled = Arc::new(AtomicBool::new(false));
    let shared = Arc::new(FetchShared {
        queue: Mutex::new(pending.into_iter()),
        writer: writer.clone(),
        gate: HostGate::new(host_delay),
        policy,
        downloaded_bytes: AtomicU64::new(downloaded_bytes),
        cancelled: Arc::clone(&cancelled),
        robots,
    });
    let writer_layout = layout.clone();
    let writer_cancelled = Arc::clone(&cancelled);
    let writer_handle = std::thread::spawn(move || {
        let result = writer_loop(receiver, expected_sequences, &writer_layout, state);
        if result.is_err() {
            writer_cancelled.store(true, Ordering::SeqCst);
        }
        result
    });
    let worker_result = std::thread::scope(|scope| -> Result<()> {
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let shared = Arc::clone(&shared);
            handles.push(scope.spawn(move || -> Result<()> {
                let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    loop {
                        if shared.cancelled.load(Ordering::SeqCst) {
                            bail!("documentation fetch cancelled after writer failure");
                        }
                        let target = {
                            let mut queue = shared.queue.lock();
                            match queue.next() {
                                Some(target) => target,
                                None => return Ok(()),
                            }
                        };
                        let url = target.url.clone();
                        let outcome = fetch_target(
                            target,
                            &shared.gate,
                            &shared.policy,
                            &shared.robots,
                            &shared.downloaded_bytes,
                        )
                        .with_context(|| format!("fetch documentation target {url}"))?;
                        if shared.cancelled.load(Ordering::SeqCst) {
                            bail!("documentation fetch cancelled after writer failure");
                        }
                        let (acknowledge, acknowledged) = mpsc::channel();
                        shared
                            .writer
                            .send(WriterMessage::Outcome(WriteRequest {
                                outcome,
                                acknowledge,
                            }))
                            .with_context(|| {
                                format!("send documentation target {url} to durable writer")
                            })?;
                        match acknowledged.recv_timeout(WRITER_LIVENESS_TIMEOUT) {
                            Ok(result) => result.map_err(anyhow::Error::msg)?,
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                shared.cancelled.store(true, Ordering::SeqCst);
                                bail!(
                                    "durable writer acknowledgement timed out after {} seconds for {url}",
                                    WRITER_LIVENESS_TIMEOUT.as_secs()
                                );
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => {
                                bail!("durable writer disconnected before acknowledging {url}");
                            }
                        }
                    }
                }));
                let error = match run {
                    Ok(Ok(())) => return Ok(()),
                    Ok(Err(error)) => error,
                    Err(_) => anyhow::anyhow!("documentation fetch worker thread panicked"),
                };
                let message = format!("{error:#}");
                shared
                    .writer
                    .send(WriterMessage::Abort(message.clone()))
                    .map_err(|send_error| {
                        anyhow::anyhow!(
                            "{message}; also failed to send worker abort to durable writer: {send_error}"
                        )
                    })?;
                Err(error)
            }));
        }
        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("documentation fetch worker thread panicked outside its failure boundary"))??;
        }
        Ok(())
    });
    drop(shared);
    drop(writer);
    let writer_result = writer_handle
        .join()
        .map_err(|_| anyhow::anyhow!("documentation corpus writer thread panicked"))?;
    let state = writer_result.context("documentation corpus writer failed")?;
    worker_result?;
    Ok(state)
}

pub(crate) fn source_root() -> PathBuf {
    super::corpus::data_root()
}

pub(crate) fn validate_worker_source(
    manifest: &super::crawl::RuntimeManifest,
    structure_dir: &Path,
) -> Result<(SiteMeta, String)> {
    if source_revision()? != manifest.source_revision {
        bail!("documentation worker source revision differs from immutable runtime manifest");
    }
    let reference_path = source_root().join(&manifest.catalog)
        .join("references")
        .join(&manifest.record)
        .join("reference.json");
    let reference_bytes = std::fs::read(&reference_path).with_context(|| {
        format!(
            "read committed documentation record {}",
            reference_path.display()
        )
    })?;
    if lib::sha256_hex(&reference_bytes) != manifest.reference_sha256 {
        bail!("documentation worker record digest differs from immutable runtime manifest");
    }
    let structure_path = structure_dir.join(format!("{}.json", manifest.record));
    let structure_bytes = std::fs::read(&structure_path).with_context(|| {
        format!(
            "read committed documentation source {}",
            structure_path.display()
        )
    })?;
    let structure_sha256 = lib::sha256_hex(&structure_bytes);
    if structure_sha256 != manifest_structure_sha256(manifest)? {
        bail!(
            "documentation content-structure digest differs from immutable runtime manifest"
        );
    }
    let meta: SiteMeta = serde_json::from_slice(&structure_bytes).with_context(|| {
        format!(
            "parse committed documentation source {}",
            structure_path.display()
        )
    })?;
    if meta.source_url != manifest.runtime_product.declared_identifier {
        bail!("documentation source URL differs from immutable runtime manifest");
    }
    Ok((meta, structure_sha256))
}
