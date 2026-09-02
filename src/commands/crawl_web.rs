//! One-record browser-evidence worker driven exclusively through the official Weles client.
//!
//! The coordinator never launches a browser and never speaks HTTP to Weles. It pins one
//! immutable runtime manifest into one exact-revision Stado job for exactly one catalog
//! record. The worker submits exactly one `generic_browser_task` through the checked-in
//! Node bridge (`weles-bridge/spis-weles-bridge.mjs`), which owns every Weles request,
//! loads the pinned official `@wisent-ai/weles-client`, and verifies every receipt.
//!
//! Everything this worker retains is re-proved locally before it is published: the live
//! service release, the signed Spis binding, the canonical official request, the
//! receipt-bound evidence manifest bytes and every retained evidence file. The documents
//! written here are the exact inputs of `crate::weles_provenance`, so each non-obvious
//! check below names the verifier invariant it satisfies.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::weles_provenance as weles;

const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";
const CATALOGS: &[&str] = &[
    "web-app-examples",
    "dashboard-console-examples",
    "onboarding-auth-examples",
    "app-store-listing-examples",
    "design-system-examples",
    "report-evidence-examples",
    "pricing-page-examples",
    "landing-page-examples",
];

const REPORT_SCHEMA: &str = "wisent.web-worker-report.v1";
const FAILURE_SCHEMA: &str = "wisent.web-worker-failure.v1";
const OBSERVATION_SCHEMA: &str = "wisent.spis-weles-observation.v1";
const EVIDENCE_MANIFEST_SCHEMA: &str = "weles.browser-evidence-manifest.v1";
const OFFICIAL_REQUEST_SCHEMA: &str = "weles.task.current";
const SERVICE_NAME: &str = "weles-admission";
const SERVICE_CONSUMER: &str = "spis";
const SERVICE_CAPABILITY: &str = "browser-evidence";
const RELEASE_PREFIX: &str = "weles-worker@";
const SCREENSHOT_KIND: &str = "screenshot";
const ACCESSIBILITY_KIND: &str = "accessibility_tree";
const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
const MAXIMUM_EVIDENCE_BYTES: u64 = 8 * 1024 * 1024;
const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// A typed worker failure. Every exit path carries an exact machine-readable code so the
/// importer can distinguish an infrastructure refusal from a rejected attempt.
struct WorkerFailure {
    code: String,
    message: String,
}

impl WorkerFailure {
    fn new(code: &str, message: impl Into<String>) -> Self {
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

type Outcome<T> = std::result::Result<T, WorkerFailure>;

fn ensure(condition: bool, code: &str, message: &str) -> Outcome<()> {
    if condition {
        Ok(())
    } else {
        Err(WorkerFailure::new(code, message))
    }
}

/// Everything obtained before a failure, so a failed attempt still reports what it proved.
#[derive(Default)]
struct Collected {
    submission: Option<weles::WelesSubmission>,
    status: Option<weles::WelesTaskStatus>,
    cancellation: Option<weles::WelesCancellation>,
    provenance: Option<weles::WelesProvenanceDocument>,
    envelope: Option<weles::WelesAttemptEnvelope>,
}

fn is_lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_sha256(value: &str) -> bool {
    is_lowercase_hex(value, 64)
}

fn is_git_revision(value: &str) -> bool {
    is_lowercase_hex(value, 40)
}

fn is_sha256_id(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(is_sha256)
}

/// `weles_provenance::is_portable_attempt_component` and the bridge's
/// `portableAttemptComponent` accept exactly this alphabet.
fn is_portable_component(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

/// A retained evidence tail: `validate_evidence_inventory` refuses anything that is not a
/// chain of `Component::Normal` portable segments.
fn is_portable_relative(value: &str) -> bool {
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
fn is_service_host(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

/// The same admission rule requires `release_id` to be exactly
/// `/^weles-worker@\d+\.\d+\.\d+$/`, compared against the release the service is running;
/// a bare `weles-worker@` prefix is not enough.
fn is_service_release_id(value: &str) -> bool {
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

fn safe_job_value(value: &str, flag: &str) -> Result<()> {
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
fn validate_api_endpoint(value: &str) -> Outcome<()> {
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

fn same_origin(value: &str, product_url: &url::Url) -> bool {
    url::Url::parse(value).is_ok_and(|parsed| {
        matches!(parsed.scheme(), "http" | "https")
            && parsed.username().is_empty()
            && parsed.password().is_none()
            && parsed.origin() == product_url.origin()
    })
}

fn attempt_base(binding: &weles::WelesAttemptBinding) -> String {
    format!(
        "stado://spis-crawls/{}/{}/{}/{}/attempts/{}/{}",
        binding.run_id,
        binding.catalog,
        binding.record,
        binding.record_key,
        binding.attempt,
        binding.attempt_id,
    )
}

fn objective(catalog: &str, name: &str, goal: &str) -> Result<String> {
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

fn submit_worker(
    host: &str,
    catalog: &str,
    record: &str,
    manifest: &super::crawl::RuntimeManifest,
    wait_seconds: u64,
) -> Result<()> {
    safe_job_value(host, "--host")?;
    safe_job_value(record, "--record")?;
    if super::crawl::build_revision()? != manifest.source_revision {
        bail!("web coordinator revision does not match immutable runtime manifest");
    }
    let service = manifest
        .service_identity
        .as_ref()
        .context("web crawls require the exact Weles service identity in the runtime manifest")?;
    if service.active_host != host {
        bail!(
            "runtime manifest Weles service identity is bound to {}, not {host}",
            service.active_host
        );
    }
    if manifest.delivery.kind != "weles-service-env" {
        bail!("web credential delivery must be weles-service-env");
    }
    if manifest.delivery.secret_env.len() != 2
        || !manifest.delivery.secret_env.contains_key("WELES_TOKEN")
        || !manifest.delivery.secret_env.contains_key("WISENT_ORGANIZATION_ID")
    {
        bail!("web delivery must carry exactly the WELES_TOKEN and WISENT_ORGANIZATION_ID secret references");
    }
    let command = format!(
        "cargo run --release -- crawl-web {catalog} --worker --record {record} --artifact-uri {} --wait-seconds {wait_seconds} --runtime-manifest-base64 '{}'",
        manifest.artifact_uri,
        manifest.encoded()?,
    );
    let mut arguments = vec![
        "submit".to_string(),
        command,
        "--run-id".to_string(),
        manifest.stado_run_id.clone(),
        "--pinned-host".to_string(),
        host.to_string(),
        "--repo".to_string(),
        REPOSITORY.to_string(),
        "--repo-ref".to_string(),
        manifest.source_revision.clone(),
        "--repo-workdir".to_string(),
        "spis".to_string(),
        "--repo-extras".to_string(),
        String::new(),
        "--output-uri".to_string(),
        manifest.output_uri.clone(),
    ];
    // `secret_env` is a BTreeMap, so the injected references are already in sorted key
    // order. The values are opaque `item#field` references and are never logged.
    for (name, reference) in &manifest.delivery.secret_env {
        arguments.push("--secret-env".to_string());
        arguments.push(format!("{name}={reference}"));
    }
    let mut stado = super::crawl::stado_command();
    stado.args(arguments);
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "submit web crawl through Stado",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "Stado refused web crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    super::crawl::print_submission(
        catalog,
        "web",
        host,
        Some(&manifest.artifact_uri),
        &manifest.output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

/// The private, never-published, never-logged bridge work area. It holds exactly one
/// file: the protected config carrying the delivered bearer. Bridge commands travel on
/// the child's stdin, so no command is ever written to the host.
struct PrivateBridge {
    config: PathBuf,
}

impl PrivateBridge {
    fn open(manifest: &super::crawl::RuntimeManifest) -> Result<Self> {
        let home = std::env::var_os("HOME")
            .context("HOME is required for the private Weles bridge work directory")?;
        let directory = PathBuf::from(home)
            .join(".stado")
            .join("work")
            .join("spis")
            .join("weles-bridge");
        std::fs::create_dir_all(&directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
        }
        prune_orphaned_configs(&directory)?;
        let config = directory.join(format!(
            "config-{}-{}.json",
            manifest.attempt_id,
            std::process::id()
        ));
        Ok(Self { config })
    }

    /// Removes the delivered bearer token from the host. `Drop` guarantees this runs on
    /// every exit this process controls, including a panic unwind; the explicit call in
    /// `run_worker` only makes it prompt. A signal kill runs neither, which is why
    /// `open` also prunes the configs earlier runs orphaned.
    fn discard(&self) {
        let _ = std::fs::remove_file(&self.config);
    }

    fn write_config(&self, endpoint: &str, bearer: &str, organization_id: &str) -> Outcome<()> {
        let document = json!({
            "schema": weles::BRIDGE_CONFIG_SCHEMA,
            "endpoint": endpoint,
            "bearer": bearer,
            "organizationId": organization_id,
        });
        write_private(&self.config, &serde_json::to_vec(&document)?)
    }
}

impl Drop for PrivateBridge {
    fn drop(&mut self) {
        self.discard();
    }
}

/// Is `file_name` one of this worker's own config names, `config-{attempt_id}-{pid}.json`,
/// whose process is gone?
///
/// Only that exact shape is recognised, so no lock, no staged file and nothing another
/// tool left in this directory is ever considered, and a config whose pid is still alive
/// — a concurrent worker's delivered bearer — is always left alone.
fn is_orphaned_config(file_name: &str) -> bool {
    let Some(body) = file_name
        .strip_prefix("config-")
        .and_then(|rest| rest.strip_suffix(".json"))
    else {
        return false;
    };
    let Some((attempt_id, pid)) = body.rsplit_once('-') else {
        return false;
    };
    if !is_portable_component(attempt_id) {
        return false;
    }
    // Parsed as the signed `pid_t` the kernel takes: a value that does not fit is not a
    // pid this worker ever wrote, and a wrapped negative would address a process group.
    let Ok(pid) = pid.parse::<i32>() else {
        return false;
    };
    pid > 0 && !process_is_live(pid)
}

/// A worker killed by a signal — which is exactly how a job that outruns its wait budget
/// ends — runs no destructor, so its config file survives on the host with the delivered
/// bearer in it. Nothing else sweeps this directory, so every worker sweeps it on the way
/// in, mirroring `crawl_docs::prune_stale_temporaries`.
fn prune_orphaned_configs(directory: &Path) -> Result<()> {
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "list the private Weles bridge directory {}",
                    directory.display()
                )
            })
        }
    };
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if is_orphaned_config(file_name) && entry.file_type()?.is_file() {
            let path = entry.path();
            std::fs::remove_file(&path)
                .with_context(|| format!("remove the orphaned Weles config {}", path.display()))?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn process_is_live(pid: i32) -> bool {
    if unsafe { libc::kill(pid, 0) } == 0 {
        return true;
    }
    // Only a definite `ESRCH` proves the process is gone. `EPERM` means it exists under
    // another user, and any other errno is unexplained, so neither authorizes deleting a
    // file that still names a live bearer.
    std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

#[cfg(not(unix))]
fn process_is_live(_pid: i32) -> bool {
    true
}

fn write_private(path: &Path, bytes: &[u8]) -> Outcome<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    let _ = std::fs::remove_file(path);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    // The bridge refuses a config that any other user could read, so the bearer is never
    // written through a mode that has to be tightened afterwards.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

/// Content-addressed byte retention: an identical file is proof, never a rewrite.
fn write_exact(path: &Path, bytes: &[u8]) -> Outcome<()> {
    if std::fs::read(path).is_ok_and(|existing| existing == bytes) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Retains one operational attempt document: the submission, the task status, the
/// cancellation, the official provenance, the attempt envelope and the failure
/// diagnostic.
///
/// `relative` must name a path OUTSIDE the `weles/` and `recordings/` subtrees.
/// `crawl::apply_web_attempt` merges exactly those two subtrees into the shared record
/// directory, and `crawl::write_immutable_file` refuses a destination that already holds
/// different bytes. Every document written through here is named after its role instead
/// of its content, and a second attempt of the same record carries a different attempt
/// id, task id and digests, so a fixed name inside a merged subtree would permanently
/// block the second import of that record. These documents therefore stay with the
/// attempt: in the attempt root, in the published archive, and in the attempt-private
/// `crawl/{attempt_id}` tree the importer installs verbatim.
///
/// The staged temp file lives BESIDE the attempt root, next to the published archive and
/// its lock, so no `.tmp` byte is ever audited by `audit_attempt_tree`, archived, or
/// installed into the record. Unlike `crawl::atomic_json_write` this leaves no
/// `.{name}.lock` sibling either: the attempt root belongs to exactly one worker,
/// because `native_attempt_root` derives it from the `attempt_id` that already binds the
/// record key, the attempt number and the executing host, and the rename below is atomic,
/// so the destination is always either absent or one complete document.
fn retain_attempt_document(attempt_root: &Path, relative: &str, value: &Value) -> Outcome<()> {
    use std::io::Write;
    let io_failed = |message: &str| WorkerFailure::new("web_worker_io_failed", message);
    let owner = attempt_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| io_failed("the attempt root has no UTF-8 name"))?;
    let staging = attempt_root
        .parent()
        .ok_or_else(|| io_failed("the attempt root has no staging parent"))?;
    let destination = attempt_root.join(relative);
    let parent = destination
        .parent()
        .ok_or_else(|| io_failed("the retained document has no parent directory"))?;
    std::fs::create_dir_all(parent)?;
    let temporary = staging.join(format!(
        ".{owner}.{}.{}.tmp",
        relative.replace('/', "."),
        std::process::id()
    ));
    let _ = std::fs::remove_file(&temporary);
    let bytes = serde_json::to_vec_pretty(value)?;
    let result = (|| -> Outcome<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        std::fs::rename(&temporary, &destination)?;
        std::fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

/// Runs one bridge operation through the single shared invoker in
/// `crate::weles_provenance`, which owns the script digest pin, the data-URL execution of
/// the verified bytes, the cleared environment, the process group, the stream bounds and
/// the canonical trust path for every operation this repository runs.
///
/// `output` is the durable destination `submit` requires and `get` refuses; `network`
/// hands the bridge the protected config, and is the only difference between the
/// credentialed task operations and the secretless local verification.
fn run_bridge(
    attempt_root: &Path,
    private: &PrivateBridge,
    label: &str,
    command: &Value,
    output: Option<&Path>,
    network: bool,
) -> Outcome<Vec<u8>> {
    // Re-read and re-validated for every operation: the exact bytes this process
    // accepted are the bytes the child is given, and an unprovisioned or altered trust
    // document stops the attempt here instead of inside the child.
    let trust = weles::CanonicalTrust::load()
        .map_err(|message| WorkerFailure::new("weles_trust_unavailable", message))?;
    weles::run_bridge_command(&weles::BridgeInvocation {
        command,
        trust: &trust,
        working_dir: attempt_root,
        output,
        config: network.then_some(private.config.as_path()),
        timeout: if network {
            weles::NETWORK_BRIDGE_TIMEOUT
        } else {
            weles::VERIFY_BRIDGE_TIMEOUT
        },
    })
    .map_err(|failure| {
        // Only the typed bridge code is surfaced; bridge stderr is never echoed, so no
        // delivered secret can reach the worker report or the Stado job log.
        WorkerFailure::new(
            &format!("weles_bridge_{}", failure.code.replace('-', "_")),
            format!("the official Weles bridge refused the {label} operation"),
        )
    })
}

/// Reads back the durable, request-bound submission the bridge persisted.
fn read_submission(path: &Path) -> Outcome<weles::WelesSubmission> {
    let text = path.to_str().ok_or_else(|| {
        WorkerFailure::new("web_worker_io_failed", "retained document path is not UTF-8")
    })?;
    Ok(crate::read_json(text)?)
}

/// Cancels a task that is still live at Weles through the bridge's `cancel` operation.
///
/// The cancellation key is derived from the retained submission's own idempotency key,
/// itself a pure function of the immutable attempt, so a resubmitted identical attempt
/// cancels exactly the same task exactly once; Weles refuses a second cancellation that
/// carries a different key or reason for the same task.
fn cancel_task(
    attempt_root: &Path,
    private: &PrivateBridge,
    identity: &weles::WelesServiceIdentity,
    identity_value: &Value,
    expected_task: &Value,
    submission: &weles::WelesSubmission,
    reason: &str,
    collected: &mut Collected,
) -> Outcome<weles::WelesCancellation> {
    let idempotency_key = format!(
        "spis-cancel-{}",
        crate::sha256_hex(format!("{}\0cancel", submission.idempotency_key).as_bytes())
    );
    let command = json!({
        "schema": weles::BRIDGE_COMMAND_SCHEMA,
        "operation": "cancel",
        "serviceIdentity": identity_value,
        "taskId": submission.task_id,
        "expectedTask": expected_task,
        "reason": reason,
        "idempotencyKey": idempotency_key,
    });
    let stdout = run_bridge(attempt_root, private, "cancel", &command, None, true)?;
    let cancellation: weles::WelesCancellation = serde_json::from_slice(&stdout)?;
    // Retained before it is judged: a cancellation this worker refuses is still the exact
    // document Weles returned for this attempt.
    retain_attempt_document(
        attempt_root,
        "weles-cancellation.json",
        &serde_json::to_value(&cancellation)?,
    )?;
    collected.cancellation = Some(cancellation.clone());
    ensure(
        cancellation.schema == weles::CANCELLATION_SCHEMA
            && cancellation.task_id == submission.task_id
            && cancellation.organization_id == submission.organization_id
            && cancellation.origin == submission.origin
            && cancellation.action == submission.action,
        "weles_cancellation_invalid",
        "the retained cancellation does not name this exact Weles task",
    )?;
    ensure(
        cancellation.idempotency_key == idempotency_key,
        "weles_cancellation_invalid",
        "the retained cancellation carries a different idempotency key",
    )?;
    ensure(
        cancellation.request_identity == submission.request_identity
            && cancellation.service_identity == *identity,
        "weles_cancellation_invalid",
        "the retained cancellation request/service identity differs from the submission",
    )?;
    Ok(cancellation)
}

/// Mirrors `weles_provenance::validate_service_identity`.
fn service_identity(
    manifest: &super::crawl::RuntimeManifest,
) -> Outcome<weles::WelesServiceIdentity> {
    let service = manifest.service_identity.as_ref().ok_or_else(|| {
        WorkerFailure::new(
            "weles_service_identity_absent",
            "the runtime manifest carries no exact Weles service identity",
        )
    })?;
    let identity = weles::WelesServiceIdentity {
        name: service.name.clone(),
        generation: service.generation,
        consumer: service.consumer.clone(),
        capability: service.capability.clone(),
        active_host: service.active_host.clone(),
        endpoint: service.endpoint.clone(),
        action: service.action.clone(),
        release_id: service.release_id.clone(),
        source_revision: service.source_revision.clone(),
    };
    ensure(
        identity.name == SERVICE_NAME
            && identity.consumer == SERVICE_CONSUMER
            && identity.capability == SERVICE_CAPABILITY
            && identity.action == weles::SPIS_WELES_ACTION
            && is_service_host(&identity.active_host),
        "weles_service_identity_invalid",
        "the runtime manifest service identity is not the exact Weles browser-evidence identity",
    )?;
    ensure(
        is_service_release_id(&identity.release_id),
        "weles_service_identity_invalid",
        "the Weles service release identifier is not a weles-worker@<major>.<minor>.<patch> release",
    )?;
    ensure(
        is_git_revision(&identity.source_revision),
        "weles_service_identity_invalid",
        "the Weles service source revision is not a 40-hex git revision",
    )?;
    validate_api_endpoint(&identity.endpoint)?;
    Ok(identity)
}

/// Independently reads the live release identity from the public version endpoint, so the
/// directory copy in the runtime manifest can never stand in for the running service.
fn confirm_service_release(identity: &weles::WelesServiceIdentity) -> Outcome<()> {
    let url = format!("{}/version", identity.endpoint);
    let mut curl = Command::new("curl");
    curl.args([
        "--fail",
        "--silent",
        "--show-error",
        "--location-trusted",
        "--max-redirs",
        "0",
        "--max-time",
        "20",
        "-H",
        "Accept: application/json",
    ])
    .arg(&url);
    let output = super::crawl::bounded_command_output(
        &mut curl,
        "read the Weles service release identity",
        Duration::from_secs(30),
        256 * 1024,
    )?;
    if !output.status.success() {
        return Err(WorkerFailure::new(
            "weles_service_release_unavailable",
            format!(
                "the Weles version endpoint refused the release readback: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    let document: Value = serde_json::from_slice(&output.stdout).map_err(|_| {
        WorkerFailure::new(
            "weles_service_release_mismatch",
            "the Weles version endpoint did not return a JSON document",
        )
    })?;
    let release_id = alternate(&document, "release_id", "releaseId").ok_or_else(|| {
        WorkerFailure::new(
            "weles_service_release_mismatch",
            "the Weles version endpoint declares no release_id",
        )
    })?;
    let source_revision =
        alternate(&document, "source_revision", "sourceRevision").ok_or_else(|| {
            WorkerFailure::new(
                "weles_service_release_mismatch",
                "the Weles version endpoint declares no source_revision",
            )
        })?;
    ensure(
        release_id == identity.release_id && source_revision == identity.source_revision,
        "weles_service_release_mismatch",
        "the live Weles release differs from the runtime manifest service directory",
    )
}

fn alternate<'a>(document: &'a Value, primary: &str, secondary: &str) -> Option<&'a str> {
    document
        .get(primary)
        .or_else(|| document.get(secondary))
        .and_then(Value::as_str)
}

/// Mirrors `weles_provenance::validate_spis_binding` before the binding is ever signed.
fn attempt_binding(
    manifest: &super::crawl::RuntimeManifest,
    identity: &weles::WelesServiceIdentity,
) -> Outcome<weles::WelesAttemptBinding> {
    let binding = weles::WelesAttemptBinding {
        schema: weles::ATTEMPT_BINDING_SCHEMA.to_string(),
        run_id: manifest.run_id.clone(),
        catalog: manifest.catalog.clone(),
        record: manifest.record.clone(),
        record_key: manifest.record_key.clone(),
        attempt: manifest.attempt,
        attempt_id: manifest.attempt_id.clone(),
        source_revision: manifest.source_revision.clone(),
        source_input_sha256: manifest.source_input_sha256.clone(),
        reference_sha256: manifest.reference_sha256.clone(),
        artifact_uri: manifest.artifact_uri.clone(),
        output_uri: manifest.output_uri.clone(),
        service: weles::WelesAttemptBindingService {
            name: identity.name.clone(),
            consumer: identity.consumer.clone(),
            capability: identity.capability.clone(),
            directory_generation: identity.generation,
            host: identity.active_host.clone(),
            endpoint: identity.endpoint.clone(),
            action: identity.action.clone(),
            release_id: identity.release_id.clone(),
            source_revision: identity.source_revision.clone(),
        },
    };
    for component in [
        binding.run_id.as_str(),
        binding.catalog.as_str(),
        binding.record.as_str(),
        binding.attempt_id.as_str(),
    ] {
        ensure(
            is_portable_component(component),
            "web_attempt_component_invalid",
            "an attempt coordinate is not a portable attempt URI component",
        )?;
    }
    ensure(
        binding.attempt >= 1,
        "web_attempt_component_invalid",
        "the attempt number must be at least one",
    )?;
    ensure(
        is_sha256(&binding.record_key)
            && is_sha256(&binding.source_input_sha256)
            && is_sha256(&binding.reference_sha256),
        "web_attempt_component_invalid",
        "record key and source/reference digests must be lowercase 64-hex SHA-256",
    )?;
    ensure(
        is_git_revision(&binding.source_revision),
        "web_attempt_component_invalid",
        "the attempt source revision is not a 40-hex git revision",
    )?;
    let base = attempt_base(&binding);
    ensure(
        binding.artifact_uri == format!("{base}/artifacts.tar.gz")
            && binding.output_uri == format!("{base}/worker-output.log"),
        "web_attempt_uri_invalid",
        "the signed Spis artifact/output URIs are not the canonical attempt coordinates",
    )?;
    Ok(binding)
}

/// The exact typed browser-evidence constraint array Weles admits.
///
/// `parseTaskRequest` in the service refuses any `input.constraints` whose canonical JSON
/// differs from `SPIS_BROWSER_EVIDENCE_POLICY.constraints`, and the browser worker
/// enforces exactly that policy while it captures. The order below is the service's own
/// order and is submitted verbatim.
const BROWSER_EVIDENCE_CONSTRAINTS: &[&str] = &[
    "browser-permission-apis:withhold",
    "notification-apis:withhold",
    "permission-notification-controls:withhold",
    "system-ui-downloads:withhold",
    "authentication-signup-recovery:withhold",
    "mfa-trusted-device:withhold",
    "message-submission:withhold",
    "commerce-payment:withhold",
    "destructive-confirmation:withhold",
    "network:exact-public-origin-pinned",
    "interactive-controls:default-deny",
];

/// The submitted constraint list.
///
/// Weles enforces one immutable withholding policy for every browser-evidence task and
/// admits only that exact array, so this worker never negotiates its own weaker
/// vocabulary: it refuses to submit unless the immutable runtime manifest asks for
/// exactly the withholding the policy performs, and then submits the policy verbatim.
/// The origin is not restated here; it is pinned by `network:exact-public-origin-pinned`
/// against the request `origin`, which the service requires to equal the exact product
/// URL origin and which is signed into the receipt as a core claim.
/// `validate_request_and_evidence_manifest` runs `validate_unique_nonempty` over exactly
/// this vector.
fn task_constraints(constraints: &super::crawl::RuntimeConstraints) -> Outcome<Vec<String>> {
    ensure(
        constraints.no_first_run_consent
            && constraints.no_system_permission_prompts
            && constraints.no_notifications
            && constraints.no_purchase
            && constraints.no_final_destructive_action
            && constraints.headless,
        "web_constraints_invalid",
        "the immutable runtime constraints do not request the exact Weles browser-evidence withholding policy",
    )?;
    Ok(BROWSER_EVIDENCE_CONSTRAINTS
        .iter()
        .map(|value| (*value).to_string())
        .collect())
}

fn text<'a>(document: &'a Value, key: &str) -> Outcome<&'a str> {
    document
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            WorkerFailure::new(
                "weles_evidence_manifest_invalid",
                format!("the signed evidence manifest has no string {key}"),
            )
        })
}

fn storage_get(uri: &str, destination: &Path) -> Outcome<()> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut stado = super::crawl::stado_command();
    stado.args(["storage", "get", uri]).arg(destination);
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "download retained Weles evidence",
        Duration::from_secs(300),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        return Err(WorkerFailure::new(
            "weles_evidence_download_failed",
            format!(
                "stado storage get refused {uri}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(())
}

fn worker_report(
    manifest: &super::crawl::RuntimeManifest,
    state: &str,
    artifact: Option<Value>,
    collected: &Collected,
    failure: Option<&WorkerFailure>,
) -> Value {
    json!({
        "schema": REPORT_SCHEMA,
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "record_key": manifest.record_key,
        "attempt": u64::from(manifest.attempt),
        "attempt_id": manifest.attempt_id,
        "engine": "web",
        "state": state,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "reference_sha256": manifest.reference_sha256,
        "bindings_file_sha256": manifest.bindings_file_sha256,
        "bindings_sha256": manifest.bindings_sha256,
        "execution_identity": serde_json::to_value(&manifest.execution_identity)
            .unwrap_or(Value::Null),
        "artifact": artifact.unwrap_or(Value::Null),
        "weles_attempt_envelope": serde_json::to_value(&collected.envelope)
            .unwrap_or(Value::Null),
        "weles_submission": serde_json::to_value(&collected.submission).unwrap_or(Value::Null),
        "weles_task_status": serde_json::to_value(&collected.status).unwrap_or(Value::Null),
        "weles_cancellation": serde_json::to_value(&collected.cancellation)
            .unwrap_or(Value::Null),
        "provenance_document": serde_json::to_value(&collected.provenance)
            .unwrap_or(Value::Null),
        "failure": failure
            .map(|failure| json!({"code": failure.code, "message": failure.message}))
            .unwrap_or(Value::Null),
    })
}

fn run_worker(
    catalog: &str,
    record: &str,
    manifest: &super::crawl::RuntimeManifest,
    wait_seconds: u64,
) -> Result<()> {
    if catalog != manifest.catalog || record != manifest.record {
        bail!("worker catalog/record differ from the immutable runtime manifest");
    }
    let base = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".spis")
        .join("crawls");
    let attempt_root = super::crawl::native_attempt_root(&base, manifest)?;
    std::fs::create_dir_all(&attempt_root)?;
    let private = PrivateBridge::open(manifest)?;
    let mut collected = Collected::default();
    let outcome = capture(
        manifest,
        wait_seconds,
        &attempt_root,
        &private,
        &mut collected,
    );
    private.discard();
    let failure = match outcome {
        // A publication failure is itself a typed attempt failure, so the single mandatory
        // report line is emitted on every path.
        Ok(()) => {
            match super::crawl::publish_attempt_archive(&attempt_root, &manifest.artifact_uri) {
                Ok(artifact) => {
                    let report = worker_report(
                        manifest,
                        "artifact_published",
                        Some(artifact),
                        &collected,
                        None,
                    );
                    println!("{}", serde_json::to_string(&report)?);
                    return Ok(());
                }
                Err(error) => WorkerFailure::new(
                    "attempt_archive_publication_failed",
                    format!("{error:#}"),
                ),
            }
        }
        Err(failure) => failure,
    };
    let document = json!({
        "schema": FAILURE_SCHEMA,
        "code": failure.code,
        "message": failure.message,
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "attempt": u64::from(manifest.attempt),
        "attempt_id": manifest.attempt_id,
    });
    if let Err(retention) = retain_attempt_document(&attempt_root, "failure.json", &document) {
        eprintln!(
            "web worker failure artifact could not be retained ({}): {}",
            retention.code, retention.message
        );
    }
    let artifact = super::crawl::publish_attempt_archive(&attempt_root, &manifest.artifact_uri);
    let report = worker_report(
        manifest,
        "failed",
        artifact.as_ref().ok().cloned(),
        &collected,
        Some(&failure),
    );
    println!("{}", serde_json::to_string(&report)?);
    match artifact {
        Ok(_) => bail!("web worker failed ({}): {}", failure.code, failure.message),
        Err(error) => bail!(
            "web worker failed ({}): {}; the attempt archive could not be published either: {error:#}",
            failure.code,
            failure.message
        ),
    }
}

fn capture(
    manifest: &super::crawl::RuntimeManifest,
    wait_seconds: u64,
    attempt_root: &Path,
    private: &PrivateBridge,
    collected: &mut Collected,
) -> Outcome<()> {
    let catalog = manifest.catalog.as_str();
    let record = manifest.record.as_str();
    let identity = service_identity(manifest)?;
    confirm_service_release(&identity)?;
    let binding = attempt_binding(manifest, &identity)?;
    let base = attempt_base(&binding);

    // `validate_request_and_evidence_manifest` binds every retained URL to the exact
    // product URL of the current record, so the manifest URL must already be canonical.
    let product_url = manifest.runtime_product.product_url.clone();
    let parsed = url::Url::parse(&product_url).map_err(|_| {
        WorkerFailure::new(
            "web_product_url_invalid",
            "the runtime product URL is not a URL",
        )
    })?;
    ensure(
        matches!(parsed.scheme(), "http" | "https")
            && parsed.username().is_empty()
            && parsed.password().is_none(),
        "web_product_url_invalid",
        "the runtime product URL must be HTTP(S) without credentials",
    )?;
    ensure(
        parsed.as_str() == product_url,
        "web_product_url_invalid",
        "the runtime product URL is not in canonical serialized form",
    )?;
    let origin = parsed.origin().ascii_serialization();
    ensure(
        !origin.is_empty() && origin != "null",
        "web_product_url_invalid",
        "the runtime product URL has an opaque origin",
    )?;
    if let Some(surface) = manifest.runtime_product.surface.as_ref() {
        ensure(
            surface.exact_url == product_url && surface.origin == origin,
            "web_surface_identity_mismatch",
            "the runtime surface identity does not name the exact product URL and origin",
        )?;
        ensure(
            surface
                .allowed_actions
                .iter()
                .any(|action| action == weles::SPIS_WELES_ACTION),
            "web_surface_identity_mismatch",
            "the runtime surface identity does not allow the Spis browser action",
        )?;
    }

    let reference_path = format!("{catalog}/references/{record}/reference.json");
    let reference: Value = crate::read_json(&reference_path)?;
    // The verifier re-derives the receipt origin from the record's own product_url, so an
    // attempt whose manifest URL differs from the committed record can never verify.
    let reference_url = reference
        .get("product_url")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            WorkerFailure::new(
                "web_reference_product_url_absent",
                "the committed record declares no product_url",
            )
        })?;
    ensure(
        url::Url::parse(reference_url).ok().as_ref() == Some(&parsed),
        "web_reference_product_url_mismatch",
        "the committed record product_url differs from the runtime manifest product URL",
    )?;
    let name = reference.get("name").and_then(Value::as_str).unwrap_or_default();
    let goal = reference
        .pointer("/journey/goal")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let objective = objective(catalog, name, goal)?;
    let constraints = task_constraints(&manifest.constraints)?;

    ensure(
        manifest.account.mode == "anonymous-read-only-probe"
            && manifest.account.credential_refs.is_empty(),
        "weles_anonymous_probe_required",
        "web crawls submit an anonymous read-only probe with no credential references",
    )?;

    // A pure function of the immutable attempt: a resubmitted identical attempt is exactly
    // idempotent at Weles, and never creates a second task.
    let idempotency_key = format!(
        "spis-{}",
        crate::sha256_hex(
            format!(
                "{}\0{}\0{}\0{}\0{}\0{}\0{}",
                manifest.source_revision,
                manifest.run_id,
                manifest.catalog,
                manifest.record,
                manifest.record_key,
                manifest.attempt,
                manifest.attempt_id,
            )
            .as_bytes()
        )
    );

    let organization_id = std::env::var("WISENT_ORGANIZATION_ID").map_err(|_| {
        WorkerFailure::new(
            "weles_delivery_env_absent",
            "WISENT_ORGANIZATION_ID was not delivered to this worker",
        )
    })?;
    let bearer = std::env::var("WELES_TOKEN").map_err(|_| {
        WorkerFailure::new(
            "weles_delivery_env_absent",
            "WELES_TOKEN was not delivered to this worker",
        )
    })?;
    ensure(
        !organization_id.trim().is_empty() && !bearer.trim().is_empty(),
        "weles_delivery_env_absent",
        "the delivered Weles organization and bearer must both be nonempty",
    )?;
    private.write_config(&identity.endpoint, &bearer, &organization_id)?;

    let identity_value = serde_json::to_value(&identity)?;
    let input = weles::WelesOfficialTaskInput {
        product_url: product_url.clone(),
        objective: objective.clone(),
        constraints: constraints.clone(),
        spis_binding: binding.clone(),
    };
    let justification = format!(
        "Spis anonymous browser-evidence capture for {catalog}/{record} attempt {} ({})",
        manifest.attempt, manifest.attempt_id
    );
    // The bridge normalizes `request` to exactly these six fields and injects `schema` and
    // `organizationId` itself before it computes the canonical request digest.
    let submit_command = json!({
        "schema": weles::BRIDGE_COMMAND_SCHEMA,
        "operation": "submit",
        "serviceIdentity": identity_value,
        "request": {
            "origin": origin,
            "action": weles::SPIS_WELES_ACTION,
            "input": serde_json::to_value(&input)?,
            "credentialRefs": [],
            "evidencePolicy": "full",
            "justification": justification,
        },
        "idempotencyKey": idempotency_key,
    });
    // The bridge persists the request-bound submission itself, so its destination is the
    // attempt root rather than the merged `weles/` subtree for the same reason
    // `retain_attempt_document` explains: this name is fixed and its bytes are unique to
    // this attempt. `weles/` is created by the content-addressed writes below.
    let submission_path = attempt_root.join("weles-submission.json");
    run_bridge(
        attempt_root,
        private,
        "submit",
        &submit_command,
        Some(&submission_path),
        true,
    )?;
    let submission = read_submission(&submission_path)?;
    collected.submission = Some(submission.clone());
    ensure(
        submission.schema == weles::SUBMISSION_SCHEMA,
        "weles_submission_invalid",
        "the retained submission does not declare the typed submission schema",
    )?;
    ensure(
        submission.organization_id == organization_id
            && submission.origin == origin
            && submission.action == weles::SPIS_WELES_ACTION,
        "weles_submission_invalid",
        "the retained submission task identity differs from the submitted request",
    )?;
    ensure(
        submission.idempotency_key == idempotency_key,
        "weles_submission_invalid",
        "the retained submission carries a different idempotency key",
    )?;
    ensure(
        submission.service_identity == identity,
        "weles_submission_invalid",
        "the retained submission service identity differs from the runtime directory",
    )?;
    ensure(
        is_sha256_id(&submission.request_digest)
            && submission.request_identity.request_digest == submission.request_digest,
        "weles_submission_invalid",
        "the retained submission request digest is not a bound sha256: identifier",
    )?;
    ensure(
        submission.request_identity.spis_binding == binding
            && submission.request_document.input.spis_binding == binding,
        "weles_submission_invalid",
        "the retained submission does not carry the exact signed Spis binding",
    )?;
    ensure(
        submission.request_document.schema == OFFICIAL_REQUEST_SCHEMA
            && submission.request_document.organization_id == organization_id
            && submission.request_document.origin == origin
            && submission.request_document.action == weles::SPIS_WELES_ACTION,
        "weles_submission_invalid",
        "the retained official request is not the canonical current Weles task",
    )?;
    ensure(
        submission.request_document.credential_refs.is_empty()
            && submission.request_document.evidence_policy == "full",
        "weles_submission_invalid",
        "the retained official request is not an anonymous full-evidence request",
    )?;
    ensure(
        submission.request_document.input.product_url == product_url
            && submission.request_document.input.objective == objective
            && submission.request_document.input.constraints == constraints,
        "weles_submission_invalid",
        "the retained official request input differs from the submitted browser task",
    )?;
    ensure(
        is_portable_component(&submission.task_id),
        "weles_task_id_invalid",
        "the Weles task identifier is not a portable recording component",
    )?;
    let weles_task_id = submission.task_id.clone();

    let expected_task = json!({
        "taskId": weles_task_id,
        "organizationId": organization_id,
        "origin": origin,
        "action": weles::SPIS_WELES_ACTION,
    });
    let get_command = json!({
        "schema": weles::BRIDGE_COMMAND_SCHEMA,
        "operation": "get",
        "serviceIdentity": identity_value,
        "taskId": weles_task_id,
        "expectedTask": expected_task,
    });
    let deadline = Instant::now() + Duration::from_secs(wait_seconds);
    let status = loop {
        let stdout = run_bridge(attempt_root, private, "get", &get_command, None, true)?;
        let observed: weles::WelesTaskStatus = serde_json::from_slice(&stdout)?;
        if observed.terminal || Instant::now() >= deadline {
            break observed;
        }
        std::thread::sleep(POLL_INTERVAL);
    };
    retain_attempt_document(
        attempt_root,
        "weles-status.json",
        &serde_json::to_value(&status)?,
    )?;
    collected.status = Some(status.clone());
    ensure(
        status.schema == weles::TASK_STATUS_SCHEMA,
        "weles_status_invalid",
        "the retained task status does not declare the typed status schema",
    )?;
    ensure(
        status.task_id == weles_task_id,
        "weles_status_invalid",
        "the retained task status names a different Weles task",
    )?;
    ensure(
        status.request_identity == submission.request_identity,
        "weles_status_invalid",
        "the retained task status request identity differs from the submission",
    )?;
    ensure(
        status.service_identity == identity,
        "weles_status_invalid",
        "the retained task status service identity differs from the runtime directory",
    )?;
    if !status.terminal {
        // The wait budget is spent while the task is still live at Weles. Leaving it
        // running would hold a browser session and a leased worker for an attempt that
        // can no longer publish evidence, so the same bridge that submitted the task
        // cancels it and the typed cancellation is retained with the attempt.
        let cancellation = cancel_task(
            attempt_root,
            private,
            &identity,
            &identity_value,
            &expected_task,
            &submission,
            // A pure function of the immutable attempt, exactly like the key: Weles
            // refuses a second cancellation of the same task under a different reason,
            // so the wait budget is reported in the failure below rather than signed
            // into the cancellation.
            &format!(
                "Spis browser-evidence attempt {} ({}) exhausted its wait budget",
                manifest.attempt, manifest.attempt_id
            ),
            collected,
        )?;
        return Err(WorkerFailure::new(
            "weles_task_not_terminal",
            format!(
                "Weles task {weles_task_id} was still {} after {wait_seconds}s and was cancelled through the official bridge (cancel status {})",
                status.status, cancellation.status
            ),
        ));
    }
    if status.outcome.as_deref() != Some("completed") {
        return Err(WorkerFailure::new(
            "weles_task_not_completed",
            format!(
                "Weles task {weles_task_id} finished as status={} outcome={}",
                status.status,
                status.outcome.as_deref().unwrap_or("none")
            ),
        ));
    }
    let checkpoint = status.receipt_checkpoint.as_ref().ok_or_else(|| {
        WorkerFailure::new(
            "weles_receipt_checkpoint_absent",
            "a completed Weles task must carry a freshly verified receipt checkpoint",
        )
    })?;
    let result_digest = status
        .result_digest
        .clone()
        .filter(|digest| is_sha256_id(digest))
        .ok_or_else(|| {
            WorkerFailure::new(
                "weles_status_invalid",
                "the completed task carries no sha256: result digest",
            )
        })?;
    // The public task-status contract returns the task identity, the request identity,
    // the outcome, the result digest and the receipt, and never a result or artifact
    // reference; the bridge therefore normalizes both to `null`/`[]`. Retained evidence is
    // addressed by this task's own recording prefix below, so nothing here may demand a
    // reference the service never signs — but any reference that IS reported has to
    // belong to exactly this recording.
    let prefix = format!("stado://weles/recordings/{weles_task_id}/");
    ensure(
        status
            .result_ref
            .iter()
            .chain(status.artifact_refs.iter())
            .all(|reference| reference.starts_with(&prefix)),
        "weles_artifact_refs_foreign",
        "a task result or artifact reference is not bound to this exact Weles recording",
    )?;

    let claims = &checkpoint.claims;
    ensure(
        claims.task_id == weles_task_id
            && claims.organization_id == organization_id
            && claims.origin == origin
            && claims.action == weles::SPIS_WELES_ACTION
            && claims.outcome == "completed",
        "weles_receipt_claims_mismatch",
        "the verified receipt claims do not name this completed task",
    )?;
    ensure(
        claims.request_digest == submission.request_digest
            && claims.result_digest == result_digest
            && claims.spis_binding == binding,
        "weles_receipt_claims_mismatch",
        "the verified receipt does not sign this exact request, result and Spis binding",
    )?;

    let stado_job_id = ["STADO_JOB_ID", "STADO_MACHINE_JOB_ID", "STADO_JOB"]
        .iter()
        .find_map(|name| {
            std::env::var(name)
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .ok_or_else(|| {
            WorkerFailure::new(
                "stado_job_id_unavailable",
                "the Stado job identifier was not delivered to this worker",
            )
        })?;
    // `verify_attempt_binding` refuses an envelope whose outer job equals the inner task.
    ensure(
        stado_job_id != weles_task_id,
        "stado_job_id_unavailable",
        "the Stado job identifier collides with the inner Weles task identifier",
    )?;

    let recordings = attempt_root.join("recordings").join(&weles_task_id);
    let evidence_manifest_uri = format!("{prefix}evidence-manifest.json");
    let evidence_manifest_path = recordings.join("evidence-manifest.json");
    storage_get(&evidence_manifest_uri, &evidence_manifest_path)?;
    // These exact bytes are the receipt-bound artifact. Re-serializing would change the
    // digest the receipt signed, so they are only ever copied.
    let manifest_bytes = std::fs::read(&evidence_manifest_path)?;
    let artifact_document_sha256 = crate::sha256_hex(&manifest_bytes);
    let artifact_relative = format!("weles/artifacts/{artifact_document_sha256}.json");
    write_exact(&attempt_root.join(&artifact_relative), &manifest_bytes)?;
    ensure(
        claims.evidence_digest == artifact_document_sha256,
        "weles_evidence_digest_mismatch",
        "the verified receipt evidenceDigest is not the retained evidence manifest digest",
    )?;

    let evidence: Value = serde_json::from_slice(&manifest_bytes)?;
    ensure(
        text(&evidence, "schema")? == EVIDENCE_MANIFEST_SCHEMA,
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest schema is unsupported",
    )?;
    ensure(
        text(&evidence, "taskId")? == weles_task_id
            && text(&evidence, "organizationId")? == claims.organization_id
            && text(&evidence, "origin")? == claims.origin
            && text(&evidence, "action")? == claims.action
            && text(&evidence, "outcome")? == "completed",
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest does not name this completed task",
    )?;
    ensure(
        text(&evidence, "requestDigest")? == submission.request_digest
            && text(&evidence, "resultDigest")? == result_digest,
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest request/result digests differ from the receipt",
    )?;
    let signed_binding: weles::WelesAttemptBinding =
        serde_json::from_value(evidence.get("spisBinding").cloned().unwrap_or(Value::Null))?;
    ensure(
        signed_binding == binding,
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest carries a different Spis binding",
    )?;
    let requested_url = text(&evidence, "requestedUrl")?;
    ensure(
        requested_url == product_url,
        "weles_evidence_manifest_invalid",
        "the signed evidence manifest requestedUrl is not the exact product URL",
    )?;
    let effective_url = text(&evidence, "effectiveUrl")?.to_string();
    let final_url = text(&evidence, "finalUrl")?.to_string();
    ensure(
        same_origin(&effective_url, &parsed) && same_origin(&final_url, &parsed),
        "weles_evidence_manifest_invalid",
        "the signed effective/final URLs are not same-origin with the product URL",
    )?;

    let entries = evidence
        .get("evidenceInventory")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            WorkerFailure::new(
                "weles_evidence_manifest_invalid",
                "the signed evidence manifest has no evidenceInventory array",
            )
        })?;
    let mut inventory: Vec<weles::WelesEvidenceInventoryEntry> = Vec::with_capacity(entries.len());
    for entry in entries {
        inventory.push(serde_json::from_value(entry.clone())?);
    }
    let screenshot_uri = format!("{prefix}artifacts/browser_evidence_final.png");
    let accessibility_uri = format!("{prefix}artifacts/browser_evidence_accessibility_tree.txt");
    let mut retained_paths: Vec<String> =
        vec![format!("recordings/{weles_task_id}/evidence-manifest.json")];
    // The uniqueness sets borrow the signed inventory, so they are scoped: the validated
    // inventory then moves into the attempt envelope without being cloned.
    {
        let mut kinds: BTreeSet<&str> = BTreeSet::new();
        let mut uris: BTreeSet<&str> = BTreeSet::new();
        let mut total_bytes = 0_u64;
        for entry in &inventory {
            let relative = entry.uri.strip_prefix(&prefix).ok_or_else(|| {
                WorkerFailure::new(
                    "weles_evidence_uri_foreign",
                    "an evidence inventory URI is not bound to this exact Weles task",
                )
            })?;
            ensure(
                is_portable_relative(relative),
                "weles_evidence_uri_invalid",
                "an evidence inventory URI is not a canonical immutable path",
            )?;
            let kind_matches_uri = match entry.kind.as_str() {
                SCREENSHOT_KIND => entry.uri == screenshot_uri,
                ACCESSIBILITY_KIND => entry.uri == accessibility_uri,
                kind => kind
                    .strip_prefix("artifact:")
                    .is_some_and(|tail| tail == relative),
            };
            ensure(
                kind_matches_uri && is_sha256(&entry.sha256) && entry.bytes > 0,
                "weles_evidence_entry_invalid",
                "an evidence inventory entry kind, digest or length is not canonical",
            )?;
            ensure(
                kinds.insert(entry.kind.as_str()) && uris.insert(entry.uri.as_str()),
                "weles_evidence_entry_duplicate",
                "the evidence inventory repeats a kind or URI",
            )?;
            total_bytes = total_bytes
                .checked_add(entry.bytes)
                .filter(|total| *total <= MAXIMUM_EVIDENCE_BYTES)
                .ok_or_else(|| {
                    WorkerFailure::new(
                        "weles_evidence_too_large",
                        "the retained evidence inventory exceeds the total byte limit",
                    )
                })?;
            let destination = recordings.join(relative);
            storage_get(&entry.uri, &destination)?;
            // The signed digest is never trusted on its own: the retained bytes are
            // re-hashed exactly the way `validate_evidence_inventory` re-hashes them.
            let bytes = std::fs::read(&destination)?;
            ensure(
                bytes.len() as u64 == entry.bytes && crate::sha256_hex(&bytes) == entry.sha256,
                "weles_evidence_bytes_differ",
                "retained evidence bytes differ from the signed inventory entry",
            )?;
            if entry.kind == SCREENSHOT_KIND {
                ensure(
                    bytes.starts_with(PNG_MAGIC),
                    "weles_evidence_screenshot_invalid",
                    "the retained final screenshot is not a PNG",
                )?;
            } else if entry.kind == ACCESSIBILITY_KIND {
                let tree = String::from_utf8(bytes).map_err(|_| {
                    WorkerFailure::new(
                        "weles_evidence_accessibility_invalid",
                        "the retained accessibility tree is not valid UTF-8",
                    )
                })?;
                ensure(
                    !tree.trim().is_empty(),
                    "weles_evidence_accessibility_invalid",
                    "the retained accessibility tree is empty",
                )?;
            }
            retained_paths.push(format!("recordings/{weles_task_id}/{relative}"));
        }
        ensure(
            kinds.contains(SCREENSHOT_KIND) && kinds.contains(ACCESSIBILITY_KIND),
            "weles_evidence_incomplete",
            "the evidence inventory lacks the required screenshot/accessibility_tree artifacts",
        )?;
    }
    retained_paths.sort();

    let mut observation: BTreeMap<&str, Value> = BTreeMap::new();
    observation.insert("schema", json!(OBSERVATION_SCHEMA));
    observation.insert("run_id", json!(manifest.run_id));
    observation.insert("catalog", json!(manifest.catalog));
    observation.insert("record", json!(manifest.record));
    observation.insert("record_key", json!(manifest.record_key));
    observation.insert("attempt", json!(u64::from(manifest.attempt)));
    observation.insert("attempt_id", json!(manifest.attempt_id));
    observation.insert("weles_task_id", json!(weles_task_id));
    observation.insert("requested_url", json!(product_url));
    observation.insert("effective_url", json!(effective_url));
    observation.insert("final_url", json!(final_url));
    observation.insert("evidence_inventory", serde_json::to_value(&inventory)?);
    observation.insert("retained_paths", json!(retained_paths));
    // A BTreeMap serializes in sorted key order regardless of the serde_json feature set,
    // so these bytes and their digest are stable.
    let observation_bytes = serde_json::to_vec(&observation)?;
    let observation_document_sha256 = crate::sha256_hex(&observation_bytes);
    write_exact(
        &attempt_root.join(format!(
            "weles/observations/{observation_document_sha256}.json"
        )),
        &observation_bytes,
    )?;

    let expected_claims = weles::ExpectedReceiptClaims {
        task_id: weles_task_id.clone(),
        organization_id: organization_id.clone(),
        request_digest: submission.request_digest.clone(),
        result_digest: result_digest.clone(),
        spis_binding: binding.clone(),
        origin: origin.clone(),
        action: weles::SPIS_WELES_ACTION.to_string(),
        outcome: "completed".to_string(),
        evidence_digest: artifact_document_sha256.clone(),
    };
    let artifact = weles::RetainedArtifact {
        path: artifact_relative,
        sha256: artifact_document_sha256.clone(),
        bytes: manifest_bytes.len() as u64,
    };
    // The bridge resolves `artifact.path` against its working directory, so the official
    // client re-reads and re-digests exactly the retained bytes.
    let verify_command = json!({
        "schema": weles::BRIDGE_COMMAND_SCHEMA,
        "operation": "verify",
        "receipt": serde_json::to_value(&checkpoint.receipt)?,
        "expectedClaims": serde_json::to_value(&expected_claims)?,
        "artifact": serde_json::to_value(&artifact)?,
    });
    let stdout = run_bridge(attempt_root, private, "verify", &verify_command, None, false)?;
    let fresh: weles::WelesProvenanceDocument = serde_json::from_slice(&stdout)?;
    ensure(
        fresh
            .id
            .strip_prefix("sha256:")
            .is_some_and(is_sha256),
        "weles_provenance_id_invalid",
        "the bridge verification document has no framed sha256: identifier",
    )?;
    let provenance = weles::WelesProvenanceDocument {
        schema: weles::PROVENANCE_DOCUMENT_SCHEMA.to_string(),
        // The framed provenance id is derived by the official bridge from the receipt, the
        // trusted key-set version and the artifact; Rust only re-checks its shape above.
        id: fresh.id.clone(),
        client: checkpoint.client.clone(),
        receipt: checkpoint.receipt.clone(),
        claims: checkpoint.claims.clone(),
        expected_claims,
        artifact,
    };
    ensure(
        fresh == provenance,
        "weles_provenance_mismatch",
        "the fresh official verification differs from the assembled provenance document",
    )?;
    retain_attempt_document(
        attempt_root,
        "weles-provenance.json",
        &serde_json::to_value(&provenance)?,
    )?;
    collected.provenance = Some(provenance);

    let envelope = weles::WelesAttemptEnvelope {
        schema: weles::ATTEMPT_ENVELOPE_SCHEMA.to_string(),
        run_id: manifest.run_id.clone(),
        catalog: manifest.catalog.clone(),
        record: manifest.record.clone(),
        record_key: manifest.record_key.clone(),
        attempt: manifest.attempt,
        attempt_id: manifest.attempt_id.clone(),
        stado_job_id,
        weles_task_id,
        state: "completed".to_string(),
        outcome: Some("completed".to_string()),
        service_identity: identity,
        source_revision: manifest.source_revision.clone(),
        source_input_sha256: manifest.source_input_sha256.clone(),
        reference_sha256: manifest.reference_sha256.clone(),
        spis_binding: binding,
        weles_request_document: submission.request_document.clone(),
        weles_request_digest: submission.request_digest.clone(),
        weles_result_digest: Some(result_digest),
        requested_url: product_url,
        final_url,
        evidence_inventory: inventory,
        weles_evidence_manifest_uri: evidence_manifest_uri,
        weles_evidence_manifest_sha256: Some(artifact_document_sha256.clone()),
        artifact_document_uri: format!("{base}/weles/artifacts/{artifact_document_sha256}.json"),
        artifact_document_sha256: Some(artifact_document_sha256),
        observation_document_uri: format!(
            "{base}/weles/observations/{observation_document_sha256}.json"
        ),
        observation_document_sha256,
    };
    retain_attempt_document(
        attempt_root,
        "attempt-envelope.json",
        &serde_json::to_value(&envelope)?,
    )?;
    collected.envelope = Some(envelope);
    Ok(())
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut host: Option<String> = None;
    let mut record: Option<String> = None;
    let mut runtime_manifest_base64: Option<String> = None;
    let mut artifact_uri: Option<String> = None;
    let mut worker = false;
    let mut wait_seconds = 7_200u64;
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
            "--runtime-manifest-base64" => {
                i += 1;
                runtime_manifest_base64 = Some(
                    rest.get(i)
                        .context("--runtime-manifest-base64 needs a value")?
                        .clone(),
                );
            }
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            "--wait-seconds" => {
                i += 1;
                wait_seconds = rest
                    .get(i)
                    .context("--wait-seconds needs a value")?
                    .parse()
                    .context("--wait-seconds must be a whole number of seconds")?;
            }
            "--worker" => worker = true,
            "--help" | "-h" => {
                println!("usage: spis crawl-web <catalog> --host TARGET --record SLUG --runtime-manifest-base64 DATA [--wait-seconds N]\nworker mode: spis crawl-web <catalog> --worker --record SLUG --artifact-uri URI --runtime-manifest-base64 DATA [--wait-seconds N]");
                return Ok(());
            }
            value if value.starts_with('-') => bail!("unknown argument: {value}"),
            value if catalog.is_none() => catalog = Some(value.to_string()),
            value => bail!("unexpected argument: {value}"),
        }
        i += 1;
    }
    let catalog = catalog.context("catalog is required")?;
    if !CATALOGS.contains(&catalog.as_str()) {
        bail!("crawl-web accepts {}", CATALOGS.join(", "));
    }
    if !(30..=86_400).contains(&wait_seconds) {
        bail!("--wait-seconds must be 30..86400");
    }
    let record = record.context("--record is required for one exact per-record job")?;
    let manifest = super::crawl::decode_runtime_manifest(
        runtime_manifest_base64
            .as_deref()
            .context("--runtime-manifest-base64 is required")?,
        &catalog,
        "web",
        Some(&record),
    )?;
    if !worker {
        if artifact_uri.is_some() {
            bail!("--artifact-uri is worker-only");
        }
        let host = host
            .context("--host is required; web crawls execute as pinned Stado jobs")?;
        return submit_worker(&host, &catalog, &record, &manifest, wait_seconds);
    }
    if host.is_some() {
        bail!("--host is coordinator-only");
    }
    let artifact_uri = artifact_uri.context("--artifact-uri is required in worker mode")?;
    if artifact_uri != manifest.artifact_uri {
        bail!("worker artifact URI does not match immutable runtime manifest");
    }
    run_worker(&catalog, &record, &manifest, wait_seconds)
}
