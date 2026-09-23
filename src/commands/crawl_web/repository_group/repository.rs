use super::*;

pub(crate) const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

pub(crate) const CATALOGS: &[&str] = &[
    "web-app-examples",
    "dashboard-console-examples",
    "onboarding-auth-examples",
    "app-store-listing-examples",
    "design-system-examples",
    "report-evidence-examples",
    "pricing-page-examples",
    "landing-page-examples",
];

pub(crate) const REPORT_SCHEMA: &str = "wisent.web-worker-report.v1";

pub(crate) const FAILURE_SCHEMA: &str = "wisent.web-worker-failure.v1";

pub(crate) const OBSERVATION_SCHEMA: &str = "wisent.spis-weles-observation.v1";

pub(crate) const EVIDENCE_MANIFEST_SCHEMA: &str = "weles.browser-evidence-manifest.v1";

/// The version the service retains for a terminal non-success: no navigation URLs, no
/// required evidence kind, and the outcome its receipt signed.
pub(crate) const NON_SUCCESS_EVIDENCE_MANIFEST_SCHEMA: &str = "weles.browser-evidence-manifest.v2";

pub(crate) const OFFICIAL_REQUEST_SCHEMA: &str = "weles.task.current";

pub(crate) const SERVICE_NAME: &str = "weles-admission";

pub(crate) const SERVICE_CONSUMER: &str = "spis";

pub(crate) const SERVICE_CAPABILITY: &str = "browser-evidence";

pub(crate) const RELEASE_PREFIX: &str = "weles-worker@";

pub(crate) const SCREENSHOT_KIND: &str = "screenshot";

pub(crate) const ACCESSIBILITY_KIND: &str = "accessibility_tree";

pub(crate) const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";

pub(crate) const MAXIMUM_EVIDENCE_BYTES: u64 = 8 * 1024 * 1024;

pub(crate) const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// A typed worker failure. Every exit path carries an exact machine-readable code so the
/// importer can distinguish an infrastructure refusal from a rejected attempt.
pub(crate) struct WorkerFailure {
    pub(crate) code: String,
    pub(crate) message: String,
}

impl WorkerFailure {
    pub(crate) fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl From<anyhow::Error> for WorkerFailure {
    fn from(error: anyhow::Error) -> Self {
        Self::new("web_worker_failed", format!("{error:#}"))
    }
}

impl From<serde_json::Error> for WorkerFailure {
    fn from(error: serde_json::Error) -> Self {
        Self::new("web_worker_json_failed", error.to_string())
    }
}

impl From<std::io::Error> for WorkerFailure {
    fn from(error: std::io::Error) -> Self {
        Self::new("web_worker_io_failed", error.to_string())
    }
}

pub(crate) type Outcome<T> = std::result::Result<T, WorkerFailure>;

pub(crate) fn ensure(condition: bool, code: &str, message: &str) -> Outcome<()> {
    if condition {
        Ok(())
    } else {
        Err(WorkerFailure::new(code, message))
    }
}

/// Everything obtained before a failure, so a failed attempt still reports what it proved.
#[derive(Default)]
pub(crate) struct Collected {
    pub(crate) submission: Option<weles::WelesSubmission>,
    pub(crate) status: Option<weles::WelesTaskStatus>,
    pub(crate) cancellation: Option<weles::WelesCancellation>,
    pub(crate) provenance: Option<weles::WelesProvenanceDocument>,
    pub(crate) envelope: Option<weles::WelesAttemptEnvelope>,
}

pub(crate) fn is_lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

pub(crate) fn is_sha256(value: &str) -> bool {
    is_lowercase_hex(value, 64)
}

pub(crate) fn is_git_revision(value: &str) -> bool {
    is_lowercase_hex(value, 40)
}

pub(crate) fn is_sha256_id(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(is_sha256)
}

/// `weles_provenance::is_portable_attempt_component` and the bridge's
/// `portableAttemptComponent` accept exactly this alphabet.
pub(crate) fn is_portable_component(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

/// A retained evidence tail: `validate_evidence_inventory` refuses anything that is not a
/// chain of `Component::Normal` portable segments.
pub(crate) fn is_portable_relative(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('\\')
        && value.split('/').all(is_portable_component)
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

/// `validatedServiceIdentity` in the deployed `scripts/worker/public-task-service.mjs`
/// admits `active_host` only against `/^[A-Za-z0-9._-]+$/`, so a host this worker would
/// accept but the service rejects must fail here, where the reason is typed, instead of
/// at admission, where it is a bare refusal.
pub(crate) fn is_service_host(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

/// The same admission rule requires `release_id` to be exactly
/// `/^weles-worker@\d+\.\d+\.\d+$/`, compared against the release the service is running;
/// a bare `weles-worker@` prefix is not enough.
pub(crate) fn is_service_release_id(value: &str) -> bool {
    let Some(version) = value.strip_prefix(RELEASE_PREFIX) else {
        return false;
    };
    let mut fields = version.split('.');
    let numeric = |field: Option<&str>| {
        field.is_some_and(|field| {
            !field.is_empty() && field.bytes().all(|byte| byte.is_ascii_digit())
        })
    };
    numeric(fields.next())
        && numeric(fields.next())
        && numeric(fields.next())
        && fields.next().is_none()
}

pub(crate) fn safe_job_value(value: &str, flag: &str) -> Result<()> {
    if value.is_empty()
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{flag} contains characters that cannot be submitted to a worker");
    }
    Ok(())
}

/// Mirrors `weles_provenance::validate_api_endpoint`: the exact canonical `/api/v1` base.
pub(crate) fn validate_api_endpoint(value: &str) -> Outcome<()> {
    let endpoint = url::Url::parse(value)
        .map_err(|_| WorkerFailure::new("weles_service_endpoint_invalid", "endpoint is not a URL"))?;
    ensure(
        matches!(endpoint.scheme(), "http" | "https")
            && endpoint.username().is_empty()
            && endpoint.password().is_none()
            && endpoint.path() == "/api/v1"
            && endpoint.query().is_none()
            && endpoint.fragment().is_none()
            && endpoint.as_str() == value,
        "weles_service_endpoint_invalid",
        "the Weles service endpoint is not the canonical exact /api/v1 base",
    )
}

pub(crate) fn same_origin(value: &str, product_url: &url::Url) -> bool {
    url::Url::parse(value).is_ok_and(|parsed| {
        matches!(parsed.scheme(), "http" | "https")
            && parsed.username().is_empty()
            && parsed.password().is_none()
            && parsed.origin() == product_url.origin()
    })
}

pub(crate) fn attempt_base(binding: &weles::WelesAttemptBinding) -> String {
    crate::crawl_attempt_base_uri(
        &binding.run_id,
        &binding.catalog,
        &binding.record,
        &binding.record_key,
        binding.attempt,
        &binding.attempt_id,
    )
}

pub(crate) fn objective(catalog: &str, name: &str, goal: &str) -> Result<String> {
    let (surface, coverage, source_guard) = match catalog {
        "web-app-examples" => (
            "browser application",
            "global navigation, the primary create/read/update workflow, search and filters, empty/loading/error states, cancellation, recovery and the first successful result",
            "Do not replace the signed-in application with its marketing site, documentation, app-store page or a guessed flow.",
        ),
        "dashboard-console-examples" => (
            "dashboard or administrative console",
            "navigation hierarchy, date and scope filters, tables, sorting, search, drill-downs, charts, export previews, permission boundaries, empty/loading/error states and recovery",
            "Do not replace the live console with its public marketing site, documentation, screenshots or a guessed flow.",
        ),
        "onboarding-auth-examples" => (
            "onboarding and authentication journey",
            "sign-in, sign-up entry, SSO choices, password recovery, MFA when available, validation failures, backtracking, cancellation and the first authenticated success state",
            "Use only the account identity bound to this task; do not invent credentials or substitute a public product page.",
        ),
        "app-store-listing-examples" => (
            "application-store listing",
            "media carousel, device or platform variants, description expansion, release history, ratings and reviews, privacy and product information, in-app purchases and visible pricing",
            "Crawl the actual store listing named by product_url, not the installed app or the vendor landing page.",
        ),
        "design-system-examples" => (
            "design-system documentation and component explorer",
            "navigation, search, component examples, variants and properties, code or installation copy controls, theming, responsive examples, accessibility guidance and error or empty states",
            "Crawl the actual design-system reference or component explorer, not its owner’s corporate homepage.",
        ),
        "report-evidence-examples" => (
            "interactive report and its evidence surfaces",
            "filters, comparisons, drill-downs, source and evidence links, tables, charts, annotations, export previews, empty/loading/error states and recovery",
            "Crawl the actual report and its linked evidence surfaces, not a summary landing page.",
        ),
        "pricing-page-examples" => (
            "pricing page",
            "billing interval, currency or region controls, seat and usage calculators, plan comparisons, feature disclosure, FAQs, CTA transitions and checkout preview up to but excluding payment",
            "Crawl the actual pricing and plan-selection surface, not a generic product homepage.",
        ),
        "landing-page-examples" => (
            "landing page",
            "global navigation, product-information routes, CTA transitions, media and carousels, forms with validation and cancellation, and desktop, tablet and mobile responsive states",
            "Crawl the exact landing page named by product_url; do not substitute another vendor page, static screenshot or guessed flow.",
        ),
        other => bail!("crawl-web has no objective for catalog {other}"),
    };
    let goal = if goal.trim().is_empty() {
        "Map the product's reachable functionality"
    } else {
        goal
    };
    Ok(format!(
        "Crawl the real {surface} for {name}. {goal}. Required coverage: {coverage}. Systematically inspect every reachable non-destructive control and retain the accessibility and visual state before and after every interaction. Execute and retain distinct cancellation, failure and recovery variants only when the real product exposes them. Retain animations, transitions, loading states and the first-success result with exact browser-history event IDs and artifact URIs. Exercise keyboard focus order, live regions, a screen-reader-relevant accessibility tree and reduced-motion media preference; name any variant that could not be executed instead of inferring it. Open destructive flows only through their final confirmation screen and never commit the final destructive control. {source_guard} Finish with one machine-readable JSON object named spis_evidence. It must contain observed_url, surface_kind, visible_pricing_comparison, canonical_interactions, canonical_journey, canonical_motion_analysis, canonical_accessibility, and artifacts. Every canonical claim must cite an exact retained event ID or stado:// artifact URI; use null or an explicit gap rather than inventing evidence. For pricing pages, visible_pricing_comparison is true only after at least two visible plans or price alternatives were actually observed. For landing pages, observed_url must be the exact requested landing URL after normalization."
    ))
}
