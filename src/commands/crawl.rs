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
use std::io::Write;
use std::os::fd::AsRawFd;
use std::time::{SystemTime, UNIX_EPOCH};
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
    if execution.resolved_product_identifier != manifest.runtime_product.identifier
        || expected_product.product_url != manifest.runtime_product.product_url
        || expected_product.surface != manifest.runtime_product.surface
    {
        bail!("runtime product differs from the committed record or resolved execution identity");
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
        || matches!(value, "." | "..")
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{name} must be one strict non-dot path component");
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

fn run_root() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/.stado-home-unavailable"))
        .join(".stado")
        .join("work")
        .join("spis")
        .join("crawl-runs")
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
        let destination = run_root().join(&id).join("run.json");
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
    Ok(run_root().join(run_id).join("run.json"))
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
fn source_snapshot_revision() -> Result<String> {
    let embedded = build_revision()?;
    let head = Command::new("git")
        .arg("-C")
        .arg(source_root())
        .args(["rev-parse", "--verify", "HEAD"])
        .output()
        .context("read current Spis source revision")?;
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
    let status = Command::new("git")
        .arg("-C")
        .arg(source_root())
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .output()
        .context("verify current Spis source snapshot")?;
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

pub(crate) fn stado_command() -> Command {
    Command::new(std::env::var_os("SPIS_STADO_BIN").unwrap_or_else(|| "stado".into()))
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

struct RecordMutationGuard {
    file: File,
}

impl RecordMutationGuard {
    fn acquire(run_id: &str, catalog: &str, record: &str) -> Result<Self> {
        safe_component(run_id, "run id")?;
        safe_component(catalog, "catalog")?;
        safe_component(record, "record")?;
        let directory = run_root()
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
            bail!("{run_id}/{catalog}/{record} is already being mutated");
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
        let directory = run_root().join(run_id);
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
    let run_id = run.get("run_id").and_then(Value::as_str).context("run has no run_id")?;
    let path = run_path(run_id)?;
    let expected = run.get("mutation_revision").and_then(Value::as_u64).unwrap_or(0);
    if path.is_file() {
        let current: Value = crate::read_json(path.to_str().context("run path is not UTF-8")?)?;
        let actual = current.get("mutation_revision").and_then(Value::as_u64).unwrap_or(0);
        if actual != expected {
            bail!("crawl run {run_id} changed concurrently: expected revision {expected}, found {actual}");
        }
    } else if expected != 0 {
        bail!("crawl run {run_id} disappeared before revision {expected} could be persisted");
    }
    let mut staged = run.clone();
    sync_attempt_history(&mut staged);
    staged["mutation_revision"] = json!(expected + 1);
    staged["updated_at"] = json!(crate::now_iso_utc());
    atomic_json_write(&path, &staged)?;
    *run = staged;
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
            let root = run_root();
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

fn compact_submission(catalog: &str, engine: &str, host: &str, artifact_uri: Option<&str>, output_uri: &str, stado_stdout: &str) -> Result<Value> {
    let receipt = stado_stdout
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .filter(|value| {
            value.get("schema").and_then(Value::as_str)
                == Some("stado.submission-receipt.v2")
        })
        .context("Stado accepted the command but returned no structured submission receipt")?;
    for digest in ["request_digest", "source_digest", "input_digest"] {
        let value = receipt.get(digest).and_then(Value::as_str).unwrap_or_default();
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("Stado submission receipt has invalid {digest}");
        }
    }
    let jobs = receipt
        .get("jobs")
        .and_then(Value::as_array)
        .context("Stado submission receipt has no jobs")?;
    if jobs.len() != 1 {
        bail!("per-record crawl submission must map to exactly one Stado job");
    }
    let job = &jobs[0];
    let job_id = job
        .get("job_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("Stado submission receipt has no exact job id")?;
    if job.get("command_index").and_then(Value::as_u64) != Some(0)
        || job.get("output_uri").and_then(Value::as_str) != Some(output_uri)
        || job.get("repo_ref").and_then(Value::as_str)
            != receipt.get("repo_ref").and_then(Value::as_str)
        || job.get("submission_request_digest").and_then(Value::as_str)
            != receipt.get("request_digest").and_then(Value::as_str)
    {
        bail!("Stado receipt command mapping, output URI, source revision or request digest does not match");
    }
    Ok(json!({
        "schema": SUBMISSION_SCHEMA,
        "catalog": catalog,
        "engine": engine,
        "host": host,
        "stado_job_id": job_id,
        "artifact_uri": artifact_uri,
        "output_uri": output_uri,
        "state": "queued",
        "stado_receipt": receipt,
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
    Command::new(executable).args(args).output().context("launch crawler coordinator")
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

fn record_directories(catalog: &str, selected: Option<&str>) -> Result<Vec<PathBuf>> {
    let root = catalog_root(catalog)?.join("references");
    if let Some(record) = selected {
        safe_component(record, "record")?;
        let path = root.join(record);
        if !path.is_dir() {
            bail!("record {record} does not exist in catalog {catalog}");
        }
        return Ok(vec![path]);
    }
    let mut records = std::fs::read_dir(&root)
        .with_context(|| format!("read crawl catalog {}", root.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
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

fn prohibited_action_constraints() -> Value {
    json!({
        "no_first_run_consent": true,
        "no_system_permission_prompts": true,
        "no_notifications": true,
        "no_purchase": true,
        "no_final_destructive_action": true,
        "headless": true,
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
    let weles_token_ref =
        weles_token_ref.context("--weles-token-ref is required; Spis will not guess a credential reference")?;
    let organization_ref = organization_ref
        .context("--organization-ref is required; Spis will not guess an organization reference")?;
    if !valid_secret_reference(&weles_token_ref)
        || !valid_secret_reference(&organization_ref)
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
                    object.insert("constraints".into(), prohibited_action_constraints());
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
        if let Some(parent) = path.parent().filter(|value| !value.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(&with_newline)?;
                file.sync_all()?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if std::fs::read(&path)? != with_newline {
                    bail!("refusing to replace differing runtime bindings {}", path.display());
                }
            }
            Err(error) => return Err(error.into()),
        }
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema": OP_SCHEMA,
                "operation": "bindings_generate",
                "path": path,
                "sha256": crate::sha256_hex(&with_newline),
            }))?
        );
    } else {
        print!("{}", String::from_utf8(with_newline)?);
    }
    Ok(())
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
        uri: format!("stado://spis-crawl-inputs/runtime-bindings/{sha256}.json"),
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
        let output = stado_command()
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
    let output = stado_command()
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
    let downloaded = stado_command()
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
    Ok(RuntimeProduct {
        kind: kind.into(),
        identifier,
        product_url,
        identity_source,
        surface,
    })
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
    let base_uri = format!(
        "stado://spis-crawls/{}/{}/{}/{}/attempts/{}/{}",
        manifest.run_id,
        manifest.catalog,
        manifest.record,
        manifest.record_key,
        manifest.attempt,
        manifest.attempt_id
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
    let coordinate = format!(
        "stado://spis-crawls/{}/{}/{}/{}/attempts/{}/{}",
        manifest.run_id,
        manifest.catalog,
        manifest.record,
        manifest.record_key,
        manifest.attempt,
        manifest.attempt_id,
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
            return json!({"record": slug, "state": "unavailable", "diagnostic": {
                "code": "reference_read_failed", "message": error.to_string(), "path": path
            }});
        }
    };
    let reference: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            return json!({"record": slug, "state": "unavailable", "diagnostic": {
                "code": "reference_invalid", "message": error.to_string(), "path": path
            }});
        }
    };
    let binding = match runtime_binding(bindings, catalog, engine, &slug) {
        Ok(binding) => binding,
        Err(error) => {
            return json!({"record": slug, "state": "unavailable", "diagnostic": {
                "code": "runtime_binding_missing_or_invalid", "message": error.to_string()
            }});
        }
    };
    if binding.constraints.headless != (engine == "web") {
        return json!({"record": slug, "state": "unavailable", "diagnostic": {
            "code": "runtime_constraint_mismatch",
            "message": format!("{catalog}/{slug}: headless constraint does not match engine {engine}")
        }});
    }
    let service_identity = match (engine, service_identity) {
        ("web", Some(service)) if service.active_host == host => Some(service.clone()),
        ("web", _) => {
            return json!({"record": slug, "state": "unavailable", "diagnostic": {
                "code": "weles_service_identity_unbound",
                "message": "authorized weles-admission/browser-evidence/generic_browser_task placement is unavailable"
            }});
        }
        (_, None) => None,
        (_, Some(_)) => {
            return json!({"record": slug, "state": "unavailable", "diagnostic": {
                "code": "unexpected_service_identity",
                "message": "non-web execution cannot carry a Weles service identity"
            }});
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
            return json!({"record": slug, "state": "unavailable", "diagnostic": {
                "code": "runtime_product_unresolved", "message": error.to_string(),
                "product_url": reference.get("product_url")
            }});
        }
    };
    let docs_structure_sha256 = match docs_structure_sha256(catalog, &slug, engine) {
        Ok(value) => value,
        Err(error) => {
            return json!({"record": slug, "state": "unavailable", "diagnostic": {
                "code": "docs_structure_missing_or_invalid", "message": error.to_string()
            }});
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
    let base_uri = format!("stado://spis-crawls/{run_id}/{catalog}/{slug}/{record_key}");
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
        resource_lease: matches!(engine, "desktop" | "mobile")
            .then(|| format!("stado-exclusive://{host}/{engine}")),
        service_identity,
    };
    if let Err(error) = finalize_manifest_identity(&mut manifest, &bytes) {
        return json!({"record": slug, "state": "unavailable", "diagnostic": {
            "code": "runtime_manifest_finalization_failed", "message": error.to_string()
        }});
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
fn registry_placements() -> Result<(BTreeMap<String, String>, Option<RuntimeServiceIdentity>)> {
    let output = stado_command().args(["registry", "pull"]).output()?;
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
            .and_then(Value::as_str)?;
        Some(RuntimeServiceIdentity {
            name: "weles-admission".into(),
            generation,
            consumer: "spis".into(),
            capability: "browser-evidence".into(),
            active_host: active_host.into(),
            endpoint: endpoint.into(),
            action: "generic_browser_task".into(),
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
    let output = stado_command()
        .args(["host", "exec", host, "--"])
        .args(arguments)
        .output();
    match output {
        Ok(output) => json!({
            "command": arguments,
            "ready": output.status.success(),
            "stdout": String::from_utf8_lossy(&output.stdout).trim(),
            "stderr": String::from_utf8_lossy(&output.stderr).trim(),
        }),
        Err(error) => json!({"command": arguments, "ready": false, "error": error.to_string()}),
    }
}

fn host_preflight(catalog: &str, engine: &str, host: &str, admission_url: &str) -> Value {
    let mut commands: Vec<Vec<&str>> = vec![vec!["hostname"]];
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
        ("desktop", _) => vec![vec!["mdfind", "kMDItemContentType == 'com.apple.application-bundle'"]],
        ("web", _) => vec![vec!["node", "--version"], vec!["curl", "--version"]],
        ("docs", _) => vec![vec!["curl", "--version"]],
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
    let admission_ready = if engine == "web" {
        let health_url = format!("{}/healthz", admission_url.trim_end_matches('/'));
        let check = host_probe(host, &["curl", "--fail", "--silent", "--show-error", &health_url]);
        let ready = check.get("ready").and_then(Value::as_bool) == Some(true);
        checks.push(check);
        ready
    } else {
        true
    };
    let ready = checks
        .iter()
        .take(commands.len())
        .all(|check| check.get("ready").and_then(Value::as_bool) == Some(true))
        && desktop_driver_ready
        && admission_ready;
    json!({
        "schema": "wisent.crawl-host-preflight.v2",
        "catalog": catalog,
        "engine": engine,
        "host": host,
        "ready": ready,
        "checks": checks,
        "service_endpoint": if engine == "web" { json!(admission_url) } else { Value::Null },
    })
}

fn observed_hostname(host_report: &Value) -> Result<String> {
    let value = host_report
        .get("checks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|check| {
            check.get("command").and_then(Value::as_array).is_some_and(|command| {
                command.len() == 1 && command[0].as_str() == Some("hostname")
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

fn ios_booted_identity(host: &str, host_report: &Value) -> Result<RuntimeExecutionIdentity> {
    let check = host_report
        .get("checks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|check| {
            check
                .get("command")
                .and_then(Value::as_array)
                .is_some_and(|parts| parts.iter().any(|part| part.as_str() == Some("booted")))
        })
        .context("iOS host preflight has no booted-device report")?;
    let document: Value = serde_json::from_str(
        check.get("stdout").and_then(Value::as_str).unwrap_or_default(),
    )
    .context("simctl booted-device report is not JSON")?;
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
    Ok(RuntimeExecutionIdentity {
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
    })
}

fn android_device_identity(host: &str, host_report: &Value) -> Result<RuntimeExecutionIdentity> {
    let check = host_report
        .get("checks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|check| {
            check
                .get("command")
                .and_then(Value::as_array)
                .is_some_and(|parts| parts.iter().any(|part| part.as_str() == Some("devices")))
        })
        .context("Android host preflight has no device report")?;
    let devices: Vec<(&str, &str)> = check
        .get("stdout")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let serial = fields.next()?;
            let state = fields.next()?;
            (state == "device").then_some((serial, state))
        })
        .collect();
    if devices.len() != 1 {
        bail!("expected exactly one authorized Android device, found {}", devices.len());
    }
    Ok(RuntimeExecutionIdentity {
        host: host.into(),
        observed_hostname: String::new(),
        platform: "android".into(),
        device_id: Some(devices[0].0.into()),
        resolved_product_identifier: String::new(),
        device_name: None,
        executable_path: None,
        product_version: None,
        executable_sha256: None,
        effective_url: None,
    })
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
    {
        bail!("prepared-runtime proof does not bind the exact product/device with first run completed, zero pending prompts, disabled permission/notification prompt invocation, and disabled notification delivery");
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
            "ios-bundle" => (ios_booted_identity(host, host_report)?, Vec::new()),
            "android-package" => (android_device_identity(host, host_report)?, Vec::new()),
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
            "url" => host_probe(
                host,
                &[
                    "curl",
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--location",
                    "--max-redirs",
                    "5",
                    "--output",
                    "/dev/null",
                    "--write-out",
                    "%{http_code} %{url_effective}",
                    product,
                ],
            ),
            _ => unreachable!(),
        };
        let output = ready_output(&check, "verify exact runtime product")?;
        if matches!(manifest.runtime_product.kind.as_str(), "cli-binary" | "tui-binary") {
            let observed = output.split_whitespace().next().unwrap_or_default();
            if identity.executable_sha256.as_deref() != Some(observed) {
                bail!("terminal executable changed during preflight");
            }
        }
        if manifest.runtime_product.kind == "url" {
            let (status, effective) = output
                .lines()
                .last()
                .unwrap_or_default()
                .split_once(' ')
                .context("URL identity probe has no status and effective URL")?;
            let status: u16 = status.parse().context("URL identity probe status is invalid")?;
            let declared = url::Url::parse(product).context("declared URL is invalid")?;
            let effective = url::Url::parse(effective).context("effective URL is invalid")?;
            if !(200..300).contains(&status)
                || declared.origin() != effective.origin()
                || effective.username() != ""
                || effective.password().is_some()
            {
                bail!("URL probe left the exact declared origin or failed");
            }
            identity.effective_url = Some(effective.to_string());
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
    let mut failures: BTreeMap<String, usize> = BTreeMap::new();
    for value in states.iter().filter(|state| failure(state)) {
        *failures.entry((*value).to_string()).or_default() += 1;
    }
    entry["state"] = json!(state);
    entry["partial"] = json!(!failures.is_empty());
    entry["failure_counts"] = serde_json::to_value(failures).unwrap_or(Value::Null);
}

fn persist_submission_receipt(
    run_id: &str,
    catalog: &str,
    record: &str,
    attempt_id: &str,
    receipt: &Value,
) -> Result<()> {
    safe_component(run_id, "run id")?;
    safe_component(catalog, "catalog")?;
    safe_component(record, "record")?;
    safe_component(attempt_id, "attempt id")?;
    let path = run_root()
        .join(run_id)
        .join("receipts")
        .join(catalog)
        .join(record)
        .join(attempt_id)
        .join("receipt.json");
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
    service_endpoint: &str,
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
    let observed = host_preflight(catalog, engine, host, service_endpoint);
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
                | "submitting"
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

    let command = if state == "preflight_passed" {
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
    if before_submit.get("state").and_then(Value::as_str) != Some("preflight_passed") {
        return Ok(());
    }
    mutate_record(run_id, catalog, record_name, |entry| {
        if entry.get("state").and_then(Value::as_str) == Some("preflight_passed")
            && !entry.get("cancel_intent").is_some_and(Value::is_object)
        {
            entry["state"] = json!("submitting");
        }
        Ok(())
    })?;
    let armed = record_snapshot(run_id, catalog, record_name)?;
    drop(record_guard);
    if armed.get("state").and_then(Value::as_str) != Some("submitting") {
        return Ok(());
    }
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
    let receipt = match parse_submission(&output.stdout) {
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
        entry["artifact_uri"] = receipt
            .get("artifact_uri")
            .cloned()
            .unwrap_or_else(|| json!(manifest.artifact_uri));
        entry["output_uri"] = receipt
            .get("output_uri")
            .cloned()
            .unwrap_or_else(|| json!(manifest.output_uri));
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
        let endpoint = catalog_entry
            .get("records")
            .and_then(Value::as_array)
            .and_then(|records| records.first())
            .and_then(|record| record.pointer("/manifest/service_identity/endpoint"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let host_report =
            ensure_host_preflight(run_id, &catalog, &engine, &host, endpoint)?;
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
                    json!({
                        "record": path.file_name().and_then(|name| name.to_str()).unwrap_or("invalid-record"),
                        "state": "unavailable",
                        "diagnostic": {
                            "code": "runtime_placement_unavailable",
                            "retryable": true,
                            "message": message,
                        },
                        "attempts": [],
                    })
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
    let stored = stado_command()
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
    let output = stado_command()
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
        let attempt_id = record
            .pointer("/manifest/attempt_id")
            .or_else(|| record.get("attempt_id"))
            .and_then(Value::as_str)
            .unwrap_or("unsubmitted-attempt");
        let attempt = record
            .pointer("/manifest/attempt")
            .and_then(Value::as_u64)
            .unwrap_or(1);
        let job_id = record.get("stado_job_id").and_then(Value::as_str);
        let stado_run_id = record
            .pointer("/manifest/stado_run_id")
            .and_then(Value::as_str);
        let intent = json!({
            "schema": "wisent.crawl-cancel-intent.v1",
            "run_id": run_id,
            "catalog": catalog_name,
            "record": record_name,
            "record_key": record.pointer("/manifest/record_key").cloned().unwrap_or(Value::Null),
            "attempt": attempt,
            "attempt_id": attempt_id,
            "stado_run_id": stado_run_id,
            "stado_job_id": job_id,
            "reason": reason,
        });
        let base_uri = record
            .pointer("/manifest/artifact_uri")
            .and_then(Value::as_str)
            .and_then(|uri| uri.rsplit_once('/').map(|(parent, _)| parent.to_string()))
            .unwrap_or_else(|| {
                format!(
                    "stado://spis-crawls/{run_id}/{catalog_name}/{record_name}/attempts/{attempt_id}"
                )
            });
        let intent_uri = format!("{base_uri}/cancel-intent.json");
        let intent_sha256 = publish_cancel_intent(&intent_uri, &intent)?;
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
            "weles_action": if record.pointer("/weles/task_id").is_some()
                || record.get("weles_task_id").is_some()
            {
                json!({"state": "official_cancel_required", "diagnostic": "retained Weles task cancellation bridge is unavailable in this source revision"})
            } else {
                json!({"state": "no_retained_task_id"})
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

fn rerun_job(job_id: &str) -> Result<String> {
    let output = super::crawl::stado_command().args(["job", "rerun", job_id, "--json"]).output()?;
    if !output.status.success() {
        bail!("Stado refused rerun of {job_id}: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    let value: Value = serde_json::from_slice(&output.stdout)?;
    value.get("new_job_id").or_else(|| value.get("job_id")).and_then(Value::as_str).map(str::to_string)
        .context("Stado rerun returned no new job id")
}

fn submit_retained_command(entry: &Value) -> Result<Value> {
    let arguments = entry
        .get("command")
        .and_then(Value::as_array)
        .context("failed submission retained no original command")?
        .iter()
        .map(|value| value.as_str().map(str::to_string).context("retained command argument is not a string"))
        .collect::<Result<Vec<_>>>()?;
    let output = invoke_engine(&arguments)?;
    if output.status.success() {
        parse_submission(&output.stdout)
    } else {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr).trim().to_string()))
    }
}

fn resume(rest: &[String]) -> Result<()> {
    let (run_id, _) = parse_run_and_record(rest, true)?;
    let mut run = load(run_id.as_deref())?;
    refresh(&mut run);
    let retained_resubmit_needed = run.get("catalogs").and_then(Value::as_array).into_iter().flatten().any(|entry| {
        matches!(entry.get("state").and_then(Value::as_str), Some("preflight_failed" | "submission_failed" | "lost"))
    });
    if retained_resubmit_needed {
        let original = run.get("source_revision").and_then(Value::as_str).context("run has no source_revision")?;
        let current = build_revision()?;
        if original != current {
            bail!("run {run_id:?} belongs to Spis revision {original}; retained-command resubmission with revision {current} is refused");
        }
    }
    let admission_url = run.get("admission_url").and_then(Value::as_str).unwrap_or_default().to_string();
    let mut reimport_partial = false;
    if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in entries {
            let state = entry.get("state").and_then(Value::as_str).unwrap_or("submission_failed").to_string();
            if state == "partial" {
                reimport_partial = true;
                continue;
            }
            if state == "preflight_failed" {
                let catalog = entry.get("catalog").and_then(Value::as_str).unwrap_or_default();
                let engine = entry.get("engine").and_then(Value::as_str).unwrap_or_default();
                let host = entry.get("host").and_then(Value::as_str).unwrap_or_default();
                entry["preflight"] = host_preflight(catalog, engine, host, &admission_url);
                if entry.pointer("/preflight/ready").and_then(Value::as_bool) != Some(true) {
                    entry["error"] = json!("host preflight still fails; no crawler job was submitted");
                    continue;
                }
            } else if !matches!(state.as_str(), "submission_failed" | "lost" | "failed" | "cancelled") {
                continue;
            }
            let result = if state == "lost" || entry.get("stado_job_id").and_then(Value::as_str).is_none() {
                submit_retained_command(entry)
            } else {
                rerun_job(entry.get("stado_job_id").and_then(Value::as_str).unwrap())
                    .map(|fresh| json!({"stado_job_id": fresh}))
            };
            match result {
                Ok(receipt) => {
                    entry["stado_job_id"] = receipt.get("stado_job_id").cloned().unwrap_or(Value::Null);
                    if receipt.get("artifact_uri").is_some() { entry["artifact_uri"] = receipt["artifact_uri"].clone(); }
                    if receipt.get("output_uri").is_some() { entry["output_uri"] = receipt["output_uri"].clone(); }
                    entry["state"] = json!("queued");
                    entry["error"] = Value::Null;
                    entry["job"] = Value::Null;
                }
                Err(error) => entry["error"] = json!(error.to_string()),
            }
        }
    }
    if reimport_partial {
        let id = run.get("run_id").and_then(Value::as_str).context("run has no id")?.to_string();
        let persisted_path = run_path(&id)?;
        let run_dir = persisted_path.parent().context("run path has no parent")?.to_path_buf();
        import_ready(&mut run, &id, &run_dir, true)?;
    }
    run["updated_at"] = json!(crate::now_iso_utc());
    update_run_state(&mut run);
    persist(&mut run)?;
    print_operation("resume", &run, None)?;
    if has_failures(&run) { bail!("one or more crawler jobs could not be resumed"); }
    Ok(())
}

fn download_uri(uri: &str, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() { std::fs::create_dir_all(parent)?; }
    let output = super::crawl::stado_command().args(["storage", "get", uri]).arg(destination).output()?;
    if !output.status.success() {
        bail!("download {uri}: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

fn unpack(archive: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    let decoder = GzDecoder::new(File::open(archive)?);
    let mut archive = tar::Archive::new(decoder);
    for member in archive.entries()? {
        let mut member = member?;
        let relative = member.path()?.into_owned();
        if relative.is_absolute() || relative.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
            bail!("crawl archive contains an unsafe path");
        }
        member.unpack_in(destination)?;
    }
    Ok(())
}

fn collect_named(root: &Path, name: &str, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() { collect_named(&path, name, out)?; }
        else if path.file_name().and_then(|value| value.to_str()) == Some(name) { out.push(path); }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() { copy_tree(&from, &to)?; } else { std::fs::copy(from, to)?; }
    }
    Ok(())
}

fn find_record_dir(catalog: &str, slug: &str) -> Option<PathBuf> {
    let root = catalog_root(catalog).ok()?.join("references");
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let tail = name.split_once('-').map(|(_, value)| value).unwrap_or(&name);
        if name == slug || tail == slug { return Some(entry.path()); }
    }
    None
}

fn record_slug(report: &Value, crawl_path: &Path) -> Option<String> {
    report.get("record").or_else(|| report.get("slug")).and_then(Value::as_str).map(str::to_string)
        .or_else(|| crawl_path.parent()?.file_name()?.to_str().map(str::to_string))
}

fn artifact_record(report: &Value, relative: &str, run_id: &str, job_id: Option<&str>, artifact_uri: Option<&str>) -> Value {
    let persisted_run: Option<Value> = run_path(run_id)
        .ok()
        .and_then(|path| path.to_str().and_then(|value| crate::read_json(value).ok()));
    let source_revision = persisted_run.and_then(|run| run.get("source_revision").cloned()).unwrap_or(Value::Null);
    let job = report.get("job").unwrap_or(report);
    json!({
        "schema": "wisent.crawl-import.v1",
        "run_id": run_id,
        "source_revision": source_revision,
        "stado_job_id": job_id,
        "artifact_uri": artifact_uri,
        "raw_report": relative,
        "engine_schema": report.get("schema").or_else(|| job.get("schema")).cloned().unwrap_or(Value::Null),
        "action": report.get("action").or_else(|| job.get("action")).cloned().unwrap_or(Value::Null),
        "idempotency_key": report.get("idempotency_key").or_else(|| job.get("idempotency_key")).cloned().unwrap_or(Value::Null),
        "receipt_evidence_digest": job.pointer("/receipt/evidence_digest").cloned().unwrap_or(Value::Null),
        "imported_at": crate::now_iso_utc(),
        "states_seen": report.get("states_seen").or_else(|| report.get("commands_crawled")).cloned().unwrap_or(json!(0)),
        "status": report.get("status").cloned().unwrap_or_else(|| json!("completed")),
        "error": report.get("error").cloned().unwrap_or(Value::Null),
    })
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(root) else { return; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { walk(&path, out); } else { out.push(path); }
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

fn capture_method(engine: &str) -> &'static str {
    match engine {
        "mobile" => "Local product run through Appium with XCUITest or UiAutomator2; screen recording and accessibility source retained",
        "desktop" => "Local product run through Cua Driver; snapshot-bound actions, screenshots, action recording and accessibility tree retained",
        "web" => "Local browser recording through Weles on a Stado-selected host; browser history, screenshots, recordings and signed result retained",
        "tui" => "Local product run in an isolated tmux pseudo-terminal; raw terminal bytes and distinct screens retained",
        "cli" => "Local product run of the real executable in an isolated tmux pseudo-terminal; stdout/stderr, argv and exit status retained",
        _ => "Unclassified Spis crawl",
    }
}

fn copy_evidence_media(
    engine: &str,
    raw_source: &Path,
    raw_destination: &Path,
    record_dir: &Path,
    run_id: &str,
    source_url: &str,
) -> Result<(Vec<Value>, Vec<Value>)> {
    let media_root = record_dir.join("media").join(run_id);
    if media_root.exists() { std::fs::remove_dir_all(&media_root)?; }
    std::fs::create_dir_all(&media_root)?;
    let mut motion = Vec::new();
    let mut states = Vec::new();
    let fallback_motion = files_under(raw_source).into_iter()
        .find(|candidate| media_kind(candidate) == Some("motion"))
        .and_then(|candidate| candidate.strip_prefix(raw_source).ok().map(Path::to_path_buf))
        .map(|relative| format!("media/{run_id}/{}", relative.to_string_lossy()));
    for source in files_under(raw_source) {
        let Some(kind) = media_kind(&source) else { continue; };
        let relative = source.strip_prefix(raw_source).unwrap_or(&source);
        let destination = media_root.join(relative);
        if let Some(parent) = destination.parent() { std::fs::create_dir_all(parent)?; }
        std::fs::copy(&source, &destination)?;
        let local_path = destination.strip_prefix(record_dir).unwrap_or(&destination).to_string_lossy().to_string();
        if kind == "motion" {
            let declared = destination.extension().and_then(|value| value.to_str()).map(|ext| match ext.to_ascii_lowercase().as_str() {
                "mp4" => "video-mp4",
                "webm" => "video-webm",
                "gif" => "animated-gif",
                "webp" => "animated-webp",
                "cast" => "terminal-cast",
                _ => "unknown",
            });
            motion.push(json!({
                "local_path": local_path,
                "source_url": source_url,
                "media_kind": declared,
                "capture_method": capture_method(engine),
                "crawl_evidence_path": raw_destination.strip_prefix(record_dir).unwrap_or(raw_destination).to_string_lossy(),
            }));
        } else {
            let sibling_motion = files_under(source.parent().unwrap_or(raw_source))
                .into_iter()
                .find(|candidate| media_kind(candidate) == Some("motion"))
                .and_then(|candidate| candidate.strip_prefix(raw_source).ok().map(Path::to_path_buf))
                .map(|relative| format!("media/{run_id}/{}", relative.to_string_lossy()))
                .or_else(|| fallback_motion.clone());
            states.push(json!({
                "name": format!("Observed {}", relative.to_string_lossy()),
                "local_path": local_path,
                "source_motion_path": sibling_motion,
            }));
        }
    }
    Ok((motion, states))
}


fn evidence_interactions(report: &Value) -> Vec<Value> {
    report.pointer("/evidence_observations/canonical_interactions")
        .and_then(Value::as_array).cloned().unwrap_or_default()
}

fn report_accessibility(report: &Value) -> Option<Value> {
    report.pointer("/evidence_observations/canonical_accessibility")
        .filter(|value| value.is_object())
        .filter(|value| value.get("measured").and_then(Value::as_bool).is_some())
        .filter(|value| value.get("observations").and_then(Value::as_array).is_some())
        .filter(|value| value.get("unknowns").and_then(Value::as_array).is_some())
        .cloned()
}

fn accessibility_evidence(raw_source: &Path, run_id: &str, report: &Value) -> Value {
    if let Some(measurement) = report_accessibility(report) {
        return measurement;
    }
    let files = files_under(raw_source);
    let trees: Vec<&PathBuf> = files.iter().filter(|path| {
        matches!(path.extension().and_then(|value| value.to_str()), Some("xml" | "html"))
            || matches!(path.file_name().and_then(|value| value.to_str()), Some("snapshot.json" | "source.json" | "axe.json"))
    }).collect();
    let bytes: u64 = trees.iter().filter_map(|path| std::fs::metadata(path).ok().map(|metadata| metadata.len())).sum();
    json!({
        "measured": false,
        "observations": if trees.is_empty() { vec![] } else { vec![format!("Retained {} accessibility/DOM source files totalling {bytes} bytes under crawl/{run_id}.", trees.len())] },
        "unknowns": [
            "No engine-supplied canonical accessibility measurement was retained.",
            "Screen-reader traversal, focus order, live regions and reduced-motion preference remain unmeasured.",
        ],
    })
}

fn journey_evidence(report: &Value) -> Value {
    report.pointer("/evidence_observations/canonical_journey")
        .cloned().unwrap_or(Value::Null)
}

fn motion_analysis(report: &Value) -> Value {
    report.pointer("/evidence_observations/canonical_motion_analysis")
        .cloned().unwrap_or(Value::Null)
}

fn adapt_canonical_record(engine: &str, run_id: &str, raw_source: &Path, raw_destination: &Path, record_dir: &Path, report: &Value, record: &mut Value) -> Result<()> {
    let source_url = record.get("product_url").and_then(Value::as_str)
        .context("reference record has no product_url")?.to_string();
    let (motion, states) = copy_evidence_media(engine, raw_source, raw_destination, record_dir, run_id, &source_url)?;
    let interactions = evidence_interactions(report);
    let journey = journey_evidence(report);
    let accessibility = accessibility_evidence(raw_source, run_id, report);
    let analysis = motion_analysis(report);
    let object = record.as_object_mut().context("reference record is not an object")?;
    object.insert("captured_at".into(), json!(crate::now_iso_utc()));
    object.insert("motion".into(), Value::Array(motion));
    object.insert("states".into(), Value::Array(states));
    object.insert("interactions".into(), Value::Array(interactions));
    object.insert("journey".into(), journey);
    object.insert("motion_analysis".into(), analysis);
    object.insert("accessibility".into(), accessibility);
    object.insert("evidence_status".into(), json!("partial"));
    object.insert("evidence_gaps".into(), json!(["crawl evidence has not yet passed verify-reference-evidence"]));
    Ok(())
}

fn merge_report(catalog: &str, engine: &str, run_id: &str, job_id: Option<&str>, artifact_uri: Option<&str>, crawl_path: &Path, report: &Value) -> Result<Value> {
    let slug = record_slug(report, crawl_path).context("crawl report has no record slug")?;
    let record_dir = find_record_dir(catalog, &slug).ok_or_else(|| anyhow!("{catalog}: no record matches {slug}"))?;
    let raw_source = crawl_path.parent().context("crawl report has no parent")?;
    let raw_destination = record_dir.join("crawl").join(run_id);
    if raw_destination.exists() { std::fs::remove_dir_all(&raw_destination)?; }
    copy_tree(raw_source, &raw_destination)?;
    let record_path = record_dir.join("reference.json");
    let mut record: Value = crate::read_json(record_path.to_str().context("record path is not UTF-8")?)?;
    adapt_canonical_record(engine, run_id, raw_source, &raw_destination, &record_dir, report, &mut record)?;
    let relative_report = format!("crawl/{run_id}/{}", crawl_path.file_name().and_then(|value| value.to_str()).unwrap_or("crawl.json"));
    let imported = artifact_record(report, &relative_report, run_id, job_id, artifact_uri);
    let runs = record.as_object_mut().context("reference record is not an object")?.entry("crawl_runs").or_insert_with(|| json!([])).as_array_mut().context("crawl_runs is not a list")?;
    if let Some(existing) = runs.iter_mut().find(|value| value.get("run_id").and_then(Value::as_str) == Some(run_id)) { *existing = imported; } else { runs.push(imported); }
    atomic_json_write(&record_path, &record)?;
    let gaps = record.get("evidence_gaps").and_then(Value::as_array).cloned().unwrap_or_default();
    Ok(json!({
        "record": record_dir.file_name().and_then(|value| value.to_str()).unwrap_or(&slug),
        "state": if report.get("status").and_then(Value::as_str) == Some("failed") { "failed" } else { "imported" },
        "states": record.get("states").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
        "interactions": record.get("interactions").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
        "media": count_media(raw_source),
        "gaps": gaps,
        "error": report.get("error").cloned().unwrap_or(Value::Null),
    }))
}

fn count_media(root: &Path) -> usize {
    fn walk(path: &Path, count: &mut usize) {
        let Ok(entries) = std::fs::read_dir(path) else { return; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { walk(&path, count); }
            else if matches!(path.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase).as_deref(), Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "mp4" | "webm" | "cast")) { *count += 1; }
        }
    }
    let mut count = 0;
    walk(root, &mut count);
    count
}

fn find_directory_named(root: &Path, name: &str) -> Option<PathBuf> {
    if root.file_name().and_then(|value| value.to_str()) == Some(name) && root.is_dir() {
        return Some(root.to_path_buf());
    }
    for entry in std::fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_directory_named(&path, name) { return Some(found); }
        }
    }
    None
}

fn import_docs_corpus(catalog: &str, run_id: &str, job_id: Option<&str>, artifact_uri: Option<&str>, root: &Path) -> Result<Vec<Value>> {
    let references = catalog_root(catalog)?.join("references");
    let mut record_dirs: Vec<PathBuf> = std::fs::read_dir(&references)?
        .filter_map(Result::ok).map(|entry| entry.path())
        .filter(|path| path.join("reference.json").is_file()).collect();
    record_dirs.sort();
    let mut out = Vec::new();
    for record_dir in record_dirs {
        let directory_name = record_dir.file_name().and_then(|value| value.to_str()).context("record directory is not UTF-8")?;
        let slug = directory_name.split_once('-').map(|(_, tail)| tail).unwrap_or(directory_name);
        let source = find_directory_named(root, slug);
        let destination = record_dir.join("crawl").join(run_id);
        if destination.exists() { std::fs::remove_dir_all(&destination)?; }
        std::fs::create_dir_all(&destination)?;
        let (state, error) = if let Some(source) = source {
            copy_tree(&source, &destination)?;
            ("imported", Value::Null)
        } else {
            ("missing", json!(format!("documentation crawl archive has no corpus directory for {slug}")))
        };
        let report = json!({
            "schema": "wisent.docs-crawl-record.v1",
            "record": slug,
            "status": state,
            "files": files_under(&destination).len(),
        });
        atomic_json_write(&destination.join("crawl.json"), &report)?;
        let record_path = record_dir.join("reference.json");
        let mut record: Value = crate::read_json(record_path.to_str().context("record path is not UTF-8")?)?;
        let imported = artifact_record(&report, &format!("crawl/{run_id}/crawl.json"), run_id, job_id, artifact_uri);
        let runs = record.as_object_mut().context("record is not an object")?.entry("crawl_runs").or_insert_with(|| json!([])).as_array_mut().context("crawl_runs is not a list")?;
        if let Some(existing) = runs.iter_mut().find(|value| value.get("run_id").and_then(Value::as_str) == Some(run_id)) { *existing = imported; } else { runs.push(imported); }
        atomic_json_write(&record_path, &record)?;
        out.push(json!({
            "record": directory_name,
            "state": state,
            "states": record.get("states").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "interactions": record.get("interactions").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "media": files_under(&destination).len(),
            "gaps": record.get("evidence_gaps").cloned().unwrap_or_else(|| json!([])),
            "error": error,
        }));
    }
    Ok(out)
}

fn import_catalog(run_id: &str, entry: &mut Value, run_dir: &Path) -> Result<()> {
    let catalog = entry.get("catalog").and_then(Value::as_str).context("catalog entry has no catalog")?.to_string();
    let engine = entry.get("engine").and_then(Value::as_str).unwrap_or_default().to_string();
    let job_id = entry.get("stado_job_id").and_then(Value::as_str).map(str::to_string);
    let artifact_uri = entry.get("artifact_uri").and_then(Value::as_str).map(str::to_string);
    let destination = run_dir.join("downloads").join(&catalog);
    std::fs::create_dir_all(&destination)?;
    if let Some(uri) = artifact_uri.as_deref() {
        let archive = destination.join("crawl.tar.gz");
        download_uri(uri, &archive)?;
        let extracted = destination.join("extracted");
        if extracted.exists() { std::fs::remove_dir_all(&extracted)?; }
        unpack(&archive, &extracted)?;
    } else if let Some(job_id) = job_id.as_deref() {
        let output = super::crawl::stado_command().args(["machine", "artifacts", job_id, "--output-dir"]).arg(&destination).output()?;
        if !output.status.success() {
            bail!("download canonical artifacts for {catalog}: {}", String::from_utf8_lossy(&output.stderr).trim());
        }
    }
    if engine == "docs" {
        let records = import_docs_corpus(&catalog, run_id, job_id.as_deref(), artifact_uri.as_deref(), &destination)?;
        entry["records"] = Value::Array(records);
        entry["state"] = json!("imported");
        entry["error"] = Value::Null;
        return Ok(());
    }
    let mut reports = Vec::new();
    collect_named(&destination, "crawl.json", &mut reports)?;
    if engine == "web" && reports.is_empty() {
        collect_named(&destination, "command_output.log", &mut reports)?;
    }
    let mut records = Vec::new();
    for path in reports {
        if path.file_name().and_then(|value| value.to_str()) == Some("command_output.log") {
            let text = std::fs::read_to_string(&path)?;
            let candidate = text.lines().rev().find_map(|line| serde_json::from_str::<Value>(line).ok());
            if let Some(report) = candidate { records.extend(import_web_report(&catalog, run_id, job_id.as_deref(), &path, &report)?); }
        } else {
            let report: Value = crate::read_json(path.to_str().context("crawl report path is not UTF-8")?)?;
            records.push(merge_report(&catalog, &engine, run_id, job_id.as_deref(), artifact_uri.as_deref(), &path, &report)?);
        }
    }
    if records.is_empty() { bail!("{catalog}: downloaded artifacts contain no importable record reports"); }
    entry["records"] = Value::Array(records);
    entry["state"] = json!("imported");
    entry["error"] = Value::Null;
    Ok(())
}

fn collect_weles_uris(value: &Value, uris: &mut Vec<String>) {
    fn collect_typed(value: &Value, uris: &mut Vec<String>) {
        match value {
            Value::String(text) if text.starts_with("stado://weles/")
                && !text.contains("/../") && !text.ends_with("/..") => {
                if !uris.contains(text) { uris.push(text.clone()); }
            }
            Value::Array(values) => values.iter().for_each(|value| collect_typed(value, uris)),
            Value::Object(values) => values.values().for_each(|value| collect_typed(value, uris)),
            _ => {}
        }
    }
    for pointer in ["/receipt/artifacts", "/result/artifacts", "/artifacts"] {
        if let Some(artifacts) = value.pointer(pointer) {
            collect_typed(artifacts, uris);
        }
    }
}

fn verified_spis_evidence(
    _value: &Value,
    _expected_job_id: &str,
    _expected_origin: &str,
    _expected_action: &str,
    _expected_idempotency_key: &str,
) -> Option<Value> {
    // Raw Weles results remain available for diagnosis. Canonical evidence is
    // withheld unless a typed Weles receipt has been cryptographically verified.
    None
}

fn web_observation(job: &Value, correlation: &Value) -> Value {
    let history = job.pointer("/result/generic_browser_task/history")
        .or_else(|| job.pointer("/result/history"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let states: Vec<Value> = history.iter().enumerate().flat_map(|(index, step)| {
        ["before", "after"].into_iter().filter_map(move |phase| {
            let artifact_id = step.get(format!("{phase}_artifact_id"))
                .or_else(|| step.get(format!("{phase}_screenshot_uri")))
                .or_else(|| step.get(format!("{phase}_state_uri")))
                .and_then(Value::as_str)?;
            Some(json!({
                "step_index": index,
                "phase": phase,
                "artifact_id": artifact_id,
                "action": step.get("tool"),
            }))
        })
    }).collect();
    let job_id = job.get("id").or_else(|| job.get("job_id")).and_then(Value::as_str).unwrap_or_default();
    let evidence = verified_spis_evidence(
        job,
        job_id,
        correlation.get("origin").and_then(Value::as_str).unwrap_or_default(),
        correlation.get("action").and_then(Value::as_str).unwrap_or_default(),
        correlation.get("idempotency_key").and_then(Value::as_str).unwrap_or_default(),
    );
    json!({
        "schema": "wisent.web-crawl-record.v1",
        "states": states,
        "states_seen": states.len(),
        "blocked_edges": job.get("error").into_iter().cloned().collect::<Vec<_>>(),
        "status": job.get("status").cloned().unwrap_or_else(|| json!("failed")),
        "error": job.get("error").cloned().unwrap_or(Value::Null),
        "evidence_observations": {
            "canonical_interactions": evidence.as_ref().and_then(|value| value.get("canonical_interactions")).cloned().unwrap_or_else(|| json!([])),
            "canonical_journey": evidence.as_ref().and_then(|value| value.get("canonical_journey")).cloned().unwrap_or(Value::Null),
            "canonical_motion_analysis": evidence.as_ref().and_then(|value| value.get("canonical_motion_analysis")).cloned().unwrap_or(Value::Null),
            "canonical_accessibility": evidence.as_ref().and_then(|value| value.get("canonical_accessibility")).cloned().unwrap_or(Value::Null),
            "surface_proof": evidence,
        },
    })
}

fn normalized_surface_url(value: &str) -> Result<String> {
    let mut url = url::Url::parse(value).context("surface proof URL is invalid")?;
    url.set_fragment(None);
    if url.path() != "/" {
        let trimmed = url.path().trim_end_matches('/').to_string();
        url.set_path(&trimmed);
    }
    Ok(url.to_string())
}

fn validate_web_surface(catalog: &str, product_url: &str, observation: &Value) -> Result<()> {
    let proof = observation.pointer("/evidence_observations/surface_proof")
        .filter(|value| value.is_object())
        .ok_or_else(|| anyhow!("{catalog}: Weles result has no machine-readable spis_evidence surface proof"))?;
    let observed_url = proof.get("observed_url").and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{catalog}: spis_evidence has no observed_url"))?;
    if catalog == "landing-page-examples" {
        if proof.get("surface_kind").and_then(Value::as_str) != Some("landing") {
            bail!("{catalog}: Weles did not identify the retained surface as a landing page");
        }
        if normalized_surface_url(observed_url)? != normalized_surface_url(product_url)? {
            bail!("{catalog}: Weles observed {observed_url}, expected exact landing URL {product_url}");
        }
    }
    if catalog == "pricing-page-examples" {
        if proof.get("surface_kind").and_then(Value::as_str) != Some("pricing") {
            bail!("{catalog}: Weles did not identify the retained surface as a pricing page");
        }
        if proof.get("visible_pricing_comparison").and_then(Value::as_bool) != Some(true) {
            bail!("{catalog}: Weles did not prove a visible comparison of at least two plans or prices");
        }
    }
    Ok(())
}

fn import_web_report(catalog: &str, run_id: &str, job_id: Option<&str>, _path: &Path, report: &Value) -> Result<Vec<Value>> {
    let records = report.get("records").and_then(Value::as_array).context("web report has no records mapping")?;
    let mut out = Vec::new();
    for item in records {
        let slug = item.get("record").and_then(Value::as_str).context("web record mapping has no record")?;
        let record_dir = find_record_dir(catalog, slug).ok_or_else(|| anyhow!("{catalog}: no record matches {slug}"))?;
        let destination = record_dir.join("crawl").join(run_id);
        if destination.exists() { std::fs::remove_dir_all(&destination)?; }
        std::fs::create_dir_all(&destination)?;
        let relative = format!("crawl/{run_id}/weles-result.json");
        atomic_json_write(&destination.join("weles-result.json"), item)?;
        let job = item.get("job").unwrap_or(item);
        let mut uris = Vec::new();
        collect_weles_uris(job, &mut uris);
        let artifacts = destination.join("artifacts");
        std::fs::create_dir_all(&artifacts)?;
        let mut downloaded = Vec::new();
        for (index, uri) in uris.iter().enumerate() {
            let basename = uri.rsplit('/').find(|part| !part.is_empty()).unwrap_or("artifact");
            let safe: String = basename.chars().map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') { character } else { '_' }
            }).collect();
            let local = artifacts.join(format!("{index:04}-{safe}"));
            download_uri(uri, &local)?;
            downloaded.push((uri.clone(), local));
        }
        let observation = web_observation(job, item);
        atomic_json_write(&destination.join("crawl.json"), &observation)?;
        let record_path = record_dir.join("reference.json");
        let mut record: Value = crate::read_json(record_path.to_str().context("record path is not UTF-8")?)?;
        validate_web_surface(catalog, record.get("product_url").and_then(Value::as_str).unwrap_or_default(), &observation)?;
        update_web_source_visual(catalog, &record_dir, &record, &downloaded)?;
        adapt_canonical_record("web", run_id, &destination, &destination, &record_dir, &observation, &mut record)?;
        let imported = artifact_record(item, &relative, run_id, job_id, None);
        let runs = record.as_object_mut().context("record is not an object")?.entry("crawl_runs").or_insert_with(|| json!([])).as_array_mut().context("crawl_runs is not a list")?;
        if let Some(existing) = runs.iter_mut().find(|value| value.get("run_id").and_then(Value::as_str) == Some(run_id)) { *existing = imported; } else { runs.push(imported); }
        atomic_json_write(&record_path, &record)?;
        let state = job.get("status").and_then(Value::as_str).unwrap_or("failed");
        let gaps = record.get("evidence_gaps").and_then(Value::as_array).cloned().unwrap_or_default();
        out.push(json!({
            "record": slug,
            "state": if state == "completed" { "imported" } else { state },
            "states": record.get("states").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "interactions": record.get("interactions").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "media": count_media(&destination),
            "gaps": gaps,
            "error": job.get("error").cloned().unwrap_or(Value::Null),
        }));
    }
    Ok(out)
}

fn run_spis_command(arguments: &[&str]) -> Result<String> {
    let executable = std::env::current_exe().context("resolve current Spis executable")?;
    let output = Command::new(executable).args(arguments).output()?;
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

fn update_web_source_visual(catalog: &str, record_dir: &Path, record: &Value, artifacts: &[(String, PathBuf)]) -> Result<()> {
    let Some((source_uri, source_path)) = artifacts.iter().find(|(_, path)| media_kind(path) == Some("state")) else {
        return Ok(());
    };
    let extension = source_path.extension().and_then(|value| value.to_str()).unwrap_or("png").to_ascii_lowercase();
    let image_name = format!("{}.{}", record_dir.file_name().and_then(|value| value.to_str()).unwrap_or("capture"), extension);
    let image_path = catalog_root(catalog)?.join("images").join(image_name);
    std::fs::copy(source_path, &image_path)?;
    let bytes = std::fs::read(&image_path)?;
    let decoded = image::open(&image_path)?;
    let sources_path = catalog_root(catalog)?.join("sources.json");
    let mut sources: Value = crate::read_json(sources_path.to_str().context("sources path is not UTF-8")?)?;
    let product_url = record.get("product_url").and_then(Value::as_str).context("record has no product_url")?;
    let examples = sources.get_mut("examples").and_then(Value::as_array_mut).context("sources examples are not a list")?;
    let example = examples.iter_mut().find(|example| example.get("source_url").and_then(Value::as_str) == Some(product_url))
        .ok_or_else(|| anyhow!("{catalog}: no source example matches {product_url}"))?;
    example["visual"] = json!({
        "source_page_url": product_url,
        "source_artifact_uri": source_uri,
        "local_path": image_path.strip_prefix(catalog).unwrap_or(&image_path).to_string_lossy(),
        "capture_kind": "local-browser-screenshot",
        "captured_at": crate::now_iso_utc(),
        "format": extension,
        "width": decoded.width(),
        "height": decoded.height(),
        "bytes": bytes.len(),
        "sha256": crate::sha256_hex(&bytes),
    });
    let visual_count = examples.iter().filter(|example| {
        example.pointer("/visual/capture_status").and_then(Value::as_str) != Some("pending-weles")
    }).count();
    sources["visual_count"] = json!(visual_count);
    atomic_json_write(&sources_path, &sources)?;
    Ok(())
}

fn summarize_catalog_records(catalog: &str, run_id: &str, entry: &mut Value) -> Result<()> {
    let reference = catalog_root(catalog)?.join("references");
    let mut summaries = Vec::new();
    if reference.is_dir() {
        let mut directories: Vec<PathBuf> = std::fs::read_dir(&reference)?
            .filter_map(Result::ok)
            .map(|item| item.path())
            .filter(|path| path.join("reference.json").is_file())
            .collect();
        directories.sort();
        for record_dir in directories {
            let record: Value = crate::read_json(record_dir.join("reference.json").to_str().context("record path is not UTF-8")?)?;
            let imported = record.get("crawl_runs").and_then(Value::as_array).is_some_and(|runs| {
                runs.iter().any(|run| run.get("run_id").and_then(Value::as_str) == Some(run_id))
            });
            let complete = record.get("evidence_status").and_then(Value::as_str) == Some("complete");
            let gaps = record.get("evidence_gaps").and_then(Value::as_array).cloned().unwrap_or_default();
            summaries.push(json!({
                "record": record_dir.file_name().and_then(|value| value.to_str()).unwrap_or("unknown"),
                "state": if imported && complete { "complete" } else if imported { "partial" } else { "missing" },
                "states": record.get("states").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
                "interactions": record.get("interactions").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
                "media": record.get("motion").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
                "gaps": gaps,
                "error": Value::Null,
            }));
        }
    }
    entry["records"] = Value::Array(summaries);
    Ok(())
}

fn import_ready(run: &mut Value, run_id: &str, run_dir: &Path, retry_partial_import: bool) -> Result<()> {
    let mut imported_catalogs = Vec::new();
    if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in entries {
            let state = entry.get("state").and_then(Value::as_str).unwrap_or_default();
            if state == "imported" { continue; }
            if !matches!(state, "completed" | "uploaded") && !(retry_partial_import && state == "partial") { continue; }
            let catalog = entry.get("catalog").and_then(Value::as_str).context("catalog entry has no catalog")?.to_string();
            let engine = entry.get("engine").and_then(Value::as_str).unwrap_or_default().to_string();
            match import_catalog(run_id, entry, run_dir)
                .and_then(|_| {
                    if engine == "web" {
                        run_spis_command(&["analyze-example-structures", &catalog]).map(|_| ())
                    } else {
                        Ok(())
                    }
                })
                .and_then(|_| run_spis_command(&["verify-reference-evidence", "--catalog", &catalog, "--apply"]).map(|_| ()))
                .and_then(|_| summarize_catalog_records(&catalog, run_id, entry))
            {
                Ok(()) => imported_catalogs.push(catalog),
                Err(error) => {
                    entry["state"] = json!("partial");
                    entry["error"] = json!(error.to_string());
                }
            }
        }
    }
    if !imported_catalogs.is_empty() {
        run_spis_command(&["generate-example-catalogs"])?;
    }
    run["updated_at"] = json!(crate::now_iso_utc());
    update_run_state(run);
    Ok(())
}

fn import(rest: &[String]) -> Result<()> {
    let (run_id, _) = parse_run_and_record(rest, true)?;
    let mut run = load(run_id.as_deref())?;
    refresh(&mut run);
    let id = run.get("run_id").and_then(Value::as_str).context("run has no id")?.to_string();
    let persisted_path = run_path(&id)?;
    let run_dir = persisted_path.parent().context("run path has no parent")?.to_path_buf();
    import_ready(&mut run, &id, &run_dir, true)?;
    if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in entries {
            let state = entry.get("state").and_then(Value::as_str).unwrap_or_default();
            if !matches!(state, "imported" | "partial" | "lost" | "failed" | "cancelled" | "submission_failed" | "preflight_failed") {
                entry["error"] = json!(format!("artifact import requires a terminal successful job, found {state}"));
            }
        }
    }
    update_run_state(&mut run);
    persist(&mut run)?;
    print_operation("import", &run, None)?;
    if has_failures(&run) || run.get("state").and_then(Value::as_str) != Some("imported") {
        bail!("one or more crawl artifacts were not imported");
    }
    Ok(())
}

fn has_failures(run: &Value) -> bool {
    run.get("catalogs").and_then(Value::as_array).into_iter().flatten().any(|entry| {
        matches!(entry.get("state").and_then(Value::as_str), Some("preflight_failed" | "submission_failed" | "lost" | "failed" | "cancelled" | "partial"))
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
    println!("usage:\n  spis crawl bindings generate --weles-token-ref ITEM#FIELD --organization-ref ITEM#FIELD [--output PATH]\n  spis crawl start [--host ENGINE=TARGET] [--catalog SLUG ...] [--record SLUG] [--bindings PATH]\n  spis crawl status [--run RUN_ID] [--record SLUG]\n  spis crawl cancel --run RUN_ID [--record SLUG | --record FILE] --reason TEXT\n  spis crawl resume --run RUN_ID\n  spis crawl import --run RUN_ID\n\nCommands emit one JSON document on stdout. The CLI is the process API; Spis does not expose a second HTTP /v1/crawl surface.");
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
