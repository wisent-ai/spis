use super::*;

pub(crate) fn validate_public_endpoint(url: &Url) -> Result<Vec<SocketAddr>> {
    let host = url
        .host_str()
        .context("declared documentation source_url has no host")?
        .to_string();
    let port = url
        .port_or_known_default()
        .context("declared documentation source_url has no effective port")?;
    let (sender, receiver) = mpsc::sync_channel(1);
    let resolver_host = host.clone();
    std::thread::spawn(move || {
        let result = (resolver_host.as_str(), port)
            .to_socket_addrs()
            .map(|addresses| addresses.collect::<Vec<_>>())
            .map_err(|error| error.to_string());
        let _ = sender.send(result);
    });
    let addresses = match receiver.recv_timeout(DNS_LOOKUP_TIMEOUT) {
        Ok(Ok(addresses)) => addresses,
        Ok(Err(error)) => {
            bail!("resolve declared documentation origin {host}:{port}: {error}")
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            bail!(
                "resolve declared documentation origin {host}:{port}: exceeded {} seconds",
                DNS_LOOKUP_TIMEOUT.as_secs()
            )
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            bail!("resolve declared documentation origin {host}:{port}: resolver stopped")
        }
    };
    if addresses.is_empty() {
        bail!("declared documentation origin resolved to no addresses");
    }
    if addresses.iter().any(|address| forbidden_ip(address.ip())) {
        bail!("declared documentation origin resolves to a non-public address");
    }
    Ok(addresses)
}

#[derive(Clone, Copy)]
pub(crate) struct ByteBudget<'a> {
    pub(crate) counter: &'a AtomicU64,
    pub(crate) limit: u64,
}

pub(crate) struct HttpResponse {
    pub(crate) status: u16,
    pub(crate) final_url: Url,
    pub(crate) content_type: Option<String>,
    pub(crate) body: Vec<u8>,
    pub(crate) downloaded_bytes: u64,
}

#[derive(Debug)]
pub(crate) struct HttpFailure {
    pub(crate) code: &'static str,
    pub(crate) message: String,
    pub(crate) downloaded_bytes: u64,
}

impl HttpFailure {
    pub(crate) fn new(code: &'static str, message: impl Into<String>, downloaded_bytes: u64) -> Self {
        Self {
            code,
            message: message.into(),
            downloaded_bytes,
        }
    }
}

impl fmt::Display for HttpFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for HttpFailure {}

pub(crate) fn claim_budget(budget: Option<ByteBudget<'_>>, requested: usize) -> usize {
    let Some(budget) = budget else {
        return requested;
    };
    loop {
        let current = budget.counter.load(Ordering::SeqCst);
        let available = budget.limit.saturating_sub(current);
        let claimed = requested.min(available as usize);
        if claimed == 0 {
            return 0;
        }
        if budget
            .counter
            .compare_exchange(
                current,
                current + claimed as u64,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
        {
            return claimed;
        }
    }
}

pub(crate) fn refund_budget(budget: Option<ByteBudget<'_>>, bytes: usize) {
    if let Some(budget) = budget {
        budget.counter.fetch_sub(bytes as u64, Ordering::SeqCst);
    }
}

pub(crate) fn read_bounded_response(
    response: ureq::Response,
    final_url: Url,
    max_bytes: usize,
    label: &str,
    budget: Option<ByteBudget<'_>>,
) -> std::result::Result<HttpResponse, HttpFailure> {
    if response
        .header("Content-Length")
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > max_bytes)
    {
        return Err(HttpFailure::new(
            "body_byte_limit",
            format!(
                "{label} exceeds the {max_bytes}-byte limit declared by Content-Length"
            ),
            0,
        ));
    }
    let status = response.status() as u16;
    let content_type = response.header("Content-Type").map(str::to_string);
    let mut body = Vec::with_capacity(max_bytes.min(64 * 1024));
    let mut reader = response.into_reader();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let local_remaining = max_bytes.saturating_add(1).saturating_sub(body.len());
        if local_remaining == 0 {
            return Err(HttpFailure::new(
                "body_byte_limit",
                format!("{label} exceeds the {max_bytes}-byte limit"),
                body.len() as u64,
            ));
        }
        let wanted = local_remaining.min(buffer.len());
        let claimed = claim_budget(budget, wanted);
        if claimed == 0 {
            return Err(HttpFailure::new(
                "total_download_byte_limit",
                format!("{label} reached the aggregate download byte limit"),
                body.len() as u64,
            ));
        }
        let read = match reader.read(&mut buffer[..claimed]) {
            Ok(read) => read,
            Err(error) => {
                refund_budget(budget, claimed);
                return Err(HttpFailure::new(
                    "response_read_failed",
                    format!("read bounded {label}: {error}"),
                    body.len() as u64,
                ));
            }
        };
        if read < claimed {
            refund_budget(budget, claimed - read);
        }
        if read == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..read]);
        if body.len() > max_bytes {
            return Err(HttpFailure::new(
                "body_byte_limit",
                format!("{label} exceeds the {max_bytes}-byte limit"),
                body.len() as u64,
            ));
        }
    }
    Ok(HttpResponse {
        status,
        final_url,
        content_type,
        downloaded_bytes: body.len() as u64,
        body,
    })
}

pub(crate) fn bounded_http_get(
    requested: &Url,
    policy: &UrlPolicy,
    max_bytes: usize,
    label: &str,
    budget: Option<ByteBudget<'_>>,
) -> std::result::Result<HttpResponse, HttpFailure> {
    let pinned_addresses = Arc::clone(&policy.pinned_addresses);
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .try_proxy_from_env(false)
        .resolver(move |_netloc: &str| Ok(pinned_addresses.as_ref().clone()))
        .timeout(Duration::from_secs(45))
        .build();
    let mut current = requested.clone();
    for redirect in 0..=MAX_REDIRECTS {
        let response = match agent
            .get(current.as_str())
            .set("User-Agent", lib::USER_AGENT)
            .call()
        {
            Ok(response) => response,
            Err(ureq::Error::Status(_, response)) => response,
            Err(error) => {
                return Err(HttpFailure::new(
                    "request_failed",
                    format!("{label} request failed: {error}"),
                    0,
                ));
            }
        };
        let observed = policy
            .canonical(response.get_url(), Some(&current), label)
            .map_err(|error| HttpFailure::new("url_rejected", format!("{error:#}"), 0))?;
        if observed != current {
            return Err(HttpFailure::new(
                "implicit_redirect",
                format!("{label} changed URL without an explicit validated redirect"),
                0,
            ));
        }
        if matches!(response.status(), 301 | 302 | 303 | 307 | 308) {
            if redirect == MAX_REDIRECTS {
                return Err(HttpFailure::new(
                    "redirect_limit",
                    format!("{label} exceeded the {MAX_REDIRECTS}-redirect limit"),
                    0,
                ));
            }
            let Some(location) = response.header("Location") else {
                return Err(HttpFailure::new(
                    "redirect_without_location",
                    "documentation redirect has no Location header",
                    0,
                ));
            };
            current = policy
                .canonical(location, Some(&current), "documentation redirect target")
                .map_err(|error| HttpFailure::new("redirect_rejected", format!("{error:#}"), 0))?;
            continue;
        }
        return read_bounded_response(response, current, max_bytes, label, budget);
    }
    unreachable!("redirect loop always returns or fails")
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct RobotsRule {
    pub(crate) pattern: String,
    pub(crate) specificity: usize,
    pub(crate) allow: bool,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub(crate) struct RobotsSnapshot {
    pub(crate) directives: Vec<RobotsRule>,
}
