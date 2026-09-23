use super::*;

pub(crate) fn writer_loop(
    receiver: mpsc::Receiver<WriterMessage>,
    expected_sequences: Vec<usize>,
    layout: &WorkLayout,
    mut state: DurableState,
) -> Result<DurableState> {
    let mut output = open_regular_file(
        &layout.pages,
        true,
        true,
        false,
        false,
        "durable documentation pages",
    )?;
    let mut journal = open_regular_file(
        &layout.journal,
        false,
        true,
        true,
        false,
        "outcome journal",
    )?;
    output.seek(SeekFrom::Start(state.committed_bytes))?;
    let mut stream_hasher = load_stream_hasher(&layout.pages, state.committed_bytes)?;
    let expected_positions = expected_sequences
        .iter()
        .copied()
        .enumerate()
        .map(|(position, sequence)| (sequence, position))
        .collect::<HashMap<_, _>>();
    let mut waiting = BTreeMap::<usize, WriteRequest>::new();
    let mut expected_index = 0usize;

    while expected_index < expected_sequences.len() {
        let expected = expected_sequences[expected_index];
        while !waiting.contains_key(&expected) {
            let message = match receiver.recv_timeout(WRITER_LIVENESS_TIMEOUT) {
                Ok(message) => message,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    bail!(
                        "documentation writer made no progress for {} seconds while waiting for target sequence {expected}",
                        WRITER_LIVENESS_TIMEOUT.as_secs()
                    );
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    bail!(
                        "documentation writer channel closed before target sequence {expected}"
                    );
                }
            };
            accept_writer_message(
                message,
                &expected_positions,
                expected_index,
                &mut waiting,
            )?;
        }

        let deadline = Instant::now() + WRITER_BATCH_WAIT;
        while waiting.len() < WRITER_BATCH_SIZE {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            match receiver.recv_timeout(remaining) {
                Ok(message) => accept_writer_message(
                    message,
                    &expected_positions,
                    expected_index,
                    &mut waiting,
                )?,
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        let mut batch = Vec::with_capacity(WRITER_BATCH_SIZE);
        while batch.len() < WRITER_BATCH_SIZE
            && expected_index + batch.len() < expected_sequences.len()
        {
            let sequence = expected_sequences[expected_index + batch.len()];
            let Some(request) = waiting.remove(&sequence) else {
                break;
            };
            batch.push(request);
        }
        let commit = (|| -> Result<()> {
            let mut journal_outcomes = Vec::with_capacity(batch.len());
            for request in &batch {
                let start = state.committed_bytes;
                let mut status = request.outcome.status.clone();
                let mut diagnostic = request.outcome.diagnostic.clone();
                let mut text_bytes = request.outcome.text_bytes;
                let (record_sha256, corpus_start, corpus_end) =
                    if let Some(line) = &request.outcome.line {
                        let mut encoder = flate2::GzBuilder::new()
                            .mtime(0)
                            .write(Vec::new(), flate2::Compression::default());
                        encoder.write_all(line)?;
                        let member = encoder.finish()?;
                        if state.committed_bytes.saturating_add(member.len() as u64)
                            > MAX_CORPUS_BYTES
                        {
                            status = json!("corpus_limit");
                            diagnostic = Some(CrawlDiagnostic {
                                code: "corpus_total_byte_limit".into(),
                                message: format!(
                                    "writing this page would exceed the {MAX_CORPUS_BYTES}-byte corpus limit"
                                ),
                                url: request.outcome.target.url.clone(),
                            });
                            text_bytes = None;
                            (None, None, None)
                        } else {
                            output.write_all(&member).with_context(|| {
                                format!(
                                    "write documentation page {} to gzip stream",
                                    request.outcome.target.url
                                )
                            })?;
                            stream_hasher.update(&member);
                            state.committed_bytes += member.len() as u64;
                            state.committed_sha256 =
                                hex::encode(stream_hasher.clone().finalize());
                            (
                                Some(lib::sha256_hex(line)),
                                Some(start),
                                Some(state.committed_bytes),
                            )
                        }
                    } else {
                        (None, None, None)
                    };
                if request.outcome.target.url == Url::parse(&state.source_url)?.as_str() {
                    state.effective_source_url = request.outcome.resolved_url.clone();
                }
                let page_outcome = PageOutcome {
                    sequence: request.outcome.target.sequence,
                    url: request.outcome.target.url.clone(),
                    resolved_url: request.outcome.resolved_url.clone(),
                    status,
                    diagnostic,
                    text_bytes,
                    downloaded_bytes: request.outcome.downloaded_bytes,
                    record_sha256,
                    corpus_start,
                    corpus_end,
                };
                state.outcomes.insert(
                    request.outcome.target.key.clone(),
                    page_outcome.clone(),
                );
                journal_outcomes.push(JournalOutcome {
                    key: request.outcome.target.key.clone(),
                    outcome: page_outcome,
                });
            }
            output
                .flush()
                .context("flush contiguous documentation gzip batch")?;
            output
                .sync_all()
                .context("fsync contiguous documentation gzip batch")?;
            let journal_batch = OutcomeJournalBatch {
                schema: "wisent.docs-outcome-batch.v1".into(),
                first_sequence: journal_outcomes
                    .first()
                    .context("empty writer batch")?
                    .outcome
                    .sequence,
                last_sequence: journal_outcomes
                    .last()
                    .context("empty writer batch")?
                    .outcome
                    .sequence,
                committed_bytes: state.committed_bytes,
                committed_sha256: state.committed_sha256.clone(),
                outcomes: journal_outcomes,
            };
            let mut journal_line = serde_json::to_vec(&journal_batch)?;
            journal_line.push(b'\n');
            let journal_length = journal.metadata()?.len();
            if journal_length.saturating_add(journal_line.len() as u64) > MAX_JOURNAL_BYTES {
                bail!(
                    "documentation outcome journal would exceed the {MAX_JOURNAL_BYTES}-byte limit"
                );
            }
            journal.write_all(&journal_line)?;
            journal.flush().context("flush outcome journal batch")?;
            journal.sync_all().context("fsync outcome journal batch")?;
            Ok(())
        })();
        if let Err(error) = commit {
            let message = format!("{error:#}");
            let mut notification_error = None;
            for request in &batch {
                if let Err(send_error) = request.acknowledge.send(Err(message.clone())) {
                    notification_error.get_or_insert(send_error.to_string());
                }
            }
            if let Some(send_error) = notification_error {
                return Err(error).with_context(|| {
                    format!(
                        "durable writer also failed to report its batch error: {send_error}"
                    )
                });
            }
            return Err(error);
        }
        expected_index += batch.len();
        for request in batch {
            request
                .acknowledge
                .send(Ok(()))
                .with_context(|| {
                    format!(
                        "acknowledge durable documentation page {}",
                        request.outcome.target.url
                    )
                })?;
        }
    }
    Ok(state)
}
