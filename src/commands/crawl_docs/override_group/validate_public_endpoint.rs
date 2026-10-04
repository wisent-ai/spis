use super::*;

pub(crate) fn validate_public_endpoint(url: &Url) -> Result<Vec<SocketAddr>> {
    let host = url
        .host_str()
        .context("declared documentation source_url has no host")?
        .to_string();
    let port = url
        .port_or_known_default()
        .context("declared documentation source_url has no effective port")?;
    // The system resolver's own answer or error is the result; no deadline
    // (cli.md rule 8).
    let addresses = (host.as_str(), port)
        .to_socket_addrs()
        .map(|addresses| addresses.collect::<Vec<_>>())
        .with_context(|| format!("resolve declared documentation origin {host}:{port}"))?;
    if addresses.is_empty() {
        bail!("declared documentation origin resolved to no addresses");
    }
    if addresses.iter().any(|address| forbidden_ip(address.ip())) {
        bail!("declared documentation origin resolves to a non-public address");
    }
    Ok(addresses)
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

/// Read one response to its end, adding what arrived to `counter`. No body
/// size is chosen here: documentation pages, sitemaps and robots files are
/// read whole, and what a run keeps is bounded by its corpus volume's room.
pub(crate) fn read_response(
    response: ureq::Response,
    final_url: Url,
    label: &str,
    counter: Option<&AtomicU64>,
) -> std::result::Result<HttpResponse, HttpFailure> {
    let status = response.status() as u16;
    let content_type = response.header("Content-Type").map(str::to_string);
    let mut body = Vec::new();
    let read = response.into_reader().read_to_end(&mut body);
    if let Some(counter) = counter {
        counter.fetch_add(body.len() as u64, Ordering::SeqCst);
    }
    if let Err(error) = read {
        return Err(HttpFailure::new(
            "response_read_failed",
            format!("read {label}: {error}"),
            body.len() as u64,
        ));
    }
    Ok(HttpResponse {
        status,
        final_url,
        content_type,
        downloaded_bytes: body.len() as u64,
        body,
    })
}

/// GET `requested`, following each validated redirect. A redirect back to an
/// address already visited is a loop and is refused with that address; no
/// count of redirects is chosen here.
pub(crate) fn http_get(
    requested: &Url,
    policy: &UrlPolicy,
    label: &str,
    counter: Option<&AtomicU64>,
) -> std::result::Result<HttpResponse, HttpFailure> {
    let pinned_addresses = Arc::clone(&policy.pinned_addresses);
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .try_proxy_from_env(false)
        .resolver(move |_netloc: &str| Ok(pinned_addresses.as_ref().clone()))
        .build();
    let mut current = requested.clone();
    let mut visited = std::collections::HashSet::new();
    loop {
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
            if !visited.insert(current.as_str().to_string()) {
                return Err(HttpFailure::new(
                    "redirect_loop",
                    format!("{label} redirects back to {current}, which it already visited"),
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
        return read_response(response, current, label, counter);
    }
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
