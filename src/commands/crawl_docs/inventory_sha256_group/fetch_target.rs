use super::*;

pub(crate) fn fetch_target(
    target: CrawlTarget,
    gate: &HostGate,
    policy: &UrlPolicy,
    robots: &CompiledRobots,
    downloaded_bytes: &AtomicU64,
) -> Result<FetchedOutcome> {
    let target_url = policy.canonical(&target.url, None, "documentation page target")?;
    if !robots.allows(&target_url) {
        return Ok(FetchedOutcome {
            diagnostic: Some(CrawlDiagnostic {
                code: "robots_disallowed".into(),
                message: "robots.txt disallows this documentation URL".into(),
                url: target.url.clone(),
            }),
            resolved_url: target.url.clone(),
            target,
            status: Value::String("robots_disallowed".into()),
            text_bytes: None,
            downloaded_bytes: 0,
            line: None,
        });
    }
    gate.wait_turn(&target.url);
    match bounded_http_get(
        &target_url,
        policy,
        MAX_PAGE_BYTES,
        "documentation page",
        Some(ByteBudget {
            counter: downloaded_bytes,
            limit: MAX_TOTAL_DOWNLOAD_BYTES,
        }),
    ) {
        Ok(response) => {
            let response_bytes = response.downloaded_bytes;
            let resolved_url = response.final_url.as_str().to_string();
            if !(200..300).contains(&response.status) {
                return Ok(FetchedOutcome {
                    diagnostic: Some(CrawlDiagnostic {
                        code: "http_status".into(),
                        message: format!(
                            "documentation page returned HTTP {}",
                            response.status
                        ),
                        url: target.url.clone(),
                    }),
                    resolved_url: resolved_url.clone(),
                    target,
                    status: Value::from(response.status),
                    text_bytes: None,
                    downloaded_bytes: response_bytes,
                    line: None,
                });
            }
            let media_type = response
                .content_type
                .as_deref()
                .and_then(|value| value.split(';').next())
                .map(str::trim)
                .map(str::to_ascii_lowercase);
            let is_html = matches!(
                media_type.as_deref(),
                Some("text/html" | "application/xhtml+xml")
            );
            let is_plain_text = matches!(
                media_type.as_deref(),
                Some(
                    "text/plain"
                        | "text/markdown"
                        | "text/x-markdown"
                        | "application/markdown"
                )
            );
            if !is_html && !is_plain_text {
                return Ok(FetchedOutcome {
                    diagnostic: Some(CrawlDiagnostic {
                        code: "unsupported_content".into(),
                        message: format!(
                            "documentation page has unsupported Content-Type {}",
                            media_type.as_deref().unwrap_or("<missing>")
                        ),
                        url: target.url.clone(),
                    }),
                    resolved_url: resolved_url.clone(),
                    target,
                    status: Value::from(response.status),
                    text_bytes: None,
                    downloaded_bytes: response_bytes,
                    line: None,
                });
            }
            let body = match std::str::from_utf8(&response.body) {
                Ok(body) => body,
                Err(error) => {
                    return Ok(FetchedOutcome {
                        diagnostic: Some(CrawlDiagnostic {
                            code: "non_utf8".into(),
                            message: format!(
                                "documentation page is not valid UTF-8 at byte {}",
                                error.valid_up_to()
                            ),
                            url: target.url.clone(),
                        }),
                        resolved_url: resolved_url.clone(),
                        target,
                        status: Value::from(response.status),
                        text_bytes: None,
                        downloaded_bytes: response_bytes,
                        line: None,
                    });
                }
            };
            let (text, title) = if is_html {
                lib::extract_text(body)
            } else {
                (body.to_string(), String::new())
            };
            let brace_density = if text.is_empty() {
                0.0
            } else {
                text.matches('{').count() as f64 * 100.0 / text.len() as f64
            };
            let (quality, diagnostic, retained_text) = if text.trim().is_empty() {
                (
                    "no_text",
                    Some(CrawlDiagnostic {
                        code: "no_text".into(),
                        message: "documentation page yielded no meaningful text".into(),
                        url: target.url.clone(),
                    }),
                    None,
                )
            } else if brace_density > 1.0 {
                (
                    "css_js_noise",
                    Some(CrawlDiagnostic {
                        code: "quality_css_js_noise".into(),
                        message: format!(
                            "documentation page text has {:.3}% brace density",
                            brace_density
                        ),
                        url: target.url.clone(),
                    }),
                    None,
                )
            } else {
                ("ok", None, Some(text))
            };
            let text_bytes = retained_text
                .as_ref()
                .map(|value| value.len() as u64)
                .unwrap_or(0);
            let record = json!({
                "url": &target.url,
                "resolved_url": response.final_url.as_str(),
                "fetched_at": lib::now_iso_utc(),
                "status": response.status,
                "content_type": media_type,
                "quality": quality,
                "sha256": lib::sha256_hex(&response.body),
                "bytes": response.body.len(),
                "title": (!title.is_empty()).then_some(title),
                "text": retained_text,
                "lastmod": &target.lastmod,
            });
            let mut line = serde_json::to_vec(&record)?;
            line.push(b'\n');
            Ok(FetchedOutcome {
                resolved_url,
                target,
                status: Value::from(response.status),
                diagnostic,
                text_bytes: Some(text_bytes),
                downloaded_bytes: response_bytes,
                line: Some(line),
            })
        }
        Err(error) => {
            Ok(FetchedOutcome {
                diagnostic: Some(CrawlDiagnostic {
                    code: match error.code {
                        "body_byte_limit" => "page_body_limit".into(),
                        code => code.into(),
                    },
                    message: error.to_string(),
                    url: target.url.clone(),
                }),
                resolved_url: target.url.clone(),
                target,
                status: Value::String("error".into()),
                text_bytes: None,
                downloaded_bytes: error.downloaded_bytes,
                line: None,
            })
        }
    }
}

pub(crate) fn load_stream_hasher(path: &Path, committed_bytes: u64) -> Result<Sha256> {
    let mut hasher = Sha256::new();
    let mut file =
        open_regular_file(path, true, false, false, false, "durable documentation pages")?;
    let mut remaining = committed_bytes;
    let mut buffer = [0u8; 64 * 1024];
    while remaining > 0 {
        let limit = usize::try_from(remaining.min(buffer.len() as u64))?;
        let read = file.read(&mut buffer[..limit])?;
        if read == 0 {
            bail!(
                "durable documentation corpus ended before its {}-byte checkpoint",
                committed_bytes
            );
        }
        hasher.update(&buffer[..read]);
        remaining -= read as u64;
    }
    Ok(hasher)
}

pub(crate) const WRITER_BATCH_SIZE: usize = 32;

pub(crate) const WRITER_BATCH_WAIT: Duration = Duration::from_millis(10);

pub(crate) fn accept_writer_message(
    message: WriterMessage,
    expected_positions: &HashMap<usize, usize>,
    expected_index: usize,
    waiting: &mut BTreeMap<usize, WriteRequest>,
) -> Result<()> {
    let request = match message {
        WriterMessage::Outcome(request) => request,
        WriterMessage::Abort(message) => {
            bail!("documentation fetch worker aborted before durable commit: {message}")
        }
    };
    let sequence = request.outcome.target.sequence;
    if expected_positions
        .get(&sequence)
        .is_none_or(|position| *position < expected_index)
        || waiting.insert(sequence, request).is_some()
    {
        bail!("documentation writer received unexpected target sequence {sequence}");
    }
    Ok(())
}
