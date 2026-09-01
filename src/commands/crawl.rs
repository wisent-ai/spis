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
const RUN_ROOT: &str = ".wisent-output/crawl-runs";

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeSurfaceIdentity {
    pub family: String,
    pub exact_url: String,
    pub origin: String,
    pub path: String,
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
    pub observed_at: String,
    pub evidence_uri: String,
    pub evidence_sha256: String,
    pub installed: bool,
    pub first_run_completed: bool,
    pub pending_permission_prompts: u32,
    pub pending_notification_prompts: u32,
}

#[derive(Clone, Debug)]
struct RuntimeBinding {
    account: RuntimeAccount,
    constraints: RuntimeConstraints,
    prepared_proof: Option<RuntimePreparedProof>,
    surface: Option<RuntimeSurfaceIdentity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RuntimeExecutionIdentity {
    pub host: String,
    pub platform: String,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    #[serde(default)]
    pub executable_path: Option<String>,
    #[serde(default)]
    pub product_version: Option<String>,
    #[serde(default)]
    pub executable_sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RuntimeManifest {
    pub schema: String,
    pub run_id: String,
    pub catalog: String,
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
    pub prepared_proof: Option<RuntimePreparedProof>,
    pub execution_identity: Option<RuntimeExecutionIdentity>,
    pub resource_lease: Option<String>,
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
    if manifest.runtime_product.identifier.is_empty()
        || manifest.source_input_sha256.len() != 64
        || manifest.correlation_id.is_empty()
        || manifest.stado_run_id.is_empty()
        || manifest.execution_identity.is_none()
    {
        bail!("runtime manifest is incomplete and cannot authorize a worker");
    }
    let reference_path = Path::new(&manifest.catalog)
        .join("references")
        .join(&manifest.record)
        .join("reference.json");
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
    if matches!(manifest.engine.as_str(), "mobile" | "web" | "docs")
        && expected_product.identifier != manifest.runtime_product.identifier
    {
        bail!("runtime manifest product identifier differs from the committed record");
    }
    let mut recomputed = manifest.clone();
    finalize_manifest_identity(&mut recomputed, &reference_bytes)?;
    if serde_json::to_value(&recomputed)? != serde_json::to_value(&manifest)? {
        bail!("runtime manifest identity, input digest, keys or artifact URIs are not canonical");
    }
    Ok(manifest)
}

fn run_path(run_id: &str) -> PathBuf {
    Path::new(RUN_ROOT).join(run_id).join("run.json")
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

struct RunMutationGuard {
    file: File,
}

impl RunMutationGuard {
    fn acquire(run_id: &str) -> Result<Self> {
        if run_id.is_empty()
            || !run_id.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            })
        {
            bail!("run id contains unsafe characters");
        }
        let directory = Path::new(RUN_ROOT).join(run_id);
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

fn persist(run: &mut Value) -> Result<()> {
    let run_id = run.get("run_id").and_then(Value::as_str).context("run has no run_id")?;
    let path = run_path(run_id);
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
    run["mutation_revision"] = json!(expected + 1);
    run["updated_at"] = json!(crate::now_iso_utc());
    atomic_json_write(&path, run)
}

fn load(run_id: Option<&str>) -> Result<Value> {
    let selected = match run_id {
        Some(value) => value.to_string(),
        None => {
            let mut ids: Vec<String> = std::fs::read_dir(RUN_ROOT)
                .with_context(|| format!("no crawl runs exist under {RUN_ROOT}"))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().join("run.json").is_file())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect();
            ids.sort();
            ids.pop().context("no persisted crawl run exists")?
        }
    };
    crate::read_json(run_path(&selected).to_str().context("run path is not UTF-8")?)
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
        "job_id": job_id,
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

fn engine_command(
    manifest: &RuntimeManifest,
    host: &str,
    admission_url: &str,
) -> Result<Vec<String>> {
    let catalog = manifest.catalog.as_str();
    let engine = manifest.engine.as_str();
    let mut args = match engine {
        "mobile" => vec!["crawl-mobile".into(), catalog.into(), "--host".into(), host.into()],
        "desktop" => vec!["crawl-desktop".into(), catalog.into(), "--host".into(), host.into()],
        "web" => vec![
            "crawl-web".into(),
            catalog.into(),
            "--host".into(),
            host.into(),
            "--admission-url".into(),
            admission_url.into(),
        ],
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
    let root = Path::new(catalog).join("references");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&root)
        .with_context(|| format!("read {}", root.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.join("reference.json").is_file())
        .collect();
    paths.sort();
    paths.retain(|path| {
        let slug = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
        selected.is_none_or(|wanted| {
            wanted == slug || slug.split_once('-').map(|(_, tail)| tail) == Some(wanted)
        })
    });
    if paths.is_empty() {
        bail!("{catalog}: no matching reference records");
    }
    Ok(paths)
}

fn runtime_binding(catalog: &str, slug: &str) -> Result<RuntimeBinding> {
    let path = std::env::var_os("SPIS_RUNTIME_BINDINGS").ok_or_else(|| {
        anyhow!(
            "{catalog}/{slug}: SPIS_RUNTIME_BINDINGS has no typed account, credential and constraint binding"
        )
    })?;
    let document: Value = crate::read_json(
        Path::new(&path)
            .to_str()
            .context("SPIS_RUNTIME_BINDINGS path is not UTF-8")?,
    )?;
    if document.get("schema").and_then(Value::as_str)
        != Some("wisent.crawl-runtime-bindings.v1")
    {
        bail!("SPIS_RUNTIME_BINDINGS must declare wisent.crawl-runtime-bindings.v1");
    }
    let binding = document
        .get("records")
        .and_then(Value::as_object)
        .and_then(|catalogs| catalogs.get(catalog))
        .and_then(Value::as_object)
        .and_then(|records| records.get(slug))
        .and_then(Value::as_object)
        .ok_or_else(|| {
            anyhow!("{catalog}/{slug}: runtime bindings have no exact catalog and record key")
        })?;
    let account: RuntimeAccount = serde_json::from_value(
        binding.get("account").cloned().context("record binding has no account declaration")?,
    )
    .context("record account declaration is invalid")?;
    match account.mode.as_str() {
        "bound" => {
            if account.account_id.as_deref().is_none_or(str::is_empty)
                || account.credential_refs.is_empty()
            {
                bail!("{catalog}/{slug}: bound account needs an exact account_id and opaque credential_refs");
            }
        }
        "anonymous-public-surface" => {
            if account.account_id.as_deref() != Some("anonymous-public-surface")
                || !account.credential_refs.is_empty()
            {
                bail!("{catalog}/{slug}: anonymous public mode must be explicit and cannot carry credentials");
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
        bail!("{catalog}/{slug}: credential_refs must be nonempty opaque references");
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
    let prepared_proof = binding
        .get("prepared_proof")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .context("prepared proof declaration is invalid")?;
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
        {
            bail!("{catalog}/{slug}: typed web surface family, origin, path or URL does not exactly match the record");
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
fn finalize_manifest_identity(manifest: &mut RuntimeManifest, reference_bytes: &[u8]) -> Result<()> {
    manifest.reference_sha256 = crate::sha256_hex(reference_bytes);
    let input_identity = json!({
        "reference_sha256": manifest.reference_sha256,
        "runtime_product": manifest.runtime_product,
        "account": manifest.account,
        "constraints": manifest.constraints,
        "prepared_proof": manifest.prepared_proof,
        "execution_identity": manifest.execution_identity,
        "resource_lease": manifest.resource_lease,
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
    manifest.correlation_id = format!("spis-{}", &manifest.record_key[..32]);
    manifest.stado_run_id = manifest.correlation_id.clone();
    let base_uri = format!(
        "stado://spis-crawls/{}/{}/{}/{}",
        manifest.run_id, manifest.catalog, manifest.record, manifest.record_key
    );
    manifest.artifact_uri = format!("{base_uri}/artifacts.tar.gz");
    manifest.output_uri = format!("{base_uri}/worker-output.log");
    Ok(())
}

fn planned_record(
    run_id: &str,
    source_revision: &str,
    catalog: &str,
    engine: &str,
    host: &str,
    record_dir: &Path,
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
    let binding = match runtime_binding(catalog, &slug) {
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
    let account = binding.account;
    let constraints = binding.constraints;
    let prepared_proof = binding.prepared_proof;
    let input_identity = json!({
        "reference_sha256": crate::sha256_hex(&bytes),
        "runtime_product": product,
        "account": account,
        "constraints": constraints,
        "prepared_proof": prepared_proof,
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
        stado_run_id: correlation_id,
        artifact_uri: format!("{base_uri}/artifacts.tar.gz"),
        output_uri: format!("{base_uri}/worker-output.log"),
        runtime_product: product,
        account,
        constraints,
        prepared_proof,
        execution_identity: None,
        resource_lease: matches!(engine, "desktop" | "mobile")
            .then(|| format!("stado-exclusive://{host}/{engine}")),
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
        "job_id": Value::Null,
        "artifact_uri": manifest.artifact_uri,
        "output_uri": manifest.output_uri,
        "submission_receipt": Value::Null,
        "preflight": Value::Null,
        "diagnostic": Value::Null,
    })
}
fn registry_placements() -> Result<(BTreeMap<String, String>, Option<String>)> {
    let output = super::crawl::stado_command().args(["registry", "pull"]).output()?;
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
    let web = registry
        .pointer("/service_directory/services/weles-admission/active_host")
        .and_then(Value::as_str)
        .map(str::to_string);
    let admission_url = web.as_deref().and_then(|host| {
        registry.pointer(&format!("/service_directory/services/weles-admission/endpoints/{host}/url"))
            .and_then(Value::as_str).map(str::to_string)
    });
    let always_on = targets.iter().find(|target| {
        target.get("role").and_then(Value::as_str) == Some("always-on")
            && target.pointer("/weles/enabled").and_then(Value::as_bool) == Some(true)
    }).and_then(|target| target.get("name")).and_then(Value::as_str).map(str::to_string);
    let cpu = targets.iter().find(|target| {
        target.get("role").and_then(Value::as_str) == Some("always-on")
    }).and_then(|target| target.get("name")).and_then(Value::as_str).map(str::to_string);
    let mobile = targets.iter().find(|target| {
        target.get("services").and_then(Value::as_array).is_some_and(|services| {
            services.iter().any(|service| {
                service.get("name").and_then(Value::as_str).is_some_and(|name| {
                    name.to_ascii_lowercase().contains("appium")
                })
            })
        })
    }).and_then(|target| target.get("name")).and_then(Value::as_str).map(str::to_string);
    let mut placements = BTreeMap::new();
    if let Some(host) = web { placements.insert("web".into(), host); }
    if let Some(host) = always_on {
        placements.insert("desktop".into(), host);
    }
    if let Some(host) = mobile { placements.insert("mobile".into(), host); }
    if let Some(host) = cpu {
        placements.insert("cli".into(), host.clone());
        placements.insert("tui".into(), host.clone());
        placements.insert("docs".into(), host);
    }
    Ok((placements, admission_url))
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


fn preflight_catalog(catalog: &str, selected_record: Option<&str>) -> Result<()> {
    let root = Path::new(catalog);
    let sources: Value = crate::read_json(root.join("sources.json").to_str().context("sources path is not UTF-8")?)
        .with_context(|| format!("{catalog}: read source manifest"))?;
    let examples = sources.get("examples").and_then(Value::as_array)
        .context(format!("{catalog}: sources.json has no examples"))?;
    let references = root.join("references");
    let mut record_count = 0usize;
    for entry in std::fs::read_dir(&references).with_context(|| format!("{catalog}: read references"))?.flatten() {
        let path = entry.path().join("reference.json");
        if !path.is_file() { continue; }
        let record: Value = crate::read_json(path.to_str().context("reference path is not UTF-8")?)?;
        let directory = entry.file_name().to_string_lossy().to_string();
        if selected_record.is_some_and(|wanted| wanted != directory && directory.split_once('-').map(|(_, slug)| slug) != Some(wanted)) {
            continue;
        }
        let url = record.get("product_url").and_then(Value::as_str)
            .filter(|value| value.starts_with("https://") || value.starts_with("http://"))
            .ok_or_else(|| anyhow!("{catalog}/{directory}: product_url must be HTTP(S)"))?;
        let source = examples.iter().find(|example| example.get("source_url").and_then(Value::as_str) == Some(url))
            .ok_or_else(|| anyhow!("{catalog}/{directory}: product_url is absent from sources.json"))?;
        if catalog == "pricing-page-examples" {
            if source.get("category").and_then(Value::as_str) != Some("pricing") {
                bail!("{catalog}/{directory}: category must be exactly pricing");
            }
            let lower = url.to_ascii_lowercase();
            if !["pricing", "plans", "plan"].iter().any(|needle| lower.contains(needle)) {
                bail!("{catalog}/{directory}: URL does not identify a pricing/plans surface");
            }
        }
        if catalog == "landing-page-examples" && source.get("category").and_then(Value::as_str) != Some("landing") {
            bail!("{catalog}/{directory}: category must be exactly landing");
        }
        record_count += 1;
    }
    if record_count == 0 {
        bail!("{catalog}: selected family is empty");
    }
    Ok(())
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
    let commands: Vec<Vec<&str>> = match (engine, catalog) {
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
        ("web", _) => vec![vec!["node", "--version"], vec!["npm", "--version"], vec!["curl", "--version"]],
        ("docs", _) => vec![vec!["git", "--version"], vec!["curl", "--version"], vec!["df", "-h"]],
        ("cli" | "tui", _) => vec![vec!["tmux", "-V"]],
        _ => Vec::new(),
    };
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
        platform: "ios".into(),
        device_id: Some(
            devices[0]
                .get("udid")
                .and_then(Value::as_str)
                .context("booted iOS device has no UDID")?
                .into(),
        ),
        device_name: devices[0].get("name").and_then(Value::as_str).map(str::to_string),
        executable_path: None,
        product_version: None,
        executable_sha256: None,
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
        platform: "android".into(),
        device_id: Some(devices[0].0.into()),
        device_name: None,
        executable_path: None,
        product_version: None,
        executable_sha256: None,
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

fn resolve_terminal_identity(
    manifest: &mut RuntimeManifest,
    host: &str,
) -> Result<(RuntimeExecutionIdentity, Vec<Value>)> {
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
    let version_check = host_probe(host, &[&path, "--version"]);
    let version = ready_output(&version_check, "read exact product version")?;
    checks.push(version_check);
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
        format!("typed host resolution: {path}; version={version}; sha256={digest}");
    Ok((
        RuntimeExecutionIdentity {
            host: host.into(),
            platform: "terminal".into(),
            device_id: None,
            device_name: None,
            executable_path: Some(path),
            product_version: Some(version),
            executable_sha256: Some(digest.to_ascii_lowercase()),
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
    manifest.runtime_product.kind = "desktop-bundle".into();
    manifest.runtime_product.identifier = bundle.clone();
    manifest.runtime_product.identity_source =
        format!("typed host display-name resolution: {app_path}");
    Ok((
        RuntimeExecutionIdentity {
            host: host.into(),
            platform: "macos".into(),
            device_id: None,
            device_name: Some(app_path),
            executable_path: None,
            product_version: None,
            executable_sha256: None,
        },
        vec![search, metadata],
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
        || proof.evidence_sha256.len() != 64
        || !proof.evidence_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        || !proof.installed
        || !proof.first_run_completed
        || proof.pending_permission_prompts != 0
        || proof.pending_notification_prompts != 0
    {
        bail!("prepared-runtime proof does not bind exact product/device and zero pending prompts");
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
    {
        bail!("prepared-runtime helper did not prove the exact safe state");
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
    let result = (|| -> Result<Vec<Value>> {
        let host = host_report.get("host").and_then(Value::as_str).context("host report has no host")?;
        let (identity, mut checks) = match manifest.runtime_product.kind.as_str() {
            "ios-bundle" => (ios_booted_identity(host, host_report)?, Vec::new()),
            "android-package" => (android_device_identity(host, host_report)?, Vec::new()),
            "desktop-display-name" => resolve_desktop_identity(manifest, host)?,
            "cli-binary" | "tui-slug" => resolve_terminal_identity(manifest, host)?,
            "url" => (
                RuntimeExecutionIdentity {
                    host: host.into(),
                    platform: if manifest.engine == "web" { "weles".into() } else { "http".into() },
                    device_id: None,
                    device_name: None,
                    executable_path: None,
                    product_version: None,
                    executable_sha256: None,
                },
                Vec::new(),
            ),
            kind => bail!("unsupported unresolved runtime product kind {kind}"),
        };
        let product = manifest.runtime_product.identifier.as_str();
        let check = match manifest.runtime_product.kind.as_str() {
            "ios-bundle" => {
                let udid = identity.device_id.as_deref().context("iOS identity has no UDID")?;
                host_probe(host, &["xcrun", "simctl", "get_app_container", udid, product])
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
                    "--head",
                    "--write-out",
                    "\n%{http_code} %{url_effective}",
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
            let last = output.lines().last().unwrap_or_default();
            let (status, effective) = last.split_once(' ').context("URL identity probe has no status and effective URL")?;
            let status: u16 = status.parse().context("URL identity probe status is invalid")?;
            if !(200..300).contains(&status) || effective != product {
                bail!("URL probe did not resolve the exact declared surface");
            }
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
        Err(error) => json!({
            "schema": "wisent.crawl-record-preflight.v2",
            "record": manifest.record,
            "ready": false,
            "runtime_product": manifest.runtime_product,
            "account": manifest.account,
            "diagnostic": {"code": "runtime_identity_or_readiness_unavailable", "message": error.to_string()},
            "checks": [],
        }),
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
    let state = if states.iter().all(|state| *state == "imported") && !states.is_empty() {
        "imported"
    } else if states.iter().all(|state| matches!(*state, "completed" | "uploaded" | "imported"))
        && !states.is_empty()
    {
        "completed"
    } else if states.iter().any(|state| *state == "running") {
        "running"
    } else if states.iter().any(|state| *state == "pending_review") {
        "pending_review"
    } else if states.iter().any(|state| *state == "queued") {
        if states.iter().any(|state| {
            matches!(
                *state,
                "unavailable" | "preflight_failed" | "submission_failed" | "lost" | "failed" | "cancelled"
            )
        }) {
            "partial"
        } else {
            "queued"
        }
    } else if states.iter().any(|state| matches!(*state, "completed" | "uploaded" | "imported" | "partial")) {
        "partial"
    } else if states.iter().any(|state| *state == "planned") {
        "planned"
    } else {
        "failed"
    };
    entry["state"] = json!(state);
}

fn persist_submission_receipt(
    run_id: &str,
    catalog: &str,
    record: &str,
    receipt: &Value,
) -> Result<()> {
    atomic_json_write(
        &Path::new(RUN_ROOT)
            .join(run_id)
            .join("receipts")
            .join(catalog)
            .join(format!("{record}.json")),
        receipt,
    )
}

fn continue_start(run: &mut Value) -> Result<()> {
    let run_id = run.get("run_id").and_then(Value::as_str).context("run has no id")?.to_string();
    let admission_url = run.get("admission_url").and_then(Value::as_str).unwrap_or_default().to_string();
    let catalog_count = run.get("catalogs").and_then(Value::as_array).map(Vec::len).unwrap_or(0);
    for catalog_index in 0..catalog_count {
        let catalog = run["catalogs"][catalog_index]["catalog"].as_str().unwrap_or_default().to_string();
        let engine = run["catalogs"][catalog_index]["engine"].as_str().unwrap_or_default().to_string();
        let host = run["catalogs"][catalog_index]["host"].as_str().unwrap_or_default().to_string();
        if run["catalogs"][catalog_index]["host_preflight"].is_null() {
            run["catalogs"][catalog_index]["host_preflight"] =
                host_preflight(&catalog, &engine, &host, &admission_url);
            run["catalogs"][catalog_index]["state"] = json!("preflighting");
            persist(run)?;
        }
        let host_report = run["catalogs"][catalog_index]["host_preflight"].clone();
        let record_count = run["catalogs"][catalog_index]["records"]
            .as_array().map(Vec::len).unwrap_or(0);
        for record_index in 0..record_count {
            let state = run["catalogs"][catalog_index]["records"][record_index]["state"]
                .as_str().unwrap_or("unavailable").to_string();
            let has_job = run["catalogs"][catalog_index]["records"][record_index]["job_id"].as_str().is_some();
            if has_job || matches!(
                state.as_str(),
                "unavailable" | "completed" | "uploaded" | "imported" | "running" | "queued" | "pending_review"
            ) {
                continue;
            }
            let mut manifest: RuntimeManifest = match serde_json::from_value(
                run["catalogs"][catalog_index]["records"][record_index]["manifest"].clone(),
            ) {
                Ok(manifest) => manifest,
                Err(error) => {
                    run["catalogs"][catalog_index]["records"][record_index]["state"] = json!("unavailable");
                    run["catalogs"][catalog_index]["records"][record_index]["diagnostic"] =
                        json!({"code": "runtime_manifest_invalid", "message": error.to_string()});
                    persist(run)?;
                    continue;
                }
            };
            let mut preflight = record_preflight(&mut manifest, &host_report);
            let mut ready = preflight.get("ready").and_then(Value::as_bool) == Some(true);
            if ready {
                let reference_path = Path::new(&manifest.catalog)
                    .join("references")
                    .join(&manifest.record)
                    .join("reference.json");
                match std::fs::read(&reference_path)
                    .map_err(anyhow::Error::from)
                    .and_then(|bytes| finalize_manifest_identity(&mut manifest, &bytes))
                {
                    Ok(()) => {}
                    Err(error) => {
                        ready = false;
                        preflight = json!({
                            "schema": "wisent.crawl-record-preflight.v2",
                            "record": manifest.record,
                            "ready": false,
                            "diagnostic": {
                                "code": "runtime_manifest_finalization_failed",
                                "message": error.to_string(),
                                "path": reference_path,
                            },
                        });
                    }
                }
            }
            run["catalogs"][catalog_index]["records"][record_index]["manifest"] =
                serde_json::to_value(&manifest)?;
            run["catalogs"][catalog_index]["records"][record_index]["preflight"] = preflight.clone();
            if ready {
                let command = engine_command(&manifest, &host, &admission_url)?;
                run["catalogs"][catalog_index]["records"][record_index]["command"] = json!(command);
                run["catalogs"][catalog_index]["records"][record_index]["state"] = json!("preflight_passed");
                run["catalogs"][catalog_index]["records"][record_index]["diagnostic"] = Value::Null;
            } else {
                run["catalogs"][catalog_index]["records"][record_index]["state"] = json!("unavailable");
                run["catalogs"][catalog_index]["records"][record_index]["diagnostic"] =
                    preflight.get("diagnostic").cloned().unwrap_or_else(|| {
                        json!({"code": "record_preflight_failed", "message": "exact record prerequisite failed"})
                    });
            }
            persist(run)?;
            if !ready {
                continue;
            }
            let command = run["catalogs"][catalog_index]["records"][record_index]["command"]
                .as_array().into_iter().flatten().filter_map(Value::as_str)
                .map(str::to_string).collect::<Vec<_>>();
            match invoke_engine(&command) {
                Ok(output) if output.status.success() => match parse_submission(&output.stdout) {
                    Ok(receipt) => {
                        let stado = receipt.get("stado_receipt");
                        if stado.and_then(|value| value.get("source_revision")).and_then(Value::as_str)
                            != Some(manifest.source_revision.as_str())
                        {
                            run["catalogs"][catalog_index]["records"][record_index]["state"] =
                                json!("submission_failed");
                            run["catalogs"][catalog_index]["records"][record_index]["diagnostic"] =
                                json!({"code": "submission_source_mismatch", "message": "Stado receipt does not bind the immutable Spis source revision"});
                        } else {
                            persist_submission_receipt(&run_id, &catalog, &manifest.record, &receipt)?;
                            run["catalogs"][catalog_index]["records"][record_index]["job_id"] =
                                receipt.get("job_id").cloned().unwrap_or(Value::Null);
                            run["catalogs"][catalog_index]["records"][record_index]["artifact_uri"] =
                                receipt.get("artifact_uri").cloned().unwrap_or_else(|| json!(manifest.artifact_uri));
                            run["catalogs"][catalog_index]["records"][record_index]["output_uri"] =
                                receipt.get("output_uri").cloned().unwrap_or_else(|| json!(manifest.output_uri));
                            run["catalogs"][catalog_index]["records"][record_index]["submission_receipt"] = receipt;
                            run["catalogs"][catalog_index]["records"][record_index]["state"] = json!("queued");
                            run["catalogs"][catalog_index]["records"][record_index]["diagnostic"] = Value::Null;
                        }
                    }
                    Err(error) => {
                        run["catalogs"][catalog_index]["records"][record_index]["state"] = json!("submission_failed");
                        run["catalogs"][catalog_index]["records"][record_index]["diagnostic"] =
                            json!({"code": "submission_receipt_invalid", "message": error.to_string()});
                    }
                },
                Ok(output) => {
                    run["catalogs"][catalog_index]["records"][record_index]["state"] = json!("submission_failed");
                    run["catalogs"][catalog_index]["records"][record_index]["diagnostic"] = json!({
                        "code": "stado_submission_failed",
                        "message": String::from_utf8_lossy(&output.stderr).trim(),
                        "stdout": String::from_utf8_lossy(&output.stdout).trim(),
                    });
                }
                Err(error) => {
                    run["catalogs"][catalog_index]["records"][record_index]["state"] = json!("submission_failed");
                    run["catalogs"][catalog_index]["records"][record_index]["diagnostic"] =
                        json!({"code": "crawler_coordinator_launch_failed", "message": format!("{error:#}")});
                }
            }
            persist(run)?;
        }
        aggregate_catalog_entry(&mut run["catalogs"][catalog_index]);
        update_run_state(run);
        persist(run)?;
    }
    update_run_state(run);
    Ok(())
}

fn start(rest: &[String]) -> Result<()> {
    let mut hosts: BTreeMap<String, String> = BTreeMap::new();
    let mut admission_url = None;
    let mut catalogs = Vec::new();
    let mut record = None;
    let mut requested_run_id = None;
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
            "--admission-url" => {
                i += 1;
                admission_url = Some(rest.get(i).context("--admission-url needs a value")?.clone());
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
    let (discovered_hosts, discovered_admission_url) = registry_placements()?;
    let needs_web = specs.iter().any(|(_, engine)| *engine == "web");
    let admission_url = admission_url.or(discovered_admission_url);
    if needs_web && admission_url.is_none() {
        bail!("Stado service directory does not expose the Weles endpoint for this caller");
    }
    let admission_url = admission_url.unwrap_or_default();
    let run_id = requested_run_id.unwrap_or_else(|| {
        format!("crawl-{}", crate::now_iso_utc().replace(':', "-").replace('T', "-"))
    });
    let _guard = RunMutationGuard::acquire(&run_id)?;
    let request_identity = json!({
        "source_revision": source_revision,
        "catalogs": specs,
        "record": record,
        "hosts": hosts,
        "admission_url": admission_url,
    });
    let request_digest = crate::sha256_hex(&serde_json::to_vec(&request_identity)?);
    if run_path(&run_id).is_file() {
        let mut run = load(Some(&run_id))?;
        if run.get("request_digest").and_then(Value::as_str) != Some(&request_digest) {
            bail!("run id {run_id} already belongs to a different exact crawl request");
        }
        continue_start(&mut run)?;
        print_operation("start", &run, None)?;
        if has_failures(&run) {
            bail!("one or more records remain unavailable or failed");
        }
        return Ok(());
    }
    let mut entries = Vec::new();
    for (catalog, engine) in specs {
        let host = host_for(catalog, engine, &hosts, &discovered_hosts)?;
        let records = record_directories(catalog, record.as_deref())?
            .iter()
            .map(|path| planned_record(&run_id, &source_revision, catalog, engine, &host, path))
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
        "admission_url": admission_url,
        "state": "planned",
        "catalogs": entries,
    });
    persist(&mut run)?;
    continue_start(&mut run)?;
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
    let Some(job_id) = entry.get("job_id").and_then(Value::as_str).map(str::to_string) else {
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
                        "job_id": job_id,
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
                "job_id": job_id,
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

fn update_run_state(run: &mut Value) {
    let states: Vec<&str> = run
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("state").and_then(Value::as_str))
        .collect();
    let state = if states.iter().all(|state| *state == "imported") && !states.is_empty() {
        "imported"
    } else if states.iter().all(|state| matches!(*state, "completed" | "uploaded" | "imported"))
        && !states.is_empty()
    {
        "completed"
    } else if states.iter().any(|state| *state == "running") {
        "running"
    } else if states.iter().any(|state| *state == "pending_review") {
        "pending_review"
    } else if states.iter().any(|state| matches!(*state, "queued" | "planned" | "preflighting")) {
        if states.iter().any(|state| matches!(*state, "failed" | "partial")) {
            "partial"
        } else {
            "queued"
        }
    } else if states.iter().any(|state| matches!(*state, "partial" | "completed" | "uploaded" | "imported")) {
        "partial"
    } else {
        "failed"
    };
    run["state"] = json!(state);
}

fn status(rest: &[String]) -> Result<()> {
    let (run_id, record) = parse_run_and_record(rest, false)?;
    let selected = load(run_id.as_deref())?;
    let selected_id = selected.get("run_id").and_then(Value::as_str).context("run has no id")?.to_string();
    let _guard = RunMutationGuard::acquire(&selected_id)?;
    let mut run = load(Some(&selected_id))?;
    refresh(&mut run);
    persist(&mut run)?;
    print_operation("status", &run, record.as_deref())
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
            let result = if state == "lost" || entry.get("job_id").and_then(Value::as_str).is_none() {
                submit_retained_command(entry)
            } else {
                rerun_job(entry.get("job_id").and_then(Value::as_str).unwrap())
                    .map(|fresh| json!({"job_id": fresh}))
            };
            match result {
                Ok(receipt) => {
                    entry["job_id"] = receipt.get("job_id").cloned().unwrap_or(Value::Null);
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
        let run_dir = run_path(&id).parent().context("run path has no parent")?.to_path_buf();
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
    let root = Path::new(catalog).join("references");
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
    let persisted_run: Option<Value> = crate::read_json(run_path(run_id).to_str().unwrap_or_default()).ok();
    let source_revision = persisted_run.and_then(|run| run.get("source_revision").cloned()).unwrap_or(Value::Null);
    let job = report.get("job").unwrap_or(report);
    json!({
        "schema": "wisent.crawl-import.v1",
        "run_id": run_id,
        "source_revision": source_revision,
        "job_id": job_id,
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
    let references = Path::new(catalog).join("references");
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
    let job_id = entry.get("job_id").and_then(Value::as_str).map(str::to_string);
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
    let image_path = Path::new(catalog).join("images").join(image_name);
    if let Some(parent) = image_path.parent() { std::fs::create_dir_all(parent)?; }
    std::fs::copy(source_path, &image_path)?;
    let bytes = std::fs::read(&image_path)?;
    let decoded = image::open(&image_path)?;
    let sources_path = Path::new(catalog).join("sources.json");
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
    let reference = Path::new(catalog).join("references");
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
    let run_dir = run_path(&id).parent().context("run path has no parent")?.to_path_buf();
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
    println!("usage:\n  spis crawl start --host TARGET [--catalog SLUG ...] [--record SLUG] [--admission-url URL]\n  spis crawl status [--run RUN_ID] [--record SLUG]\n  spis crawl resume --run RUN_ID\n  spis crawl import --run RUN_ID\n\nAll commands emit one wisent.crawl-operation.v1 JSON document on stdout. The CLI is the process API; Spis does not expose a second HTTP /v1/crawl surface.");
}

pub fn run(rest: &[String]) -> Result<()> {
    match rest.first().map(String::as_str) {
        Some("start") => start(&rest[1..]),
        Some("status") => status(&rest[1..]),
        Some("resume") => resume(&rest[1..]),
        Some("import") => import(&rest[1..]),
        Some("--help" | "-h") | None => { usage(); Ok(()) }
        Some(other) => bail!("unknown crawl operation: {other}"),
    }
}
