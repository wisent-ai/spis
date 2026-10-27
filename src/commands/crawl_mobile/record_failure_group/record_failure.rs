use super::*;

pub(crate) use crate::commands::crawl::RecordFailure;

pub(crate) fn failure_code(error: &anyhow::Error) -> &'static str {
    crate::commands::crawl::record_failure_code(error, "mobile_record_failed")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Platform {
    Ios,
    Android,
}

impl Platform {
    pub(crate) fn from_catalog(catalog: &str) -> Result<Self> {
        match catalog {
            "ios-app-examples" => Ok(Self::Ios),
            "android-app-examples" => Ok(Self::Android),
            _ => bail!("crawler run mobile accepts ios-app-examples or android-app-examples"),
        }
    }

    pub(crate) fn appium_name(self) -> &'static str {
        match self {
            Self::Ios => "iOS",
            Self::Android => "Android",
        }
    }

    pub(crate) fn automation(self) -> &'static str {
        match self {
            Self::Ios => "XCUITest",
            Self::Android => "UiAutomator2",
        }
    }

    pub(crate) fn app_key(self) -> &'static str {
        match self {
            Self::Ios => "appium:bundleId",
            Self::Android => "appium:appPackage",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct Action {
    pub(crate) selector: String,
    pub(crate) label: String,
    pub(crate) safety: crate::commands::control_safety::Safety,
    pub(crate) kind: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct PathStep {
    pub(crate) selector: String,
    pub(crate) label: String,
}

pub(crate) fn canonical_driver_url(value: &str) -> Result<String> {
    let parsed = url::Url::parse(value).context("--driver-url must be a URL")?;
    // Loopback is read from the address itself; `localhost` is the name
    // reserved for it.
    let local = match parsed.host() {
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        Some(url::Host::Domain(name)) => name == "localhost",
        None => false,
    };
    // The driver is reached over TLS, or in the clear only on this machine.
    let scheme = crate::WebScheme::of(&parsed);
    let encrypted = scheme.is_some_and(crate::WebScheme::encrypted);
    let plain = scheme == Some(crate::WebScheme::Http);
    if parsed.host_str().is_none() || !(encrypted || plain && local) {
        bail!("--driver-url must be HTTPS or loopback HTTP");
    }
    if !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !matches!(parsed.path(), "" | "/")
    {
        bail!("--driver-url may contain only scheme, host and port");
    }
    Ok(parsed.as_str().trim_end_matches('/').to_string())
}

pub(crate) struct Appium {
    pub(crate) base: String,
    pub(crate) agent: ureq::Agent,
}
