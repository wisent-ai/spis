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

pub(crate) fn site_rules(
    slug: &str,
    meta: &SiteMeta,
    map: &HashMap<&'static str, Override>,
) -> SiteRules {
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

#[derive(Clone)]
pub(crate) struct UrlPolicy {
    pub(crate) declared_source_url: String,
    pub(crate) source_url: Url,
    pub(crate) origin: String,
    pub(crate) pinned_addresses: Arc<Vec<SocketAddr>>,
}

pub(crate) static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

impl UrlPolicy {
    pub(crate) fn new(source_url: &str) -> Result<Self> {
        let parsed =
            Url::parse(source_url).context("declared documentation source_url is invalid")?;
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
    if crate::WebScheme::of(url).is_none() {
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

/// Whether a destination is not a public address, by the one rule Wisent's
/// egress guards share: `wisent_public_destination::is_public`, read from the
/// IANA special-purpose registries, an IPv6 address that delivers to an IPv4
/// address judged as that IPv4 address.
pub(crate) fn forbidden_ip(address: IpAddr) -> bool {
    !wisent_public_destination::is_public(address)
}
