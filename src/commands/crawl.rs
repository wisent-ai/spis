//! Durable coordinator for every Spis crawler.
//!
//! The six surface-specific commands remain the execution engines. This command
//! is the single operator and desktop contract for planning, submission, status,
//! resumption, artifact retrieval and idempotent record import.

use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const OP_SCHEMA: &str = "wisent.crawl-operation.v1";
const RUN_SCHEMA: &str = "wisent.crawl-run.v1";
const SUBMISSION_SCHEMA: &str = "wisent.crawl-submission.v1";

const CATALOGS: &[(&str, &str)] = &[
    ("ios-app-examples", "mobile"),
    ("android-app-examples", "mobile"),
    ("macos-app-examples", "desktop"),
    ("desktop-app-examples", "desktop"),
    ("web-app-examples", "web"),
    ("dashboard-console-examples", "web"),
    ("tui-examples", "tui"),
    ("cli-examples", "cli"),
    ("onboarding-auth-examples", "web"),
    ("documentation-site-examples", "docs"),
    ("app-store-listing-examples", "web"),
    ("design-system-examples", "web"),
    ("report-evidence-examples", "web"),
    ("pricing-page-examples", "web"),
    ("landing-page-examples", "web"),
];

const RUNTIME_MANIFEST_SCHEMA: &str = "wisent.crawl-runtime-manifest.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RuntimeSurfaceIdentity {
    pub family: String,
    pub exact_url: String,
    pub origin: String,
    pub path: String,
    pub allowed_origins: Vec<String>,
    pub allowed_actions: Vec<String>,
    pub terminal_outcomes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeProduct {
    pub kind: String,
    /// The identity committed in `reference.json`, never rewritten.
    ///
    /// `record_preflight` resolves a display name into a bundle id and a slug
    /// into a binary name, so `identifier` legitimately changes mid-flight. The
    /// declared value stays fixed, which lets `decode_runtime_manifest` compare
    /// every engine against the committed record instead of exempting the three
    /// engines that resolve.
    pub declared_identifier: String,
    pub identifier: String,
    pub product_url: String,
    pub identity_source: String,
    pub surface: Option<RuntimeSurfaceIdentity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeAccount {
    pub mode: String,
    pub account_id: Option<String>,
    #[serde(default)]
    pub credential_refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeConstraints {
    pub no_first_run_consent: bool,
    pub no_system_permission_prompts: bool,
    pub no_notifications: bool,
    pub no_purchase: bool,
    pub no_final_destructive_action: bool,
    pub headless: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimePreparedProof {
    pub schema: String,
    pub product_identifier: String,
    pub device_id: Option<String>,
    pub observed_by: String,
    pub product_version: String,
    pub executable_sha256: String,
    pub observed_at: String,
    pub evidence_uri: String,
    pub evidence_sha256: String,
    pub installed: bool,
    pub first_run_completed: bool,
    pub pending_permission_prompts: u32,
    pub pending_notification_prompts: u32,
    pub notification_delivery_disabled: bool,
    pub permission_prompt_invocation_disabled: bool,
    pub notification_prompt_invocation_disabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeDelivery {
    pub kind: String,
    #[serde(default)]
    pub secret_env: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
struct RuntimeBinding {
    account: RuntimeAccount,
    constraints: RuntimeConstraints,
    prepared_proof: Option<RuntimePreparedProof>,
    delivery: RuntimeDelivery,
    surface: Option<RuntimeSurfaceIdentity>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeServiceIdentity {
    pub name: String,
    pub generation: u64,
    pub consumer: String,
    pub capability: String,
    pub active_host: String,
    pub endpoint: String,
    pub action: String,
    /// Exact deployed Weles worker release advertised by the Stado service
    /// directory and re-confirmed against `{endpoint}/version` before a task is
    /// submitted. It is signed into the receipt through `spisBinding.service`.
    pub release_id: String,
    /// Exact Weles source revision behind `release_id`, from the same two
    /// independent observations.
    pub source_revision: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeExecutionIdentity {
    pub host: String,
    pub observed_hostname: String,
    pub platform: String,
    pub device_id: Option<String>,
    pub resolved_product_identifier: String,
    pub device_name: Option<String>,
    #[serde(default)]
    pub executable_path: Option<String>,
    #[serde(default)]
    pub product_version: Option<String>,
    #[serde(default)]
    pub executable_sha256: Option<String>,
    #[serde(default)]
    pub effective_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeManifest {
    pub schema: String,
    pub run_id: String,
    pub catalog: String,
    pub attempt: u32,
    pub attempt_id: String,
    pub record: String,
    pub engine: String,
    pub source_revision: String,
    pub source_input_sha256: String,
    pub reference_sha256: String,
    pub catalog_key: String,
    pub record_key: String,
    pub correlation_id: String,
    pub stado_run_id: String,
    pub artifact_uri: String,
    pub output_uri: String,
    pub runtime_product: RuntimeProduct,
    pub account: RuntimeAccount,
    pub constraints: RuntimeConstraints,
    pub docs_structure_sha256: Option<String>,
    pub bindings_file_sha256: String,
    pub bindings_source: String,
    pub bindings_sha256: String,
    pub bindings_uri: String,
    pub delivery: RuntimeDelivery,
    pub prepared_proof: Option<RuntimePreparedProof>,
    pub execution_identity: Option<RuntimeExecutionIdentity>,
    pub resource_lease: Option<String>,
    pub service_identity: Option<RuntimeServiceIdentity>,
}

impl RuntimeManifest {
    pub(crate) fn encoded(&self) -> Result<String> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        Ok(STANDARD.encode(serde_json::to_vec(self)?))
    }
}

pub(crate) fn decode_runtime_manifest(
    encoded: &str,
    catalog: &str,
    engine: &str,
    record: Option<&str>,
) -> Result<RuntimeManifest> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let manifest: RuntimeManifest = serde_json::from_slice(
        &STANDARD.decode(encoded).context("runtime manifest is not valid base64")?,
    )?;
    if manifest.schema != RUNTIME_MANIFEST_SCHEMA
        || manifest.catalog != catalog
        || manifest.engine != engine
        || record.is_some_and(|value| value != manifest.record)
        || manifest.source_revision != build_revision()?
    {
        bail!("runtime manifest does not match this exact worker, record, engine and source revision");
    }
    safe_component(&manifest.run_id, "run id")?;
    safe_component(&manifest.catalog, "catalog")?;
    safe_component(&manifest.record, "record")?;
    safe_component(&manifest.attempt_id, "attempt id")?;
    if manifest.runtime_product.identifier.is_empty()
        || manifest.source_input_sha256.len() != 64
        || manifest.correlation_id.is_empty()
        || manifest.stado_run_id.is_empty()
        || manifest.execution_identity.is_none()
    {
        bail!("runtime manifest is incomplete and cannot authorize a worker");
    }
    if manifest.attempt == 0 || manifest.attempt_id.is_empty() {
        bail!("runtime manifest has no immutable execution attempt");
    }
    let execution_identity = manifest
        .execution_identity
        .as_ref()
        .context("runtime manifest has no execution identity")?;
    if execution_identity.host.is_empty() || execution_identity.observed_hostname.is_empty() {
        bail!("runtime manifest execution identity lacks registry host alias or observed hostname");
    }
    if manifest.engine == "web" {
        let service = manifest
            .service_identity
            .as_ref()
            .context("web runtime manifest has no exact Weles service identity")?;
        if service.name != "weles-admission"
            || service.consumer != "spis"
            || service.capability != "browser-evidence"
            || service.action != "generic_browser_task"
            || service.active_host != execution_identity.host
        {
            bail!("web runtime manifest Weles service identity is invalid");
        }
        let (_, current) = registry_placements()?;
        if serde_json::to_value(current)? != serde_json::to_value(&manifest.service_identity)? {
            bail!("Weles service directory generation or exact placement changed after planning");
        }
    } else if manifest.service_identity.is_some() {
        bail!("non-web runtime manifest cannot bind a Weles service identity");
    }
    let bindings = runtime_bindings_for_worker(&manifest)?;
    if bindings.sha256 != manifest.bindings_file_sha256 {
        bail!("runtime bindings whole-file digest differs from the immutable manifest");
    }
    let authoritative = runtime_binding(
        &bindings,
        &manifest.catalog,
        &manifest.engine,
        &manifest.record,
    )?;
    let authoritative_sha256 =
        crate::sha256_hex(&serde_json::to_vec(&authoritative)?);
    if authoritative_sha256 != manifest.bindings_sha256 {
        bail!("normalized catalog+record binding digest differs from the immutable manifest");
    }
    if serde_json::to_value(&authoritative.account)? != serde_json::to_value(&manifest.account)?
        || serde_json::to_value(&authoritative.constraints)?
            != serde_json::to_value(&manifest.constraints)?
        || serde_json::to_value(&authoritative.delivery)?
            != serde_json::to_value(&manifest.delivery)?
        || serde_json::to_value(&authoritative.prepared_proof)?
            != serde_json::to_value(&manifest.prepared_proof)?
        || serde_json::to_value(&authoritative.surface)?
            != serde_json::to_value(&manifest.runtime_product.surface)?
    {
        bail!("runtime manifest differs from the exact committed catalog+record binding");
    }
    let reference_path = reference_path(&manifest.catalog, &manifest.record)?;
    let reference_bytes = std::fs::read(&reference_path)
        .with_context(|| format!("read committed worker record {}", reference_path.display()))?;
    let reference: Value = serde_json::from_slice(&reference_bytes)?;
    let expected_product = runtime_product(
        &manifest.catalog,
        &manifest.engine,
        &manifest.record,
        &reference,
        manifest.runtime_product.surface.clone(),
    )?;
    let expected_docs_structure =
        docs_structure_sha256(&manifest.catalog, &manifest.record, &manifest.engine)?;
    if expected_docs_structure != manifest.docs_structure_sha256 {
        bail!("docs crawl definition digest differs from the committed exact structure");
    }
    let execution = manifest.execution_identity.as_ref().expect("checked above");
    let resolving_engine = matches!(manifest.engine.as_str(), "desktop" | "cli" | "tui");
    // The declared identity is never rewritten, so this comparison covers every
    // engine — including desktop, cli and tui, whose resolved `identifier` is a
    // bundle id or binary name that legitimately differs from the record.
    if expected_product.declared_identifier != manifest.runtime_product.declared_identifier
        || execution.resolved_product_identifier != manifest.runtime_product.identifier
        || expected_product.product_url != manifest.runtime_product.product_url
        || expected_product.surface != manifest.runtime_product.surface
    {
        bail!(
            "runtime product declared identity, URL, surface or resolved execution identity differs from the committed record"
        );
    }
    if resolving_engine && !is_host_query_literal(&manifest.runtime_product.declared_identifier) {
        bail!("declared runtime identity is not a safe host resolution literal");
    }
    if !resolving_engine && expected_product.kind != manifest.runtime_product.kind {
        bail!("runtime product kind differs from the committed record");
    }
    match manifest.engine.as_str() {
        "mobile" | "web" | "docs" => {
            if serde_json::to_value(&expected_product)?
                != serde_json::to_value(&manifest.runtime_product)?
            {
                bail!("canonical runtime product differs from the committed record");
            }
        }
        "cli" => {
            let path = execution.executable_path.as_deref().context("CLI identity has no path")?;
            let digest = execution
                .executable_sha256
                .as_deref()
                .context("CLI identity has no executable digest")?;
            if expected_product.identifier != manifest.runtime_product.identifier
                || manifest.runtime_product.kind != "cli-binary"
                || manifest.runtime_product.identity_source
                    != format!("typed isolated host path resolution: {path}; sha256={digest}")
            {
                bail!("canonical CLI runtime product is invalid");
            }
        }
        "tui" => {
            let path = execution.executable_path.as_deref().context("TUI identity has no path")?;
            let digest = execution
                .executable_sha256
                .as_deref()
                .context("TUI identity has no executable digest")?;
            if execution.device_name.as_deref() != Some(expected_product.identifier.as_str())
                || manifest.runtime_product.kind != "tui-binary"
                || manifest.runtime_product.identity_source
                    != format!("typed isolated host path resolution: {path}; sha256={digest}")
            {
                bail!("canonical TUI runtime product is invalid");
            }
        }
        "desktop" => {
            let path = execution
                .executable_path
                .as_deref()
                .context("desktop identity has no executable path")?;
            let version = execution
                .product_version
                .as_deref()
                .context("desktop identity has no product version")?;
            let digest = execution
                .executable_sha256
                .as_deref()
                .context("desktop identity has no executable digest")?;
            let app_path = path
                .split_once("/Contents/MacOS/")
                .map(|(value, _)| value)
                .context("desktop executable is outside a canonical app bundle")?;
            if execution.device_name.as_deref() != Some(expected_product.identifier.as_str())
                || manifest.runtime_product.kind != "desktop-bundle"
                || manifest.runtime_product.identity_source
                    != format!("typed host display-name resolution: bundle={app_path}; executable={path}; version={version}; sha256={digest}")
            {
                bail!("canonical desktop runtime product is invalid");
            }
        }
        _ => bail!("unsupported runtime engine"),
    }
    let mut recomputed = manifest.clone();
    finalize_manifest_identity(&mut recomputed, &reference_bytes)?;
    if serde_json::to_value(&recomputed)? != serde_json::to_value(&manifest)? {
        bail!("runtime manifest identity, input digest, keys or artifact URIs are not canonical");
    }
    Ok(manifest)
}

fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn safe_component(value: &str, name: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || matches!(value, "." | "..")
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
        })
    {
        bail!("{name} must be one strict ASCII path component of at most 128 bytes");
    }
    Ok(())
}

fn catalog_root(catalog: &str) -> Result<PathBuf> {
    safe_component(catalog, "catalog")?;
    if !CATALOGS.iter().any(|(known, _)| *known == catalog) {
        bail!("unknown crawl catalog {catalog}");
    }
    Ok(source_root().join(catalog))
}

fn reference_path(catalog: &str, record: &str) -> Result<PathBuf> {
    safe_component(record, "record")?;
    Ok(catalog_root(catalog)?
        .join("references")
        .join(record)
        .join("reference.json"))
}

fn run_root() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").context("HOME is required for durable crawl run state")?;
    Ok(PathBuf::from(home)
        .join(".stado")
        .join("work")
        .join("spis")
        .join("crawl-runs"))
}

fn legacy_run_root() -> PathBuf {
    source_root().join(".wisent-output").join("crawl-runs")
}

fn migrate_run_state(run_id: Option<&str>) -> Result<()> {
    let legacy = legacy_run_root();
    if !legacy.is_dir() {
        return Ok(());
    }
    let selected = if let Some(run_id) = run_id {
        safe_component(run_id, "run id")?;
        vec![run_id.to_string()]
    } else {
        std::fs::read_dir(&legacy)?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().join("run.json").is_file())
            .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
            .collect()
    };
    for id in selected {
        safe_component(&id, "run id")?;
        let source = legacy.join(&id).join("run.json");
        let destination = run_root()?.join(&id).join("run.json");
        if !source.is_file() || destination.is_file() {
            continue;
        }
        let document: Value =
            crate::read_json(source.to_str().context("legacy run path is not UTF-8")?)?;
        atomic_json_write(&destination, &document)?;
        let recovered: Value =
            crate::read_json(destination.to_str().context("migrated run path is not UTF-8")?)?;
        if recovered != document {
            bail!("legacy crawl run {id} migration read-back differs");
        }
    }
    Ok(())
}

fn run_path(run_id: &str) -> Result<PathBuf> {
    safe_component(run_id, "run id")?;
    Ok(run_root()?.join(run_id).join("run.json"))
}

pub(crate) fn build_revision() -> Result<String> {
    let revision = env!("SPIS_GIT_REVISION");
    let dirty = env!("SPIS_GIT_DIRTY");
    if dirty != "false" {
        bail!("this Spis binary was built from a dirty source tree; exact-revision crawl submission is refused");
    }
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("this Spis binary has no identifiable build-time Git revision");
    }
    Ok(revision.to_string())
}
fn bounded_git(arguments: &[&str], operation: &str) -> Result<Output> {
    let mut command = Command::new("git");
    command.arg("-C").arg(source_root()).args(arguments);
    bounded_command_output(
        &mut command,
        operation,
        Duration::from_secs(30),
        4 * 1024 * 1024,
    )
}

fn source_snapshot_revision() -> Result<String> {
    let embedded = build_revision()?;
    let head = bounded_git(
        &["rev-parse", "--verify", "HEAD"],
        "read current Spis source revision",
    )?;
    if !head.status.success() {
        bail!(
            "cannot identify current Spis source revision: {}",
            String::from_utf8_lossy(&head.stderr).trim()
        );
    }
    let runtime = String::from_utf8(head.stdout)
        .context("current source revision is not UTF-8")?
        .trim()
        .to_string();
    if runtime != embedded {
        bail!(
            "stale Spis binary: embedded revision {embedded} differs from runtime source revision {runtime}"
        );
    }
    let status = bounded_git(
        &["status", "--porcelain=v1", "--untracked-files=all"],
        "verify current Spis source snapshot",
    )?;
    if !status.status.success() {
        bail!(
            "cannot verify current Spis source snapshot: {}",
            String::from_utf8_lossy(&status.stderr).trim()
        );
    }
    if !status.stdout.is_empty() {
        bail!("current Spis source snapshot is dirty; crawl planning is refused");
    }
    Ok(runtime)
}

/// Environment Stado itself reads to find its configuration, authenticate, and
/// select the backend a `stado://` URI resolves against. Nothing else reaches a
/// crawl subprocess: the worker environment carries unrelated credentials that
/// Stado has no use for, and a child that never receives them cannot leak them.
///
/// Every name here is a documented Stado binding, not a guess:
/// `STADO_CONFIG` is `config_file::FILE_ENV`, the explicit config-file override
/// that precedes `./stado.config.json` and `~/.config/stado/config.json`;
/// `STADO_API_TOKEN` and `STADO_API_TOKEN_FILE` are the object API bearer, read
/// directly by the storage client before it falls back to
/// `storage.stado.token_file`; `WC_STORAGE_BACKEND` selects the storage adapter;
/// `WC_STADO_STORAGE_URL`, `WC_STADO_STORAGE_TOKEN_FILE`,
/// `WC_STADO_STORAGE_NAMESPACE` and `WC_STADO_STORAGE_CA_FILE` are that
/// adapter's catalog fields (endpoint, token file, namespace, private CA root);
/// `WC_LOCAL_STORAGE_PATH` is the same field for a device-local backend; and
/// `STADO_RESOLVER_SSH_KEY_FILE` relocates the resolver key `stado host exec`
/// authenticates with. Attribution-only variables (`USER`, `LOGNAME`,
/// `HOSTNAME`) are deliberately excluded: they feed a registry actor string that
/// already defaults to empty, and they are not needed to reach any backend.
const STADO_PASSTHROUGH_ENV: [&str; 12] = [
    "PATH",
    "HOME",
    "STADO_CONFIG",
    "STADO_API_TOKEN",
    "STADO_API_TOKEN_FILE",
    "STADO_RESOLVER_SSH_KEY_FILE",
    "WC_STORAGE_BACKEND",
    "WC_STADO_STORAGE_URL",
    "WC_STADO_STORAGE_TOKEN_FILE",
    "WC_STADO_STORAGE_NAMESPACE",
    "WC_STADO_STORAGE_CA_FILE",
    "WC_LOCAL_STORAGE_PATH",
];

pub(crate) fn stado_command() -> Command {
    let mut command =
        Command::new(std::env::var_os("SPIS_STADO_BIN").unwrap_or_else(|| "stado".into()));
    command.env_clear();
    for name in STADO_PASSTHROUGH_ENV {
        // Only when set: an empty value is a configured value to Stado, and
        // would mask the config file entry the operator actually wrote.
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}

/// Stado invocation for objects Spis owns, i.e. everything under
/// [`crate::CRAWL_NAMESPACE`].
///
/// One object request carries exactly one bearer, and Stado compares it against
/// the credential item of the namespace being addressed. The coordinator
/// therefore needs two: its own for `spis-crawls`, and whatever the host
/// already uses for the queue plane the job submission reads and writes
/// (`probierz/...`). Forcing one bearer for both made `crawl start` choose
/// which half to break: with the crawl bearer the queue write was refused,
/// without it the runtime-bindings read-back was.
///
/// `SPIS_CRAWL_OBJECT_TOKEN_FILE` names the owner-only file holding the crawl
/// namespace bearer. It is injected as `STADO_API_TOKEN_FILE` on exactly the
/// invocations that address Spis's own objects and on no other, so the queue
/// plane keeps the host's configured bearer. Unset, everything behaves as
/// before and a single-bearer deployment is unaffected.
pub(crate) fn crawl_storage_command() -> Command {
    let mut command = stado_command();
    if let Some(token_file) = std::env::var_os("SPIS_CRAWL_OBJECT_TOKEN_FILE") {
        command.env("STADO_API_TOKEN_FILE", token_file);
        command.env_remove("STADO_API_TOKEN");
    }
    command
}
fn read_bounded<R: Read>(mut reader: R, maximum: usize) -> std::io::Result<(Vec<u8>, bool)> {
    let mut retained = Vec::with_capacity(maximum.min(64 * 1024));
    let mut buffer = [0_u8; 16 * 1024];
    let mut overflow = false;
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let remaining = maximum.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..count.min(remaining)]);
        overflow |= count > remaining;
    }
    Ok((retained, overflow))
}

pub(crate) fn bounded_command_output(
    command: &mut Command,
    operation: &str,
    timeout: Duration,
    maximum_stream_bytes: usize,
) -> Result<Output> {
    use std::process::Stdio;
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("start {operation}"))?;
    let stdout = child.stdout.take().context("capture bounded child stdout")?;
    let stderr = child.stderr.take().context("capture bounded child stderr")?;
    let stdout_reader = std::thread::spawn(move || read_bounded(stdout, maximum_stream_bytes));
    let stderr_reader = std::thread::spawn(move || read_bounded(stderr, maximum_stream_bytes));
    let deadline = Instant::now() + timeout;
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait()? {
            break (status, false);
        }
        if Instant::now() >= deadline {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            break (child.wait()?, true);
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let (stdout, stdout_overflow) = stdout_reader
        .join()
        .map_err(|_| anyhow!("{operation} stdout reader panicked"))??;
    let (stderr, stderr_overflow) = stderr_reader
        .join()
        .map_err(|_| anyhow!("{operation} stderr reader panicked"))??;
    if timed_out {
        bail!(
            "{operation} exceeded hard timeout {:?}; stdout={:?}; stderr={:?}",
            timeout,
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr)
        );
    }
    if stdout_overflow || stderr_overflow {
        bail!("{operation} exceeded the {maximum_stream_bytes}-byte stdout/stderr bound");
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

pub(crate) fn atomic_json_write(path: &Path, value: &Value) -> Result<()> {
    let parent = path.parent().context("JSON path has no parent")?;
    std::fs::create_dir_all(parent)?;
    let file_name = path.file_name().and_then(|name| name.to_str()).context("JSON filename is not UTF-8")?;
    let lock_path = parent.join(format!(".{file_name}.lock"));
    let lock = OpenOptions::new().read(true).write(true).create(true).open(&lock_path)?;
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        bail!("another process is updating {}", path.display());
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<()> {
        let mut output = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        output.write_all((serde_json::to_string_pretty(value)? + "\n").as_bytes())?;
        output.sync_all()?;
        std::fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    let _ = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) };
    drop(lock);
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

const MAX_ATTEMPT_TREE_ENTRIES: usize = 20_000;
const MAX_ATTEMPT_TREE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_ATTEMPT_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;

/// Walk `root` and prove it is a self-contained tree of ordinary directories and
/// regular files. A symlink, device, socket, FIFO, hard-link fan-out or an
/// out-of-bound entry count/byte total is refused before anything is archived,
/// so a compromised or racing worker cannot smuggle host content into a
/// published crawl artifact.
fn audit_attempt_tree(root: &Path) -> Result<(usize, u64)> {
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    let mut pending = vec![root.to_path_buf()];
    let mut entries = 0_usize;
    let mut bytes = 0_u64;
    while let Some(directory) = pending.pop() {
        let metadata = std::fs::symlink_metadata(&directory)
            .with_context(|| format!("read attempt tree entry {}", directory.display()))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            bail!("attempt artifact tree {} is not a real directory", directory.display());
        }
        for entry in std::fs::read_dir(&directory)
            .with_context(|| format!("list attempt tree {}", directory.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("read attempt tree entry {}", path.display()))?;
            let file_type = metadata.file_type();
            entries += 1;
            if entries > MAX_ATTEMPT_TREE_ENTRIES {
                bail!(
                    "attempt artifact tree exceeds the {MAX_ATTEMPT_TREE_ENTRIES}-entry bound"
                );
            }
            if file_type.is_symlink() {
                bail!("attempt artifact {} is a symbolic link", path.display());
            }
            if file_type.is_dir() {
                pending.push(path);
                continue;
            }
            if !file_type.is_file() {
                bail!("attempt artifact {} is not a regular file", path.display());
            }
            #[cfg(unix)]
            if metadata.nlink() != 1 {
                bail!("attempt artifact {} is a hard link", path.display());
            }
            bytes = bytes
                .checked_add(metadata.len())
                .filter(|total| *total <= MAX_ATTEMPT_TREE_BYTES)
                .with_context(|| {
                    format!(
                        "attempt artifact tree exceeds the {MAX_ATTEMPT_TREE_BYTES}-byte bound"
                    )
                })?;
        }
    }
    Ok((entries, bytes))
}

fn hash_regular_file(path: &Path, maximum: u64) -> Result<(String, u64)> {
    use sha2::{Digest, Sha256};
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("read {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{} is not a regular file", path.display());
    }
    if metadata.len() > maximum {
        bail!("{} exceeds the {maximum}-byte bound", path.display());
    }
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 128 * 1024];
    let mut bytes = 0_u64;
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash {}", path.display()))?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        if bytes > maximum {
            bail!("{} grew past the {maximum}-byte bound while hashing", path.display());
        }
        digest.update(&buffer[..count]);
    }
    Ok((hex::encode(digest.finalize()), bytes))
}

/// Archive one audited attempt tree, publish it once to `uri`, then read it back
/// and prove the stored bytes. Every engine shares this path so archive content
/// safety, the exclusive per-archive lock and the digest proof cannot drift.
pub(crate) fn publish_attempt_archive(root: &Path, uri: &str) -> Result<Value> {
    if !uri.starts_with(&format!("{}/", crate::CRAWL_ATTEMPT_ROOT)) {
        bail!("attempt artifact URI is outside the Spis crawl namespace");
    }
    let attempt_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .context("attempt artifact root has no UTF-8 name")?
        .to_string();
    let parent = root.parent().context("attempt artifact root has no parent")?;
    let (entries, tree_bytes) = audit_attempt_tree(root)?;
    let lock_path = parent.join(format!(".{attempt_name}.archive.lock"));
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)?;
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        bail!("another worker is publishing the {attempt_name} attempt archive");
    }
    let result = (|| -> Result<Value> {
        let archive = parent.join(format!("{attempt_name}.tar.gz"));
        // Always rebuild. A surviving `<attempt_id>.tar.gz` proves nothing about
        // the tree just audited above: an earlier publish attempt may have built
        // it, the run may then have resumed and appended pages, and reusing that
        // file would publish stale content under the current attempt's receipt.
        // `storage archive` is deterministic (mtime 0, sorted members, symlinks
        // refused), so rebuilding is cheap and always correct.
        //
        // It also refuses to overwrite (`create_new`), so build under a staged
        // sibling and swap it in with a rename. That keeps the crash window safe:
        // `archive` is only ever the previous complete archive or the new
        // complete one, never a truncated stream. `flock` above still fences a
        // second publisher, so the staged path cannot be contended.
        let staged = parent.join(format!(".{attempt_name}.tar.gz.staged"));
        match std::fs::remove_file(&staged) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("remove staged crawl attempt archive {}", staged.display())
                })
            }
        }
        let mut stado = stado_command();
        stado.args(["storage", "archive"]).arg(root).arg(&staged);
        let output = bounded_command_output(
            &mut stado,
            "archive crawl attempt",
            Duration::from_secs(600),
            4 * 1024 * 1024,
        )?;
        if !output.status.success() {
            bail!(
                "stado storage archive refused the crawl attempt: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        std::fs::rename(&staged, &archive).with_context(|| {
            format!("install rebuilt crawl attempt archive {}", archive.display())
        })?;
        let (sha256, bytes) = hash_regular_file(&archive, MAX_ATTEMPT_ARCHIVE_BYTES)?;
        let mut stado = crawl_storage_command();
        stado
            .args(["storage", "put", "--if-absent", "--content-type", "application/gzip", uri])
            .arg(&archive);
        let output = bounded_command_output(
            &mut stado,
            "publish crawl attempt",
            Duration::from_secs(600),
            4 * 1024 * 1024,
        )?;
        if !output.status.success() {
            bail!(
                "stado storage put refused the crawl attempt: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        let readback = parent.join(format!(".{attempt_name}.{}.readback", std::process::id()));
        let _ = std::fs::remove_file(&readback);
        let mut stado = crawl_storage_command();
        stado.args(["storage", "get", uri]).arg(&readback);
        let output = bounded_command_output(
            &mut stado,
            "read back crawl attempt",
            Duration::from_secs(600),
            4 * 1024 * 1024,
        )?;
        if !output.status.success() {
            let _ = std::fs::remove_file(&readback);
            bail!(
                "stado storage get refused the published crawl attempt: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        let observed = hash_regular_file(&readback, MAX_ATTEMPT_ARCHIVE_BYTES);
        let _ = std::fs::remove_file(&readback);
        let (observed_sha256, observed_bytes) = observed?;
        if observed_sha256 != sha256 || observed_bytes != bytes {
            bail!(
                "published crawl attempt read-back differs: expected sha256={sha256} bytes={bytes}, observed sha256={observed_sha256} bytes={observed_bytes}"
            );
        }
        Ok(json!({
            "uri": uri,
            "sha256": sha256,
            "bytes": bytes,
            "media_type": "application/gzip",
            "tree_entries": entries,
            "tree_bytes": tree_bytes,
        }))
    })();
    let _ = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) };
    result
}

#[derive(Debug)]
struct RecordLockBusy {
    run_id: String,
    catalog: String,
    record: String,
}

impl std::fmt::Display for RecordLockBusy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}/{}/{} is already being mutated",
            self.run_id, self.catalog, self.record
        )
    }
}

impl std::error::Error for RecordLockBusy {}

struct RecordMutationGuard {
    file: File,
}

impl RecordMutationGuard {
    fn acquire(run_id: &str, catalog: &str, record: &str) -> Result<Self> {
        safe_component(run_id, "run id")?;
        safe_component(catalog, "catalog")?;
        safe_component(record, "record")?;
        let directory = run_root()?
            .join(run_id)
            .join("record-locks")
            .join(catalog);
        std::fs::create_dir_all(&directory)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(directory.join(format!("{record}.lock")))?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(RecordLockBusy {
                run_id: run_id.to_string(),
                catalog: catalog.to_string(),
                record: record.to_string(),
            }
            .into());
        }
        Ok(Self { file })
    }
}

impl Drop for RecordMutationGuard {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

struct RunMutationGuard {
    file: File,
}

impl RunMutationGuard {
    fn acquire(run_id: &str) -> Result<Self> {
        safe_component(run_id, "run id")?;
        let directory = run_root()?.join(run_id);
        std::fs::create_dir_all(&directory)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(directory.join(".mutation.lock"))?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            bail!("crawl run {run_id} is already being mutated by another process");
        }
        Ok(Self { file })
    }
}

impl Drop for RunMutationGuard {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

fn sync_attempt_history(run: &mut Value) {
    let Some(catalogs) = run.get_mut("catalogs").and_then(Value::as_array_mut) else {
        return;
    };
    for record in catalogs
        .iter_mut()
        .filter_map(|catalog| catalog.get_mut("records").and_then(Value::as_array_mut))
        .flatten()
    {
        let attempt_id = record
            .get("manifest")
            .and_then(|manifest| manifest.get("attempt_id"))
            .and_then(Value::as_str)
            .or_else(|| record.get("attempt_id").and_then(Value::as_str))
            .map(str::to_string)
            .or_else(|| {
                record
                    .get("stado_job_id")
                    .and_then(Value::as_str)
                    .map(|job| format!("legacy-job-{job}"))
            });
        let Some(attempt_id) = attempt_id else {
            continue;
        };
        let mut snapshot = record.clone();
        snapshot.as_object_mut().map(|object| object.remove("attempts"));
        snapshot["attempt_id"] = json!(attempt_id);
        let attempts = record
            .as_object_mut()
            .expect("crawl record must be an object")
            .entry("attempts")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .expect("crawl record attempts must be an array");
        if let Some(existing) = attempts.iter_mut().find(|attempt| {
            attempt.get("attempt_id").and_then(Value::as_str) == Some(attempt_id.as_str())
        }) {
            *existing = snapshot;
        } else {
            attempts.push(snapshot);
        }
    }
}

fn persist(run: &mut Value) -> Result<()> {
    let run_id = run
        .get("run_id")
        .and_then(Value::as_str)
        .context("run has no run_id")?;
    let path = run_path(run_id)?;
    let parent = path.parent().context("crawl run path has no parent")?;
    std::fs::create_dir_all(parent)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(parent.join(".run.json.lock"))?;
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        bail!("crawl run {run_id} is already being persisted");
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = parent.join(format!(
        ".run.json.{}.{}.tmp",
        std::process::id(),
        nonce
    ));
    let result = (|| -> Result<Value> {
        let expected = run
            .get("mutation_revision")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if path.is_file() {
            let current: Value =
                crate::read_json(path.to_str().context("run path is not UTF-8")?)?;
            let actual = current
                .get("mutation_revision")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            if actual != expected {
                bail!(
                    "crawl run {run_id} changed concurrently: expected revision {expected}, found {actual}"
                );
            }
        } else if expected != 0 {
            bail!("crawl run {run_id} disappeared before revision {expected} could be persisted");
        }
        let mut staged = run.clone();
        sync_attempt_history(&mut staged);
        staged["mutation_revision"] = json!(expected + 1);
        staged["updated_at"] = json!(crate::now_iso_utc());
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        output.write_all((serde_json::to_string_pretty(&staged)? + "\n").as_bytes())?;
        output.sync_all()?;
        std::fs::rename(&temporary, &path)?;
        File::open(parent)?.sync_all()?;
        Ok(staged)
    })();
    let _ = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) };
    drop(lock);
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    *run = result?;
    Ok(())
}

fn load(run_id: Option<&str>) -> Result<Value> {
    migrate_run_state(run_id)?;
    let selected = match run_id {
        Some(value) => {
            safe_component(value, "run id")?;
            value.to_string()
        }
        None => {
            let root = run_root()?;
            let mut ids: Vec<String> = std::fs::read_dir(&root)
                .with_context(|| format!("no crawl runs exist under {}", root.display()))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().join("run.json").is_file())
                .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
                .filter(|id| safe_component(id, "run id").is_ok())
                .collect();
            ids.sort();
            ids.pop().context("no persisted crawl run exists")?
        }
    };
    let path = run_path(&selected)?;
    crate::read_json(path.to_str().context("run path is not UTF-8")?)
}

const STADO_SUBMISSION_RECEIPT_SCHEMA: &str = "stado.submission-receipt.v3";
pub(crate) const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";
/// The checkout subdirectory every crawl worker runs from. It is no longer a
/// receipt field, so it is asserted where it is actually declared: the
/// `--repo-workdir` each engine passes to `stado submit`.
pub(crate) const STADO_REPO_WORKDIR: &str = "spis";

/// Where the executor Stado resolved for a job is reported. Retained whole:
/// the placement a crawl actually ran on is evidence, and refusing to read it
/// would be refusing the receipt.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StadoResolvedExecutor {
    provider: String,
    machine_type: String,
    gpu_type: String,
    platform_os: String,
    architecture: String,
}

/// One job of a `stado.submission-receipt.v3` document, field for field as
/// `stado-rs/src/cli/submit.rs` serialises it.
///
/// This side denies unknown fields on purpose, so it has to name every field
/// the producer emits and no field it does not. Three that were declared here
/// never existed in the receipt — `repo`, `executor` and `state` at job level —
/// and three that do exist were missing: `command`, `job_key` and
/// `resolved_executor`. The result was that a receipt proving a successful
/// submission read as a contract violation while the job was already queued on
/// the host.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StadoSubmissionJob {
    command_index: u64,
    /// The exact command line Stado accepted. Verified below against
    /// `command_digest`, so it is proof rather than decoration.
    command: String,
    command_digest: String,
    /// Stado derives `job_id` from this key, which is checked below.
    job_key: String,
    job_id: String,
    output_uri: String,
    pinned_host: String,
    resolved_executor: StadoResolvedExecutor,
    repo_ref: String,
    submission_request_digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StadoSubmissionReceipt {
    schema: String,
    run_id: String,
    source_revision: String,
    request_digest: String,
    source_digest: String,
    input_digest: String,
    repo: String,
    repo_ref: String,
    jobs: Vec<StadoSubmissionJob>,
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Accept only an exact typed Stado v3 submission receipt for one job.
///
/// Every digest must be lowercase SHA-256, the repository/ref/workdir/pinned host
/// must equal the immutable attempt the caller is submitting, the single job must
/// map to command index 0 with the receipt's own command and request digests, and
/// its output URI must be the canonical attempt output coordinate. Unknown fields
/// are refused so a v2 or extended receipt can never be mistaken for proof.
fn compact_submission(catalog: &str, engine: &str, host: &str, artifact_uri: Option<&str>, output_uri: &str, stado_stdout: &str) -> Result<Value> {
    let raw = stado_stdout
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .find(|value| {
            value.get("schema").and_then(Value::as_str)
                == Some(STADO_SUBMISSION_RECEIPT_SCHEMA)
        })
        .context("Stado accepted the command but returned no exact typed v3 submission receipt")?;
    let receipt: StadoSubmissionReceipt = serde_json::from_value(raw.clone())
        .context("Stado submission receipt does not match the exact typed v3 contract")?;
    let expected_revision = build_revision()?;
    for (label, digest) in [
        ("request_digest", receipt.request_digest.as_str()),
        ("source_digest", receipt.source_digest.as_str()),
        ("input_digest", receipt.input_digest.as_str()),
    ] {
        if !is_lower_sha256(digest) {
            bail!("Stado submission receipt {label} is not a lowercase SHA-256 digest");
        }
    }
    if receipt.schema != STADO_SUBMISSION_RECEIPT_SCHEMA
        || receipt.run_id.trim().is_empty()
        || receipt.repo != REPOSITORY
        || receipt.repo_ref != expected_revision
        || receipt.source_revision != expected_revision
    {
        bail!(
            "Stado submission receipt run/repository/ref does not bind this exact attempt"
        );
    }
    if receipt.jobs.len() != 1 {
        bail!("per-record crawl submission must map to exactly one Stado job");
    }
    let job = &receipt.jobs[0];
    if job.job_id.trim().is_empty() || safe_component(&job.job_id, "stado job id").is_err() {
        bail!("Stado submission receipt has no portable exact job id");
    }
    // The pinned host and the repository ref are per job in the emitted
    // receipt, not per receipt, so they are compared where they live.
    if job.command_index != 0
        || !is_lower_sha256(&job.command_digest)
        || !is_lower_sha256(&job.job_key)
        || job.submission_request_digest != receipt.request_digest
        || job.repo_ref != receipt.repo_ref
        || job.pinned_host != host
        || job.output_uri != output_uri
    {
        // No assertion on `resolved_executor`: a job pinned to a registry host
        // needs no provider resolution, and Stado reports every one of its
        // fields empty for exactly that case — measured on this attempt. It is
        // retained evidence of the placement, not a precondition.
        bail!(
            "Stado receipt job mapping, digests, host or output URI does not match the submitted attempt"
        );
    }
    // Two derivations Stado performs and this side re-performs, so the receipt
    // is proof and not a claim: the accepted command line hashes to its own
    // command digest, and the job id is the first 24 hex of the job key.
    if crate::sha256_hex(job.command.as_bytes()) != job.command_digest {
        bail!("Stado receipt job command does not hash to its own command digest");
    }
    if job.job_key.len() < 24 || job.job_id != format!("job-{}", &job.job_key[..24]) {
        bail!("Stado receipt job id is not derived from its own job key");
    }
    Ok(json!({
        "schema": SUBMISSION_SCHEMA,
        "catalog": catalog,
        "engine": engine,
        "host": host,
        "stado_job_id": job.job_id,
        "stado_run_id": receipt.run_id,
        "artifact_uri": artifact_uri,
        "output_uri": output_uri,
        "state": "queued",
        // The placement a crawl actually ran on, lifted out of the receipt so
        // a reader of the compact line does not have to parse the whole one.
        "stado_executor": {
            "provider": job.resolved_executor.provider,
            "machine_type": job.resolved_executor.machine_type,
            "gpu_type": job.resolved_executor.gpu_type,
            "platform_os": job.resolved_executor.platform_os,
            "architecture": job.resolved_executor.architecture,
        },
        "stado_job_key": job.job_key,
        "stado_receipt": raw,
    }))
}

/// Surface-specific coordinators call this after Stado accepts a job. The final
/// compact line is stable machine input while the preceding Stado text remains
/// useful to a person invoking the low-level engine directly.
pub fn print_submission(catalog: &str, engine: &str, host: &str, artifact_uri: Option<&str>, output_uri: &str, stado_stdout: &str) -> Result<()> {
    print!("{stado_stdout}");
    let report = compact_submission(catalog, engine, host, artifact_uri, output_uri, stado_stdout)?;
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn parse_submission(stdout: &[u8]) -> Result<Value> {
    let text = String::from_utf8_lossy(stdout);
    text.lines()
        .rev()
        .find_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .filter(|value| value.get("schema").and_then(Value::as_str) == Some(SUBMISSION_SCHEMA))
        .context("crawler returned no machine-readable submission")
}

fn engine_command(manifest: &RuntimeManifest, host: &str) -> Result<Vec<String>> {
    let execution = manifest
        .execution_identity
        .as_ref()
        .context("cannot build worker command without execution identity")?;
    if execution.host != host {
        bail!("worker command host differs from the bound execution identity");
    }
    if manifest.engine == "web"
        && manifest
            .service_identity
            .as_ref()
            .is_none_or(|service| service.active_host != host)
    {
        bail!("web worker command host differs from the bound Weles service placement");
    }
    let catalog = manifest.catalog.as_str();
    let engine = manifest.engine.as_str();
    let mut args = match engine {
        "mobile" => vec!["crawl-mobile".into(), catalog.into(), "--host".into(), host.into()],
        "desktop" => vec!["crawl-desktop".into(), catalog.into(), "--host".into(), host.into()],
        "web" => vec!["crawl-web".into(), catalog.into(), "--host".into(), host.into()],
        "tui" => vec!["crawl-tui".into(), "--host".into(), host.into()],
        "cli" => vec!["crawl-cli".into(), "--host".into(), host.into()],
        "docs" => vec!["crawl-docs".into(), "--host".into(), host.into()],
        _ => bail!("unknown crawler engine {engine}"),
    };
    args.push("--record".into());
    args.push(manifest.record.clone());
    args.push("--runtime-manifest-base64".into());
    args.push(manifest.encoded()?);
    Ok(args)
}

fn invoke_engine(args: &[String]) -> Result<Output> {
    let executable = std::env::current_exe().context("locate running spis binary")?;
    let mut command = Command::new(executable);
    command.args(args);
    bounded_command_output(
        &mut command,
        "crawler coordinator",
        Duration::from_secs(180),
        8 * 1024 * 1024,
    )
}

fn selected_specs(selected: &[String]) -> Result<Vec<(&'static str, &'static str)>> {
    if selected.is_empty() {
        return Ok(CATALOGS.to_vec());
    }
    let mut out = Vec::new();
    for wanted in selected {
        let spec = CATALOGS.iter().find(|(catalog, _)| catalog == wanted).copied()
            .ok_or_else(|| anyhow!("unknown crawl catalog {wanted}"))?;
        if !out.contains(&spec) {
            out.push(spec);
        }
    }
    Ok(out)
}

/// Enumerate the committed record directories of one catalog.
///
/// A record directory must be a real directory with a portable UTF-8 name. A
/// symbolic link, a non-UTF-8 name, an unreadable entry or a non-directory is an
/// error rather than a silently dropped record, so a planted link can never
/// redirect a crawl and a broken checkout can never shrink the plan in silence.
fn record_directories(catalog: &str, selected: Option<&str>) -> Result<Vec<PathBuf>> {
    let root = catalog_root(catalog)?.join("references");
    let require_real_directory = |path: &Path| -> Result<()> {
        let metadata = std::fs::symlink_metadata(path)
            .with_context(|| format!("read crawl record {}", path.display()))?;
        if metadata.file_type().is_symlink() {
            bail!("crawl record {} is a symbolic link", path.display());
        }
        if !metadata.is_dir() {
            bail!("crawl record {} is not a directory", path.display());
        }
        Ok(())
    };
    if let Some(record) = selected {
        safe_component(record, "record")?;
        let path = root.join(record);
        if !path.exists() {
            bail!("record {record} does not exist in catalog {catalog}");
        }
        require_real_directory(&path)?;
        return Ok(vec![path]);
    }
    let mut records = Vec::new();
    for entry in std::fs::read_dir(&root)
        .with_context(|| format!("read crawl catalog {}", root.display()))?
    {
        let entry = entry.with_context(|| format!("read crawl catalog {}", root.display()))?;
        let name = entry.file_name();
        let name = name.to_str().with_context(|| {
            format!("crawl catalog {} contains a non-UTF-8 record name", root.display())
        })?;
        if name.starts_with('.') {
            continue;
        }
        safe_component(name, "record")?;
        let path = entry.path();
        require_real_directory(&path)?;
        records.push(path);
    }
    records.sort();
    Ok(records)
}

fn require_exact_object_keys(value: &Value, expected: &[&str], context: &str) -> Result<()> {
    let object = value
        .as_object()
        .with_context(|| format!("{context} must be an object"))?;
    let mut actual = object.keys().map(String::as_str).collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    actual.sort_unstable();
    expected.sort_unstable();
    if actual != expected {
        bail!(
            "{context} fields differ: expected {}, found {}",
            expected.join(", "),
            actual.join(", ")
        );
    }
    Ok(())
}

fn validate_runtime_bindings_document(document: &Value) -> Result<()> {
    require_exact_object_keys(document, &["schema", "records"], "runtime bindings")?;
    if document.get("schema").and_then(Value::as_str)
        != Some("wisent.crawl-runtime-bindings.v1")
    {
        bail!("runtime bindings must declare wisent.crawl-runtime-bindings.v1");
    }
    let catalogs = document
        .get("records")
        .and_then(Value::as_object)
        .context("runtime bindings records must be an object")?;
    let mut actual_catalogs = catalogs.keys().map(String::as_str).collect::<Vec<_>>();
    let mut expected_catalogs = CATALOGS.iter().map(|(name, _)| *name).collect::<Vec<_>>();
    actual_catalogs.sort_unstable();
    expected_catalogs.sort_unstable();
    if actual_catalogs != expected_catalogs {
        bail!("runtime bindings must contain every and only the checked-in crawl catalogs");
    }
    for (catalog, _) in CATALOGS {
        let records = catalogs
            .get(*catalog)
            .and_then(Value::as_object)
            .with_context(|| format!("runtime bindings {catalog} must be a record object"))?;
        let mut actual_records = records.keys().map(String::as_str).collect::<Vec<_>>();
        let expected_paths = record_directories(catalog, None)?;
        let mut expected_records = expected_paths
            .iter()
            .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
            .collect::<Vec<_>>();
        actual_records.sort_unstable();
        expected_records.sort_unstable();
        if actual_records != expected_records {
            bail!("{catalog}: runtime bindings must contain every and only checked-in record slug");
        }
        for (slug, binding) in records {
            match binding.get("configured").and_then(Value::as_bool) {
                Some(false) => require_exact_object_keys(
                    binding,
                    &["configured", "diagnostic"],
                    &format!("{catalog}/{slug} binding"),
                )?,
                Some(true) => {
                    let object = binding
                        .as_object()
                        .with_context(|| format!("{catalog}/{slug} binding must be an object"))?;
                    let allowed = [
                        "configured",
                        "account",
                        "constraints",
                        "delivery",
                        "prepared_proof",
                        "surface",
                    ];
                    let unknown = object
                        .keys()
                        .filter(|key| !allowed.contains(&key.as_str()))
                        .cloned()
                        .collect::<Vec<_>>();
                    if !unknown.is_empty() {
                        bail!("{catalog}/{slug}: unknown runtime binding fields: {}", unknown.join(", "));
                    }
                    for required in ["account", "constraints", "delivery"] {
                        if !object.contains_key(required) {
                            bail!("{catalog}/{slug}: configured binding is missing {required}");
                        }
                    }
                }
                _ => bail!("{catalog}/{slug}: configured must be an explicit boolean"),
            }
        }
    }
    Ok(())
}
fn safe_unconfigured_binding(engine: &str) -> Value {
    json!({
        "configured": false,
        "diagnostic": format!(
            "{engine} requires an explicit record binding and independently observed authorization proof"
        ),
    })
}

fn anonymous_probe_account() -> Value {
    json!({
        "mode": "anonymous-read-only-probe",
        "account_id": "anonymous-read-only-probe",
        "credential_refs": [],
    })
}

/// Every crawl refuses first-run consent, system permission prompts,
/// notifications, purchases and final destructive actions.
///
/// `headless` is the one engine-specific control: only the Weles browser engine
/// executes headless. `planned_record` requires `headless == (engine == "web")`,
/// so a generator that hardcoded `true` made every documentation, terminal and
/// native record permanently unavailable.
fn prohibited_action_constraints(engine: &str) -> Value {
    json!({
        "no_first_run_consent": true,
        "no_system_permission_prompts": true,
        "no_notifications": true,
        "no_purchase": true,
        "no_final_destructive_action": true,
        "headless": engine == "web",
    })
}

fn generate_runtime_bindings(rest: &[String]) -> Result<()> {
    let mut output_path: Option<PathBuf> = None;
    let mut weles_token_ref: Option<String> = None;
    let mut organization_ref: Option<String> = None;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--output" => {
                index += 1;
                output_path = Some(PathBuf::from(
                    rest.get(index).context("--output needs a path")?,
                ));
            }
            "--weles-token-ref" => {
                index += 1;
                weles_token_ref = Some(
                    rest.get(index)
                        .context("--weles-token-ref needs ITEM#FIELD")?
                        .clone(),
                );
            }
            "--organization-ref" => {
                index += 1;
                organization_ref = Some(
                    rest.get(index)
                        .context("--organization-ref needs ITEM#FIELD")?
                        .clone(),
                );
            }
            other => bail!("unknown crawl bindings generate option: {other}"),
        }
        index += 1;
    }
    // The two references authorize exactly one engine: the Weles browser
    // engine. Requiring them for the whole document made every other family
    // hostage to a credential coordinate it never reads — a documentation
    // crawl is bounded HTTP on a Stado host and its delivery is
    // `{"kind": "none"}` — so a fleet without a provisioned Weles bearer could
    // not plan a docs record at all. Supply them and the web families are
    // configured exactly as before; omit them and the web records become
    // explicitly unconfigured with a typed diagnostic, which is the same
    // treatment a native record without an authorization proof already gets
    // and which `planned_record` already turns into one `unavailable` attempt
    // rather than a silent crawl. Both must still be supplied together: half a
    // browser credential is a misconfiguration, not a narrower plan.
    if weles_token_ref.is_some() != organization_ref.is_some() {
        bail!(
            "--weles-token-ref and --organization-ref are supplied together or not at all; \
             one without the other cannot authorize a browser record"
        );
    }
    if weles_token_ref
        .as_deref()
        .into_iter()
        .chain(organization_ref.as_deref())
        .any(|value| !valid_secret_reference(value))
    {
        bail!("binding secret references must use exact ITEM#FIELD syntax");
    }

    let mut catalogs = serde_json::Map::new();
    for (catalog, engine) in CATALOGS {
        let mut records = serde_json::Map::new();
        for directory in record_directories(catalog, None)? {
            let slug = directory
                .file_name()
                .and_then(|value| value.to_str())
                .context("record directory name is not UTF-8")?
                .to_string();
            let binding = match *engine {
                // A web record without the pair of references has no
                // authorized delivery, so it is declared unconfigured here
                // rather than written as a configured record whose secret
                // environment names nothing.
                "web" if weles_token_ref.is_none() => safe_unconfigured_binding(engine),
                "web" | "docs" => {
                    let reference: Value = crate::read_json(
                        directory
                            .join("reference.json")
                            .to_str()
                            .context("record path is not UTF-8")?,
                    )?;
                    let exact_url = reference
                        .get("product_url")
                        .and_then(Value::as_str)
                        .filter(|value| !value.is_empty())
                        .with_context(|| format!("{catalog}/{slug}: reference has no product_url"))?;
                    let parsed = url::Url::parse(exact_url)
                        .with_context(|| format!("{catalog}/{slug}: product_url is invalid"))?;
                    let delivery = if *engine == "web" {
                        json!({
                            "kind": "weles-service-env",
                            "secret_env": {
                                "WELES_TOKEN": weles_token_ref,
                                "WISENT_ORGANIZATION_ID": organization_ref,
                            },
                        })
                    } else {
                        json!({"kind": "none", "secret_env": {}})
                    };
                    let mut object = serde_json::Map::new();
                    object.insert("configured".into(), json!(true));
                    object.insert("account".into(), anonymous_probe_account());
                    object.insert("constraints".into(), prohibited_action_constraints(engine));
                    object.insert("delivery".into(), delivery);
                    if *engine == "web" {
                        object.insert(
                            "surface".into(),
                            json!({
                                "family": catalog.strip_suffix("-examples")
                                    .context("web catalog has no canonical family")?,
                                "exact_url": exact_url,
                                "origin": parsed.origin().ascii_serialization(),
                                "path": parsed.path(),
                                "allowed_origins": [parsed.origin().ascii_serialization()],
                                "allowed_actions": ["generic_browser_task"],
                                "terminal_outcomes": ["blocked", "completed", "failed"],
                            }),
                        );
                    }
                    Value::Object(object)
                }
                other => safe_unconfigured_binding(other),
            };
            records.insert(slug, binding);
        }
        catalogs.insert((*catalog).to_string(), Value::Object(records));
    }
    let document = json!({
        "schema": "wisent.crawl-runtime-bindings.v1",
        "records": catalogs,
    });
    validate_runtime_bindings_document(&document)?;
    let bytes = serde_json::to_vec_pretty(&document)?;
    let mut with_newline = bytes;
    with_newline.push(b'\n');
    if let Some(path) = output_path {
        let outcome = write_generated_bindings(&path, &with_newline)?;
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema": OP_SCHEMA,
                "operation": "bindings_generate",
                "path": path,
                "sha256": crate::sha256_hex(&with_newline),
                "outcome": outcome,
            }))?
        );
    } else {
        print!("{}", String::from_utf8(with_newline)?);
    }
    Ok(())
}

/// Install one freshly validated generated bindings document at `path`.
///
/// The document has already passed `validate_runtime_bindings_document`, so an
/// existing file that differs is stale generated output, not a reason to refuse
/// forever: stage the new bytes beside it, fsync, rename atomically, fsync the
/// directory, then read the installed file back and prove the exact bytes. A
/// crash therefore leaves either the previous or the new complete document, and
/// regeneration after a catalog change is idempotent rather than blocked.
fn write_generated_bindings(path: &Path, bytes: &[u8]) -> Result<&'static str> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&parent)?;
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            bail!(
                "refusing to replace {}: generated runtime bindings must be a regular file",
                path.display()
            );
        }
        Ok(_) if std::fs::read(path)? == bytes => return Ok("unchanged"),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let existed = path.exists();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .context("generated runtime bindings path has no UTF-8 file name")?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let staged = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&staged)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&staged, path)?;
        File::open(&parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result?;
    let installed = std::fs::read(path)?;
    if installed != bytes {
        bail!(
            "generated runtime bindings read-back differs from the validated document at {}",
            path.display()
        );
    }
    Ok(if existed { "replaced" } else { "created" })
}


struct RuntimeBindings {
    source: String,
    local_path: Option<PathBuf>,
    uri: String,
    sha256: String,
    document: Value,
}

fn load_runtime_bindings(explicit: Option<&str>) -> Result<RuntimeBindings> {
    let project_template = source_root().join("crawl-runtime-bindings.json");
    let (source, selected) = if let Some(path) = explicit {
        ("explicit".to_string(), PathBuf::from(path))
    } else if let Some(path) = std::env::var_os("SPIS_RUNTIME_BINDINGS") {
        ("env:SPIS_RUNTIME_BINDINGS".to_string(), PathBuf::from(path))
    } else if let Some(path) = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/spis/crawl-runtime-bindings.json"))
        .filter(|path| path.is_file())
    {
        ("default:user-config".to_string(), path)
    } else if project_template.is_file() {
        ("template:project".to_string(), project_template)
    } else {
        bail!("no runtime bindings: pass --bindings PATH, set SPIS_RUNTIME_BINDINGS, generate ~/.config/spis/crawl-runtime-bindings.json, or provide the project template");
    };
    let bytes = std::fs::read(&selected)
        .with_context(|| format!("read runtime bindings {}", selected.display()))?;
    let document: Value = serde_json::from_slice(&bytes)?;
    validate_runtime_bindings_document(&document)?;
    let sha256 = crate::sha256_hex(&bytes);
    Ok(RuntimeBindings {
        source,
        local_path: Some(selected),
        uri: format!("{}/{sha256}.json", crate::CRAWL_INPUT_ROOT),
        sha256,
        document,
    })
}

fn runtime_bindings_for_worker(manifest: &RuntimeManifest) -> Result<RuntimeBindings> {
    let home = std::env::var_os("HOME").context("HOME is required for private Stado work cache")?;
    let directory = PathBuf::from(home)
        .join(".stado")
        .join("work")
        .join("spis")
        .join("runtime-bindings");
    std::fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    }
    let cache = directory.join(format!("{}.json", manifest.bindings_file_sha256));
    if !cache.is_file() {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let temporary = directory.join(format!(
            ".{}.{}.{}.tmp",
            manifest.bindings_file_sha256,
            std::process::id(),
            nonce
        ));
        let output = crawl_storage_command()
            .args(["storage", "get", &manifest.bindings_uri])
            .arg(&temporary)
            .output()
            .context("download immutable runtime bindings")?;
        if !output.status.success() {
            let _ = std::fs::remove_file(&temporary);
            bail!(
                "download runtime bindings {}: {}",
                manifest.bindings_uri,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        let downloaded = std::fs::read(&temporary)?;
        if crate::sha256_hex(&downloaded) != manifest.bindings_file_sha256 {
            let _ = std::fs::remove_file(&temporary);
            bail!("downloaded runtime bindings digest does not match manifest");
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
        }
        if cache.is_file() {
            let _ = std::fs::remove_file(&temporary);
        } else {
            std::fs::rename(&temporary, &cache)?;
            File::open(&directory)?.sync_all()?;
        }
    }
    let bytes = std::fs::read(&cache)?;
    if crate::sha256_hex(&bytes) != manifest.bindings_file_sha256 {
        bail!("private runtime bindings cache conflicts with manifest digest");
    }
    let document: Value = serde_json::from_slice(&bytes)?;
    validate_runtime_bindings_document(&document)?;
    Ok(RuntimeBindings {
        source: manifest.bindings_source.clone(),
        local_path: None,
        uri: manifest.bindings_uri.clone(),
        sha256: manifest.bindings_file_sha256.clone(),
        document,
    })
}

fn publish_runtime_bindings(bindings: &RuntimeBindings) -> Result<()> {
    let source = bindings
        .local_path
        .as_ref()
        .context("runtime binding input has no local immutable source")?;
    let bytes = std::fs::read(source)?;
    if crate::sha256_hex(&bytes) != bindings.sha256 {
        bail!("runtime binding input changed after planning");
    }
    let output = crawl_storage_command()
        .args([
            "storage",
            "put",
            "--if-absent",
            "--content-type",
            "application/json",
            bindings.uri.as_str(),
        ])
        .arg(source)
        .output()
        .context("publish immutable runtime bindings")?;
    if !output.status.success() {
        bail!(
            "Stado refused immutable runtime bindings: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let home = std::env::var_os("HOME").context("HOME is required for private Stado work cache")?;
    let directory = PathBuf::from(home)
        .join(".stado")
        .join("work")
        .join("spis")
        .join("runtime-bindings");
    std::fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let target = directory.join(format!(
        ".{}.{}.{}.readback",
        bindings.sha256,
        std::process::id(),
        nonce
    ));
    let downloaded = crawl_storage_command()
        .args(["storage", "get", bindings.uri.as_str()])
        .arg(&target)
        .output()
        .context("read back immutable runtime bindings")?;
    if !downloaded.status.success() {
        let _ = std::fs::remove_file(&target);
        bail!(
            "runtime bindings read-back failed: {}",
            String::from_utf8_lossy(&downloaded.stderr).trim()
        );
    }
    let stored = std::fs::read(&target)?;
    let _ = std::fs::remove_file(&target);
    if crate::sha256_hex(&stored) != bindings.sha256 || stored != bytes {
        bail!("immutable runtime bindings read-back differs from the planned exact input");
    }
    Ok(())
}

fn valid_secret_reference(reference: &str) -> bool {
    reference
        .split_once('#')
        .is_some_and(|(item, field)| {
            !item.is_empty()
                && !field.is_empty()
                && item
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
                && field
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        })
}

fn runtime_binding(
    bindings: &RuntimeBindings,
    catalog: &str,
    engine: &str,
    slug: &str,
) -> Result<RuntimeBinding> {
    let binding = bindings
        .document
        .get("records")
        .and_then(Value::as_object)
        .and_then(|catalogs| catalogs.get(catalog))
        .and_then(Value::as_object)
        .and_then(|records| records.get(slug))
        .and_then(Value::as_object)
        .ok_or_else(|| {
            anyhow!("{catalog}/{slug}: runtime bindings have no exact catalog and record key")
        })?;
    if binding.get("configured").and_then(Value::as_bool) != Some(true) {
        bail!(
            "{catalog}/{slug}: {}",
            binding
                .get("diagnostic")
                .and_then(Value::as_str)
                .unwrap_or("runtime binding is explicitly unconfigured")
        );
    }
    let account: RuntimeAccount = serde_json::from_value(
        binding.get("account").cloned().context("record binding has no account declaration")?,
    )
    .context("record account declaration is invalid")?;
    match account.mode.as_str() {
        "anonymous-read-only-probe" => {
            if account.account_id.as_deref() != Some("anonymous-read-only-probe")
                || !account.credential_refs.is_empty()
            {
                bail!("{catalog}/{slug}: anonymous read-only probe mode must be explicit and cannot carry credentials or an account claim");
            }
        }
        "none" => {
            if account.account_id.is_some() || !account.credential_refs.is_empty() {
                bail!("{catalog}/{slug}: explicit none account mode cannot carry identity or credentials");
            }
        }
        mode => bail!("{catalog}/{slug}: unsupported account mode {mode}"),
    }
    if account
        .credential_refs
        .iter()
        .any(|reference| reference.is_empty() || reference.chars().any(char::is_whitespace))
    {
        bail!("{catalog}/{slug}: credentialRefs must be nonempty opaque identifiers");
    }
    let constraints: RuntimeConstraints = serde_json::from_value(
        binding
            .get("constraints")
            .cloned()
            .context("record binding has no constraint declaration")?,
    )
    .context("record constraint declaration is invalid")?;
    if !constraints.no_first_run_consent
        || !constraints.no_system_permission_prompts
        || !constraints.no_notifications
        || !constraints.no_purchase
        || !constraints.no_final_destructive_action
    {
        bail!("{catalog}/{slug}: runtime constraints would permit a prohibited crawl action");
    }
    let delivery: RuntimeDelivery = serde_json::from_value(
        binding
            .get("delivery")
            .cloned()
            .context("record binding has no typed credential delivery")?,
    )
    .context("record credential delivery is invalid")?;
    if delivery.secret_env.values().any(|reference| !valid_secret_reference(reference)) {
        bail!("{catalog}/{slug}: secret_env must contain exact NAME=item#field references");
    }
    let prepared_proof = binding
        .get("prepared_proof")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .context("prepared proof declaration is invalid")?;
    match engine {
        "web" => {
            let expected = ["WELES_TOKEN", "WISENT_ORGANIZATION_ID"];
            if delivery.kind != "weles-service-env"
                || delivery.secret_env.len() != expected.len()
                || expected.iter().any(|name| !delivery.secret_env.contains_key(*name))
            {
                bail!("{catalog}/{slug}: web binding needs only exact bearer and organization secret references; public receipt trust is checked in");
            }
        }
        "mobile" | "desktop" => {
            if delivery.kind != "preauthenticated-device"
                || !delivery.secret_env.is_empty()
                || prepared_proof.is_none()
            {
                bail!("{catalog}/{slug}: native binding needs preauthenticated-device delivery, no secret injection, and prepared proof");
            }
        }
        "cli" | "tui" => {
            let expected = if delivery.secret_env.is_empty() { "none" } else { "stado-secret-env" };
            let delivered: std::collections::BTreeSet<&str> =
                delivery.secret_env.values().map(String::as_str).collect();
            let declared: std::collections::BTreeSet<&str> =
                account.credential_refs.iter().map(String::as_str).collect();
            if delivery.kind != expected || delivered != declared {
                bail!("{catalog}/{slug}: terminal delivery must exactly bind account refs through Stado secret-env");
            }
        }
        "docs" => {
            if delivery.kind != "none" || !delivery.secret_env.is_empty() {
                bail!("{catalog}/{slug}: documentation public delivery must be explicit none");
            }
        }
        _ => bail!("{catalog}/{slug}: unsupported engine delivery"),
    }
    let surface = binding
        .get("surface")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .context("surface identity declaration is invalid")?;
    Ok(RuntimeBinding {
        account,
        constraints,
        prepared_proof,
        delivery,
        surface,
    })
}

fn runtime_product(
    catalog: &str,
    engine: &str,
    slug: &str,
    record: &Value,
    surface: Option<RuntimeSurfaceIdentity>,
) -> Result<RuntimeProduct> {
    let product_url = record
        .get("product_url")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("record has no product_url")?
        .to_string();
    let (kind, identifier, identity_source) = match (engine, catalog) {
        ("mobile", "ios-app-examples") => {
            let (bundle, source) = super::crawl_mobile::ios_bundle_id_for(&product_url)?;
            ("ios-bundle", bundle, source)
        }
        ("mobile", "android-app-examples") => {
            let parsed = url::Url::parse(&product_url)?;
            let package = parsed
                .query_pairs()
                .find(|(key, _)| key == "id")
                .map(|(_, value)| value.into_owned())
                .filter(|value| !value.is_empty())
                .context("Play product_url has no exact package id")?;
            ("android-package", package, product_url.clone())
        }
        ("desktop", _) => {
            let display_name = record
                .get("name")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .context("reference has no exact product display name")?;
            (
                "desktop-display-name",
                display_name.to_string(),
                "pending unique typed host resolution".into(),
            )
        }
        ("cli", _) => (
            "cli-binary",
            super::crawl_cli::binary_for(slug),
            "Spis exact CLI catalog mapping".into(),
        ),
        ("tui", _) => {
            let display_name = record
                .get("name")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .context("reference has no exact product display name")?;
            (
                "tui-slug",
                display_name.to_string(),
                "pending unique typed host executable/version resolution".into(),
            )
        }
        ("web" | "docs", _) => ("url", product_url.clone(), "reference.json product_url".into()),
        _ => bail!("{catalog}/{slug}: unsupported runtime identity for {engine}"),
    };
    let surface = if engine == "web" {
        let surface = surface.context("web record has no exact typed surface identity")?;
        let expected_family = catalog
            .strip_suffix("-examples")
            .context("web catalog has no canonical family suffix")?;
        let parsed = url::Url::parse(&product_url).context("web product URL is invalid")?;
        if surface.family != expected_family
            || surface.exact_url != product_url
            || surface.origin != parsed.origin().ascii_serialization()
            || surface.path != parsed.path()
            || surface.allowed_origins.is_empty()
            || !surface.allowed_origins.iter().any(|origin| origin == &surface.origin)
            || surface.allowed_origins.iter().any(|origin| url::Url::parse(origin).is_err())
            || surface.allowed_actions.is_empty()
            || surface.allowed_actions.iter().any(|action| action.is_empty() || action.chars().any(char::is_whitespace))
            || surface.terminal_outcomes.is_empty()
            || surface.terminal_outcomes.iter().any(|outcome| {
                !matches!(outcome.as_str(), "completed" | "failed" | "blocked")
            })
        {
            bail!("{catalog}/{slug}: typed web surface family, origin, path, URL, allowed actions or terminal outcomes are not exact");
        }
        Some(surface)
    } else {
        surface
    };
    if matches!(engine, "desktop" | "cli" | "tui") && !is_host_query_literal(&identifier) {
        bail!(
            "{catalog}/{slug}: {engine} product identity {identifier:?} contains characters that cannot be embedded in a host resolution query"
        );
    }
    Ok(RuntimeProduct {
        kind: kind.into(),
        declared_identifier: identifier.clone(),
        identifier,
        product_url,
        identity_source,
        surface,
    })
}

/// A display name, bundle id or binary name that is safe to embed verbatim in a
/// Spotlight predicate or an argv element.
///
/// Escaping a single quote with a backslash is not how the Spotlight query
/// language quotes literals, so the only safe posture is to refuse any identity
/// that could close a literal or append a predicate.
pub(crate) fn is_host_query_literal(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, ' ' | '.' | '_' | '-'))
        && !value.starts_with(' ')
        && !value.ends_with(' ')
}
fn docs_structure_sha256(catalog: &str, record: &str, engine: &str) -> Result<Option<String>> {
    if engine != "docs" {
        return Ok(None);
    }
    safe_component(record, "record")?;
    let path = catalog_root(catalog)?
        .join("content-structure")
        .join(format!("{record}.json"));
    let bytes = std::fs::read(&path)
        .with_context(|| format!("read docs crawl definition {}", path.display()))?;
    Ok(Some(crate::sha256_hex(&bytes)))
}

fn finalize_manifest_identity(manifest: &mut RuntimeManifest, reference_bytes: &[u8]) -> Result<()> {
    manifest.reference_sha256 = crate::sha256_hex(reference_bytes);
    let input_identity = json!({
        "reference_sha256": manifest.reference_sha256,
        "runtime_product": manifest.runtime_product,
        "account": manifest.account,
        "constraints": manifest.constraints,
        "delivery": manifest.delivery,
        "bindings_sha256": manifest.bindings_sha256,
        "bindings_file_sha256": manifest.bindings_file_sha256,
        "prepared_proof": manifest.prepared_proof,
        "execution_identity": manifest.execution_identity,
        "resource_lease": manifest.resource_lease,
        "docs_structure_sha256": manifest.docs_structure_sha256,
        "service_identity": manifest.service_identity,
    });
    manifest.source_input_sha256 =
        crate::sha256_hex(&serde_json::to_vec(&input_identity)?);
    manifest.catalog_key = crate::sha256_hex(
        format!(
            "{}\0{}\0{}",
            manifest.source_revision, manifest.run_id, manifest.catalog
        )
        .as_bytes(),
    );
    manifest.record_key = crate::sha256_hex(
        format!(
            "{}\0{}\0{}",
            manifest.catalog_key, manifest.record, manifest.source_input_sha256
        )
        .as_bytes(),
    );
    manifest.attempt_id = format!(
        "attempt-{}-{}",
        manifest.attempt,
        &crate::sha256_hex(
            format!(
                "{}\0{}\0{}",
                manifest.record_key, manifest.attempt, manifest.execution_identity.as_ref().map(|value| value.host.as_str()).unwrap_or("")
            )
            .as_bytes()
        )[..16]
    );
    manifest.correlation_id = format!("spis-{}-{}", &manifest.record_key[..24], manifest.attempt);
    manifest.stado_run_id = format!("{}-{}", manifest.correlation_id, manifest.attempt_id);
    let base_uri = crate::crawl_attempt_base_uri(
        &manifest.run_id,
        &manifest.catalog,
        &manifest.record,
        &manifest.record_key,
        manifest.attempt,
        &manifest.attempt_id,
    );
    manifest.artifact_uri = format!("{base_uri}/artifacts.tar.gz");
    manifest.output_uri = format!("{base_uri}/worker-output.log");
    Ok(())
}
pub(crate) fn native_attempt_root(
    base: &Path,
    manifest: &RuntimeManifest,
) -> Result<PathBuf> {
    for (name, component) in [
        ("run_id", manifest.run_id.as_str()),
        ("catalog", manifest.catalog.as_str()),
        ("record", manifest.record.as_str()),
        ("record_key", manifest.record_key.as_str()),
        ("attempt_id", manifest.attempt_id.as_str()),
    ] {
        safe_component(component, name)?;
    }
    if manifest.attempt == 0 {
        bail!("runtime manifest attempt must be a nonzero u32");
    }
    let coordinate = crate::crawl_attempt_base_uri(
        &manifest.run_id,
        &manifest.catalog,
        &manifest.record,
        &manifest.record_key,
        manifest.attempt,
        &manifest.attempt_id,
    );
    let artifact_uri = format!("{coordinate}/artifacts.tar.gz");
    let output_uri = format!("{coordinate}/worker-output.log");
    if manifest.artifact_uri != artifact_uri || manifest.output_uri != output_uri {
        bail!("runtime manifest artifact/output URIs are not the canonical attempt coordinates");
    }
    Ok(base
        .join(&manifest.run_id)
        .join(&manifest.catalog)
        .join(&manifest.record)
        .join(&manifest.record_key)
        .join("attempts")
        .join(manifest.attempt.to_string())
        .join(&manifest.attempt_id))
}

/// One explicit typed attempt entry for a record that cannot be planned.
///
/// An unconfigured native binding, an unreadable reference or an unbound Weles
/// placement is still an attempted record: it keeps a deterministic attempt id so
/// `sync_attempt_history` retains exactly one durable diagnostic instead of
/// letting the record disappear from the run.
fn unavailable_record(
    run_id: &str,
    catalog: &str,
    slug: &str,
    code: &str,
    message: String,
    detail: Value,
) -> Value {
    let attempt_id = format!(
        "unattempted-{}",
        &crate::sha256_hex(format!("{run_id}\0{catalog}\0{slug}\0{code}").as_bytes())[..16]
    );
    json!({
        "record": slug,
        "state": "unavailable",
        "attempt": 1,
        "attempt_id": attempt_id,
        "manifest": Value::Null,
        "command": Value::Null,
        "stado_job_id": Value::Null,
        "artifact_uri": Value::Null,
        "output_uri": Value::Null,
        "submission_receipt": Value::Null,
        "preflight": Value::Null,
        "diagnostic": {
            "code": code,
            "retryable": true,
            "message": message,
            "detail": detail,
        },
        "attempts": [],
    })
}

fn planned_record(
    run_id: &str,
    source_revision: &str,
    catalog: &str,
    engine: &str,
    host: &str,
    record_dir: &Path,
    bindings: &RuntimeBindings,
    service_identity: Option<&RuntimeServiceIdentity>,
) -> Value {
    let slug = record_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let path = record_dir.join("reference.json");
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "reference_read_failed",
                error.to_string(),
                json!({"path": path}),
            );
        }
    };
    let reference: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "reference_invalid",
                error.to_string(),
                json!({"path": path}),
            );
        }
    };
    let binding = match runtime_binding(bindings, catalog, engine, &slug) {
        Ok(binding) => binding,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "runtime_binding_missing_or_invalid",
                error.to_string(),
                Value::Null,
            );
        }
    };
    if binding.constraints.headless != (engine == "web") {
        return unavailable_record(
            run_id,
            catalog,
            &slug,
            "runtime_constraint_mismatch",
            format!("{catalog}/{slug}: headless constraint does not match engine {engine}"),
            json!({"headless": binding.constraints.headless, "engine": engine}),
        );
    }
    let service_identity = match (engine, service_identity) {
        ("web", Some(service)) if service.active_host == host => Some(service.clone()),
        ("web", _) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "weles_service_identity_unbound",
                "authorized weles-admission/browser-evidence/generic_browser_task placement is unavailable".into(),
                json!({"host": host}),
            );
        }
        (_, None) => None,
        (_, Some(_)) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "unexpected_service_identity",
                "non-web execution cannot carry a Weles service identity".into(),
                json!({"engine": engine}),
            );
        }
    };
    let product = match runtime_product(
        catalog,
        engine,
        &slug,
        &reference,
        binding.surface.clone(),
    ) {
        Ok(product) => product,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "runtime_product_unresolved",
                error.to_string(),
                json!({"product_url": reference.get("product_url")}),
            );
        }
    };
    let docs_structure_sha256 = match docs_structure_sha256(catalog, &slug, engine) {
        Ok(value) => value,
        Err(error) => {
            return unavailable_record(
                run_id,
                catalog,
                &slug,
                "docs_structure_missing_or_invalid",
                error.to_string(),
                Value::Null,
            );
        }
    };
    let bindings_sha256 = crate::sha256_hex(
        &serde_json::to_vec(&binding).expect("typed runtime binding serializes"),
    );
    let account = binding.account;
    let constraints = binding.constraints;
    let prepared_proof = binding.prepared_proof;
    let delivery = binding.delivery;
    let input_identity = json!({
        "reference_sha256": crate::sha256_hex(&bytes),
        "runtime_product": product,
        "account": account,
        "constraints": constraints,
        "prepared_proof": prepared_proof,
        "delivery": delivery,
        "bindings_file_sha256": bindings.sha256,
        "bindings_sha256": bindings_sha256,
        "service_identity": service_identity,
        "docs_structure_sha256": docs_structure_sha256,
    });
    let source_input_sha256 = crate::sha256_hex(
        &serde_json::to_vec(&input_identity).expect("typed runtime input serializes"),
    );
    let catalog_key = crate::sha256_hex(
        format!("{source_revision}\0{run_id}\0{catalog}").as_bytes(),
    );
    let record_key = crate::sha256_hex(
        format!("{catalog_key}\0{slug}\0{source_input_sha256}").as_bytes(),
    );
    let correlation_id = format!("spis-{}", &record_key[..32]);
    let base_uri = crate::crawl_record_base_uri(run_id, catalog, &slug, &record_key);
    let mut manifest = RuntimeManifest {
        schema: RUNTIME_MANIFEST_SCHEMA.into(),
        run_id: run_id.into(),
        catalog: catalog.into(),
        record: slug.clone(),
        engine: engine.into(),
        source_revision: source_revision.into(),
        source_input_sha256,
        reference_sha256: crate::sha256_hex(&bytes),
        catalog_key,
        record_key,
        correlation_id: correlation_id.clone(),
        attempt: 1,
        attempt_id: String::new(),
        stado_run_id: correlation_id,
        artifact_uri: format!("{base_uri}/artifacts.tar.gz"),
        output_uri: format!("{base_uri}/worker-output.log"),
        runtime_product: product,
        account,
        constraints,
        docs_structure_sha256,
        delivery,
        bindings_file_sha256: bindings.sha256.clone(),
        bindings_source: bindings.source.clone(),
        bindings_sha256,
        bindings_uri: bindings.uri.clone(),
        prepared_proof,
        execution_identity: None,
        // Terminal surfaces share the registry host's tmux namespace, PATH and CPU,
        // so they need a host-level exclusivity lease even though they need no device.
        resource_lease: matches!(engine, "desktop" | "mobile" | "cli" | "tui")
            .then(|| format!("stado-exclusive://{host}/{engine}")),
        service_identity,
    };
    if let Err(error) = finalize_manifest_identity(&mut manifest, &bytes) {
        return unavailable_record(
            run_id,
            catalog,
            &slug,
            "runtime_manifest_finalization_failed",
            error.to_string(),
            Value::Null,
        );
    }
    json!({
        "record": slug,
        "state": "planned",
        "manifest": manifest,
        "command": Value::Null,
        "stado_job_id": Value::Null,
        "artifact_uri": manifest.artifact_uri,
        "output_uri": manifest.output_uri,
        "submission_receipt": Value::Null,
        "preflight": Value::Null,
        "diagnostic": Value::Null,
    })
}
fn is_git_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// A canonical Weles API base is exactly `<scheme>://<host>[:port]/api/v1` with
/// no credentials, query or fragment. Both the bridge and the Rust verifier
/// enforce the same shape, so an endpoint that would be rejected downstream must
/// never enter a plan.
fn canonical_api_endpoint(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|endpoint| {
        matches!(endpoint.scheme(), "http" | "https")
            && endpoint.username().is_empty()
            && endpoint.password().is_none()
            && endpoint.path() == "/api/v1"
            && endpoint.query().is_none()
            && endpoint.fragment().is_none()
            && endpoint.as_str() == value
    })
}

fn registry_placements() -> Result<(BTreeMap<String, String>, Option<RuntimeServiceIdentity>)> {
    let mut command = stado_command();
    command.args(["registry", "pull"]);
    let output = bounded_command_output(
        &mut command,
        "Stado registry pull",
        Duration::from_secs(60),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "Stado registry could not select crawler hosts: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let registry: Value = serde_json::from_slice(&output.stdout)?;
    let targets = registry
        .get("targets")
        .and_then(Value::as_array)
        .context("Stado registry has no targets")?;
    let service = registry.pointer("/service_directory/services/weles-admission");
    let service_identity = service.and_then(|service| {
        let generation = registry
            .pointer("/service_directory/generation")
            .and_then(Value::as_u64)?;
        let capabilities = service
            .pointer("/consumers/spis/capabilities")
            .and_then(Value::as_array)?;
        if !capabilities
            .iter()
            .any(|capability| capability.as_str() == Some("browser-evidence"))
        {
            return None;
        }
        let active_host = service.get("active_host").and_then(Value::as_str)?;
        let target = targets.iter().find(|target| {
            target.get("name").and_then(Value::as_str) == Some(active_host)
        })?;
        let actions = target.pointer("/weles/actions").and_then(Value::as_array)?;
        if !actions
            .iter()
            .any(|action| action.as_str() == Some("generic_browser_task"))
        {
            return None;
        }
        let endpoint = service
            .pointer(&format!("/endpoints/{active_host}/url"))
            .and_then(Value::as_str)
            .filter(|value| canonical_api_endpoint(value))?;
        let release_id = service
            .pointer(&format!("/endpoints/{active_host}/release_id"))
            .or_else(|| service.get("release_id"))
            .and_then(Value::as_str)
            .filter(|value| value.starts_with("weles-worker@") && *value != "weles-worker@")?;
        let source_revision = service
            .pointer(&format!("/endpoints/{active_host}/source_revision"))
            .or_else(|| service.get("source_revision"))
            .and_then(Value::as_str)
            .filter(|value| is_git_revision(value))?;
        Some(RuntimeServiceIdentity {
            name: "weles-admission".into(),
            generation,
            consumer: "spis".into(),
            capability: "browser-evidence".into(),
            active_host: active_host.into(),
            endpoint: endpoint.into(),
            action: "generic_browser_task".into(),
            release_id: release_id.into(),
            source_revision: source_revision.into(),
        })
    });
    let always_on = targets
        .iter()
        .find(|target| {
            target.get("role").and_then(Value::as_str) == Some("always-on")
                && target.pointer("/weles/enabled").and_then(Value::as_bool) == Some(true)
        })
        .and_then(|target| target.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let cpu = targets
        .iter()
        .find(|target| target.get("role").and_then(Value::as_str) == Some("always-on"))
        .and_then(|target| target.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let mobile = targets
        .iter()
        .find(|target| {
            target
                .get("services")
                .and_then(Value::as_array)
                .is_some_and(|services| {
                    services.iter().any(|service| {
                        service.get("name").and_then(Value::as_str).is_some_and(|name| {
                            name.to_ascii_lowercase().contains("appium")
                        })
                    })
                })
        })
        .and_then(|target| target.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let mut placements = BTreeMap::new();
    if let Some(service) = &service_identity {
        placements.insert("web".into(), service.active_host.clone());
    }
    if let Some(host) = always_on {
        placements.insert("desktop".into(), host);
    }
    if let Some(host) = mobile {
        placements.insert("mobile".into(), host);
    }
    if let Some(host) = cpu {
        placements.insert("cli".into(), host.clone());
        placements.insert("tui".into(), host.clone());
        placements.insert("docs".into(), host);
    }
    Ok((placements, service_identity))
}

fn host_for(
    catalog: &str,
    engine: &str,
    explicit: &BTreeMap<String, String>,
    discovered: &BTreeMap<String, String>,
) -> Result<String> {
    explicit
        .get(catalog)
        .or_else(|| explicit.get(engine))
        .or_else(|| explicit.get("*"))
        .or_else(|| discovered.get(engine))
        .cloned()
        .ok_or_else(|| anyhow!(
            "no Stado host advertises the {engine} execution boundary for {catalog}; pass --host {engine}=TARGET after registering that capability"
        ))
}



fn host_probe(host: &str, arguments: &[&str]) -> Value {
    let mut command = stado_command();
    command
        .args(["host", "exec", host, "--json", "--"])
        .args(arguments);
    let result = (|| -> Result<Value> {
        let output = bounded_command_output(
            &mut command,
            "Stado host probe",
            Duration::from_secs(30),
            1024 * 1024,
        )?;
        if !output.status.success() {
            bail!(
                "Stado host probe failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        let receipt: Value =
            serde_json::from_slice(&output.stdout).context("host probe receipt is not JSON")?;
        let object = receipt
            .as_object()
            .context("host probe receipt must be an object")?;
        let allowed = [
            "schema",
            "target",
            "ssh",
            "ssh_fallbacks",
            "command",
            "argv",
            "stdout",
            "stderr",
            "exit_code",
            "status",
            "program_candidates",
            // Stado reports which of an entry's candidate paths the host
            // actually execed. It is retained evidence, not a surprise: a
            // strict allowlist means the field can only ever name a path the
            // entry itself declares.
            "resolved_executable",
            "error",
        ];
        if object.keys().any(|key| !allowed.contains(&key.as_str()))
            || receipt.get("schema").and_then(Value::as_str)
                != Some("stado.host-exec-receipt.v1")
            || receipt.get("target").and_then(Value::as_str) != Some(host)
            || !receipt.get("ssh").is_some_and(|value| value.is_null() || value.is_string())
            || !receipt.get("ssh_fallbacks").is_some_and(Value::is_array)
            || !receipt.get("command").is_some_and(Value::is_string)
            || !receipt.get("stdout").is_some_and(Value::is_string)
            || !receipt.get("stderr").is_some_and(Value::is_string)
            || !receipt.get("exit_code").is_some_and(Value::is_i64)
        {
            bail!("host probe receipt does not match the exact typed Stado contract");
        }
        // `argv` is what the HOST ran: an absolute program path followed by the
        // entry's fixed arguments. Comparing it word-for-word against the
        // requested spelling refused every receipt whose program Stado resolves
        // per host — which is every multi-candidate entry, and even
        // `hostname -f`, resolved to `/bin/hostname`. What must hold is that
        // the receipt answers exactly the command that was requested and that
        // no argument was added, removed or rewritten on the way.
        if receipt.get("command").and_then(Value::as_str) != Some(arguments.join(" ").as_str()) {
            bail!("host probe receipt answers a different command than the one requested");
        }
        let argv = receipt
            .get("argv")
            .and_then(Value::as_array)
            .context("host probe receipt has no argv")?;
        let (program, rest) = argv
            .split_first()
            .context("host probe receipt argv is empty")?;
        let program = program
            .as_str()
            .context("host probe receipt argv program is not a string")?;
        let requested_program = arguments
            .first()
            .context("host probe requires at least one word")?;
        let program_matches = program == *requested_program
            || program
                .rsplit('/')
                .next()
                .is_some_and(|name| name == *requested_program);
        let arguments_match = rest.len() == arguments.len() - 1
            && rest
                .iter()
                .zip(arguments.iter().skip(1))
                .all(|(observed, expected)| observed.as_str() == Some(*expected));
        if !program_matches || !arguments_match {
            bail!("host probe receipt argv differs from the approved exact command");
        }
        Ok(receipt)
    })();
    match result {
        Ok(receipt) => json!({
            "command": arguments,
            "ready": receipt.get("status").and_then(Value::as_str) == Some("ok")
                && receipt.get("exit_code").and_then(Value::as_i64) == Some(0),
            "stdout": receipt.get("stdout").and_then(Value::as_str).unwrap_or_default(),
            "stderr": receipt.get("stderr").and_then(Value::as_str).unwrap_or_default(),
            "stado_receipt": receipt,
        }),
        Err(error) => {
            json!({"command": arguments, "ready": false, "error": error.to_string()})
        }
    }
}

/// Independently confirm the deployed Weles release on the pinned host.
///
/// The Stado service directory is one observation; `{endpoint}/version` on the
/// selected host is the second. Both must report the same `release_id` and
/// `source_revision`, and redirects are refused, so a relocated or rolled-back
/// worker cannot silently sign attempts as the planned release.
fn weles_version_check(host: &str, service: &RuntimeServiceIdentity) -> Value {
    let url = format!("{}/version", service.endpoint);
    let mut check = host_probe(
        host,
        &[
            "curl",
            "--fail",
            "--silent",
            "--show-error",
            "--max-redirs",
            "0",
            "--max-time",
            "20",
            "--header",
            "Accept: application/json",
            url.as_str(),
        ],
    );
    if check.get("ready").and_then(Value::as_bool) != Some(true) {
        return check;
    }
    let reported: Option<Value> = check
        .get("stdout")
        .and_then(Value::as_str)
        .and_then(|stdout| serde_json::from_str(stdout.trim()).ok());
    let field = |document: &Value, snake: &str, camel: &str| -> Option<String> {
        document
            .get(snake)
            .or_else(|| document.get(camel))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let agreed = reported.as_ref().is_some_and(|document| {
        field(document, "release_id", "releaseId").as_deref() == Some(service.release_id.as_str())
            && field(document, "source_revision", "sourceRevision").as_deref()
                == Some(service.source_revision.as_str())
    });
    if !agreed {
        check["ready"] = json!(false);
        check["error"] = json!(format!(
            "{url} does not report the service-directory release {} at revision {}",
            service.release_id, service.source_revision
        ));
    }
    check["weles_version"] = reported.unwrap_or(Value::Null);
    check
}

fn host_preflight(
    catalog: &str,
    engine: &str,
    host: &str,
    service_identity: Option<&RuntimeServiceIdentity>,
) -> Value {
    // `hostname -f` and not bare `hostname`: Stado's host-exec allowlist
    // matches an entry exactly and never appends operator words, and the entry
    // it carries is the fully-qualified form. Asking for the bare program made
    // every placement preflight fail with "'hostname' is not an approved
    // host-exec command", which reads as a missing host capability and is
    // really a spelling this side chose.
    let mut commands: Vec<Vec<&str>> = vec![vec!["hostname", "-f"]];
    commands.extend(match (engine, catalog) {
        ("mobile", "ios-app-examples") => vec![
            vec!["appium", "--version"],
            vec!["appium", "driver", "list", "--installed"],
            vec!["xcrun", "simctl", "list", "devices", "booted", "--json"],
        ],
        ("mobile", _) => vec![
            vec!["appium", "--version"],
            vec!["appium", "driver", "list", "--installed"],
            vec!["adb", "version"],
            vec!["adb", "devices", "-l"],
        ],
        ("desktop", _) => Vec::new(),
        ("web", _) => vec![vec!["node", "--version"]],
        ("docs", _) => Vec::new(),
        ("cli" | "tui", _) => vec![vec!["tmux", "-V"]],
        _ => Vec::new(),
    });
    let mut checks: Vec<Value> = commands.iter().map(|command| host_probe(host, command)).collect();
    let desktop_driver_ready = if engine == "desktop" {
        let candidates = [
            "/Applications/CuaDriver.app/Contents/MacOS/cua-driver",
            "/Applications/CuaDriver.app/Contents/MacOS/CuaDriver",
        ];
        let driver_checks: Vec<Value> = candidates
            .iter()
            .map(|candidate| host_probe(host, &[*candidate, "doctor", "--json"]))
            .collect();
        let ready = driver_checks
            .iter()
            .any(|check| check.get("ready").and_then(Value::as_bool) == Some(true));
        checks.extend(driver_checks);
        ready
    } else {
        true
    };
    let admission = match (engine, service_identity) {
        ("web", Some(service)) => {
            let check = weles_version_check(host, service);
            let ready = check.get("ready").and_then(Value::as_bool) == Some(true);
            checks.push(check);
            ready
        }
        ("web", None) => false,
        _ => true,
    };
    let ready = checks
        .iter()
        .take(commands.len())
        .all(|check| check.get("ready").and_then(Value::as_bool) == Some(true))
        && desktop_driver_ready
        && admission;
    json!({
        "schema": "wisent.crawl-host-preflight.v2",
        "catalog": catalog,
        "engine": engine,
        "host": host,
        "ready": ready,
        "checks": checks,
        "service_identity": service_identity,
    })
}

fn observed_hostname(host_report: &Value) -> Result<String> {
    let value = host_report
        .get("checks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|check| {
            // The approved spelling is `hostname -f`, so the retained check
            // carries two words. Matching only the one-word form silently
            // dropped the observed hostname and every record then refused with
            // runtime_identity_or_readiness_unavailable.
            check.get("command").and_then(Value::as_array).is_some_and(|command| {
                command.len() == 2
                    && command[0].as_str() == Some("hostname")
                    && command[1].as_str() == Some("-f")
            })
        })
        .and_then(|check| check.get("stdout"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| {
            !value.is_empty()
                && !value.chars().any(char::is_whitespace)
                && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b".-_".contains(&byte))
        })
        .context("host preflight has no exact observed hostname")?;
    Ok(value.to_string())
}

/// Resolve the booted iOS simulator with a probe taken for THIS record.
///
/// The catalog-level host preflight is cached once per catalog, so parsing the
/// device list out of it attributed every later record to a possibly shut-down
/// simulator — and `resume` re-derived identity from the same stale text days
/// later. The desktop and terminal resolvers already probe per record; these two
/// now do the same and return the fresh check as retained evidence.
fn ios_booted_identity(host: &str) -> Result<(RuntimeExecutionIdentity, Vec<Value>)> {
    let check = host_probe(
        host,
        &["xcrun", "simctl", "list", "devices", "booted", "--json"],
    );
    let stdout = ready_output(&check, "fresh iOS booted-device probe")?;
    let document: Value =
        serde_json::from_str(&stdout).context("simctl booted-device report is not JSON")?;
    let mut devices = Vec::new();
    for values in document
        .get("devices")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|map| map.values())
    {
        for device in values.as_array().into_iter().flatten() {
            if device.get("state").and_then(Value::as_str) == Some("Booted")
                && device.get("isAvailable").and_then(Value::as_bool).unwrap_or(true)
            {
                devices.push(device);
            }
        }
    }
    if devices.len() != 1 {
        bail!("expected exactly one booted available iOS device, found {}", devices.len());
    }
    let identity = RuntimeExecutionIdentity {
        host: host.into(),
        observed_hostname: String::new(),
        platform: "ios".into(),
        device_id: Some(
            devices[0]
                .get("udid")
                .and_then(Value::as_str)
                .context("booted iOS device has no UDID")?
                .into(),
        ),
        device_name: devices[0].get("name").and_then(Value::as_str).map(str::to_string),
        resolved_product_identifier: String::new(),
        executable_path: None,
        product_version: None,
        executable_sha256: None,
        effective_url: None,
    };
    Ok((identity, vec![check]))
}

/// Resolve the authorized Android device with a probe taken for THIS record.
fn android_device_identity(host: &str) -> Result<(RuntimeExecutionIdentity, Vec<Value>)> {
    let check = host_probe(host, &["adb", "devices", "-l"]);
    let stdout = ready_output(&check, "fresh Android device probe")?;
    let devices: Vec<&str> = stdout
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let serial = fields.next()?;
            let state = fields.next()?;
            (state == "device").then_some(serial)
        })
        .collect();
    if devices.len() != 1 {
        bail!("expected exactly one authorized Android device, found {}", devices.len());
    }
    let identity = RuntimeExecutionIdentity {
        host: host.into(),
        observed_hostname: String::new(),
        platform: "android".into(),
        device_id: Some(devices[0].into()),
        resolved_product_identifier: String::new(),
        device_name: None,
        executable_path: None,
        product_version: None,
        executable_sha256: None,
        effective_url: None,
    };
    Ok((identity, vec![check]))
}

fn ready_output(check: &Value, context: &str) -> Result<String> {
    if check.get("ready").and_then(Value::as_bool) != Some(true) {
        bail!("{context}: {}", check.get("stderr").and_then(Value::as_str).unwrap_or("host command failed"));
    }
    let output = check
        .get("stdout")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if output.is_empty() {
        bail!("{context}: command returned no identity");
    }
    Ok(output)
}

fn resolve_mobile_install_identity(
    manifest: &RuntimeManifest,
    identity: &mut RuntimeExecutionIdentity,
    host: &str,
    install_check: &Value,
) -> Result<Vec<Value>> {
    let device = identity.device_id.as_deref().context("mobile identity has no device id")?;
    let product = manifest.runtime_product.identifier.as_str();
    if identity.platform == "ios" {
        let app_path = ready_output(install_check, "resolve installed iOS app bundle")?;
        if !app_path.ends_with(".app") {
            bail!("iOS application container is not an app bundle");
        }
        let info = format!("{app_path}/Info.plist");
        let executable_check = host_probe(
            host,
            &["/usr/libexec/PlistBuddy", "-c", "Print:CFBundleExecutable", &info],
        );
        let executable_name = ready_output(&executable_check, "resolve installed iOS executable")?;
        if executable_name.contains('/') || executable_name.chars().any(char::is_whitespace) {
            bail!("installed iOS executable name is invalid");
        }
        let executable_path = format!("{app_path}/{executable_name}");
        let version_check = host_probe(
            host,
            &["/usr/libexec/PlistBuddy", "-c", "Print:CFBundleShortVersionString", &info],
        );
        let version = ready_output(&version_check, "resolve installed iOS version")?;
        let digest_check = host_probe(
            host,
            &["shasum", "-a", "256", &executable_path],
        );
        let digest_output = ready_output(&digest_check, "hash installed iOS executable")?;
        let digest = digest_output.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("installed iOS executable SHA-256 is invalid");
        }
        identity.executable_path = Some(executable_path);
        identity.product_version = Some(version);
        identity.executable_sha256 = Some(digest);
        return Ok(vec![executable_check, version_check, digest_check]);
    }
    let package_path = ready_output(install_check, "resolve installed Android package")?
        .lines()
        .filter_map(|line| line.trim().strip_prefix("package:"))
        .next()
        .context("Android pm path returned no base package path")?
        .to_string();
    let version_check = host_probe(
        host,
        &["adb", "-s", device, "shell", "dumpsys", "package", product],
    );
    let version_output = ready_output(&version_check, "resolve installed Android version")?;
    let version = version_output
        .lines()
        .find_map(|line| line.trim().strip_prefix("versionName="))
        .filter(|value| !value.is_empty())
        .context("Android package has no versionName")?
        .to_string();
    let digest_check = host_probe(
        host,
        &["adb", "-s", device, "shell", "sha256sum", &package_path],
    );
    let digest_output = ready_output(&digest_check, "hash installed Android package")?;
    let digest = digest_output.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("installed Android package SHA-256 is invalid");
    }
    identity.executable_path = Some(package_path);
    identity.product_version = Some(version);
    identity.executable_sha256 = Some(digest);
    Ok(vec![version_check, digest_check])
}

fn resolve_terminal_identity(
    manifest: &mut RuntimeManifest,
    host: &str,
) -> Result<(RuntimeExecutionIdentity, Vec<Value>)> {
    let declared_identifier = manifest.runtime_product.identifier.clone();
    let candidates = if manifest.runtime_product.kind == "tui-slug" {
        super::crawl_tui::binary_candidates(&manifest.runtime_product.identifier)
    } else {
        vec![manifest.runtime_product.identifier.clone()]
    };
    let mut checks = Vec::new();
    let mut resolved = std::collections::BTreeSet::new();
    for candidate in candidates {
        let check = host_probe(host, &["which", &candidate]);
        if check.get("ready").and_then(Value::as_bool) == Some(true) {
            for path in check
                .get("stdout")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|path| path.starts_with('/') && !path.chars().any(char::is_whitespace))
            {
                resolved.insert(path.to_string());
            }
        }
        checks.push(check);
    }
    if resolved.len() != 1 {
        bail!(
            "expected one unique executable for {}, found {}",
            manifest.runtime_product.identifier,
            resolved.len()
        );
    }
    let path = resolved.into_iter().next().expect("one executable");
    let digest_check = host_probe(host, &["shasum", "-a", "256", &path]);
    let digest_output = ready_output(&digest_check, "hash exact executable")?;
    let digest = digest_output.split_whitespace().next().unwrap_or_default();
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("exact executable SHA-256 is invalid");
    }
    checks.push(digest_check);
    let binary = Path::new(&path)
        .file_name()
        .and_then(|value| value.to_str())
        .context("resolved executable path has no UTF-8 filename")?
        .to_string();
    manifest.runtime_product.kind = if manifest.engine == "tui" {
        "tui-binary".into()
    } else {
        "cli-binary".into()
    };
    manifest.runtime_product.identifier = binary;
    manifest.runtime_product.identity_source =
        format!("typed isolated host path resolution: {path}; sha256={digest}");
    Ok((
        RuntimeExecutionIdentity {
            host: host.into(),
            observed_hostname: String::new(),
            platform: "terminal".into(),
            device_id: None,
            resolved_product_identifier: String::new(),
            device_name: Some(declared_identifier),
            executable_path: Some(path),
            product_version: None,
            executable_sha256: Some(digest.to_ascii_lowercase()),
            effective_url: None,
        },
        checks,
    ))
}

fn resolve_desktop_identity(
    manifest: &mut RuntimeManifest,
    host: &str,
) -> Result<(RuntimeExecutionIdentity, Vec<Value>)> {
    let display_name = manifest.runtime_product.identifier.replace('\'', "\\'");
    let query = format!(
        "kMDItemDisplayName == '{display_name}' && kMDItemContentType == 'com.apple.application-bundle'"
    );
    let search = host_probe(host, &["mdfind", &query]);
    let search_output = ready_output(&search, "resolve exact desktop display name")?;
    let paths: Vec<&str> = search_output
        .lines()
        .map(str::trim)
        .filter(|path| path.ends_with(".app"))
        .collect();
    if paths.len() != 1 {
        bail!(
            "desktop display name {} resolved to {} application bundles",
            manifest.runtime_product.identifier,
            paths.len()
        );
    }
    let app_path = paths[0].to_string();
    let metadata = host_probe(
        host,
        &["mdls", "-raw", "-name", "kMDItemCFBundleIdentifier", &app_path],
    );
    let bundle = ready_output(&metadata, "resolve exact desktop bundle identifier")?;
    if bundle == "(null)" || bundle.chars().any(char::is_whitespace) {
        bail!("desktop bundle identifier is missing or invalid");
    }
    let info = format!("{app_path}/Contents/Info.plist");
    let executable_check = host_probe(
        host,
        &["/usr/libexec/PlistBuddy", "-c", "Print:CFBundleExecutable", &info],
    );
    let executable_name = ready_output(&executable_check, "resolve desktop executable")?;
    if executable_name.contains('/') || executable_name.chars().any(char::is_whitespace) {
        bail!("desktop CFBundleExecutable is invalid");
    }
    let executable_path = format!("{app_path}/Contents/MacOS/{executable_name}");
    let version_check = host_probe(
        host,
        &["/usr/libexec/PlistBuddy", "-c", "Print:CFBundleShortVersionString", &info],
    );
    let version = ready_output(&version_check, "resolve desktop product version")?;
    let digest_check = host_probe(host, &["shasum", "-a", "256", &executable_path]);
    let digest_output = ready_output(&digest_check, "hash desktop executable")?;
    let digest = digest_output.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("desktop executable SHA-256 is invalid");
    }
    let hardware_check = host_probe(host, &["ioreg", "-rd1", "-c", "IOPlatformExpertDevice"]);
    let hardware_output = ready_output(&hardware_check, "resolve exact desktop hardware identity")?;
    let hardware_id = hardware_output
        .lines()
        .find(|line| line.contains("\"IOPlatformUUID\""))
        .and_then(|line| line.split('"').nth(3))
        .filter(|value| {
            !value.is_empty()
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
        .context("desktop hardware report has no valid IOPlatformUUID")?
        .to_string();
    manifest.runtime_product.kind = "desktop-bundle".into();
    manifest.runtime_product.identifier = bundle.clone();
    manifest.runtime_product.identity_source =
        format!("typed host display-name resolution: bundle={app_path}; executable={executable_path}; version={version}; sha256={digest}");
    Ok((
        RuntimeExecutionIdentity {
            host: host.into(),
            observed_hostname: String::new(),
            platform: "macos".into(),
            device_id: Some(hardware_id),
            resolved_product_identifier: String::new(),
            device_name: Some(display_name),
            executable_path: Some(executable_path),
            product_version: Some(version),
            executable_sha256: Some(digest),
            effective_url: None,
        },
        vec![search, metadata, executable_check, version_check, digest_check, hardware_check],
    ))
}

fn prepared_runtime_check(
    manifest: &RuntimeManifest,
    identity: &RuntimeExecutionIdentity,
    host: &str,
) -> Result<Value> {
    let proof = manifest
        .prepared_proof
        .as_ref()
        .context("no independently observed prepared-runtime proof is bound")?;
    let product = &manifest.runtime_product.identifier;
    let device = identity.device_id.as_deref().unwrap_or("");
    if proof.schema != "wisent.runtime-preparation-proof.v1"
        || proof.product_identifier != *product
        || proof.device_id.as_deref().unwrap_or("") != device
        || proof.observed_by != "stado-runtime-readiness"
        || !proof.evidence_uri.starts_with("stado://")
        || identity.product_version.as_deref() != Some(proof.product_version.as_str())
        || identity.executable_sha256.as_deref() != Some(proof.executable_sha256.as_str())
        || proof.evidence_sha256.len() != 64
        || !proof.evidence_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        || !proof.installed
        || !proof.first_run_completed
        || proof.pending_permission_prompts != 0
        || proof.pending_notification_prompts != 0
        || !proof.notification_delivery_disabled
        || !proof.permission_prompt_invocation_disabled
        || !proof.notification_prompt_invocation_disabled
        || !is_rfc3339_utc(&proof.observed_at)
    {
        bail!("prepared-runtime proof does not bind the exact product/device with an RFC 3339 UTC observation time, first run completed, zero pending prompts, disabled permission/notification prompt invocation, and disabled notification delivery");
    }
    let check = host_probe(
        host,
        &[
            "stado-runtime-readiness",
            "verify",
            "--json",
            "--product",
            product,
            "--device",
            device,
            "--evidence-uri",
            &proof.evidence_uri,
            "--evidence-sha256",
            &proof.evidence_sha256,
        ],
    );
    let observation: Value =
        serde_json::from_str(&ready_output(&check, "verify prepared-runtime proof")?)
            .context("prepared-runtime helper output is not JSON")?;
    if observation.get("schema").and_then(Value::as_str)
        != Some("wisent.runtime-readiness-observation.v1")
        || observation.get("ready").and_then(Value::as_bool) != Some(true)
        || observation.get("product_identifier").and_then(Value::as_str) != Some(product)
        || observation.get("device_id").and_then(Value::as_str).unwrap_or("") != device
        || observation.get("evidence_sha256").and_then(Value::as_str)
            != Some(proof.evidence_sha256.as_str())
        || observation.get("pending_permission_prompts").and_then(Value::as_u64) != Some(0)
        || observation.get("pending_notification_prompts").and_then(Value::as_u64) != Some(0)
        || observation.get("notification_delivery_disabled").and_then(Value::as_bool)
            != Some(true)
        || observation
            .get("permission_prompt_invocation_disabled")
            .and_then(Value::as_bool)
            != Some(true)
        || observation
            .get("notification_prompt_invocation_disabled")
            .and_then(Value::as_bool)
            != Some(true)
        || observation.get("product_version").and_then(Value::as_str)
            != identity.product_version.as_deref()
        || observation.get("executable_sha256").and_then(Value::as_str)
            != identity.executable_sha256.as_deref()
    {
        bail!("prepared-runtime helper did not attest the exact safe state, version and executable/package digest, including disabled permission and notification prompt invocation");
    }
    Ok(check)
}

/// `YYYY-MM-DDTHH:MM:SSZ`, the exact shape `crate::now_iso_utc` emits.
///
/// The prepared-runtime proof's `observed_at` used to be a dead field that
/// implied a staleness check nobody performed. Freshness itself still comes from
/// the live `stado-runtime-readiness verify` re-check, but the timestamp is now
/// required to be a well-formed UTC instant rather than arbitrary text.
fn is_rfc3339_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 20
        && bytes[19] == b'Z'
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18]
            .iter()
            .all(|index| bytes[*index].is_ascii_digit())
}

fn record_preflight(manifest: &mut RuntimeManifest, host_report: &Value) -> Value {
    let host_ready = host_report.get("ready").and_then(Value::as_bool) == Some(true);
    if !host_ready {
        return json!({
            "schema": "wisent.crawl-record-preflight.v2",
            "record": manifest.record,
            "ready": false,
            "diagnostic": {"code": "host_unavailable", "message": "host capability preflight failed"},
            "checks": [],
        });
    }
    if manifest.engine == "web" && manifest.service_identity.is_none() {
        return json!({
            "schema": "wisent.crawl-record-preflight.v2",
            "record": manifest.record,
            "ready": false,
            "diagnostic": {
                "code": "weles_service_identity_unbound",
                "message": "web execution is unavailable until the exact authorized Weles service directory generation, active host, endpoint, consumer capability and action are bound"
            },
            "checks": [],
        });
    }
    let original_manifest = manifest.clone();
    let result = (|| -> Result<Vec<Value>> {
        let host = host_report.get("host").and_then(Value::as_str).context("host report has no host")?;
        let (mut identity, mut checks) = match manifest.runtime_product.kind.as_str() {
            "ios-bundle" => ios_booted_identity(host)?,
            "android-package" => android_device_identity(host)?,
            "desktop-display-name" => resolve_desktop_identity(manifest, host)?,
            "cli-binary" | "tui-slug" => resolve_terminal_identity(manifest, host)?,
            "url" => (
                RuntimeExecutionIdentity {
                    host: host.into(),
                    observed_hostname: String::new(),
                    platform: if manifest.engine == "web" { "weles".into() } else { "http".into() },
                    device_id: None,
                    resolved_product_identifier: String::new(),
                    device_name: None,
                    executable_path: None,
                    product_version: None,
                    executable_sha256: None,
                    effective_url: None,
                },
                Vec::new(),
            ),
            kind => bail!("unsupported unresolved runtime product kind {kind}"),
        };
        identity.observed_hostname = observed_hostname(host_report)?;
        identity.resolved_product_identifier = manifest.runtime_product.identifier.clone();
        let product = manifest.runtime_product.identifier.as_str();
        let check = match manifest.runtime_product.kind.as_str() {
            "ios-bundle" => {
                let udid = identity.device_id.as_deref().context("iOS identity has no UDID")?;
                host_probe(host, &["xcrun", "simctl", "get_app_container", udid, product, "app"])
            }
            "android-package" => {
                let serial = identity.device_id.as_deref().context("Android identity has no serial")?;
                host_probe(host, &["adb", "-s", serial, "shell", "pm", "path", product])
            }
            "desktop-bundle" => {
                let query = format!("kMDItemCFBundleIdentifier == '{}'", product.replace('\'', "\\'"));
                host_probe(host, &["mdfind", &query])
            }
            "cli-binary" | "tui-binary" => {
                let path = identity.executable_path.as_deref().context("terminal identity has no exact executable path")?;
                host_probe(host, &["shasum", "-a", "256", path])
            }
            "url" => {
                let parsed = url::Url::parse(product)
                    .context("declared URL is invalid")?;
                if parsed.scheme() != "https"
                    || parsed.username() != ""
                    || parsed.password().is_some()
                    || parsed.host_str().is_none()
                {
                    bail!("URL identity must be an exact credential-free HTTPS URL");
                }
                // A URL product has no host command to run: the identity IS
                // the exact committed URL, already parsed and refused above if
                // it carried credentials or a non-HTTPS scheme. It is reported
                // as this check's observed output because the caller proves
                // readiness through `ready_output`, and a synthetic check with
                // empty stdout made every documentation and browser record
                // refuse with "verify exact runtime product: command returned
                // no identity".
                json!({
                    "command": [],
                    "ready": true,
                    "stdout": product,
                    "network_policy_owner": manifest.engine,
                    "declared_url": product,
                })
            }
            _ => unreachable!(),
        };
        let output = ready_output(&check, "verify exact runtime product")?;
        if matches!(manifest.runtime_product.kind.as_str(), "cli-binary" | "tui-binary") {
            let observed = output.split_whitespace().next().unwrap_or_default();
            if identity.executable_sha256.as_deref() != Some(observed) {
                bail!("terminal executable changed during preflight");
            }
        }
        if manifest.engine == "mobile" {
            checks.extend(resolve_mobile_install_identity(manifest, &mut identity, host, &check)?);
        }
        checks.push(check);
        if matches!(manifest.engine.as_str(), "mobile" | "desktop") {
            checks.push(prepared_runtime_check(manifest, &identity, host)?);
        }
        manifest.execution_identity = Some(identity.clone());
        if manifest.resource_lease.is_some() {
            manifest.resource_lease = Some(format!(
                "stado-exclusive://{}/{}",
                host,
                identity.device_id.as_deref().unwrap_or(product)
            ));
        }
        Ok(checks)
    })();
    match result {
        Ok(checks) => json!({
            "schema": "wisent.crawl-record-preflight.v2",
            "record": manifest.record,
            "ready": true,
            "runtime_product": manifest.runtime_product,
            "account": manifest.account,
            "execution_identity": manifest.execution_identity,
            "resource_lease": manifest.resource_lease,
            "prepared_runtime_proof": manifest.prepared_proof,
            "checks": checks,
        }),
        Err(error) => {
            *manifest = original_manifest;
            json!({
                "schema": "wisent.crawl-record-preflight.v2",
                "record": manifest.record,
                "ready": false,
                "runtime_product": manifest.runtime_product,
                "account": manifest.account,
                "diagnostic": {"code": "runtime_identity_or_readiness_unavailable", "message": error.to_string()},
                "checks": [],
            })
        }
    }
}

fn aggregate_catalog_entry(entry: &mut Value) {
    let states: Vec<&str> = entry
        .get("records")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|record| record.get("state").and_then(Value::as_str))
        .collect();
    let failure = |state: &&str| {
        matches!(
            *state,
            "unavailable"
                | "preflight_failed"
                | "submission_failed"
                | "lost"
                | "failed"
                | "cancelled"
                | "partial"
        )
    };
    let state = if states.iter().any(|state| *state == "running") {
        "running"
    } else if states.iter().any(|state| *state == "cancel_pending") {
        "cancel_pending"
    } else if states.iter().any(|state| *state == "pending_review") {
        "pending_review"
    } else if states.iter().any(|state| {
        matches!(*state, "queued" | "submitting" | "preflight_passed")
    }) {
        "queued"
    } else if states.iter().any(|state| matches!(*state, "planned" | "preflighting")) {
        "planned"
    } else if states.iter().all(|state| *state == "imported") && !states.is_empty() {
        "imported"
    } else if states
        .iter()
        .all(|state| matches!(*state, "completed" | "uploaded" | "imported"))
        && !states.is_empty()
    {
        "completed"
    } else if states.iter().any(failure) {
        "partial"
    } else {
        "failed"
    };
    // Captured while `states` still borrows `entry`, used after the writes.
    let no_planned_records = states.is_empty();
    let mut failures: BTreeMap<String, usize> = BTreeMap::new();
    for value in states.iter().filter(|state| failure(state)) {
        *failures.entry((*value).to_string()).or_default() += 1;
    }
    entry["state"] = json!(state);
    entry["partial"] = json!(!failures.is_empty());
    entry["failure_counts"] = serde_json::to_value(failures).unwrap_or(Value::Null);
    // A catalog with no records at all reaches the final `else` above and is
    // reported `failed` with `error: null` and no failure counts, which is
    // what the 2026-09-01 documentation catalog looked like after a refresh:
    // a whole family declared failed with nothing anywhere saying why. That
    // state is not a crawl outcome, it is an empty plan — a run written in the
    // retired catalog-level shape, or a checked-out catalog whose references
    // directory is empty — so it says so, in the same typed diagnostic shape
    // every record-level refusal uses.
    if no_planned_records {
        entry["diagnostic"] = json!({
            "code": "no_planned_records",
            "message": "catalog carries no record attempts; nothing was planned, submitted or imported for it",
        });
    }
}

fn submission_receipt_path(
    run_id: &str,
    catalog: &str,
    record: &str,
    attempt_id: &str,
) -> Result<PathBuf> {
    safe_component(run_id, "run id")?;
    safe_component(catalog, "catalog")?;
    safe_component(record, "record")?;
    safe_component(attempt_id, "attempt id")?;
    Ok(run_root()?
        .join(run_id)
        .join("receipts")
        .join(catalog)
        .join(record)
        .join(attempt_id)
        .join("receipt.json"))
}

fn load_submission_receipt(
    run_id: &str,
    catalog: &str,
    record: &str,
    attempt_id: &str,
) -> Result<Option<Value>> {
    let path = submission_receipt_path(run_id, catalog, record, attempt_id)?;
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(crate::read_json(
        path.to_str().context("receipt path is not UTF-8")?,
    )?))
}

fn persist_submission_receipt(
    run_id: &str,
    catalog: &str,
    record: &str,
    attempt_id: &str,
    receipt: &Value,
) -> Result<()> {
    let path = submission_receipt_path(run_id, catalog, record, attempt_id)?;
    if path.is_file() {
        let existing: Value =
            crate::read_json(path.to_str().context("receipt path is not UTF-8")?)?;
        if existing == *receipt {
            return Ok(());
        }
        bail!("immutable submission receipt already exists with different content");
    }
    atomic_json_write(&path, receipt)?;
    let recovered: Value =
        crate::read_json(path.to_str().context("receipt path is not UTF-8")?)?;
    if recovered != *receipt {
        bail!("submission receipt read-back differs from accepted content");
    }
    Ok(())
}

fn record_snapshot(run_id: &str, catalog: &str, record: &str) -> Result<Value> {
    let run = load(Some(run_id))?;
    run.get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| entry.get("catalog").and_then(Value::as_str) == Some(catalog))
        .and_then(|entry| entry.get("records"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| entry.get("record").and_then(Value::as_str) == Some(record))
        .cloned()
        .with_context(|| format!("{catalog}/{record}: crawl record disappeared"))
}

fn mutate_record<F>(run_id: &str, catalog: &str, record: &str, mutation: F) -> Result<()>
where
    F: FnOnce(&mut Value) -> Result<()>,
{
    let _guard = RunMutationGuard::acquire(run_id)?;
    let mut run = load(Some(run_id))?;
    let catalog_index = run
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .position(|entry| entry.get("catalog").and_then(Value::as_str) == Some(catalog))
        .with_context(|| format!("{catalog}: crawl catalog disappeared"))?;
    let record_index = run["catalogs"][catalog_index]
        .get("records")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .position(|entry| entry.get("record").and_then(Value::as_str) == Some(record))
        .with_context(|| format!("{catalog}/{record}: crawl record disappeared"))?;
    mutation(&mut run["catalogs"][catalog_index]["records"][record_index])?;
    aggregate_catalog_entry(&mut run["catalogs"][catalog_index]);
    update_run_state(&mut run);
    persist(&mut run)
}

fn mark_record_failure(
    run_id: &str,
    catalog: &str,
    record: &str,
    state: &str,
    code: &str,
    message: String,
) -> Result<()> {
    mutate_record(run_id, catalog, record, |entry| {
        if entry.get("cancel_intent").is_some_and(Value::is_object) {
            entry["state"] = json!("cancelled");
            entry["diagnostic"] = json!({
                "code": "cancelled_during_submission",
                "message": "durable cancel intent takes precedence over the coordinator result",
                "underlying": {"code": code, "message": message},
            });
        } else {
            entry["state"] = json!(state);
            entry["diagnostic"] = json!({"code": code, "message": message});
        }
        Ok(())
    })
}

fn ensure_host_preflight(
    run_id: &str,
    catalog: &str,
    engine: &str,
    host: &str,
    service_identity: Option<&RuntimeServiceIdentity>,
) -> Result<Value> {
    let snapshot = load(Some(run_id))?;
    let existing = snapshot
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| entry.get("catalog").and_then(Value::as_str) == Some(catalog))
        .and_then(|entry| entry.get("host_preflight"))
        .cloned()
        .context("crawl catalog disappeared before host preflight")?;
    if !existing.is_null() {
        return Ok(existing);
    }
    let observed = host_preflight(catalog, engine, host, service_identity);
    let _guard = RunMutationGuard::acquire(run_id)?;
    let mut run = load(Some(run_id))?;
    let catalog_index = run
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .position(|entry| entry.get("catalog").and_then(Value::as_str) == Some(catalog))
        .context("crawl catalog disappeared while retaining host preflight")?;
    if run["catalogs"][catalog_index]["host_preflight"].is_null() {
        run["catalogs"][catalog_index]["host_preflight"] = observed;
        run["catalogs"][catalog_index]["state"] = json!("preflighting");
        persist(&mut run)?;
    }
    Ok(run["catalogs"][catalog_index]["host_preflight"].clone())
}

fn continue_record(
    run_id: &str,
    catalog: &str,
    host: &str,
    host_report: &Value,
    record_name: &str,
) -> Result<()> {
    let record_guard = RecordMutationGuard::acquire(run_id, catalog, record_name)?;
    let snapshot = record_snapshot(run_id, catalog, record_name)?;
    let state = snapshot
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("unavailable")
        .to_string();
    if snapshot.get("stado_job_id").and_then(Value::as_str).is_some()
        || matches!(
            state.as_str(),
            "unavailable"
                | "completed"
                | "uploaded"
                | "imported"
                | "running"
                | "queued"
                | "cancel_pending"
                | "pending_review"
                | "cancelled"
        )
    {
        return Ok(());
    }
    if snapshot.get("cancel_intent").is_some_and(Value::is_object) {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "cancelled",
            "cancelled_before_submission",
            "durable cancel intent exists; worker submission is prohibited".into(),
        );
    }
    let mut manifest: RuntimeManifest = match serde_json::from_value(
        snapshot.get("manifest").cloned().unwrap_or(Value::Null),
    ) {
        Ok(manifest) => manifest,
        Err(error) => {
            return mark_record_failure(
                run_id,
                catalog,
                record_name,
                "unavailable",
                "runtime_manifest_invalid",
                error.to_string(),
            );
        }
    };

    let command = if matches!(state.as_str(), "preflight_passed" | "submitting") {
        let retained = snapshot
            .get("command")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>();
        match engine_command(&manifest, host) {
            Ok(expected) if !retained.is_empty() && expected == retained => retained,
            Ok(_) => {
                return mark_record_failure(
                    run_id,
                    catalog,
                    record_name,
                    "unavailable",
                    "retained_command_mismatch",
                    "preflight-persisted command differs from the immutable attempt".into(),
                );
            }
            Err(error) => {
                return mark_record_failure(
                    run_id,
                    catalog,
                    record_name,
                    "unavailable",
                    "worker_command_unavailable",
                    error.to_string(),
                );
            }
        }
    } else {
        let mut preflight = record_preflight(&mut manifest, host_report);
        let mut ready = preflight.get("ready").and_then(Value::as_bool) == Some(true);
        if ready {
            let path = reference_path(&manifest.catalog, &manifest.record);
            let display = path
                .as_ref()
                .map(|value| value.display().to_string())
                .unwrap_or_else(|error| error.to_string());
            if let Err(error) = path.and_then(|value| {
                std::fs::read(&value)
                    .map_err(anyhow::Error::from)
                    .and_then(|bytes| finalize_manifest_identity(&mut manifest, &bytes))
            }) {
                ready = false;
                preflight = json!({
                    "schema": "wisent.crawl-record-preflight.v2",
                    "record": manifest.record,
                    "ready": false,
                    "diagnostic": {
                        "code": "runtime_manifest_finalization_failed",
                        "message": error.to_string(),
                        "path": display,
                    },
                });
            }
        }
        let command = if ready {
            engine_command(&manifest, host)
        } else {
            Err(anyhow!("exact record preflight failed"))
        };
        let command = match command {
            Ok(command) => command,
            Err(error) => {
                let diagnostic = preflight
                    .get("diagnostic")
                    .cloned()
                    .unwrap_or_else(|| {
                        json!({"code": "worker_command_unavailable", "message": error.to_string()})
                    });
                mutate_record(run_id, catalog, record_name, |entry| {
                    entry["manifest"] = serde_json::to_value(&manifest)?;
                    entry["preflight"] = preflight;
                    entry["state"] = json!("unavailable");
                    entry["diagnostic"] = diagnostic;
                    Ok(())
                })?;
                return Ok(());
            }
        };
        mutate_record(run_id, catalog, record_name, |entry| {
            if entry.get("stado_job_id").and_then(Value::as_str).is_some()
                || entry.get("cancel_intent").is_some_and(Value::is_object)
            {
                return Ok(());
            }
            entry["manifest"] = serde_json::to_value(&manifest)?;
            entry["preflight"] = preflight;
            entry["command"] = json!(command);
            entry["state"] = json!("preflight_passed");
            entry["diagnostic"] = Value::Null;
            Ok(())
        })?;
        command
    };

    let before_submit = record_snapshot(run_id, catalog, record_name)?;
    if before_submit.get("cancel_intent").is_some_and(Value::is_object) {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "cancelled",
            "cancelled_before_submission",
            "durable cancel intent won the submission race".into(),
        );
    }
    if !matches!(
        before_submit.get("state").and_then(Value::as_str),
        Some("preflight_passed" | "submitting")
    ) {
        return Ok(());
    }
    if before_submit.get("state").and_then(Value::as_str) == Some("preflight_passed") {
        mutate_record(run_id, catalog, record_name, |entry| {
            if entry.get("state").and_then(Value::as_str) == Some("preflight_passed")
                && !entry.get("cancel_intent").is_some_and(Value::is_object)
            {
                entry["state"] = json!("submitting");
                entry["submission_transition"] = json!({
                    "state": "intent_persisted",
                    "attempt_id": manifest.attempt_id,
                });
            }
            Ok(())
        })?;
    }
    let armed = record_snapshot(run_id, catalog, record_name)?;
    if armed.get("state").and_then(Value::as_str) != Some("submitting") {
        return Ok(());
    }
    let recovered = load_submission_receipt(
        run_id,
        catalog,
        &manifest.record,
        &manifest.attempt_id,
    )?;
    drop(record_guard);
    let receipt = if let Some(receipt) = recovered {
        receipt
    } else {
        let output = match invoke_engine(&command) {
            Ok(output) => output,
            Err(error) => {
                return mark_record_failure(
                    run_id,
                    catalog,
                    record_name,
                    "submission_failed",
                    "crawler_coordinator_launch_failed",
                    format!("{error:#}"),
                );
            }
        };
        if !output.status.success() {
            return mark_record_failure(
                run_id,
                catalog,
                record_name,
                "submission_failed",
                "stado_submission_failed",
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout).trim(),
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            );
        }
        match parse_submission(&output.stdout) {
            Ok(receipt) => receipt,
            Err(error) => {
                return mark_record_failure(
                    run_id,
                    catalog,
                    record_name,
                    "submission_failed",
                    "submission_receipt_invalid",
                    error.to_string(),
                );
            }
        }
    };
    let stado = receipt.get("stado_receipt");
    if stado
        .and_then(|value| value.get("source_revision"))
        .and_then(Value::as_str)
        != Some(manifest.source_revision.as_str())
    {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "submission_failed",
            "submission_source_mismatch",
            "Stado receipt does not bind the immutable Spis source revision".into(),
        );
    }
    // The coordinator derived both attempt URIs from the record key. The child's
    // reported values are checked against them and never adopted, so a divergent
    // final JSON line cannot relocate where this attempt's evidence is expected.
    if receipt.get("artifact_uri").and_then(Value::as_str) != Some(manifest.artifact_uri.as_str())
        || receipt.get("output_uri").and_then(Value::as_str) != Some(manifest.output_uri.as_str())
    {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "submission_failed",
            "submission_uri_mismatch",
            "crawler submission reported artifact or output URIs that are not the canonical attempt coordinates".into(),
        );
    }
    if let Err(error) = persist_submission_receipt(
        run_id,
        catalog,
        &manifest.record,
        &manifest.attempt_id,
        &receipt,
    ) {
        return mark_record_failure(
            run_id,
            catalog,
            record_name,
            "submission_failed",
            "submission_receipt_persistence_failed",
            error.to_string(),
        );
    }
    mutate_record(run_id, catalog, record_name, |entry| {
        entry["stado_job_id"] = receipt
            .get("stado_job_id")
            .cloned()
            .unwrap_or(Value::Null);
        entry["artifact_uri"] = json!(manifest.artifact_uri);
        entry["output_uri"] = json!(manifest.output_uri);
        entry["submission_receipt"] = receipt;
        if entry.get("cancel_intent").is_some_and(Value::is_object) {
            entry["state"] = json!("cancel_pending");
            entry["diagnostic"] = json!({
                "code": "cancel_won_submission_race",
                "message": "the durable cancel intent will be dispatched against the retained Stado job",
            });
        } else {
            entry["state"] = json!("queued");
            entry["diagnostic"] = Value::Null;
        }
        Ok(())
    })?;
    let retained = record_snapshot(run_id, catalog, record_name)?;
    if !retained.get("cancel_intent").is_some_and(Value::is_object) {
        return Ok(());
    }
    let Some(job_id) = retained.get("stado_job_id").and_then(Value::as_str) else {
        return Ok(());
    };
    let cancellation = match machine_status(job_id) {
        Ok(job) if terminal_machine_state(machine_state(&job)) => {
            Ok(json!({"state": "noop_terminal", "observed_job": job}))
        }
        Ok(job) => {
            let output = stado_command()
                .args(["machine", "cancel", job_id])
                .output()
                .context("cancel Stado job after submission race")?;
            if output.status.success() {
                let response = serde_json::from_slice(&output.stdout)
                    .unwrap_or_else(|_| json!({"stdout": String::from_utf8_lossy(&output.stdout).trim()}));
                Ok(json!({"state": "cancel_dispatched", "observed_job": job, "response": response}))
            } else {
                Err(anyhow!(
                    "Stado refused race cancellation: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ))
            }
        }
        Err(error) => Err(anyhow!(
            "status-first race cancellation failed: {}",
            error.diagnostic
        )),
    };
    mutate_record(run_id, catalog, record_name, |entry| {
        match cancellation {
            Ok(result) => {
                entry["state"] = json!("cancelled");
                entry["cancel_result"] = result;
                entry["diagnostic"] = Value::Null;
            }
            Err(error) => {
                entry["state"] = json!("cancel_pending");
                entry["diagnostic"] = json!({
                    "code": "cancel_dispatch_failed",
                    "message": error.to_string(),
                });
            }
        }
        Ok(())
    })
}

fn continue_start(run_id: &str) -> Result<Value> {
    let catalogs = load(Some(run_id))?
        .get("catalogs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for catalog_entry in catalogs {
        let catalog = catalog_entry
            .get("catalog")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let engine = catalog_entry
            .get("engine")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let host = catalog_entry
            .get("host")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if host.is_empty() {
            continue;
        }
        let service_identity: Option<RuntimeServiceIdentity> = catalog_entry
            .get("records")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find_map(|record| record.pointer("/manifest/service_identity"))
            .filter(|value| value.is_object())
            .and_then(|value| serde_json::from_value(value.clone()).ok());
        let host_report = ensure_host_preflight(
            run_id,
            &catalog,
            &engine,
            &host,
            service_identity.as_ref(),
        )?;
        let records = catalog_entry
            .get("records")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for record in records {
            let Some(record_name) = record.get("record").and_then(Value::as_str) else {
                continue;
            };
            if let Err(error) = continue_record(
                run_id,
                &catalog,
                &host,
                &host_report,
                record_name,
            ) {
                if error.downcast_ref::<RecordLockBusy>().is_some() {
                    continue;
                }
                mark_record_failure(
                    run_id,
                    &catalog,
                    record_name,
                    "submission_failed",
                    "record_coordinator_failed",
                    format!("{error:#}"),
                )?;
            }
        }
    }
    load(Some(run_id))
}

fn start(rest: &[String]) -> Result<()> {
    let mut hosts: BTreeMap<String, String> = BTreeMap::new();
    let mut catalogs = Vec::new();
    let mut record = None;
    let mut requested_run_id = None;
    let mut bindings_path = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                let value = rest.get(i).context("--host needs a value")?;
                if let Some((scope, target)) = value.split_once('=') {
                    if scope.is_empty() || target.is_empty() {
                        bail!("--host mapping must be ENGINE=TARGET or CATALOG=TARGET");
                    }
                    hosts.insert(scope.to_string(), target.to_string());
                } else {
                    hosts.insert("*".into(), value.clone());
                }
            }
            "--catalog" => {
                i += 1;
                catalogs.push(rest.get(i).context("--catalog needs a value")?.clone());
            }
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--run-id" => {
                i += 1;
                requested_run_id = Some(rest.get(i).context("--run-id needs a value")?.clone());
            }
            "--bindings" => {
                i += 1;
                bindings_path = Some(rest.get(i).context("--bindings needs a value")?.clone());
            }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    let specs = selected_specs(&catalogs)?;
    if record.is_some() && specs.len() != 1 {
        bail!("--record requires exactly one --catalog");
    }
    let source_revision = source_snapshot_revision()?;
    let bindings = load_runtime_bindings(bindings_path.as_deref())?;
    let (discovered_hosts, service_identity, registry_diagnostic) =
        match registry_placements() {
            Ok((hosts, service)) => (hosts, service, None),
            Err(error) => (
                BTreeMap::new(),
                None,
                Some(format!("Stado registry placement discovery failed: {error}")),
            ),
        };
    let run_id = requested_run_id.unwrap_or_else(|| {
        format!("crawl-{}", crate::now_iso_utc().replace(':', "-").replace('T', "-"))
    });
    let request_identity = json!({
        "source_revision": source_revision,
        "catalogs": specs,
        "record": record,
        "hosts": hosts,
        "service_identity": service_identity,
        "bindings_source": bindings.source,
        "bindings_sha256": bindings.sha256,
        "bindings_uri": bindings.uri,
    });
    let request_digest = crate::sha256_hex(&serde_json::to_vec(&request_identity)?);
    if run_path(&run_id)?.is_file() {
        {
            let _guard = RunMutationGuard::acquire(&run_id)?;
            let run = load(Some(&run_id))?;
            if run.get("request_digest").and_then(Value::as_str) != Some(&request_digest) {
                bail!("run id {run_id} already belongs to a different exact crawl request");
            }
        }
        publish_runtime_bindings(&bindings)?;
        let run = continue_start(&run_id)?;
        print_operation("start", &run, None)?;
        if has_failures(&run) {
            bail!("one or more records remain unavailable or failed");
        }
        return Ok(());
    }
    let mut entries = Vec::new();
    for (catalog, engine) in specs {
        let paths = record_directories(catalog, record.as_deref())?;
        let host = host_for(catalog, engine, &hosts, &discovered_hosts);
        let unavailable = host
            .as_ref()
            .err()
            .map(ToString::to_string)
            .or_else(|| {
                (engine == "web" && service_identity.is_none()).then(|| {
                    registry_diagnostic.clone().unwrap_or_else(|| {
                        "Stado service directory does not authorize consumer spis for weles-admission browser-evidence on a host advertising generic_browser_task".into()
                    })
                })
            });
        if let Some(message) = unavailable {
            let records = paths
                .iter()
                .map(|path| {
                    unavailable_record(
                        &run_id,
                        catalog,
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("invalid-record"),
                        "runtime_placement_unavailable",
                        message.clone(),
                        json!({"engine": engine}),
                    )
                })
                .collect::<Vec<_>>();
            entries.push(json!({
                "catalog": catalog,
                "engine": engine,
                "host": Value::Null,
                "state": "unavailable",
                "host_preflight": {
                    "ready": false,
                    "diagnostic": {
                        "code": "runtime_placement_unavailable",
                        "message": message,
                    }
                },
                "records": records,
            }));
            continue;
        }
        let host = host.expect("placement error handled");
        let service = (engine == "web")
            .then_some(service_identity.as_ref())
            .flatten();
        let records = paths
            .iter()
            .map(|path| {
                planned_record(
                    &run_id,
                    &source_revision,
                    catalog,
                    engine,
                    &host,
                    path,
                    &bindings,
                    service,
                )
            })
            .collect::<Vec<_>>();
        entries.push(json!({
            "catalog": catalog,
            "engine": engine,
            "host": host,
            "state": "planned",
            "host_preflight": Value::Null,
            "records": records,
        }));
    }
    let mut run = json!({
        "schema": RUN_SCHEMA,
        "run_id": run_id,
        "source_revision": source_revision,
        "request_digest": request_digest,
        "request": request_identity,
        "created_at": crate::now_iso_utc(),
        "updated_at": crate::now_iso_utc(),
        "mutation_revision": 0,
        "hosts": hosts,
        "bindings_source": bindings.source,
        "bindings_sha256": bindings.sha256,
        "bindings_uri": bindings.uri,
        "state": "planned",
        "catalogs": entries,
    });
    {
        let _guard = RunMutationGuard::acquire(&run_id)?;
        if run_path(&run_id)?.is_file() {
            bail!("run id {run_id} was concurrently created");
        }
        persist(&mut run)?;
    }
    publish_runtime_bindings(&bindings)?;
    run = continue_start(&run_id)?;
    print_operation("start", &run, None)?;
    if has_failures(&run) {
        bail!("one or more records remain unavailable or failed");
    }
    Ok(())
}

#[derive(Debug)]
struct LookupFailure {
    diagnostic: Value,
    not_found: bool,
}

fn machine_status(job_id: &str) -> std::result::Result<Value, LookupFailure> {
    let output = super::crawl::stado_command().args(["machine", "status", job_id]).output()
        .map_err(|error| LookupFailure {
            diagnostic: json!({"code": "transport_error", "retryable": true, "message": error.to_string()}),
            not_found: false,
        })?;
    let document: Value = serde_json::from_slice(&output.stdout).map_err(|error| LookupFailure {
        diagnostic: json!({"code": "invalid_response", "retryable": true, "message": error.to_string(), "stderr": String::from_utf8_lossy(&output.stderr).trim()}),
        not_found: false,
    })?;
    if !output.status.success() || document.get("ok").and_then(Value::as_bool) != Some(true) {
        let error = document.get("error").cloned().unwrap_or_else(|| json!({
            "code": "status_failed",
            "retryable": true,
            "message": String::from_utf8_lossy(&output.stderr).trim(),
        }));
        let code = error
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_uppercase();
        let not_found = matches!(code.as_str(), "NOT_FOUND" | "JOB_NOT_FOUND");
        return Err(LookupFailure { diagnostic: error, not_found });
    }
    document.pointer("/result/job").cloned().ok_or_else(|| LookupFailure {
        diagnostic: json!({"code": "invalid_response", "retryable": true, "message": "Stado status has no result.job"}),
        not_found: false,
    })
}

fn refresh_item(entry: &mut Value) {
    let Some(job_id) = entry.get("stado_job_id").and_then(Value::as_str).map(str::to_string) else {
        return;
    };
    match machine_status(&job_id) {
        Ok(job) => {
            if let Some(expected) = entry
                .pointer("/manifest/source_revision")
                .and_then(Value::as_str)
                .map(str::to_string)
            {
                let observed = job.get("repo_ref").and_then(Value::as_str).unwrap_or_default();
                if observed != expected {
                    entry["state"] = json!("failed");
                    entry["diagnostic"] = json!({
                        "code": "runtime_revision_mismatch",
                        "expected": expected,
                        "observed": observed,
                        "stado_job_id": job_id,
                    });
                    entry["job"] = job;
                    return;
                }
            }
            if entry.get("state").and_then(Value::as_str) != Some("imported") {
                entry["state"] = job.get("state").cloned().unwrap_or_else(|| json!("failed"));
            }
            entry["job"] = job;
            entry["lookup_error"] = Value::Null;
            if entry.get("state").and_then(Value::as_str) != Some("partial") {
                entry["diagnostic"] = Value::Null;
                entry["error"] = Value::Null;
            }
        }
        Err(error) if error.not_found => {
            entry["state"] = json!("lost");
            entry["lookup_error"] = error.diagnostic.clone();


            entry["diagnostic"] = json!({
                "code": "stado_job_not_found",
                "message": format!("Stado returned semantic NOT_FOUND for job {job_id}"),
                "stado_job_id": job_id,
            });
            entry["error"] = json!(format!("current Stado lookup confirmed that job {job_id} does not exist"));
            entry["job"] = Value::Null;
        }
        Err(error) => {
            entry["lookup_error"] = error.diagnostic;
        }
    }
}

fn refresh(run: &mut Value) {
    if let Some(catalogs) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in catalogs {
            if let Some(records) = entry.get_mut("records").and_then(Value::as_array_mut) {
                for record in records {
                    refresh_item(record);
                }
                aggregate_catalog_entry(entry);
            } else {
                refresh_item(entry);
            }
        }
    }
    run["updated_at"] = json!(crate::now_iso_utc());
    update_run_state(run);
}
fn migrate_legacy_catalog_jobs(run: &mut Value) {
    let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) else {
        return;
    };
    for entry in entries {
        let has_records = entry
            .get("records")
            .and_then(Value::as_array)
            .is_some_and(|records| !records.is_empty());
        let Some(job_id) = entry.get("job_id").and_then(Value::as_str).map(str::to_string) else {
            continue;
        };
        if has_records {
            continue;
        }
        let suffix = job_id
            .chars()
            .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
            .collect::<String>();
        let record = format!("legacy-job-{suffix}");
        entry["records"] = json!([{
            "record": record,
            "state": entry.get("state").cloned().unwrap_or_else(|| json!("unknown")),
            "stado_job_id": job_id,
            "attempt_id": format!("legacy-attempt-{suffix}"),
            "artifact_uri": entry.get("artifact_uri").cloned().unwrap_or(Value::Null),
            "output_uri": entry.get("output_uri").cloned().unwrap_or(Value::Null),
            "submission_receipt": entry.get("submission_receipt").cloned().unwrap_or(Value::Null),
            "diagnostic": {
                "code": "legacy_catalog_attempt_migrated",
                "message": "legacy catalog-level job retained as one explicit synthetic record attempt"
            },
            "attempts": [],
        }]);
    }
}

fn machine_state(job: &Value) -> &str {
    job.get("state")
        .or_else(|| job.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
}

fn terminal_machine_state(state: &str) -> bool {
    matches!(
        state,
        "completed"
            | "succeeded"
            | "failed"
            | "cancelled"
            | "canceled"
            | "lost"
            | "reaped"
    )
}

fn publish_cancel_intent(uri: &str, intent: &Value) -> Result<String> {
    let bytes = serde_json::to_vec(intent)?;
    let digest = crate::sha256_hex(&bytes);
    let home = std::env::var_os("HOME").context("HOME is required for private Stado work state")?;
    let directory = PathBuf::from(home)
        .join(".stado")
        .join("work")
        .join("spis")
        .join("cancel-intents");
    std::fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    }
    let source = directory.join(format!("{digest}.json"));
    if source.is_file() {
        if std::fs::read(&source)? != bytes {
            bail!("private cancel-intent cache conflicts with its content digest");
        }
    } else {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let temporary = directory.join(format!(".{digest}.{}.{}.tmp", std::process::id(), nonce));
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
        }
        std::fs::rename(&temporary, &source)?;
        File::open(&directory)?.sync_all()?;
    }
    let stored = crawl_storage_command()
        .args(["storage", "put", "--if-absent", "--content-type", "application/json", uri])
        .arg(&source)
        .output()
        .context("persist immutable crawl cancel intent")?;
    if !stored.status.success() {
        bail!(
            "Stado refused immutable cancel intent: {}",
            String::from_utf8_lossy(&stored.stderr).trim()
        );
    }
    let readback = directory.join(format!(".{digest}.{}.readback", std::process::id()));
    let output = crawl_storage_command()
        .args(["storage", "get", uri])
        .arg(&readback)
        .output()
        .context("read back immutable crawl cancel intent")?;
    if !output.status.success() {
        let _ = std::fs::remove_file(&readback);
        bail!(
            "cancel intent read-back failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let recovered = std::fs::read(&readback)?;
    let _ = std::fs::remove_file(&readback);
    if recovered != bytes || crate::sha256_hex(&recovered) != digest {
        bail!("cancel intent read-back differs from the exact requested cancellation");
    }
    Ok(digest)
}

fn cancel(rest: &[String]) -> Result<()> {
    let mut run_id = None;
    let mut selected_record = None;
    let mut reason = None;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--run" => {
                index += 1;
                run_id = Some(rest.get(index).context("--run needs a value")?.clone());
            }
            "--record" => {
                index += 1;
                selected_record = Some(rest.get(index).context("--record needs a value")?.clone());
            }
            "--reason" => {
                index += 1;
                reason = Some(rest.get(index).context("--reason needs a value")?.clone());
            }
            value => bail!("unknown argument: {value}"),
        }
        index += 1;
    }
    let run_id = run_id.context("--run is required")?;
    let reason = reason
        .filter(|value| !value.trim().is_empty() && value.len() <= 1024)
        .context("--reason must be nonempty and at most 1024 bytes")?;
    {
        let _guard = RunMutationGuard::acquire(&run_id)?;
        let mut run = load(Some(&run_id))?;
        let before = run.clone();
        migrate_legacy_catalog_jobs(&mut run);
        if run != before {
            persist(&mut run)?;
        }
    }
    let snapshot = load(Some(&run_id))?;
    let mut targets = Vec::new();
    for catalog in snapshot
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let catalog_name = catalog
            .get("catalog")
            .and_then(Value::as_str)
            .unwrap_or_default();
        for record in catalog
            .get("records")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let record_name = record
                .get("record")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if selected_record.as_deref().is_none_or(|wanted| {
                record_name == wanted
                    || record_name.split_once('-').map(|(_, tail)| tail) == Some(wanted)
            }) {
                targets.push((catalog_name.to_string(), record_name.to_string()));
            }
        }
    }
    if targets.is_empty() {
        bail!("no crawl record matches the cancellation selection");
    }
    if selected_record.is_some() && targets.len() != 1 {
        bail!("--record must resolve to exactly one retained crawl record");
    }
    for (catalog_name, record_name) in targets {
        let _record_guard =
            RecordMutationGuard::acquire(&run_id, &catalog_name, &record_name)?;
        let current = load(Some(&run_id))?;
        let record = current
            .get("catalogs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|catalog| {
                catalog.get("catalog").and_then(Value::as_str) == Some(catalog_name.as_str())
            })
            .and_then(|catalog| catalog.get("records"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|record| {
                record.get("record").and_then(Value::as_str) == Some(record_name.as_str())
            })
            .cloned()
            .context("crawl record disappeared during cancellation")?;
        let manifest = record
            .get("manifest")
            .filter(|value| value.is_object())
            .cloned();
        let attempt_id = manifest
            .as_ref()
            .and_then(|value| value.get("attempt_id"))
            .or_else(|| record.get("attempt_id"))
            .and_then(Value::as_str)
            .context("crawl record has no attempt id to cancel")?
            .to_string();
        let attempt = manifest
            .as_ref()
            .and_then(|value| value.get("attempt"))
            .and_then(Value::as_u64)
            .or_else(|| record.get("attempt").and_then(Value::as_u64))
            .context("crawl record has no attempt number to cancel")?;
        let job_id = record
            .get("stado_job_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let stado_run_id = manifest
            .as_ref()
            .and_then(|value| value.get("stado_run_id"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let record_key = manifest
            .as_ref()
            .and_then(|value| value.get("record_key"))
            .cloned()
            .unwrap_or(Value::Null);
        let intent = json!({
            "schema": "wisent.crawl-cancel-intent.v1",
            "run_id": run_id,
            "catalog": catalog_name,
            "record": record_name,
            "record_key": record_key,
            "attempt": attempt,
            "attempt_id": attempt_id,
            "stado_run_id": stado_run_id,
            "stado_job_id": job_id,
            "reason": reason,
        });
        // Persist the durable local intent BEFORE any external effect. Every
        // submission path in `continue_record` refuses to submit, and refuses to
        // leave a submitted job running, once this object exists — so a crash
        // between here and the Stado cancellation can never resurrect the record.
        mutate_record(&run_id, &catalog_name, &record_name, |entry| {
            if !entry.get("cancel_intent").is_some_and(Value::is_object) {
                entry["cancel_intent"] = intent.clone();
            }
            if matches!(
                entry.get("state").and_then(Value::as_str),
                Some("planned" | "preflighting" | "preflight_passed" | "unavailable")
            ) {
                entry["state"] = json!("cancelled");
                entry["diagnostic"] = json!({
                    "code": "cancelled_before_submission",
                    "message": "durable cancel intent recorded before any submission",
                });
            }
            Ok(())
        })?;
        let intent = record_snapshot(&run_id, &catalog_name, &record_name)?
            .get("cancel_intent")
            .cloned()
            .context("durable cancel intent disappeared after persistence")?;
        // The canonical attempt coordinate is the artifact URI's parent; a record
        // with no manifest has no attempt to cancel and was rejected above.
        let base_uri = manifest
            .as_ref()
            .and_then(|value| value.get("artifact_uri"))
            .and_then(Value::as_str)
            .and_then(|uri| uri.rsplit_once('/').map(|(parent, _)| parent.to_string()))
            .context("crawl record has no canonical attempt artifact coordinate")?;
        let intent_uri = format!("{base_uri}/cancel-intent.json");
        let intent_sha256 = publish_cancel_intent(&intent_uri, &intent)?;
        let job_id = job_id.as_deref();
        let status_before = match job_id {
            Some(job_id) => match machine_status(job_id) {
                Ok(job) => json!({"state": machine_state(&job), "job": job}),
                Err(error) => json!({
                    "state": if error.not_found { "not_found" } else { "lookup_failed" },
                    "diagnostic": error.diagnostic,
                }),
            },
            None => json!({"state": "not_submitted"}),
        };
        let before_state = status_before
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("lookup_failed");
        let (action, status_after) = if let Some(job_id) = job_id {
            if matches!(before_state, "queued" | "running") {
                let output = stado_command()
                    .args(["machine", "cancel", job_id])
                    .output()
                    .context("cancel Stado machine job")?;
                if !output.status.success() {
                    (
                        json!({
                            "state": "cancel_failed",
                            "stderr": String::from_utf8_lossy(&output.stderr).trim(),
                        }),
                        status_before.clone(),
                    )
                } else {
                    let after = machine_status(job_id)
                        .map(|job| json!({"state": machine_state(&job), "job": job}))
                        .unwrap_or_else(|error| {
                            json!({
                                "state": if error.not_found { "not_found" } else { "lookup_failed" },
                                "diagnostic": error.diagnostic,
                            })
                        });
                    (json!({"state": "cancel_dispatched"}), after)
                }
            } else if terminal_machine_state(before_state) {
                (json!({"state": "already_terminal"}), status_before.clone())
            } else {
                (json!({"state": "not_cancellable"}), status_before.clone())
            }
        } else {
            (json!({"state": "cancelled_before_submission"}), status_before.clone())
        };
        let result = json!({
            "schema": "wisent.crawl-cancel-result.v1",
            "intent_uri": intent_uri,
            "intent_sha256": intent_sha256,
            "reason": reason,
            "status_before": status_before,
            "stado_action": action,
            "status_after": status_after,
            // The coordinator holds only secret *references*; the bearer is
            // injected by Stado into the pinned worker. Cancelling the Stado job
            // therefore terminates the process that owns the Weles task, and that
            // is the authoritative boundary rather than a coordinator-side API call.
            "weles_action": match record.get("weles_task_id").and_then(Value::as_str) {
                Some(task_id) => json!({
                    "state": "stado_job_cancellation_is_authoritative",
                    "weles_task_id": task_id,
                    "diagnostic": "the inner Weles task is owned by the cancelled Stado worker; the coordinator holds no admission bearer",
                }),
                None => json!({"state": "no_retained_task_id"}),
            },
        });
        let _guard = RunMutationGuard::acquire(&run_id)?;
        let mut run = load(Some(&run_id))?;
        let target = run
            .get_mut("catalogs")
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
            .find(|catalog| {
                catalog.get("catalog").and_then(Value::as_str) == Some(catalog_name.as_str())
            })
            .and_then(|catalog| catalog.get_mut("records"))
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
            .find(|record| {
                record.get("record").and_then(Value::as_str) == Some(record_name.as_str())
            })
            .context("crawl record disappeared before cancellation result persistence")?;
        target["cancel"] = result;
        let final_state = target
            .pointer("/cancel/status_after/state")
            .and_then(Value::as_str)
            .unwrap_or("lookup_failed");
        if matches!(final_state, "cancelled" | "canceled")
            || target.pointer("/cancel/stado_action/state").and_then(Value::as_str)
                == Some("cancelled_before_submission")
        {
            target["state"] = json!("cancelled");
        }
        if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
            for entry in entries {
                aggregate_catalog_entry(entry);
            }
        }
        update_run_state(&mut run);
        persist(&mut run)?;
    }
    let run = load(Some(&run_id))?;
    print_operation("cancel", &run, selected_record.as_deref())
}

fn update_run_state(run: &mut Value) {
    let states: Vec<&str> = run
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("state").and_then(Value::as_str))
        .collect();
    let state = if states.iter().any(|state| *state == "running") {
        "running"
    } else if states.iter().any(|state| *state == "cancel_pending") {
        "cancel_pending"
    } else if states.iter().any(|state| *state == "pending_review") {
        "pending_review"
    } else if states.iter().any(|state| *state == "queued") {
        "queued"
    } else if states.iter().any(|state| *state == "planned") {
        "planned"
    } else if states.iter().all(|state| *state == "imported") && !states.is_empty() {
        "imported"
    } else if states
        .iter()
        .all(|state| matches!(*state, "completed" | "uploaded" | "imported"))
        && !states.is_empty()
    {
        "completed"
    } else if states
        .iter()
        .any(|state| matches!(*state, "partial" | "completed" | "uploaded" | "imported"))
    {
        "partial"
    } else {
        "failed"
    };
    let partial = states.iter().any(|state| *state == "partial");
    run["state"] = json!(state);
    run["partial"] = json!(partial);
}

fn status(rest: &[String]) -> Result<()> {
    let (run_id, record) = parse_run_and_record(rest, false)?;
    let mut selected = load(run_id.as_deref())?;
    let selected_id = selected.get("run_id").and_then(Value::as_str).context("run has no id")?.to_string();
    match RunMutationGuard::acquire(&selected_id) {
        Ok(_guard) => {
            selected = load(Some(&selected_id))?;
            refresh(&mut selected);
            persist(&mut selected)?;
        }
        Err(error) => {
            selected["status_refresh"] = json!({
                "state": "read_only_snapshot",
                "diagnostic": error.to_string(),
            });
        }
    }
    print_operation("status", &selected, record.as_deref())
}

fn parse_run_and_record(rest: &[String], require_run: bool) -> Result<(Option<String>, Option<String>)> {
    let mut run = None;
    let mut record = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--run" => { i += 1; run = Some(rest.get(i).context("--run needs a value")?.clone()); }
            "--record" => { i += 1; record = Some(rest.get(i).context("--record needs a value")?.clone()); }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    if require_run && run.is_none() { bail!("--run is required"); }
    Ok((run, record))
}

/// Re-arm one terminal-after-acceptance record as attempt N+1.
///
/// A `failed`, `cancelled`, `lost`, `submission_failed` or `preflight_failed`
/// attempt is immutable history. Resumption never reruns the old Stado job:
/// it increments the attempt, drops the previous execution identity, and
/// recomputes every derived identity value — input digest, catalog and record
/// keys, attempt id, correlation id, Stado run id and both attempt URIs — so the
/// next submission is a genuinely distinct idempotent attempt. `queued`,
/// `running`, `submitting`, `preflight_passed`, `cancel_pending`,
/// `pending_review`, `completed`, `uploaded` and `imported` are left untouched.
fn rearm_record_attempt(
    run_id: &str,
    catalog: &str,
    record: &str,
) -> Result<Option<u32>> {
    let mut rearmed = None;
    mutate_record(run_id, catalog, record, |entry| {
        let state = entry
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if !matches!(
            state.as_str(),
            "failed" | "cancelled" | "lost" | "submission_failed" | "preflight_failed"
        ) {
            return Ok(());
        }
        if entry.get("cancel_intent").is_some_and(Value::is_object) {
            entry["diagnostic"] = json!({
                "code": "cancel_intent_blocks_resume",
                "message": "a durable cancel intent exists; this record will not be re-armed",
            });
            return Ok(());
        }
        let mut manifest: RuntimeManifest =
            serde_json::from_value(entry.get("manifest").cloned().unwrap_or(Value::Null))
                .context("terminal record retains no typed runtime manifest to re-arm")?;
        manifest.attempt = manifest
            .attempt
            .checked_add(1)
            .context("record has exhausted the attempt counter")?;
        manifest.execution_identity = None;
        let reference = reference_path(&manifest.catalog, &manifest.record)?;
        let bytes = std::fs::read(&reference)
            .with_context(|| format!("read committed record {}", reference.display()))?;
        finalize_manifest_identity(&mut manifest, &bytes)?;
        entry["manifest"] = serde_json::to_value(&manifest)?;
        entry["attempt"] = json!(manifest.attempt);
        entry["attempt_id"] = json!(manifest.attempt_id);
        entry["artifact_uri"] = json!(manifest.artifact_uri);
        entry["output_uri"] = json!(manifest.output_uri);
        entry["state"] = json!("planned");
        for cleared in [
            "command",
            "stado_job_id",
            "submission_receipt",
            "submission_transition",
            "preflight",
            "job",
            "lookup_error",
            "cancel",
            "cancel_result",
            "error",
            "import",
        ] {
            entry[cleared] = Value::Null;
        }
        entry["diagnostic"] = json!({
            "code": "attempt_rearmed",
            "message": format!(
                "terminal {state} attempt retained in history; attempt {} planned with fresh identity",
                manifest.attempt
            ),
        });
        rearmed = Some(manifest.attempt);
        Ok(())
    })?;
    Ok(rearmed)
}

fn resume(rest: &[String]) -> Result<()> {
    let (run_id, selected_record) = parse_run_and_record(rest, true)?;
    let run_id = run_id.context("--run is required")?;
    {
        let _guard = RunMutationGuard::acquire(&run_id)?;
        let mut run = load(Some(&run_id))?;
        migrate_legacy_catalog_jobs(&mut run);
        refresh(&mut run);
        persist(&mut run)?;
    }
    let snapshot = load(Some(&run_id))?;
    let original = snapshot
        .get("source_revision")
        .and_then(Value::as_str)
        .context("run has no source_revision")?
        .to_string();
    let current = build_revision()?;
    if original != current {
        bail!(
            "run {run_id} belongs to Spis revision {original}; resumption with revision {current} is refused"
        );
    }
    let mut targets = Vec::new();
    for catalog in snapshot
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let catalog_name = catalog
            .get("catalog")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        for record in catalog
            .get("records")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let record_name = record
                .get("record")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if selected_matches(&record_name, selected_record.as_deref()) {
                targets.push((catalog_name.clone(), record_name));
            }
        }
    }
    if targets.is_empty() {
        bail!("no crawl record matches the resume selection");
    }
    for (catalog, record) in &targets {
        match RecordMutationGuard::acquire(&run_id, catalog, record) {
            Ok(_guard) => {
                rearm_record_attempt(&run_id, catalog, record)?;
            }
            Err(error) if error.downcast_ref::<RecordLockBusy>().is_some() => continue,
            Err(error) => return Err(error),
        }
    }
    continue_start(&run_id)?;
    let mut run = import_ready(&run_id, selected_record.as_deref())?;
    print_operation("resume", &run, selected_record.as_deref())?;
    update_run_state(&mut run);
    if has_failures(&run) {
        bail!("one or more crawl records remain unresumable");
    }
    Ok(())
}

fn selected_matches(record: &str, selected: Option<&str>) -> bool {
    selected.is_none_or(|wanted| {
        record == wanted || record.split_once('-').map(|(_, tail)| tail) == Some(wanted)
    })
}

const MAX_WORKER_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;
const MAX_EXTRACTED_ENTRIES: usize = 20_000;
const MAX_EXTRACTED_BYTES: u64 = 512 * 1024 * 1024;

fn download_uri(uri: &str, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match std::fs::symlink_metadata(destination) {
        Ok(_) => std::fs::remove_file(destination)
            .with_context(|| format!("clear stale download {}", destination.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let mut command = crawl_storage_command();
    command.args(["storage", "get", uri]).arg(destination);
    let output = bounded_command_output(
        &mut command,
        "download retained crawl object",
        Duration::from_secs(600),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "download {uri}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// Extract one crawl attempt archive into an empty staging directory.
///
/// Only ordinary files and directories are accepted. Absolute paths, `..`
/// components, symlinks, hard links, devices, duplicate members and archives
/// beyond the entry/byte bounds are refused, and every member is created with
/// `create_new` so a pre-existing path can never be followed or overwritten.
fn extract_attempt_archive(archive: &Path, destination: &Path) -> Result<Vec<String>> {
    if destination.exists() {
        std::fs::remove_dir_all(destination)?;
    }
    std::fs::create_dir_all(destination)?;
    let mut tar = tar::Archive::new(GzDecoder::new(File::open(archive)?));
    let mut entries = 0_usize;
    let mut total = 0_u64;
    let mut extracted = Vec::new();
    for member in tar.entries()? {
        let mut member = member?;
        let kind = member.header().entry_type();
        let relative = member.path()?.into_owned();
        if relative
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            bail!(
                "crawl attempt archive contains the unsafe path {}",
                relative.display()
            );
        }
        let target = destination.join(&relative);
        if kind.is_dir() {
            std::fs::create_dir_all(&target)?;
            continue;
        }
        if !kind.is_file() {
            bail!(
                "crawl attempt archive member {} is not a regular file",
                relative.display()
            );
        }
        entries += 1;
        if entries > MAX_EXTRACTED_ENTRIES {
            bail!("crawl attempt archive exceeds the {MAX_EXTRACTED_ENTRIES}-entry bound");
        }
        let size = member.header().size()?;
        total = total
            .checked_add(size)
            .filter(|value| *value <= MAX_EXTRACTED_BYTES)
            .context("crawl attempt archive exceeds the extracted byte bound")?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .with_context(|| format!("extract crawl member {}", relative.display()))?;
        let written = std::io::copy(&mut member.by_ref().take(size), &mut file)?;
        if written != size {
            bail!(
                "crawl attempt archive member {} is truncated",
                relative.display()
            );
        }
        file.sync_all()?;
        extracted.push(
            relative
                .to_str()
                .context("crawl attempt archive member name is not UTF-8")?
                .to_string(),
        );
    }
    extracted.sort();
    Ok(extracted)
}

fn fsync_tree(root: &Path) -> Result<()> {
    let mut directories = vec![root.to_path_buf()];
    let mut index = 0;
    while index < directories.len() {
        let directory = directories[index].clone();
        index += 1;
        for entry in std::fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)?;
            if metadata.is_dir() {
                directories.push(path);
            } else if metadata.is_file() {
                File::open(&path)?.sync_all()?;
            }
        }
    }
    for directory in directories.iter().rev() {
        File::open(directory)?.sync_all()?;
    }
    Ok(())
}

/// Atomically install a fully staged, fsynced tree over `destination`.
///
/// The staged tree is durable before the rename, the previous tree is moved
/// aside rather than deleted in place, and the parent directory is fsynced, so a
/// crash always leaves either the complete previous tree or the complete new
/// one — never a half-copied record.
fn install_staged_tree(staged: &Path, destination: &Path) -> Result<()> {
    let parent = destination
        .parent()
        .context("import destination has no parent")?;
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .context("import destination has no UTF-8 name")?;
    std::fs::create_dir_all(parent)?;
    fsync_tree(staged)?;
    let superseded = parent.join(format!(".{name}.superseded"));
    if superseded.exists() {
        std::fs::remove_dir_all(&superseded)?;
    }
    if destination.exists() {
        std::fs::rename(destination, &superseded)?;
    }
    std::fs::rename(staged, destination)?;
    File::open(parent)?.sync_all()?;
    if superseded.exists() {
        std::fs::remove_dir_all(&superseded)?;
    }
    Ok(())
}

fn worker_report_schema(engine: &str) -> &'static str {
    match engine {
        "web" => "wisent.web-worker-report.v1",
        "docs" => "wisent.docs-worker-report.v1",
        _ => "wisent.native-worker-report.v1",
    }
}

/// Read the exact typed worker report out of one attempt's retained output log.
///
/// The importer accepts only the engine's declared report schema on its own
/// line. There is no `command_output.log` fallback and no heuristic scan, so a
/// worker that failed to print its typed report is an import failure rather than
/// an invitation to guess.
fn retained_worker_report(engine: &str, output_log: &Path) -> Result<Value> {
    let metadata = std::fs::symlink_metadata(output_log)
        .with_context(|| format!("read retained worker output {}", output_log.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!(
            "retained worker output {} is not a regular file",
            output_log.display()
        );
    }
    if metadata.len() > MAX_WORKER_OUTPUT_BYTES {
        bail!(
            "retained worker output {} exceeds the {MAX_WORKER_OUTPUT_BYTES}-byte bound",
            output_log.display()
        );
    }
    let bytes = std::fs::read(output_log)?;
    let text = String::from_utf8_lossy(&bytes);
    let schema = worker_report_schema(engine);
    text.lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .find(|value| value.get("schema").and_then(Value::as_str) == Some(schema))
        .with_context(|| format!("retained worker output carries no {schema} report"))
}

/// Prove the worker report describes exactly this attempt.
///
/// Every identity field must equal the immutable manifest, the artifact URI must
/// be the canonical attempt coordinate, and the retained Stado submission
/// receipt must bind the same job and source revision.
///
/// What this document is NOT: content-addressed. Unlike the evidence manifest, the
/// provenance document and every retained artifact, the report has no digest committed
/// anywhere before it is read — `retained_worker_report` takes the last matching line of
/// the worker's own output log, and the artifact digest it declares is proved against the
/// bytes in durable storage rather than against an independently signed value. What the
/// report carries is therefore trusted only as far as something else re-proves it: the
/// checks below bind it to the immutable attempt and to the retained submission receipt,
/// the archive is re-hashed, and every field of the attempt envelope that matters is
/// re-compared against the SIGNED receipt claims by
/// `weles_provenance::verify_attempt_binding` at record-verification time. Nothing here
/// may be read as proof of a fact that no signature or digest covers.
fn verify_worker_report(
    report: &Value,
    manifest: &RuntimeManifest,
    entry: &Value,
    receipt: &Value,
    // `artifact_published` for the accepted attempt, `failed` for a non-success attempt
    // whose signed failure proof is being imported. Everything else this function proves is
    // identical for both, so neither path gets its own weaker identity rules.
    expected_state: &str,
) -> Result<Value> {
    let expected_strings = [
        ("run_id", manifest.run_id.as_str()),
        ("catalog", manifest.catalog.as_str()),
        ("record", manifest.record.as_str()),
        ("record_key", manifest.record_key.as_str()),
        ("attempt_id", manifest.attempt_id.as_str()),
        ("engine", manifest.engine.as_str()),
        ("source_revision", manifest.source_revision.as_str()),
        ("source_input_sha256", manifest.source_input_sha256.as_str()),
        ("reference_sha256", manifest.reference_sha256.as_str()),
        (
            "bindings_file_sha256",
            manifest.bindings_file_sha256.as_str(),
        ),
        ("bindings_sha256", manifest.bindings_sha256.as_str()),
    ];
    for (field, expected) in expected_strings {
        let observed = report.get(field).and_then(Value::as_str);
        if observed != Some(expected) {
            bail!(
                "worker report {field} is {observed:?} but the immutable attempt declares {expected:?}"
            );
        }
    }
    if report.get("attempt").and_then(Value::as_u64) != Some(u64::from(manifest.attempt)) {
        bail!("worker report attempt differs from the immutable attempt");
    }
    if report.get("state").and_then(Value::as_str) != Some(expected_state) {
        bail!(
            "worker report state is {:?}, not the {expected_state} state this import requires",
            report.get("state")
        );
    }
    let artifact = report
        .get("artifact")
        .filter(|value| value.is_object())
        .cloned()
        .context("worker report has no typed published artifact")?;
    if artifact.get("uri").and_then(Value::as_str) != Some(manifest.artifact_uri.as_str()) {
        bail!("worker report artifact URI is not the canonical attempt coordinate");
    }
    let sha256 = artifact
        .get("sha256")
        .and_then(Value::as_str)
        .filter(|value| is_lower_sha256(value))
        .context("worker report artifact has no lowercase SHA-256 digest")?;
    let bytes = artifact
        .get("bytes")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .context("worker report artifact has no positive byte count")?;
    let job_id = entry
        .get("stado_job_id")
        .and_then(Value::as_str)
        .context("imported record retains no Stado job id")?;
    if receipt.get("stado_job_id").and_then(Value::as_str) != Some(job_id) {
        bail!("retained submission receipt names a different Stado job");
    }
    if receipt
        .pointer("/stado_receipt/source_revision")
        .and_then(Value::as_str)
        != Some(manifest.source_revision.as_str())
    {
        bail!("retained submission receipt does not bind the attempt source revision");
    }
    if receipt
        .pointer("/stado_receipt/jobs/0/output_uri")
        .and_then(Value::as_str)
        != Some(manifest.output_uri.as_str())
    {
        bail!("retained submission receipt output URI is not the canonical attempt coordinate");
    }
    Ok(json!({"sha256": sha256, "bytes": bytes, "artifact": artifact}))
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    files
}

fn media_kind(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" => Some("state"),
        "gif" | "mp4" | "webm" | "cast" => Some("motion"),
        _ => None,
    }
}

fn declared_motion_kind(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mp4") => "video-mp4",
        Some("webm") => "video-webm",
        Some("gif") => "animated-gif",
        Some("webp") => "animated-webp",
        Some("cast") => "terminal-cast",
        _ => "unknown",
    }
}

fn capture_method(engine: &str) -> &'static str {
    match engine {
        "mobile" => "Local product run through Appium with XCUITest or UiAutomator2; screen recording and accessibility source retained",
        "desktop" => "Local product run through Cua Driver; snapshot-bound actions, screenshots, action recording and accessibility tree retained",
        "web" => "Real browser session executed by Weles on a Stado-pinned host; signed receipt, evidence manifest, screenshot and accessibility tree retained",
        "tui" => "Local product run in an isolated tmux pseudo-terminal; raw terminal bytes and distinct screens retained",
        "cli" => "Local product run of the real executable in an isolated tmux pseudo-terminal; stdout/stderr, argv and exit status retained",
        "docs" => "Rate-limited full-text documentation crawl; per-site gzipped JSONL corpus retained",
        _ => "Unclassified Spis crawl",
    }
}

/// Retained media descriptors for one attempt, addressed relative to the record.
fn attempt_media(
    engine: &str,
    attempt_dir: &Path,
    record_dir: &Path,
    source_url: &str,
) -> Result<(Vec<Value>, Vec<Value>)> {
    let mut motion = Vec::new();
    let mut states = Vec::new();
    let files = files_under(attempt_dir);
    let first_motion = files
        .iter()
        .find(|path| media_kind(path) == Some("motion"))
        .and_then(|path| path.strip_prefix(record_dir).ok())
        .map(|relative| relative.to_string_lossy().to_string());
    for path in &files {
        let Some(kind) = media_kind(path) else {
            continue;
        };
        let relative = path
            .strip_prefix(record_dir)
            .context("retained media escaped the record directory")?
            .to_string_lossy()
            .to_string();
        let bytes = std::fs::read(path)?;
        if kind == "motion" {
            motion.push(json!({
                "local_path": relative,
                "sha256": crate::sha256_hex(&bytes),
                "bytes": bytes.len(),
                "source_url": source_url,
                "media_kind": declared_motion_kind(path),
                "capture_method": capture_method(engine),
            }));
        } else {
            states.push(json!({
                "name": format!("Observed {relative}"),
                "local_path": relative,
                "sha256": crate::sha256_hex(&bytes),
                "bytes": bytes.len(),
                "source_motion_path": first_motion,
            }));
        }
    }
    Ok((motion, states))
}

fn accessibility_gap(engine: &str, attempt_dir: &Path) -> Value {
    let trees: Vec<PathBuf> = files_under(attempt_dir)
        .into_iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("xml" | "html" | "txt")
            ) || matches!(
                path.file_name().and_then(|value| value.to_str()),
                Some("snapshot.json" | "source.json" | "axe.json")
            )
        })
        .collect();
    let bytes: u64 = trees
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok().map(|value| value.len()))
        .sum();
    json!({
        "measured": false,
        "observations": if trees.is_empty() {
            Vec::new()
        } else {
            vec![format!(
                "Retained {} accessibility or DOM source files totalling {bytes} bytes from the {engine} attempt.",
                trees.len()
            )]
        },
        "unknowns": [
            "No engine-supplied canonical accessibility measurement was retained.",
            "Screen-reader traversal, focus order, live regions and reduced-motion preference remain unmeasured.",
        ],
    })
}

/// One durable `crawl_runs` entry, keyed by the immutable attempt id.
fn crawl_run_entry(
    manifest: &RuntimeManifest,
    entry: &Value,
    report: &Value,
    artifact: &Value,
    relative_report: &str,
) -> Value {
    let mut run = json!({
        "schema": "wisent.crawl-import.v2",
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "record_key": manifest.record_key,
        "attempt": manifest.attempt,
        "attempt_id": manifest.attempt_id,
        "engine": manifest.engine,
        "state": "completed",
        "outcome": "completed",
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "reference_sha256": manifest.reference_sha256,
        "bindings_file_sha256": manifest.bindings_file_sha256,
        "bindings_sha256": manifest.bindings_sha256,
        "stado_job_id": entry.get("stado_job_id").cloned().unwrap_or(Value::Null),
        "stado_run_id": manifest.stado_run_id,
        "artifact_uri": manifest.artifact_uri,
        "artifact_sha256": artifact.get("sha256").cloned().unwrap_or(Value::Null),
        "artifact_bytes": artifact.get("bytes").cloned().unwrap_or(Value::Null),
        "output_uri": manifest.output_uri,
        "worker_report": relative_report,
        "capture_method": capture_method(&manifest.engine),
        "imported_at": crate::now_iso_utc(),
    });
    if let Some(execution) = report.get("execution_identity") {
        run["execution_identity"] = execution.clone();
    }
    run
}

/// Write one immutable retained document, or prove the existing bytes are identical.
///
/// Every path handled here is content-addressed or digest-verified, so a second
/// attempt that legitimately retains the same object must find the same bytes. A
/// staged temporary plus rename keeps the destination either absent or complete.
fn write_immutable_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("retained document has no parent")?;
    std::fs::create_dir_all(parent)?;
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            bail!("retained document {} is not a regular file", path.display());
        }
        Ok(_) => {
            if std::fs::read(path)? != bytes {
                bail!(
                    "retained document {} already exists with different content",
                    path.display()
                );
            }
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .context("retained document has no UTF-8 name")?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let staged = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&staged)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&staged, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

/// Merge one immutable retained subtree into the record without disturbing the
/// objects earlier attempts still reference.
fn merge_immutable_tree(source: &Path, destination: &Path) -> Result<()> {
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        let metadata = std::fs::symlink_metadata(&from)?;
        if metadata.file_type().is_symlink() {
            bail!("retained crawl content {} is a symbolic link", from.display());
        }
        if metadata.is_dir() {
            merge_immutable_tree(&from, &to)?;
        } else if metadata.is_file() {
            write_immutable_file(&to, &std::fs::read(&from)?)?;
        } else {
            bail!("retained crawl content {} is not a regular file", from.display());
        }
    }
    Ok(())
}

/// Copy the typed Weles attempt facts into the crawl-run entry and the record.
///
/// `verify_attempt_binding` in the receipt verifier compares every one of these
/// outer fields with the inner `weles_attempt_envelope`, so they are written from
/// the envelope itself rather than restated.
fn apply_web_attempt(
    record: &mut Value,
    run: &mut Value,
    report: &Value,
    attempt_dir: &Path,
    record_dir: &Path,
    // True only for the accepted, completed attempt. A non-success attempt is imported so
    // that its signed failure proof is re-verified with the record, and it must never
    // write the record-level evidence inventory: see the comment at that write for what
    // this guard does and does not decide.
    confirms_record: bool,
) -> Result<()> {
    let envelope = report
        .get("weles_attempt_envelope")
        .filter(|value| value.is_object())
        .context("web worker report has no typed Weles attempt envelope")?;
    let envelope: crate::weles_provenance::WelesAttemptEnvelope =
        serde_json::from_value(envelope.clone())
            .context("web worker report envelope does not match the typed schema")?;
    let manifest_sha256 = envelope
        .weles_evidence_manifest_sha256
        .clone()
        .context("web attempt envelope has no evidence manifest digest")?;
    let artifact_sha256 = envelope
        .artifact_document_sha256
        .clone()
        .context("web attempt envelope has no artifact document digest")?;
    run["weles_task_id"] = json!(envelope.weles_task_id);
    run["weles_request_digest"] = json!(envelope.weles_request_digest);
    run["weles_result_digest"] = json!(envelope.weles_result_digest);
    run["weles_evidence_manifest_uri"] = json!(envelope.weles_evidence_manifest_uri);
    run["weles_evidence_manifest_sha256"] = json!(manifest_sha256);
    run["artifact_document_uri"] = json!(envelope.artifact_document_uri);
    run["artifact_document_sha256"] = json!(artifact_sha256);
    run["observation_document_uri"] = json!(envelope.observation_document_uri);
    run["observation_document_sha256"] = json!(envelope.observation_document_sha256);
    run["requested_url"] = json!(envelope.requested_url);
    run["final_url"] = json!(envelope.final_url);
    run["state"] = json!(envelope.state);
    run["outcome"] = json!(envelope.outcome);
    run["weles_attempt_envelope"] = serde_json::to_value(&envelope)?;
    // The verifier resolves `artifact.path` and every inventory tail relative to the
    // RECORD directory, so the attempt's content-addressed `weles/` documents and its
    // task-scoped `recordings/` tree must exist there, merged rather than replaced:
    // earlier attempts' provenance documents still point at their own digests.
    //
    // This is why every object a worker places in these two subtrees is addressed by its
    // own content or by its Weles task, never by its role: `write_immutable_file` refuses
    // a name that already holds different bytes, so one role-named document here would
    // permanently block the second import of the same record. Operational documents that
    // differ per attempt by construction stay in the attempt root instead, and reach the
    // record through the attempt-private `crawl/{attempt_id}` tree installed above.
    for subtree in ["weles", "recordings"] {
        let source = attempt_dir.join(subtree);
        if source.is_dir() {
            merge_immutable_tree(&source, &record_dir.join(subtree))?;
        }
    }
    let provenance = report
        .get("provenance_document")
        .filter(|value| value.is_object())
        .context("web worker report has no official provenance document")?;
    let provenance: crate::weles_provenance::WelesProvenanceDocument =
        serde_json::from_value(provenance.clone())
            .context("web provenance document does not match the typed schema")?;
    let provenance_id = provenance
        .id
        .strip_prefix("sha256:")
        .filter(|value| is_lower_sha256(value))
        .context("official provenance document has no framed sha256: identifier")?
        .to_string();
    let provenance_relative = format!("weles/provenance/{provenance_id}.json");
    let provenance_bytes = serde_json::to_vec_pretty(&provenance)?;
    write_immutable_file(&record_dir.join(&provenance_relative), &provenance_bytes)?;
    let reference = crate::weles_provenance::WelesProvenanceDocumentRef {
        schema: crate::weles_provenance::PROVENANCE_DOCUMENT_REF_SCHEMA.to_string(),
        path: provenance_relative,
        sha256: crate::sha256_hex(&provenance_bytes),
    };
    let references = record
        .as_object_mut()
        .context("reference record is not an object")?
        .entry("provenance_documents")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .context("provenance_documents is not a list")?;
    let reference = serde_json::to_value(&reference)?;
    if let Some(existing) = references.iter_mut().find(|value| {
        value.get("path").and_then(Value::as_str) == reference.get("path").and_then(Value::as_str)
    }) {
        *existing = reference;
    } else {
        references.push(reference);
    }
    let inventory: Vec<Value> = envelope
        .evidence_inventory
        .iter()
        .map(|item| {
            let tail = item
                .uri
                .strip_prefix(&format!(
                    "stado://weles/recordings/{}/",
                    envelope.weles_task_id
                ))
                .context("evidence inventory URI is not bound to the attempt task")?;
            let relative = format!("recordings/{}/{tail}", envelope.weles_task_id);
            let retained = record_dir.join(&relative);
            let bytes = std::fs::read(&retained).with_context(|| {
                format!("read retained Weles evidence {}", retained.display())
            })?;
            if bytes.len() as u64 != item.bytes || crate::sha256_hex(&bytes) != item.sha256 {
                bail!("retained Weles evidence {relative} differs from the signed inventory");
            }
            Ok(json!({
                "kind": item.kind,
                "uri": item.uri,
                "local_path": relative,
                "sha256": item.sha256,
                "bytes": item.bytes,
            }))
        })
        .collect::<Result<_>>()?;
    run["evidence_inventory"] = Value::Array(inventory.clone());
    // The importer side of the boundary. `weles_evidence_inventory` is the RECORD-level
    // statement that this record has confirmed browser material, so only the accepted
    // completed attempt writes it; a non-success attempt contributes its per-run
    // `evidence_inventory` and its provenance reference and nothing record-level. No
    // command in this repository reads `weles_evidence_inventory` today, so this is a
    // guard on the record's own claim, NOT what stops a failure from being counted as
    // confirmation: that is enforced by `VerifiedProvenanceSet::supports_value`, which
    // refuses any document whose signed outcome is not the successful one and through
    // which both consumers classify every item.
    if confirms_record {
        let object = record
            .as_object_mut()
            .context("reference record is not an object")?;
        object.insert("weles_evidence_inventory".into(), Value::Array(inventory));
    }
    let _ = attempt_dir;
    Ok(())
}

/// Import the signed failure proofs of this record's other published web attempts.
///
/// A non-success attempt is never the record's source. Its receipt, evidence manifest and
/// retained evidence are signed and delivered all the same, so its provenance document
/// belongs in `provenance_documents`, where `VerifiedProvenanceSet::verify_record`
/// re-verifies it on every run instead of leaving it outside every path. Nothing about the
/// record's confirmed material changes: `apply_web_attempt` is called with
/// `confirms_record: false`, so no record-level evidence inventory is written, and
/// `supports_value` refuses a document whose outcome is not the successful one, so the
/// proof cannot back a single claim.
///
/// One unreadable or unpublished failed attempt is a diagnostic, never a reason to fail
/// the accepted attempt's import: the returned values are recorded on the import summary.
fn import_non_success_attempts(
    run_id: &str,
    catalog: &str,
    accepted: &RuntimeManifest,
    entry: &Value,
    record: &mut Value,
    record_dir: &Path,
    run_dir: &Path,
) -> Vec<Value> {
    let mut imported = Vec::new();
    let attempts = entry
        .get("attempts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for snapshot in &attempts {
        let Some(manifest) = snapshot
            .get("manifest")
            .cloned()
            .and_then(|value| serde_json::from_value::<RuntimeManifest>(value).ok())
        else {
            continue;
        };
        if manifest.engine != "web"
            || manifest.run_id != accepted.run_id
            || manifest.catalog != accepted.catalog
            || manifest.record != accepted.record
            || manifest.record_key != accepted.record_key
            || manifest.attempt_id == accepted.attempt_id
        {
            continue;
        }
        match import_non_success_attempt(&manifest, snapshot, record, record_dir, run_dir) {
            Ok(Some(summary)) => imported.push(summary),
            Ok(None) => {}
            Err(error) => imported.push(json!({
                "attempt_id": manifest.attempt_id,
                "state": "not_imported",
                "message": format!("{error:#}"),
            })),
        }
    }
    let _ = (run_id, catalog);
    imported
}

/// One non-success attempt, or `None` when this attempt is not a published web attempt
/// that carries a signed non-success proof.
///
/// Every record mutation happens on a COPY that replaces `record` only after the whole
/// import has succeeded. `apply_web_attempt` appends the provenance reference before it
/// runs the fallible loop that re-reads and re-hashes each retained evidence file, and the
/// caller turns any error here into a diagnostic while persisting the record regardless,
/// so mutating `record` in place would leave a reference behind whose verification is
/// guaranteed to fail on the same read — a permanent record-verification failure created
/// by an attempt the summary reports as not imported. Ordering the loop earlier would fix
/// only today's arrangement; substituting the copy keeps the guarantee whatever a later
/// change does inside `apply_web_attempt`.
fn import_non_success_attempt(
    manifest: &RuntimeManifest,
    snapshot: &Value,
    record: &mut Value,
    record_dir: &Path,
    run_dir: &Path,
) -> Result<Option<Value>> {
    let staging = run_dir
        .join("imports")
        .join(&manifest.catalog)
        .join(&manifest.record)
        .join(format!("{}-non-success", manifest.attempt_id));
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;
    let output_log = staging.join("worker-output.log");
    download_uri(&manifest.output_uri, &output_log)?;
    let report = retained_worker_report("web", &output_log)?;
    let envelope_outcome = report
        .pointer("/weles_attempt_envelope/outcome")
        .and_then(Value::as_str);
    let (Some(outcome), true) = (
        envelope_outcome,
        report
            .get("provenance_document")
            .is_some_and(Value::is_object),
    ) else {
        // Nothing signed to verify: this attempt never reached a terminal outcome with a
        // provenance document, so there is no proof to import and nothing to diagnose.
        let _ = std::fs::remove_dir_all(&staging);
        return Ok(None);
    };
    if outcome == crate::weles_provenance::SUCCESSFUL_OUTCOME {
        bail!("a non-accepted attempt reports the successful outcome");
    }
    if !crate::weles_provenance::is_terminal_outcome(outcome) {
        bail!("failed worker report envelope outcome {outcome} is not a terminal outcome");
    }
    // The identical identity proof the accepted attempt gets, against the same immutable
    // Stado submission receipt: a failure proof is not imported on weaker evidence than a
    // success, only on a different reported state.
    let receipt = load_submission_receipt(
        &manifest.run_id,
        &manifest.catalog,
        &manifest.record,
        &manifest.attempt_id,
    )?
    .context("non-success attempt has no immutable submission receipt")?;
    let proof = verify_worker_report(&report, manifest, snapshot, &receipt, "failed")?;
    let expected_sha256 = proof
        .get("sha256")
        .and_then(Value::as_str)
        .expect("verified artifact digest")
        .to_string();
    let expected_bytes = proof
        .get("bytes")
        .and_then(Value::as_u64)
        .expect("verified artifact byte count");
    let archive = staging.join("artifacts.tar.gz");
    download_uri(&manifest.artifact_uri, &archive)?;
    let (observed_sha256, observed_bytes) =
        hash_regular_file(&archive, MAX_ATTEMPT_ARCHIVE_BYTES)?;
    if observed_sha256 != expected_sha256 || observed_bytes != expected_bytes {
        bail!("retained failed-attempt artifact differs from the worker report");
    }
    let extracted_root = staging.join("extracted");
    let members = extract_attempt_archive(&archive, &extracted_root)?;
    let attempt_relative = format!("crawl/{}", manifest.attempt_id);
    let attempt_destination = record_dir.join(&attempt_relative);
    let attempt_staged = record_dir
        .join("crawl")
        .join(format!(".{}.staging", manifest.attempt_id));
    if attempt_staged.exists() {
        std::fs::remove_dir_all(&attempt_staged)?;
    }
    std::fs::create_dir_all(attempt_staged.parent().expect("crawl parent"))?;
    std::fs::rename(&extracted_root, &attempt_staged)?;
    std::fs::write(
        attempt_staged.join("worker-report.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    install_staged_tree(&attempt_staged, &attempt_destination)?;
    let relative_report = format!("{attempt_relative}/worker-report.json");
    let mut staged_record = record.clone();
    let mut run = crawl_run_entry(manifest, snapshot, &report, &proof, &relative_report);
    run["retained_members"] = json!(members.len());
    apply_web_attempt(
        &mut staged_record,
        &mut run,
        &report,
        &attempt_destination,
        record_dir,
        false,
    )?;
    let runs = staged_record
        .as_object_mut()
        .context("reference record is not an object")?
        .entry("crawl_runs")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .context("crawl_runs is not a list")?;
    if let Some(existing) = runs.iter_mut().find(|value| {
        value.get("attempt_id").and_then(Value::as_str) == Some(manifest.attempt_id.as_str())
    }) {
        *existing = run.clone();
    } else {
        runs.push(run.clone());
    }
    runs.sort_by(|left, right| {
        left.get("attempt")
            .and_then(Value::as_u64)
            .cmp(&right.get("attempt").and_then(Value::as_u64))
    });
    // Nothing above this line has touched the caller's record.
    *record = staged_record;
    let _ = std::fs::remove_dir_all(&staging);
    Ok(Some(json!({
        "attempt": manifest.attempt,
        "attempt_id": manifest.attempt_id,
        "state": "provenance_imported",
        "outcome": outcome,
        "artifact_sha256": expected_sha256,
        "retained_members": members.len(),
        "weles_task_id": run.get("weles_task_id").cloned().unwrap_or(Value::Null),
        "supports_confirmed_material": false,
    })))
}
/// Import exactly one accepted attempt of one record.
///
/// The whole record transaction is staged and fsynced before any rename, so an
/// interrupted import leaves the previous attempt content intact and can be
/// retried; the previous attempts and their diagnostics are preserved because
/// each attempt owns its own `crawl/{attempt_id}` subtree and its own
/// `crawl_runs` entry keyed by `attempt_id`.
fn import_record_attempt(
    run_id: &str,
    catalog: &str,
    engine: &str,
    entry: &Value,
    run_dir: &Path,
) -> Result<Value> {
    let manifest: RuntimeManifest =
        serde_json::from_value(entry.get("manifest").cloned().unwrap_or(Value::Null))
            .context("accepted record retains no typed runtime manifest")?;
    if manifest.engine != engine || manifest.catalog != catalog || manifest.run_id != run_id {
        bail!("retained runtime manifest does not belong to this run, catalog and engine");
    }
    let receipt = load_submission_receipt(run_id, catalog, &manifest.record, &manifest.attempt_id)?
        .context("accepted record has no immutable submission receipt")?;
    let staging = run_dir
        .join("imports")
        .join(catalog)
        .join(&manifest.record)
        .join(&manifest.attempt_id);
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;
    let output_log = staging.join("worker-output.log");
    download_uri(&manifest.output_uri, &output_log)?;
    let report = retained_worker_report(engine, &output_log)?;
    let proof = verify_worker_report(&report, &manifest, entry, &receipt, "artifact_published")?;
    let archive = staging.join("artifacts.tar.gz");
    download_uri(&manifest.artifact_uri, &archive)?;
    let expected_sha256 = proof
        .get("sha256")
        .and_then(Value::as_str)
        .expect("verified artifact digest");
    let expected_bytes = proof
        .get("bytes")
        .and_then(Value::as_u64)
        .expect("verified artifact byte count");
    let (observed_sha256, observed_bytes) =
        hash_regular_file(&archive, MAX_ATTEMPT_ARCHIVE_BYTES)?;
    if observed_sha256 != expected_sha256 || observed_bytes != expected_bytes {
        bail!(
            "retained attempt artifact differs from the worker report: expected sha256={expected_sha256} bytes={expected_bytes}, observed sha256={observed_sha256} bytes={observed_bytes}"
        );
    }
    let extracted_root = staging.join("extracted");
    let members = extract_attempt_archive(&archive, &extracted_root)?;
    let record_dir = reference_path(catalog, &manifest.record)?
        .parent()
        .context("record reference has no directory")?
        .to_path_buf();
    let attempt_relative = format!("crawl/{}", manifest.attempt_id);
    let attempt_destination = record_dir.join(&attempt_relative);
    let attempt_staged = record_dir
        .join("crawl")
        .join(format!(".{}.staging", manifest.attempt_id));
    if attempt_staged.exists() {
        std::fs::remove_dir_all(&attempt_staged)?;
    }
    std::fs::create_dir_all(attempt_staged.parent().expect("crawl parent"))?;
    std::fs::rename(&extracted_root, &attempt_staged)?;
    // A plain write, not `atomic_json_write`: that helper leaves a `.lock` sibling,
    // and the staged tree is published verbatim as the attempt artifact.
    std::fs::write(
        attempt_staged.join("worker-report.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    install_staged_tree(&attempt_staged, &attempt_destination)?;
    let record_path = record_dir.join("reference.json");
    let mut record: Value =
        crate::read_json(record_path.to_str().context("record path is not UTF-8")?)?;
    let source_url = record
        .get("product_url")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let relative_report = format!("{attempt_relative}/worker-report.json");
    let mut run = crawl_run_entry(&manifest, entry, &report, &proof, &relative_report);
    run["retained_members"] = json!(members.len());
    match engine {
        "web" => apply_web_attempt(
            &mut record,
            &mut run,
            &report,
            &attempt_destination,
            &record_dir,
            // The accepted attempt is the one the record's confirmed material comes from.
            true,
        )?,
        "docs" => {
            // One corpus attempt. The typed report is validated in exactly one
            // place, shared with `docs-corpus import`, so this document is not
            // checked twice by two independent rules; that validation also proves
            // the artifact coordinates, digests and corpus/tree agreement, and it
            // returns them, so the artifact this attempt actually downloaded is
            // compared here rather than through transitivity via the manifest URI
            // equality above and the archive readback.
            let (artifact_uri, artifact_sha256) =
                super::docs_corpus::validate_docs_worker_report(&report)?;
            if artifact_uri != manifest.artifact_uri || artifact_sha256 != expected_sha256 {
                bail!(
                    "docs worker report artifact ({artifact_uri} {artifact_sha256}) is not the immutable attempt artifact ({} {expected_sha256})",
                    manifest.artifact_uri
                );
            }
            let corpus = report
                .get("corpus")
                .cloned()
                .context("docs worker report has no typed corpus summary")?;
            // Manifest equality stays here: only this side knows the immutable
            // attempt's committed content-structure digest.
            if report.get("docs_structure_sha256").and_then(Value::as_str)
                != manifest.docs_structure_sha256.as_deref()
            {
                bail!("docs worker report crawl-definition digest differs from the immutable attempt");
            }
            run["docs_structure_sha256"] = json!(manifest.docs_structure_sha256);
            run["corpus"] = corpus;
        }
        _ => {}
    }
    let (motion, states) = attempt_media(engine, &attempt_destination, &record_dir, &source_url)?;
    let accessibility = accessibility_gap(engine, &attempt_destination);
    let object = record
        .as_object_mut()
        .context("reference record is not an object")?;
    object.insert("captured_at".into(), json!(crate::now_iso_utc()));
    object.insert("motion".into(), Value::Array(motion.clone()));
    object.insert("states".into(), Value::Array(states.clone()));
    object.insert("accessibility".into(), accessibility);
    object.insert("evidence_status".into(), json!("partial"));
    object.insert(
        "evidence_gaps".into(),
        json!(["crawl evidence has not yet passed verify-reference-evidence"]),
    );
    let runs = object
        .entry("crawl_runs")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .context("crawl_runs is not a list")?;
    if let Some(existing) = runs.iter_mut().find(|value| {
        value.get("attempt_id").and_then(Value::as_str) == Some(manifest.attempt_id.as_str())
    }) {
        *existing = run.clone();
    } else {
        runs.push(run.clone());
    }
    runs.sort_by(|left, right| {
        left.get("attempt")
            .and_then(Value::as_u64)
            .cmp(&right.get("attempt").and_then(Value::as_u64))
    });
    // In the same record transaction, so the failure proofs and the accepted attempt are
    // persisted by the one `atomic_json_write` below or not at all.
    let non_success = if engine == "web" {
        import_non_success_attempts(
            run_id,
            catalog,
            &manifest,
            entry,
            &mut record,
            &record_dir,
            run_dir,
        )
    } else {
        Vec::new()
    };
    atomic_json_write(&record_path, &record)?;
    let _ = std::fs::remove_dir_all(&staging);
    Ok(json!({
        "state": "imported",
        "attempt": manifest.attempt,
        "attempt_id": manifest.attempt_id,
        "artifact_uri": manifest.artifact_uri,
        "artifact_sha256": expected_sha256,
        "artifact_bytes": expected_bytes,
        "retained_members": members.len(),
        "states": states.len(),
        "motion": motion.len(),
        "worker_report": relative_report,
        "weles_task_id": run.get("weles_task_id").cloned().unwrap_or(Value::Null),
        "non_success_attempts": non_success,
        "imported_at": crate::now_iso_utc(),
    }))
}

fn run_spis_command(arguments: &[&str]) -> Result<String> {
    let executable = std::env::current_exe().context("resolve current Spis executable")?;
    let mut command = Command::new(executable);
    command.args(arguments);
    let output = bounded_command_output(
        &mut command,
        "Spis catalog maintenance command",
        Duration::from_secs(900),
        8 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "spis {} failed: {}{}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Import every accepted attempt of one run, then run the catalog validators.
///
/// Each record is its own durable transaction under its own record lock, so a
/// busy peer, a single failing record or a crash never blocks or corrupts the
/// others. Validators and the catalog generator run once, after every record has
/// been installed, and their failure is retained as a typed run diagnostic rather
/// than left as unrecoverable dirty state.
fn import_ready(run_id: &str, selected_record: Option<&str>) -> Result<Value> {
    let run_dir = run_path(run_id)?
        .parent()
        .context("run path has no parent")?
        .to_path_buf();
    let snapshot = load(Some(run_id))?;
    let mut touched_catalogs: BTreeMap<String, String> = BTreeMap::new();
    for catalog in snapshot
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let catalog_name = catalog
            .get("catalog")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let engine = catalog
            .get("engine")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        for record in catalog
            .get("records")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let record_name = record
                .get("record")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if !selected_matches(&record_name, selected_record) {
                continue;
            }
            let state = record
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !matches!(state, "completed" | "uploaded") {
                continue;
            }
            let guard = match RecordMutationGuard::acquire(run_id, &catalog_name, &record_name) {
                Ok(guard) => guard,
                Err(error) if error.downcast_ref::<RecordLockBusy>().is_some() => continue,
                Err(error) => return Err(error),
            };
            let current = record_snapshot(run_id, &catalog_name, &record_name)?;
            if !matches!(
                current.get("state").and_then(Value::as_str),
                Some("completed" | "uploaded")
            ) {
                continue;
            }
            let outcome =
                import_record_attempt(run_id, &catalog_name, &engine, &current, &run_dir);
            match outcome {
                Ok(import) => {
                    mutate_record(run_id, &catalog_name, &record_name, |entry| {
                        entry["state"] = json!("imported");
                        entry["weles_task_id"] =
                            import.get("weles_task_id").cloned().unwrap_or(Value::Null);
                        entry["import"] = import;
                        entry["diagnostic"] = Value::Null;
                        Ok(())
                    })?;
                    touched_catalogs.insert(catalog_name.clone(), engine.clone());
                }
                Err(error) => {
                    mutate_record(run_id, &catalog_name, &record_name, |entry| {
                        entry["state"] = json!("partial");
                        entry["diagnostic"] = json!({
                            "code": "attempt_import_failed",
                            "message": format!("{error:#}"),
                        });
                        Ok(())
                    })?;
                }
            }
            drop(guard);
        }
    }
    let mut maintenance = Vec::new();
    for (catalog, engine) in &touched_catalogs {
        if engine == "web" {
            match run_spis_command(&["analyze-example-structures", catalog]) {
                Ok(_) => {}
                Err(error) => maintenance.push(json!({
                    "command": format!("analyze-example-structures {catalog}"),
                    "state": "failed",
                    "message": format!("{error:#}"),
                })),
            }
        }
        if let Err(error) =
            run_spis_command(&["verify-reference-evidence", "--catalog", catalog, "--apply"])
        {
            maintenance.push(json!({
                "command": format!("verify-reference-evidence --catalog {catalog} --apply"),
                "state": "failed",
                "message": format!("{error:#}"),
            }));
        }
    }
    if !touched_catalogs.is_empty() {
        if let Err(error) = run_spis_command(&["generate-example-catalogs"]) {
            maintenance.push(json!({
                "command": "generate-example-catalogs",
                "state": "failed",
                "message": format!("{error:#}"),
            }));
        }
    }
    let _guard = RunMutationGuard::acquire(run_id)?;
    let mut run = load(Some(run_id))?;
    run["import_maintenance"] = json!({
        "catalogs": touched_catalogs.keys().collect::<Vec<_>>(),
        "failures": maintenance,
    });
    if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in entries {
            aggregate_catalog_entry(entry);
        }
    }
    update_run_state(&mut run);
    persist(&mut run)?;
    Ok(run)
}

fn import(rest: &[String]) -> Result<()> {
    let (run_id, selected_record) = parse_run_and_record(rest, true)?;
    let run_id = run_id.context("--run is required")?;
    {
        let _guard = RunMutationGuard::acquire(&run_id)?;
        let mut run = load(Some(&run_id))?;
        migrate_legacy_catalog_jobs(&mut run);
        refresh(&mut run);
        persist(&mut run)?;
    }
    let run = import_ready(&run_id, selected_record.as_deref())?;
    print_operation("import", &run, selected_record.as_deref())?;
    let pending: Vec<String> = run
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|catalog| {
            catalog
                .get("records")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter(|record| {
            selected_matches(
                record.get("record").and_then(Value::as_str).unwrap_or_default(),
                selected_record.as_deref(),
            ) && record.get("state").and_then(Value::as_str) != Some("imported")
        })
        .filter_map(|record| {
            record
                .get("record")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect();
    if run
        .pointer("/import_maintenance/failures")
        .and_then(Value::as_array)
        .is_some_and(|failures| !failures.is_empty())
    {
        bail!("crawl evidence was imported but a catalog validator or generator failed");
    }
    if !pending.is_empty() {
        bail!("{} crawl records were not imported: {}", pending.len(), pending.join(", "));
    }
    Ok(())
}

fn has_failures(run: &Value) -> bool {
    run.get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|entry| {
            matches!(
                entry.get("state").and_then(Value::as_str),
                Some(
                    "unavailable"
                        | "preflight_failed"
                        | "submission_failed"
                        | "lost"
                        | "failed"
                        | "cancelled"
                        | "partial"
                )
            )
        })
}

fn print_operation(operation: &str, run: &Value, record_filter: Option<&str>) -> Result<()> {
    let mut catalogs = run.get("catalogs").and_then(Value::as_array).cloned().unwrap_or_default();
    if let Some(record) = record_filter {
        for catalog in &mut catalogs {
            if let Some(records) = catalog.get_mut("records").and_then(Value::as_array_mut) {
                records.retain(|item| item.get("record").and_then(Value::as_str).is_some_and(|value| value == record || value.split_once('-').map(|(_, tail)| tail) == Some(record)));
            }
        }
        catalogs.retain(|catalog| catalog.get("records").and_then(Value::as_array).is_some_and(|records| !records.is_empty()));
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for catalog in &catalogs {
        *counts.entry(catalog.get("state").and_then(Value::as_str).unwrap_or("unknown").to_string()).or_default() += 1;
        if let Some(records) = catalog.get("records").and_then(Value::as_array) {
            for record in records {
                *counts.entry(format!("record_{}", record.get("state").and_then(Value::as_str).unwrap_or("unknown"))).or_default() += 1;
            }
        }
    }
    let document = json!({
        "schema": OP_SCHEMA,
        "operation": operation,
        "run_id": run.get("run_id"),
        "state": run.get("state"),
        "source_revision": run.get("source_revision"),
        "updated_at": run.get("updated_at"),
        "counts": counts,
        "catalogs": catalogs,
    });
    println!("{}", serde_json::to_string(&document)?);
    Ok(())
}

fn usage() {
    println!(
        "usage:
  spis crawl bindings generate --weles-token-ref ITEM#FIELD --organization-ref ITEM#FIELD [--output PATH]
  spis crawl start [--host ENGINE=TARGET] [--catalog SLUG ...] [--record SLUG] [--run-id ID] [--bindings PATH]
  spis crawl status [--run RUN_ID] [--record SLUG]
  spis crawl cancel --run RUN_ID [--record SLUG] --reason TEXT
  spis crawl resume --run RUN_ID [--record SLUG]
  spis crawl import --run RUN_ID [--record SLUG]

Every command emits exactly one JSON document on stdout; the CLI is the process
API and Spis exposes no second HTTP crawl surface.

Each record is one immutable attempt: identity, both stado:// attempt URIs and
the Stado run id are derived from the record key, and every state transition is
persisted under a durable per-record lock before the external effect it authorizes.

  bindings generate  Writes the exact typed binding for every checked-in record.
                     With --output an existing generated document is replaced
                     atomically after validation and read-back; the reported
                     outcome is created, replaced or unchanged.
  start              Idempotent. Re-running the same request digest continues the
                     existing run; planned, preflight_passed and submitting
                     records are driven forward and a record held by another
                     process is skipped, never failed.
  status             Refreshes from Stado when the run lock is free, otherwise
                     returns a read-only snapshot.
  cancel             Status-first, durable and idempotent. The intent is recorded
                     locally and published immutably before any cancellation is
                     dispatched, so a crash can never resurrect the record.
  resume             Never reruns a Stado job. A terminal failed, cancelled, lost
                     or submission_failed attempt becomes attempt N+1 with fully
                     recomputed identity; queued and running records are left
                     alone; completed records are imported.
  import             Per record and per attempt. The typed worker report, the
                     Stado submission receipt, the attempt artifact digest and
                     byte count and every retained evidence hash are verified
                     before a staged, fsynced, atomically installed record
                     transaction; earlier attempts and their diagnostics are kept."
    );
}

pub fn run(rest: &[String]) -> Result<()> {
    match rest.first().map(String::as_str) {
        Some("start") => start(&rest[1..]),
        Some("status") => status(&rest[1..]),
        Some("cancel") => cancel(&rest[1..]),
        Some("resume") => resume(&rest[1..]),
        Some("import") => import(&rest[1..]),
        Some("--help" | "-h") | None => { usage(); Ok(()) }
        Some("bindings") if rest.get(1).map(String::as_str) == Some("generate") => {
            generate_runtime_bindings(&rest[2..])
        }
        Some(other) => bail!("unknown crawl operation: {other}"),
    }
}
