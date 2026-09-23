use super::*;

pub(crate) struct Override {
    pub(crate) sitemaps: &'static [&'static str],
    pub(crate) prefixes: &'static [&'static str],
    pub(crate) llms: &'static [&'static str],
}

pub(crate) fn overrides() -> HashMap<&'static str, Override> {
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
pub(crate) struct SiteMeta {
    pub(crate) name: String,
    pub(crate) source_url: String,
    #[serde(default)]
    pub(crate) inventory_source: String,
    #[serde(default)]
    pub(crate) landing_nav: Vec<LandingNavItem>,
}

#[derive(serde::Deserialize, Clone)]
pub(crate) struct LandingNavItem {
    pub(crate) path: String,
}

pub(crate) struct SiteRules {
    pub(crate) sitemaps: Vec<String>,
    pub(crate) llms: Vec<String>,
    pub(crate) prefixes: Vec<String>,
}

pub(crate) fn site_rules(slug: &str, meta: &SiteMeta, map: &HashMap<&'static str, Override>) -> SiteRules {
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

pub(crate) const MAX_INVENTORY_BYTES: usize = 8 * 1024 * 1024;

pub(crate) const MAX_ROBOTS_BYTES: usize = 512 * 1024;

pub(crate) const MAX_PAGE_BYTES: usize = 16 * 1024 * 1024;

pub(crate) const MAX_TARGETS: usize = 50_000;

/// How many distinct excluded URLs one run will count before it reports a
/// floor instead of a total. Held well above the largest inventory this
/// family declares (216,092) so every real site yields an exact number, and
/// bounded regardless because the count must not become its own memory leak.
pub(crate) const MAX_COUNTED_EXCLUDED_KEYS: usize = 1_000_000;

pub(crate) const MAX_INVENTORY_SOURCES: usize = 256;

pub(crate) const MAX_INVENTORY_DIAGNOSTICS: usize = 512;

pub(crate) const MAX_ROBOTS_RULES: usize = 4_096;

pub(crate) const MAX_REDIRECTS: usize = 5;

#[derive(Clone)]
pub(crate) struct UrlPolicy {
    pub(crate) declared_source_url: String,
    pub(crate) source_url: Url,
    pub(crate) origin: String,
    pub(crate) pinned_addresses: Arc<Vec<SocketAddr>>,
}

pub(crate) const MAX_WORKERS: usize = 8;

pub(crate) const MAX_HOST_DELAY_SECONDS: f64 = 30.0;

pub(crate) const MAX_TOTAL_INVENTORY_BYTES: u64 = 64 * 1024 * 1024;

pub(crate) const MAX_TOTAL_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub(crate) const MAX_CORPUS_BYTES: u64 = 1024 * 1024 * 1024;

pub(crate) const WRITER_LIVENESS_TIMEOUT: Duration = Duration::from_secs(180);

pub(crate) const STADO_OUTPUT_LIMIT: usize = 1024 * 1024;

pub(crate) const STADO_COMMAND_TIMEOUT: Duration = Duration::from_secs(30 * 60);

pub(crate) const DNS_LOOKUP_TIMEOUT: Duration = Duration::from_secs(15);

/// Longest `Allow:`/`Disallow:` value accepted from a served robots.txt. Real
/// robots.txt paths are far shorter; the cap keeps a hostile origin from handing
/// us a rule whose compiled program is unbounded.
pub(crate) const MAX_ROBOTS_PATTERN_BYTES: usize = 1024;

/// Explicit compiled-program ceiling for one robots rule, so the bound is ours
/// rather than whatever `regex` happens to default to.
pub(crate) const MAX_ROBOTS_PROGRAM_BYTES: usize = 1024 * 1024;

pub(crate) static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) const MAX_JOURNAL_BYTES: u64 = 256 * 1024 * 1024;

pub(crate) const MAX_STATE_BYTES: u64 = 256 * 1024 * 1024;

impl UrlPolicy {
    pub(crate) fn new(source_url: &str) -> Result<Self> {
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

    pub(crate) fn canonical(&self, raw: &str, base: Option<&Url>, label: &str) -> Result<Url> {
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

pub(crate) fn validate_url_shape(url: &Url, label: &str) -> Result<()> {
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

pub(crate) fn forbidden_ip(address: IpAddr) -> bool {
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
