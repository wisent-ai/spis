use super::*;

/// Typed record failure so the worker report can name a stable machine code
/// instead of a free-text diagnostic.
#[derive(Debug)]
pub(crate) struct RecordFailure {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

impl std::fmt::Display for RecordFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for RecordFailure {}

pub(crate) fn failure_code(error: &anyhow::Error) -> &'static str {
    error
        .chain()
        .find_map(|cause| {
            cause
                .downcast_ref::<RecordFailure>()
                .map(|failure| failure.code)
        })
        .unwrap_or("mobile_record_failed")
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
            _ => bail!("crawl-mobile accepts ios-app-examples or android-app-examples"),
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
    pub(crate) destructive: bool,
    pub(crate) kind: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct PathStep {
    pub(crate) selector: String,
    pub(crate) label: String,
}

pub(crate) fn canonical_driver_url(value: &str) -> Result<String> {
    let parsed = url::Url::parse(value).context("--driver-url must be a URL")?;
    let local = matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    if parsed.host_str().is_none()
        || (parsed.scheme() != "https" && !(parsed.scheme() == "http" && local))
    {
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

/// Per-endpoint response bound in bytes.
pub(crate) fn response_limit(path: &str) -> u64 {
    if path.ends_with("/screenshot") {
        32 * 1024 * 1024
    } else if path.ends_with("/stop_recording_screen") {
        256 * 1024 * 1024
    } else {
        8 * 1024 * 1024
    }
}

pub(crate) struct Appium {
    pub(crate) base: String,
    pub(crate) agent: ureq::Agent,
}
