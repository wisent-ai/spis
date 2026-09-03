//! `spis crawl-docs` — parallel full-text crawler for the documentation set.
//!
//! Every immutable runtime manifest owns one durable corpus beneath
//! `~/.stado/work`. Fetches may run concurrently, but a single ordered writer
//! commits complete gzip members before atomically checkpointing their full
//! URL hashes. A resumed worker therefore truncates only an uncommitted tail
//! and never borrows pages from another crawl run.

use crate as lib;
use anyhow::{bail, Context, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::os::unix::fs::OpenOptionsExt;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use url::Url;

struct Override {
    sitemaps: &'static [&'static str],
    prefixes: &'static [&'static str],
    llms: &'static [&'static str],
}

fn overrides() -> HashMap<&'static str, Override> {
    HashMap::from([
        (
            "01-mdn-web-docs",
            Override {
                sitemaps: &["https://developer.mozilla.org/sitemap.xml"],
                prefixes: &["/en-US/docs/Web"],
                llms: &[],
            },
        ),
        (
            "12-net-documentation",
            Override {
                sitemaps: &[],
                prefixes: &["/dotnet"],
                llms: &[],
            },
        ),
        (
            "21-google-cloud-documentation",
            Override {
                sitemaps: &[],
                prefixes: &["/docs"],
                llms: &[],
            },
        ),
        (
            "22-microsoft-azure-documentation",
            Override {
                sitemaps: &[],
                prefixes: &["/azure"],
                llms: &[],
            },
        ),
        (
            "38-postgresql-documentation",
            Override {
                sitemaps: &[],
                prefixes: &["/docs/current", "/docs/17"],
                llms: &[],
            },
        ),
        (
            "48-atlassian-design-system",
            Override {
                sitemaps: &[],
                prefixes: &[],
                llms: &["https://atlassian.design/llms.txt"],
            },
        ),
    ])
}

#[derive(serde::Deserialize, Clone)]
struct SiteMeta {
    name: String,
    source_url: String,
    #[serde(default)]
    inventory_source: String,
    #[serde(default)]
    landing_nav: Vec<LandingNavItem>,
}

#[derive(serde::Deserialize, Clone)]
struct LandingNavItem {
    path: String,
}

struct SiteRules {
    sitemaps: Vec<String>,
    llms: Vec<String>,
    prefixes: Vec<String>,
}

fn site_rules(slug: &str, meta: &SiteMeta, map: &HashMap<&'static str, Override>) -> SiteRules {
    let ov = map.get(slug);
    let mut prefixes: Vec<String> = ov
        .map(|o| o.prefixes.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();
    if prefixes.is_empty() {
        if let Some(inv) = meta.inventory_source.strip_prefix("scoped sitemap (") {
            if let Some(inner) = inv.strip_suffix(')') {
                prefixes = inner.split(',').map(|p| p.trim().to_string()).collect();
            }
        }
    }
    SiteRules {
        sitemaps: ov
            .map(|o| o.sitemaps.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default(),
        llms: ov
            .map(|o| o.llms.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default(),
        prefixes,
    }
}

const MAX_INVENTORY_BYTES: usize = 8 * 1024 * 1024;
const MAX_ROBOTS_BYTES: usize = 512 * 1024;
const MAX_PAGE_BYTES: usize = 16 * 1024 * 1024;
const MAX_TARGETS: usize = 50_000;
const MAX_INVENTORY_SOURCES: usize = 256;
const MAX_INVENTORY_DIAGNOSTICS: usize = 512;
const MAX_ROBOTS_RULES: usize = 4_096;
const MAX_REDIRECTS: usize = 5;

#[derive(Clone)]
struct UrlPolicy {
    declared_source_url: String,
    source_url: Url,
    origin: String,
    pinned_addresses: Arc<Vec<SocketAddr>>,
}
const MAX_WORKERS: usize = 8;
const MAX_HOST_DELAY_SECONDS: f64 = 30.0;
const MAX_TOTAL_INVENTORY_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOTAL_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_CORPUS_BYTES: u64 = 1024 * 1024 * 1024;
const WRITER_LIVENESS_TIMEOUT: Duration = Duration::from_secs(180);
pub(crate) const STADO_OUTPUT_LIMIT: usize = 1024 * 1024;
const STADO_COMMAND_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_secs(15);
/// Longest `Allow:`/`Disallow:` value accepted from a served robots.txt. Real
/// robots.txt paths are far shorter; the cap keeps a hostile origin from handing
/// us a rule whose compiled program is unbounded.
const MAX_ROBOTS_PATTERN_BYTES: usize = 1024;
/// Explicit compiled-program ceiling for one robots rule, so the bound is ours
/// rather than whatever `regex` happens to default to.
const MAX_ROBOTS_PROGRAM_BYTES: usize = 1024 * 1024;
static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

const MAX_JOURNAL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_STATE_BYTES: u64 = 256 * 1024 * 1024;
impl UrlPolicy {
    fn new(source_url: &str) -> Result<Self> {
        let parsed = Url::parse(source_url).context("declared documentation source_url is invalid")?;
        validate_url_shape(&parsed, "declared documentation source_url")?;
        let pinned_addresses = validate_public_endpoint(&parsed)?;
        Ok(Self {
            declared_source_url: source_url.to_string(),
            origin: parsed.origin().ascii_serialization(),
            source_url: parsed,
            pinned_addresses: Arc::new(pinned_addresses),
        })
    }

    fn canonical(&self, raw: &str, base: Option<&Url>, label: &str) -> Result<Url> {
        let parsed = match Url::parse(raw) {
            Ok(url) => url,
            Err(url::ParseError::RelativeUrlWithoutBase) => base
                .context("relative documentation URL has no base")?
                .join(raw)
                .with_context(|| format!("{label} is not a valid relative URL"))?,
            Err(error) => return Err(error).with_context(|| format!("{label} is invalid")),
        };
        validate_url_shape(&parsed, label)?;
        if parsed.origin().ascii_serialization() != self.origin {
            bail!("{label} is outside the exact declared documentation origin");
        }
        Ok(parsed)
    }
}

fn validate_url_shape(url: &Url, label: &str) -> Result<()> {
    if !matches!(url.scheme(), "http" | "https") {
        bail!("{label} must use http or https");
    }
    if !url.username().is_empty() || url.password().is_some() {
        bail!("{label} must not contain credentials");
    }
    if url.fragment().is_some() {
        bail!("{label} must not contain a fragment");
    }
    let host = url.host().context("documentation URL has no host")?;
    match host {
        url::Host::Domain(domain)
            if domain.eq_ignore_ascii_case("localhost")
                || domain.to_ascii_lowercase().ends_with(".localhost") =>
        {
            bail!("{label} resolves to a loopback hostname")
        }
        url::Host::Ipv4(address) if forbidden_ip(IpAddr::V4(address)) => {
            bail!("{label} uses a non-public IPv4 target")
        }
        url::Host::Ipv6(address) if forbidden_ip(IpAddr::V6(address)) => {
            bail!("{label} uses a non-public IPv6 target")
        }
        _ => {}
    }
    Ok(())
}

fn forbidden_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            let [a, b, c, _] = address.octets();
            a == 0
                || a == 10
                || a == 127
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192 && b == 0 && c == 0)
                || (a == 192 && b == 0 && c == 2)
                || (a == 192 && b == 88 && c == 99)
                || (a == 192 && b == 168)
                || (a == 198 && (b == 18 || b == 19))
                || (a == 198 && b == 51 && c == 100)
                || (a == 203 && b == 0 && c == 113)
                || a >= 224
        }
        IpAddr::V6(address) => {
            if let Some(mapped) = address.to_ipv4_mapped() {
                return forbidden_ip(IpAddr::V4(mapped));
            }
            let segments = address.segments();
            let global_unicast = (segments[0] & 0xe000) == 0x2000;
            let ietf_special_2001 = segments[0] == 0x2001 && segments[1] <= 0x01ff;
            let documentation = (segments[0] == 0x2001 && segments[1] == 0x0db8)
                || (segments[0] == 0x3fff && (segments[1] & 0xf000) == 0);
            let transition = segments[0] == 0x2002;
            !global_unicast || ietf_special_2001 || documentation || transition
        }
    }
}

fn validate_public_endpoint(url: &Url) -> Result<Vec<SocketAddr>> {
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
struct ByteBudget<'a> {
    counter: &'a AtomicU64,
    limit: u64,
}

struct HttpResponse {
    status: u16,
    final_url: Url,
    content_type: Option<String>,
    body: Vec<u8>,
    downloaded_bytes: u64,
}

#[derive(Debug)]
struct HttpFailure {
    code: &'static str,
    message: String,
    downloaded_bytes: u64,
}

impl HttpFailure {
    fn new(code: &'static str, message: impl Into<String>, downloaded_bytes: u64) -> Self {
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

fn claim_budget(budget: Option<ByteBudget<'_>>, requested: usize) -> usize {
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

fn refund_budget(budget: Option<ByteBudget<'_>>, bytes: usize) {
    if let Some(budget) = budget {
        budget.counter.fetch_sub(bytes as u64, Ordering::SeqCst);
    }
}

fn read_bounded_response(
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

fn bounded_http_get(
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
struct RobotsRule {
    pattern: String,
    specificity: usize,
    allow: bool,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct RobotsSnapshot {
    directives: Vec<RobotsRule>,
}

impl RobotsSnapshot {
    /// The deny-all snapshot every robots failure path falls back to. Kept as one
    /// constructor so the serialized shape cannot drift between failure paths; it
    /// is persisted in `DurableState.robots` and feeds `inventory_sha256`.
    fn deny_all() -> Self {
        Self {
            directives: vec![RobotsRule {
                pattern: "^/".into(),
                specificity: 1,
                allow: false,
            }],
        }
    }
}

/// `RobotsSnapshot` is the persisted wire form; this is the matcher. Patterns are
/// compiled exactly once per run instead of once per rule per URL, which used to
/// cost up to `MAX_ROBOTS_RULES * MAX_TARGETS` compilations of identical
/// programs. Compilation is fallible here and never ignored, so a rule can no
/// longer be silently dropped — dropping a `Disallow` would have turned it into
/// an allow, the one fail-open path in this file.
struct CompiledRobots {
    rules: Vec<CompiledRobotsRule>,
}

struct CompiledRobotsRule {
    pattern: regex::Regex,
    specificity: usize,
    allow: bool,
}

impl CompiledRobots {
    fn compile(snapshot: &RobotsSnapshot) -> Result<Self> {
        let rules = snapshot
            .directives
            .iter()
            .map(|rule| {
                Ok(CompiledRobotsRule {
                    pattern: compile_robots_pattern(&rule.pattern)?,
                    specificity: rule.specificity,
                    allow: rule.allow,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { rules })
    }

    fn allows(&self, url: &Url) -> bool {
        let mut path = url.path().to_string();
        if let Some(query) = url.query() {
            path.push('?');
            path.push_str(query);
        }
        self.rules
            .iter()
            .filter(|rule| rule.pattern.is_match(&path))
            .max_by(|left, right| {
                left.specificity
                    .cmp(&right.specificity)
                    .then(left.allow.cmp(&right.allow))
            })
            .is_none_or(|rule| rule.allow)
    }
}

/// Compile one robots pattern under an explicit program-size ceiling, so the
/// bound is ours rather than whatever `regex` defaults to.
fn compile_robots_pattern(pattern: &str) -> Result<regex::Regex> {
    regex::RegexBuilder::new(pattern)
        .size_limit(MAX_ROBOTS_PROGRAM_BYTES)
        .build()
        .with_context(|| format!("robots pattern {pattern:?} does not compile"))
}

struct InventoryResolution {
    pages: Vec<(String, Option<String>)>,
    diagnostics: Vec<CrawlDiagnostic>,
    robots: RobotsSnapshot,
    downloaded_bytes: u64,
}

fn push_inventory_diagnostic(
    diagnostics: &mut Vec<CrawlDiagnostic>,
    code: &str,
    message: impl Into<String>,
    url: impl Into<String>,
) {
    if diagnostics.len() < MAX_INVENTORY_DIAGNOSTICS {
        diagnostics.push(CrawlDiagnostic {
            code: code.into(),
            message: message.into(),
            url: url.into(),
        });
    } else if diagnostics.len() == MAX_INVENTORY_DIAGNOSTICS {
        diagnostics.push(CrawlDiagnostic {
            code: "inventory_diagnostics_truncated".into(),
            message: format!(
                "inventory diagnostics exceeded the {MAX_INVENTORY_DIAGNOSTICS}-entry limit"
            ),
            url: String::new(),
        });
    }
}

/// Build the persisted snapshot *and* its compiled matcher together, so a
/// snapshot can never reach the crawl loop without every pattern having compiled
/// successfully. Every recoverable robots problem yields the deny-all policy plus
/// a durable diagnostic; only a failure to compile our own deny-all constant is
/// an error, and that is still fail-closed because it aborts the run.
fn parse_robots(
    body: &[u8],
    diagnostics: &mut Vec<CrawlDiagnostic>,
    robots_url: &Url,
) -> Result<(RobotsSnapshot, CompiledRobots, Vec<String>)> {
    let deny_all = |diagnostics: &mut Vec<CrawlDiagnostic>,
                    code: &str,
                    message: String|
     -> Result<(RobotsSnapshot, CompiledRobots, Vec<String>)> {
        push_inventory_diagnostic(diagnostics, code, message, robots_url.as_str());
        let snapshot = RobotsSnapshot::deny_all();
        let compiled = CompiledRobots::compile(&snapshot)?;
        Ok((snapshot, compiled, Vec::new()))
    };
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return deny_all(
                diagnostics,
                "robots_non_utf8",
                format!("robots.txt is not valid UTF-8 at byte {}", error.valid_up_to()),
            );
        }
    };
    let mut groups: Vec<(Vec<String>, Vec<RobotsRule>)> = Vec::new();
    let mut agents = Vec::new();
    let mut directives = Vec::new();
    let mut sitemaps = Vec::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let Some((field, value)) = line.split_once(':') else {
            continue;
        };
        let field = field.trim();
        let value = value.trim();
        if field.eq_ignore_ascii_case("sitemap") {
            if sitemaps.len() < MAX_INVENTORY_SOURCES {
                sitemaps.push(value.to_string());
            }
            continue;
        }
        if field.eq_ignore_ascii_case("user-agent") {
            if !directives.is_empty() {
                groups.push((
                    std::mem::take(&mut agents),
                    std::mem::take(&mut directives),
                ));
            }
            agents.push(value.to_ascii_lowercase());
        } else if (field.eq_ignore_ascii_case("allow")
            || field.eq_ignore_ascii_case("disallow"))
            && !agents.is_empty()
            && !value.is_empty()
        {
            if directives.len() >= MAX_ROBOTS_RULES {
                push_inventory_diagnostic(
                    diagnostics,
                    "robots_rules_truncated",
                    format!("robots directives exceeded the {MAX_ROBOTS_RULES}-rule limit"),
                    robots_url.as_str(),
                );
                continue;
            }
            let terminal = value.ends_with('$');
            let source = value.strip_suffix('$').unwrap_or(value);
            // A robots.txt line is otherwise bounded only by MAX_ROBOTS_BYTES, so
            // an origin could serve `Disallow: /` plus ~250 000 `*` characters and
            // push the compiled program past any sane ceiling. Refuse the whole
            // file fail-closed rather than dropping the offending rule, because a
            // dropped `Disallow` reads as an allow.
            if source.len() > MAX_ROBOTS_PATTERN_BYTES {
                return deny_all(
                    diagnostics,
                    "robots_rule_too_long",
                    format!(
                        "robots directive exceeds the {MAX_ROBOTS_PATTERN_BYTES}-byte pattern limit"
                    ),
                );
            }
            directives.push(RobotsRule {
                pattern: format!(
                    "^{}{}",
                    regex::escape(source).replace(r"\*", ".*"),
                    if terminal { "$" } else { "" }
                ),
                specificity: source.chars().filter(|character| *character != '*').count(),
                allow: field.eq_ignore_ascii_case("allow"),
            });
        }
    }
    if !agents.is_empty() {
        groups.push((agents, directives));
    }
    let user_agent = lib::USER_AGENT.to_ascii_lowercase();
    let specificity = groups
        .iter()
        .flat_map(|(agents, _)| agents)
        .filter_map(|agent| {
            if agent == "*" {
                Some(0)
            } else if user_agent.starts_with(agent) {
                Some(agent.len())
            } else {
                None
            }
        })
        .max();
    let directives = specificity
        .map(|wanted| {
            groups
                .into_iter()
                .filter(|(agents, _)| {
                    agents.iter().any(|agent| {
                        (agent == "*" && wanted == 0)
                            || (agent != "*"
                                && agent.len() == wanted
                                && user_agent.starts_with(agent))
                    })
                })
                .flat_map(|(_, rules)| rules)
                .take(MAX_ROBOTS_RULES)
                .collect()
        })
        .unwrap_or_default();
    let snapshot = RobotsSnapshot { directives };
    match CompiledRobots::compile(&snapshot) {
        Ok(compiled) => Ok((snapshot, compiled, sitemaps)),
        // A served rule that will not compile must deny, never allow.
        Err(error) => deny_all(
            diagnostics,
            "robots_rule_uncompilable",
            format!("robots directive could not be compiled: {error:#}"),
        ),
    }
}

fn path_is_in_scope(url: &Url, prefixes: &[String]) -> Result<bool> {
    if prefixes.is_empty() {
        return Ok(true);
    }
    for prefix in prefixes {
        if !prefix.starts_with('/') || prefix.contains(['?', '#']) {
            bail!("documentation path prefix {prefix:?} is not an absolute path prefix");
        }
        let normalized = prefix.trim_end_matches('/');
        if normalized.is_empty()
            || url.path() == normalized
            || url
                .path()
                .strip_prefix(normalized)
                .is_some_and(|suffix| suffix.starts_with('/'))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn resolve_urls(meta: &SiteMeta, rules: &SiteRules, policy: &UrlPolicy) -> Result<InventoryResolution> {
    let mut diagnostics = Vec::new();
    let total_inventory_bytes = AtomicU64::new(0);
    let inventory_budget = Some(ByteBudget {
        counter: &total_inventory_bytes,
        limit: MAX_TOTAL_INVENTORY_BYTES,
    });
    let robots_url = policy.source_url.join("/robots.txt")?;
    let (robots, compiled_robots, discovered_sitemaps) = match bounded_http_get(
        &robots_url,
        policy,
        MAX_ROBOTS_BYTES,
        "robots.txt",
        inventory_budget,
    ) {
        Ok(response) if (200..300).contains(&response.status) => {
            parse_robots(&response.body, &mut diagnostics, &robots_url)?
        }
        Ok(response) if matches!(response.status, 401 | 403) => {
            push_inventory_diagnostic(
                &mut diagnostics,
                "robots_access_denied",
                format!("robots.txt returned HTTP {}", response.status),
                robots_url.as_str(),
            );
            let snapshot = RobotsSnapshot::deny_all();
            let compiled = CompiledRobots::compile(&snapshot)?;
            (snapshot, compiled, Vec::new())
        }
        // An origin that answers "there is no policy here" is genuinely
        // unconstrained, and only these two statuses say that.
        Ok(response) if matches!(response.status, 404 | 410) => {
            let snapshot = RobotsSnapshot::default();
            let compiled = CompiledRobots::compile(&snapshot)?;
            (snapshot, compiled, Vec::new())
        }
        // `bounded_http_get` reports a served error status as `Ok`, so 429 and every 5xx
        // arrive here rather than in the transport-error branch below. An origin that
        // rate-limits or fails is an origin whose robots.txt was never observed: the
        // rules it does serve may forbid this sweep, so an unobserved policy denies
        // everything and leaves the same durable diagnostic as a transport failure.
        Ok(response) => {
            push_inventory_diagnostic(
                &mut diagnostics,
                "robots_unavailable",
                format!("robots.txt returned HTTP {}", response.status),
                robots_url.as_str(),
            );
            let snapshot = RobotsSnapshot::deny_all();
            let compiled = CompiledRobots::compile(&snapshot)?;
            (snapshot, compiled, Vec::new())
        }
        Err(error) => {
            push_inventory_diagnostic(
                &mut diagnostics,
                error.code,
                error.to_string(),
                robots_url.as_str(),
            );
            let snapshot = RobotsSnapshot::deny_all();
            let compiled = CompiledRobots::compile(&snapshot)?;
            (snapshot, compiled, Vec::new())
        }
    };

    let mut pages = vec![(policy.source_url.as_str().to_string(), None)];
    let mut queue = VecDeque::<Url>::new();
    let seeds = if rules.sitemaps.is_empty() && rules.llms.is_empty() {
        let mut seeds = discovered_sitemaps;
        seeds.push(policy.source_url.join("/sitemap.xml")?.to_string());
        seeds
    } else {
        rules.sitemaps.clone()
    };
    for raw in seeds.into_iter().chain(rules.llms.iter().cloned()) {
        match policy.canonical(&raw, Some(&policy.source_url), "inventory source") {
            Ok(url) if queue.len() < MAX_INVENTORY_SOURCES => queue.push_back(url),
            Ok(url) => push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_limit",
                format!("inventory sources exceeded the {MAX_INVENTORY_SOURCES}-source limit"),
                url.as_str(),
            ),
            Err(error) => push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_rejected",
                format!("{error:#}"),
                raw,
            ),
        }
    }

    let mut visited = HashSet::new();
    let mut target_limit_reported = false;
    while let Some(source) = queue.pop_front() {
        if !visited.insert(source.as_str().to_string()) {
            continue;
        }
        if visited.len() > MAX_INVENTORY_SOURCES {
            push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_limit",
                format!("inventory sources exceeded the {MAX_INVENTORY_SOURCES}-source limit"),
                source.as_str(),
            );
            break;
        }
        if !compiled_robots.allows(&source) {
            push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_robots_disallowed",
                "robots.txt disallows this inventory source",
                source.as_str(),
            );
            continue;
        }
        let response = match bounded_http_get(
            &source,
            policy,
            MAX_INVENTORY_BYTES,
            "documentation inventory source",
            inventory_budget,
        ) {
            Ok(response) if (200..300).contains(&response.status) => response,
            Ok(response) => {
                push_inventory_diagnostic(
                    &mut diagnostics,
                    "inventory_source_http_status",
                    format!("inventory source returned HTTP {}", response.status),
                    source.as_str(),
                );
                continue;
            }
            Err(error) => {
                push_inventory_diagnostic(
                    &mut diagnostics,
                    error.code,
                    error.to_string(),
                    source.as_str(),
                );
                if error.code == "total_download_byte_limit" {
                    break;
                }
                continue;
            }
        };
        let payload = match std::str::from_utf8(&response.body) {
            Ok(payload) => payload,
            Err(error) => {
                push_inventory_diagnostic(
                    &mut diagnostics,
                    "inventory_source_non_utf8",
                    format!(
                        "inventory source is not valid UTF-8 at byte {}",
                        error.valid_up_to()
                    ),
                    source.as_str(),
                );
                continue;
            }
        };
        if payload.contains("<urlset") || payload.contains("<sitemapindex") {
            let (children, discovered_pages) = lib::parse_sitemap(&response.body);
            for child in children {
                match policy.canonical(&child, Some(&response.final_url), "child sitemap") {
                    Ok(url) if queue.len() + visited.len() < MAX_INVENTORY_SOURCES => {
                        queue.push_back(url)
                    }
                    Ok(url) => push_inventory_diagnostic(
                        &mut diagnostics,
                        "inventory_source_limit",
                        format!(
                            "inventory sources exceeded the {MAX_INVENTORY_SOURCES}-source limit"
                        ),
                        url.as_str(),
                    ),
                    Err(error) => push_inventory_diagnostic(
                        &mut diagnostics,
                        "child_sitemap_rejected",
                        format!("{error:#}"),
                        child,
                    ),
                }
            }
            for (raw, lastmod) in discovered_pages {
                if pages.len() >= MAX_TARGETS {
                    if !target_limit_reported {
                        push_inventory_diagnostic(
                            &mut diagnostics,
                            "target_limit",
                            format!(
                                "documentation targets exceeded the {MAX_TARGETS}-target limit"
                            ),
                            source.as_str(),
                        );
                        target_limit_reported = true;
                    }
                    break;
                }
                match policy.canonical(&raw, Some(&response.final_url), "sitemap page") {
                    Ok(url) if path_is_in_scope(&url, &rules.prefixes)? => {
                        pages.push((url.to_string(), lastmod))
                    }
                    Ok(_) => {}
                    Err(error) => push_inventory_diagnostic(
                        &mut diagnostics,
                        "sitemap_page_rejected",
                        format!("{error:#}"),
                        raw,
                    ),
                }
            }
        } else if source.path().ends_with(".txt") && source.path().contains("llms") {
            for line in payload.lines() {
                let trimmed = line.trim();
                let Some(rest) = trimmed.strip_prefix("- [") else {
                    continue;
                };
                let Some(close) = rest.find("](") else {
                    continue;
                };
                let after = &rest[close + 2..];
                let Some(end) = after.find(')') else {
                    continue;
                };
                let raw = &after[..end];
                if pages.len() >= MAX_TARGETS {
                    if !target_limit_reported {
                        push_inventory_diagnostic(
                            &mut diagnostics,
                            "target_limit",
                            format!(
                                "documentation targets exceeded the {MAX_TARGETS}-target limit"
                            ),
                            source.as_str(),
                        );
                        target_limit_reported = true;
                    }
                    break;
                }
                match policy.canonical(raw, Some(&response.final_url), "llms.txt page") {
                    Ok(url) if path_is_in_scope(&url, &rules.prefixes)? => {
                        pages.push((url.to_string(), None))
                    }
                    Ok(_) => {}
                    Err(error) => push_inventory_diagnostic(
                        &mut diagnostics,
                        "llms_page_rejected",
                        format!("{error:#}"),
                        raw,
                    ),
                }
            }
        } else {
            push_inventory_diagnostic(
                &mut diagnostics,
                "inventory_source_unrecognized",
                "inventory source was neither sitemap XML nor llms.txt",
                source.as_str(),
            );
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    if pages.len() == 1 && meta.inventory_source.starts_with("landing-nav") {
        for item in &meta.landing_nav {
            if pages.len() >= MAX_TARGETS {
                push_inventory_diagnostic(
                    &mut diagnostics,
                    "target_limit",
                    format!("documentation targets exceeded the {MAX_TARGETS}-target limit"),
                    policy.source_url.as_str(),
                );
                break;
            }
            match policy.canonical(&item.path, Some(&policy.source_url), "landing navigation page") {
                Ok(url) if path_is_in_scope(&url, &rules.prefixes)? => {
                    pages.push((url.to_string(), None))
                }
                Ok(_) => {}
                Err(error) => push_inventory_diagnostic(
                    &mut diagnostics,
                    "landing_page_rejected",
                    format!("{error:#}"),
                    item.path.clone(),
                ),
            }
        }
    }
    if pages.len() == 1 {
        push_inventory_diagnostic(
            &mut diagnostics,
            "inventory_degraded",
            "inventory resolution produced no canonical targets beyond exact source_url",
            policy.source_url.as_str(),
        );
    }
    Ok(InventoryResolution {
        pages,
        diagnostics,
        robots,
        downloaded_bytes: total_inventory_bytes.load(Ordering::SeqCst),
    })
}

// ---------- parallel fetch engine ----------

struct HostGate {
    next_allowed: Mutex<HashMap<String, std::time::Instant>>,
    host_delay: f64,
}

impl HostGate {
    fn new(host_delay: f64) -> Self {
        Self {
            next_allowed: Mutex::new(HashMap::new()),
            host_delay,
        }
    }

    /// Block until this host's next slot, then reserve it.
    fn wait_turn(&self, url: &str) {
        loop {
            let now = std::time::Instant::now();
            let host = lib::origin_of(url);
            let mut slots = self.next_allowed.lock();
            let slot = slots.entry(host).or_insert(now);
            if *slot <= now {
                *slot = now + std::time::Duration::from_secs_f64(self.host_delay);
                return;
            }
            let wait = *slot - now;
            drop(slots);
            std::thread::sleep(wait);
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct CrawlTarget {
    sequence: usize,
    key: String,
    url: String,
    lastmod: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct CrawlDiagnostic {
    code: String,
    message: String,
    url: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct PageOutcome {
    sequence: usize,
    url: String,
    resolved_url: String,
    status: Value,
    diagnostic: Option<CrawlDiagnostic>,
    text_bytes: Option<u64>,
    downloaded_bytes: u64,
    record_sha256: Option<String>,
    corpus_start: Option<u64>,
    corpus_end: Option<u64>,
}
#[derive(Clone, Deserialize, Serialize)]
struct JournalOutcome {
    key: String,
    outcome: PageOutcome,
}

#[derive(Deserialize, Serialize)]
struct OutcomeJournalBatch {
    schema: String,
    first_sequence: usize,
    last_sequence: usize,
    committed_bytes: u64,
    committed_sha256: String,
    outcomes: Vec<JournalOutcome>,
}


#[derive(Clone, Deserialize, Serialize)]
struct DurableState {
    schema: String,
    run_id: String,
    source_revision: String,
    source_input_sha256: String,
    record_key: String,
    record: String,
    attempt: u32,
    attempt_id: String,
    source_url: String,
    started_at: String,
    effective_source_url: String,
    inventory_complete: bool,
    inventory_downloaded_bytes: u64,
    inventory_sha256: Option<String>,
    inventory_diagnostics: Vec<CrawlDiagnostic>,
    robots: Option<RobotsSnapshot>,
    targets: Vec<CrawlTarget>,
    outcomes: BTreeMap<String, PageOutcome>,
    committed_bytes: u64,
    committed_sha256: String,
    completed_at: Option<String>,
    report_sha256: Option<String>,
}

struct FetchedOutcome {
    target: CrawlTarget,
    status: Value,
    diagnostic: Option<CrawlDiagnostic>,
    text_bytes: Option<u64>,
    downloaded_bytes: u64,
    line: Option<Vec<u8>>,
    resolved_url: String,
}

struct WriteRequest {
    outcome: FetchedOutcome,
    acknowledge: mpsc::Sender<std::result::Result<(), String>>,
}

enum WriterMessage {
    Outcome(WriteRequest),
    Abort(String),
}

struct FetchShared {
    queue: Mutex<std::vec::IntoIter<CrawlTarget>>,
    writer: mpsc::Sender<WriterMessage>,
    gate: HostGate,
    policy: UrlPolicy,
    downloaded_bytes: AtomicU64,
    cancelled: Arc<AtomicBool>,
    robots: CompiledRobots,
}

#[derive(Clone)]
struct WorkLayout {
    root: PathBuf,
    corpus: PathBuf,
    state: PathBuf,
    pages: PathBuf,
    journal: PathBuf,
    report: PathBuf,
}

struct WorkLock {
    file: File,
}

impl WorkLock {
    fn acquire(layout: &WorkLayout) -> Result<Self> {
        std::fs::create_dir_all(&layout.root)
            .with_context(|| format!("create durable work directory {}", layout.root.display()))?;
        let parent = layout
            .root
            .parent()
            .context("durable work directory has no parent")?;
        let name = layout
            .root
            .file_name()
            .and_then(|value| value.to_str())
            .context("durable work directory has no UTF-8 name")?;
        let file = open_regular_file(
            &parent.join(format!(".{name}.crawl.lock")),
            true,
            true,
            false,
            true,
            "durable work lock",
        )?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            bail!(
                "documentation crawl {} is already active in another worker",
                layout.root.display()
            );
        }
        Ok(Self { file })
    }
}

impl Drop for WorkLock {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

#[derive(Clone)]
struct WorkerOptions {
    site: Option<String>,
    all: bool,
    exclude: Vec<String>,
    workers: usize,
    host_delay: f64,
    refresh: bool,
}

impl WorkerOptions {
    fn parse(rest: &[String]) -> Result<Self> {
        let mut options = Self {
            site: None,
            all: false,
            exclude: Vec::new(),
            workers: MAX_WORKERS,
            host_delay: 0.3,
            refresh: false,
        };
        let mut i = 0;
        while i < rest.len() {
            match rest[i].as_str() {
                "--site" => {
                    i += 1;
                    options.site = Some(rest.get(i).context("--site needs a value")?.clone());
                }
                "--all" => options.all = true,
                "--exclude" => {
                    i += 1;
                    options
                        .exclude
                        .push(rest.get(i).context("--exclude needs a value")?.clone());
                }
                "--workers" => {
                    i += 1;
                    options.workers = rest.get(i).context("--workers needs a value")?.parse()?;
                }
                "--host-delay" => {
                    i += 1;
                    options.host_delay =
                        rest.get(i).context("--host-delay needs a value")?.parse()?;
                }
                "--refresh" => options.refresh = true,
                other => bail!("unknown argument: {other}"),
            }
            i += 1;
        }
        if options.site.is_none() && !options.all {
            bail!("pass --site <NN-slug> or --all");
        }
        if options.workers == 0 || options.workers > MAX_WORKERS {
            bail!("--workers must be between 1 and {MAX_WORKERS}");
        }
        if !options.host_delay.is_finite()
            || !(0.0..=MAX_HOST_DELAY_SECONDS).contains(&options.host_delay)
        {
            bail!(
                "--host-delay must be a finite number between 0 and {MAX_HOST_DELAY_SECONDS} seconds"
            );
        }
        Ok(options)
    }
}

fn safe_path_component(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{label} contains unsafe durable-path characters");
    }
    Ok(())
}

fn exact_lower_hex(value: &str, length: usize, label: &str) -> Result<()> {
    if value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not an exact {length}-character lowercase hexadecimal digest");
    }
    Ok(())
}

fn manifest_attempt(manifest: &super::crawl::RuntimeManifest) -> Result<(u32, String)> {
    let encoded = serde_json::to_value(manifest)?;
    let attempt = encoded
        .get("attempt")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .context("runtime manifest has no positive immutable attempt")?;
    let attempt_id = encoded
        .get("attempt_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("runtime manifest has no immutable attempt_id")?
        .to_string();
    for (value, label) in [
        (manifest.run_id.as_str(), "runtime manifest run_id"),
        (manifest.catalog.as_str(), "runtime manifest catalog"),
        (manifest.record.as_str(), "runtime manifest record"),
        (attempt_id.as_str(), "runtime manifest attempt_id"),
    ] {
        safe_path_component(value, label)?;
    }
    exact_lower_hex(&manifest.record_key, 64, "runtime manifest record_key")?;
    let base = crate::crawl_attempt_base_uri(
        &manifest.run_id,
        &manifest.catalog,
        &manifest.record,
        &manifest.record_key,
        attempt,
        &attempt_id,
    );
    if manifest.artifact_uri != format!("{base}/artifacts.tar.gz")
        || manifest.output_uri != format!("{base}/worker-output.log")
    {
        bail!(
            "runtime manifest artifact/output URIs do not exactly match canonical run/catalog/record/record_key/attempt/attempt_id coordinates"
        );
    }
    Ok((attempt, attempt_id))
}
fn manifest_structure_sha256(manifest: &super::crawl::RuntimeManifest) -> Result<String> {
    let encoded = serde_json::to_value(manifest)?;
    let digest = encoded
        .get("docs_structure_sha256")
        .and_then(Value::as_str)
        .context("documentation runtime manifest has no docs_structure_sha256")?;
    exact_lower_hex(
        digest,
        64,
        "runtime manifest docs_structure_sha256",
    )?;
    Ok(digest.to_string())
}

fn work_layout(manifest: &super::crawl::RuntimeManifest) -> Result<WorkLayout> {
    safe_path_component(&manifest.run_id, "runtime manifest run_id")?;
    safe_path_component(&manifest.catalog, "runtime manifest catalog")?;
    safe_path_component(&manifest.record, "runtime manifest record")?;
    exact_lower_hex(&manifest.source_revision, 40, "runtime manifest source_revision")?;
    exact_lower_hex(
        &manifest.source_input_sha256,
        64,
        "runtime manifest source_input_sha256",
    )?;
    exact_lower_hex(&manifest.record_key, 64, "runtime manifest record_key")?;
    let (_, attempt_id) = manifest_attempt(manifest)?;
    safe_path_component(&attempt_id, "runtime manifest attempt_id")?;
    let home = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .context("HOME is not set; cannot locate the durable crawl work root")?;
    let base = PathBuf::from(home).join(".spis/crawls");
    let root = super::crawl::native_attempt_root(&base, manifest)?;
    let corpus = root.clone();
    Ok(WorkLayout {
        state: corpus.join("state.json"),
        pages: corpus.join("pages.jsonl.gz"),
        journal: corpus.join("outcomes.jsonl"),
        report: corpus.join("docs-retrieval-run.json"),
        root,
        corpus,
    })
}

fn regular_file_exists(path: &Path, label: &str) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                bail!("{label} is not a regular non-symlink file: {}", path.display());
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}


fn open_regular_file(
    path: &Path,
    read: bool,
    write: bool,
    append: bool,
    create: bool,
    label: &str,
) -> Result<File> {
    let exists = match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                bail!("{label} is not a regular non-symlink file: {}", path.display());
            }
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error).with_context(|| format!("inspect {label}")),
    };
    let mut options = OpenOptions::new();
    options
        .read(read)
        .write(write)
        .append(append)
        .custom_flags(libc::O_NOFOLLOW);
    if create && !exists {
        options.create_new(true);
    }
    let file = options
        .open(path)
        .with_context(|| format!("open {label} {}", path.display()))?;
    if !file.metadata()?.is_file() {
        bail!("{label} opened as a non-regular file: {}", path.display());
    }
    Ok(file)
}
pub(crate) fn staging_directory(root: &Path, label: &str) -> Result<PathBuf> {
    for _ in 0..128 {
        let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::SeqCst);
        let directory = root.join(format!(".{label}-{}-{sequence}", std::process::id()));
        match std::fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("create staging directory {}", directory.display()))
            }
        }
    }
    bail!("could not allocate a unique {label} staging directory")
}

/// The exact artifact set an attempt root may contain. `corpus_summary` enforces
/// it and `audit_attempt_tree` archives it, so nothing else may ever be staged
/// or left inside `layout.root`.
const CORPUS_ARTIFACTS: [&str; 4] = [
    "docs-retrieval-run.json",
    "outcomes.jsonl",
    "pages.jsonl.gz",
    "state.json",
];

/// Name for a staged temp file, scoped by the owning attempt directory so two
/// workers on sibling attempt_ids under the same `attempts/<n>` parent never
/// share a temp namespace. This mirrors the `.{attempt}.crawl.lock` and
/// `.{attempt}.archive.lock` names that already live in that directory.
fn temporary_name(owner: &str, name: &str, sequence: u64) -> String {
    format!(".{owner}.{name}.{}-{sequence}.tmp", std::process::id())
}

/// Does `file_name` denote a staged temp file for one of this attempt's own
/// artifacts? Recognises the current attempt-scoped shape
/// `.<owner>.<artifact>.<pid>-<sequence>.tmp` and the legacy in-root shape
/// `.<artifact>.<pid>-<sequence>.tmp` that earlier workers left behind. Anything
/// that does not match one of those exact shapes is never touched, so a sibling
/// attempt's temp and every lock, archive and read-back file are left alone.
fn is_own_temporary(file_name: &str, owner: &str) -> bool {
    let Some(body) = file_name
        .strip_prefix('.')
        .and_then(|rest| rest.strip_suffix(".tmp"))
    else {
        return false;
    };
    let body = body
        .strip_prefix(owner)
        .and_then(|rest| rest.strip_prefix('.'))
        .unwrap_or(body);
    let Some(suffix) = CORPUS_ARTIFACTS
        .iter()
        .find_map(|artifact| body.strip_prefix(*artifact))
    else {
        return false;
    };
    let Some((pid, sequence)) = suffix
        .strip_prefix('.')
        .and_then(|rest| rest.split_once('-'))
    else {
        return false;
    };
    !pid.is_empty()
        && !sequence.is_empty()
        && pid.bytes().all(|byte| byte.is_ascii_digit())
        && sequence.bytes().all(|byte| byte.is_ascii_digit())
}

/// Drop temp files this attempt orphaned in an earlier, killed run. Called at
/// resume while the exclusive work lock is held, so no live writer can own a
/// matching name. Without this, a SIGKILL inside the old in-root `write_all` +
/// `sync_all` window left a `.state.json.<pid>-<n>.tmp` that made
/// `corpus_summary` reject the attempt for good.
fn prune_stale_temporaries(layout: &WorkLayout) -> Result<()> {
    let owner = layout
        .root
        .file_name()
        .and_then(|value| value.to_str())
        .context("durable work directory has no UTF-8 name")?;
    let staging_parent = layout
        .root
        .parent()
        .context("durable work directory has no staging parent")?;
    for directory in [staging_parent, layout.root.as_path()] {
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("list staging directory {}", directory.display()))
            }
        };
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            if !is_own_temporary(file_name, owner) || !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            std::fs::remove_file(&path)
                .with_context(|| format!("remove stale staged file {}", path.display()))?;
        }
    }
    Ok(())
}

/// Path of the retained worker failure diagnostic. It sits beside the attempt
/// root, alongside `<attempt_id>.tar.gz`, so it is neither audited by
/// `corpus_summary` nor archived by `publish_attempt_archive`.
fn failure_diagnostic_path(layout: &WorkLayout) -> Result<PathBuf> {
    let owner = layout
        .root
        .file_name()
        .and_then(|value| value.to_str())
        .context("durable work directory has no UTF-8 name")?;
    let parent = layout
        .root
        .parent()
        .context("durable work directory has no parent")?;
    Ok(parent.join(format!("{owner}.failure.json")))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
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

fn state_bytes(state: &DurableState) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(state)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn checkpoint_state(path: &Path, state: &DurableState) -> Result<()> {
    atomic_write(path, &state_bytes(state)?)
        .with_context(|| format!("checkpoint documentation crawl state {}", path.display()))
}

fn read_state(path: &Path) -> Result<DurableState> {
    let mut file = open_regular_file(path, true, false, false, false, "durable state")?;
    if file.metadata()?.len() > MAX_STATE_BYTES {
        bail!("durable documentation state exceeds its byte limit");
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse durable documentation state {}", path.display()))
}
fn reset_outcome_journal(layout: &WorkLayout) -> Result<()> {
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

fn replay_outcome_journal(layout: &WorkLayout, state: &mut DurableState) -> Result<()> {
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

fn fresh_state(
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

fn inventory_sha256(
    targets: &[CrawlTarget],
    diagnostics: &[CrawlDiagnostic],
    robots: &RobotsSnapshot,
    downloaded_bytes: u64,
) -> Result<String> {
    Ok(lib::sha256_hex(&serde_json::to_vec(&json!({
        "targets": targets,
        "diagnostics": diagnostics,
        "robots": robots,
        "downloaded_bytes": downloaded_bytes,
    }))?))
}

fn validate_state(
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

fn hash_reader(mut reader: impl Read) -> Result<(String, u64)> {
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

fn reconcile_corpus(layout: &WorkLayout, state: &DurableState) -> Result<()> {
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

fn fetch_target(
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

fn load_stream_hasher(path: &Path, committed_bytes: u64) -> Result<Sha256> {
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


const WRITER_BATCH_SIZE: usize = 32;
const WRITER_BATCH_WAIT: Duration = Duration::from_millis(10);

fn accept_writer_message(
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

fn writer_loop(
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

fn run_fetch_workers(
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
fn source_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn validate_worker_source(
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

fn load_or_create_state(
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

fn report_bytes(report: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(report)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn build_report(
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
    let retrieval_status = if state.targets.is_empty() || retrieved_count == 0 {
        "retrieval_empty"
    } else if text_page_count == 0 {
        "retrieval_no_text"
    } else if !diagnostics.is_empty()
        || retrieved_count != state.targets.len()
        || ok_count != state.targets.len()
        || text_page_count != state.targets.len()
    {
        "retrieval_partial"
    } else {
        "retrieval_complete"
    };
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

fn finish_run(
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

fn corpus_summary(layout: &WorkLayout, state: &DurableState) -> Result<Value> {
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

fn worker_report(
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

fn crawl_attempt(
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

fn run_worker(rest: &[String], manifest: &super::crawl::RuntimeManifest) -> Result<()> {
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

const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    safe_path_component(value, flag)
        .map_err(|_| anyhow::anyhow!("{flag} contains characters that cannot be submitted to a worker"))
}

fn source_revision() -> Result<String> {
    super::crawl::build_revision()
}



#[derive(Deserialize)]
struct StorageStatReceipt {
    schema: String,
    path: String,
    state: String,
    size: Option<u64>,
}

fn storage_artifact_present(uri: &str, context: &str) -> Result<bool> {
    let mut command = super::crawl::crawl_storage_command();
    command.args(["storage", "stat", uri, "--json"]);
    let output = super::crawl::bounded_command_output(
        &mut command,
        context,
        Duration::from_secs(60),
        STADO_OUTPUT_LIMIT,
    )?;
    if !output.status.success() {
        bail!(
            "cannot determine whether immutable documentation attempt artifact is published: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let response: StorageStatReceipt =
        serde_json::from_slice(&output.stdout).context("parse Stado storage stat response")?;
    if response.schema != "stado.storage-stat-receipt.v1" || response.path != uri {
        bail!("Stado storage stat receipt has the wrong schema or URI identity");
    }
    match response.state.as_str() {
        "absent" if response.size.is_none() => Ok(false),
        "present" if response.size.is_some() => Ok(true),
        state => bail!(
            "immutable documentation attempt URI {uri} has unsupported or inconsistent storage state {state}"
        ),
    }
}
fn refuse_published_refresh(uri: &str) -> Result<()> {
    if storage_artifact_present(
        uri,
        "check immutable documentation attempt artifact before --refresh",
    )? {
        bail!(
            "--refresh refuses published immutable attempt URI {uri}; create a fresh crawl attempt"
        );
    }
    Ok(())
}


fn shell_quote(value: &str) -> Result<String> {
    if value.contains('\0') {
        bail!("documentation worker argument contains a NUL byte");
    }
    Ok(format!("'{}'", value.replace('\'', "'\\''")))
}

fn submit_worker(
    host: &str,
    record: &str,
    manifest: &super::crawl::RuntimeManifest,
    forwarded: &[String],
) -> Result<()> {
    safe_job_value(host, "--host")?;
    safe_job_value(record, "--record")?;
    if source_revision()? != manifest.source_revision {
        bail!("documentation coordinator revision does not match immutable runtime manifest");
    }
    let _ = manifest_attempt(manifest)?;
    let mut worker_options = forwarded.to_vec();
    if !worker_options
        .iter()
        .any(|argument| matches!(argument.as_str(), "--site" | "--all"))
    {
        worker_options.insert(0, record.to_string());
        worker_options.insert(0, "--site".into());
    }
    let parsed = WorkerOptions::parse(&worker_options)?;
    if parsed.all || parsed.site.as_deref() != Some(record) {
        bail!("documentation worker arguments do not select the immutable manifest record");
    }
    if parsed.refresh {
        refuse_published_refresh(&manifest.artifact_uri)?;
    }
    let artifact = manifest.artifact_uri.clone();
    let output_uri = manifest.output_uri.clone();
    // The absolute path Stado resolved on this host, never the bare name. The
    // submitted command runs under a non-login `/bin/sh` that reads no
    // profile, so `cargo` alone resolved to nothing and
    // job-545551889f9e88be30daa81f died sixteen minutes into a claimed slot
    // with `/bin/sh: cargo: command not found`.
    let cargo = super::crawl::resolved_program(host, &["cargo", "--version"])?;
    let mut command_arguments = vec![
        cargo,
        "run".to_string(),
        "--release".to_string(),
        "--".to_string(),
        "crawl-docs".to_string(),
        "--worker".to_string(),
        "--artifact-uri".to_string(),
        artifact.clone(),
        "--runtime-manifest-base64".to_string(),
        manifest.encoded()?,
    ];
    command_arguments.extend(worker_options);
    let command = command_arguments
        .iter()
        .map(|argument| shell_quote(argument))
        .collect::<Result<Vec<_>>>()?
        .join(" ");
    let mut stado = super::crawl::stado_command();
    stado.args([
        "submit",
        &command,
        "--run-id",
        &manifest.stado_run_id,
        "--pinned-host",
        host,
        "--repo",
        REPOSITORY,
        "--repo-ref",
        &manifest.source_revision,
        "--repo-workdir",
        super::crawl::STADO_REPO_WORKDIR,
        "--repo-extras",
        "",
        "--output-uri",
        &output_uri,
    ]);
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "submit documentation crawl through Stado",
        STADO_COMMAND_TIMEOUT,
        STADO_OUTPUT_LIMIT,
    )?;
    if !output.status.success() {
        bail!(
            "Stado refused documentation crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    super::crawl::print_submission(
        "documentation-site-examples",
        "docs",
        host,
        Some(&artifact),
        &output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut host = None;
    let mut worker = false;
    let mut artifact_uri = None;
    let mut record = None;
    let mut runtime_manifest_base64 = None;
    let mut forwarded = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--site" => {
                i += 1;
                let value = rest.get(i).context("--site needs a value")?.clone();
                record = Some(value.clone());
                forwarded.push("--site".into());
                forwarded.push(value);
            }
            "--runtime-manifest-base64" => {
                i += 1;
                runtime_manifest_base64 =
                    Some(rest.get(i).context("--runtime-manifest-base64 needs a value")?.clone());
            }
            "--worker" => worker = true,
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            value => forwarded.push(value.to_string()),
        }
        i += 1;
    }
    let record = record.context("--record is required for one exact per-record job")?;
    let manifest = super::crawl::decode_runtime_manifest(
        runtime_manifest_base64.as_deref().context("--runtime-manifest-base64 is required")?,
        "documentation-site-examples",
        "docs",
        Some(&record),
    )?;
    if !worker {
        return submit_worker(
            &host.context("--host is required; documentation crawls execute as pinned Stado jobs")?,
            &record,
            &manifest,
            &forwarded,
        );
    }
    if host.is_some() {
        bail!("--host cannot be used with --worker");
    }
    let artifact_uri = artifact_uri.context("--artifact-uri is required in worker mode")?;
    if artifact_uri != manifest.artifact_uri {
        bail!("worker artifact URI does not match immutable runtime manifest");
    }
    run_worker(&forwarded, &manifest)?;
    Ok(())
}
