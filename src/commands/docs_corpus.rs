//! `spis docs-corpus` — JSON views and immutable Stado artifact import for
//! documentation retrieval attempts. stdout carries exactly one JSON document.

use crate as lib;
use anyhow::{bail, Context as _, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::io::{BufRead, Read, Write};
use std::path::{Component, Path, PathBuf};

const MAX_DISCOVERY_DEPTH: usize = 16;
const MAX_DISCOVERY_DIRECTORIES: usize = 100_000;
const MAX_IMPORTED_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_IMPORTED_CORPUS_BYTES: u64 = 1536 * 1024 * 1024;
const MAX_OUTCOME_JOURNAL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_PAGE_RECORDS: usize = 50_000;
const MAX_PAGE_RECORD_BYTES: usize = 128 * 1024 * 1024;
const MAX_DECOMPRESSED_CORPUS_BYTES: u64 = 3 * 1024 * 1024 * 1024;
const MAX_TOTAL_PAGE_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_TOTAL_INVENTORY_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOTAL_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_METADATA_BYTES: u64 = 256 * 1024 * 1024;
const CORPUS_FILES: [&str; 4] = [
    "docs-retrieval-run.json",
    "outcomes.jsonl",
    "pages.jsonl.gz",
    "state.json",
];

fn engine_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("documentation-site-examples/content-structure")
}

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .context("HOME is not set; cannot locate the durable Stado work root")
}

fn crawl_root() -> Result<PathBuf> {
    Ok(home_dir()?.join(".spis/crawls"))
}

fn imports_root() -> Result<PathBuf> {
    Ok(home_dir()?.join(".stado/work/spis/docs-corpus-imports"))
}

/// Where a discovered corpus came from. `Imported` corpora were installed by
/// `import_artifact` and therefore have a sibling `artifact.tar.gz` receipt
/// archive; `Local` corpora were written in place by `crawl-docs --worker` and
/// have no archive next to them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CorpusOrigin {
    Local,
    Imported,
}

#[derive(Clone)]
struct AttemptCorpus {
    slug: String,
    corpus_dir: PathBuf,
    origin: CorpusOrigin,
    completed_at: String,
    attempt: u64,
    attempt_id: String,
    retrieval_status: String,
    state: Value,
    report: Value,
}

struct SiteInfo {
    slug: String,
    name: String,
    category: String,
    source_url: String,
    inventory_url_count: i64,
    seen: usize,
    cumulative_ok: usize,
    noise: usize,
    retrieval_status: Option<String>,
    attempt: Option<u64>,
    attempt_id: Option<String>,
    corpus_dir: Option<PathBuf>,
}

fn open_regular_read(path: &Path, label: &str) -> Result<File> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        bail!("{label} is not a regular non-symlink file: {}", path.display());
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .with_context(|| format!("open {label} {}", path.display()))?;
    if !file.metadata()?.is_file() {
        bail!("{label} opened as a non-regular file");
    }
    Ok(file)
}
fn existing_regular_directory(path: &Path, label: &str) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
                bail!("{label} is not a regular non-symlink directory: {}", path.display());
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}


fn read_json(path: &Path) -> Result<Value> {
    let mut file = open_regular_read(path, "documentation corpus metadata")?;
    if file.metadata()?.len() > MAX_METADATA_BYTES {
        bail!("documentation corpus metadata exceeds its byte limit");
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}
fn read_last_worker_report(path: &Path) -> Result<Value> {
    let mut file = open_regular_read(path, "documentation worker output")?;
    if file.metadata()?.len() > MAX_METADATA_BYTES {
        bail!("documentation worker output exceeds its byte limit");
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    bytes
        .split(|byte| *byte == b'\n')
        .rev()
        .filter_map(|line| serde_json::from_slice::<Value>(line).ok())
        .find(|value| {
            value.get("schema").and_then(Value::as_str)
                == Some("wisent.docs-worker-report.v1")
        })
        .context("documentation worker output has no typed documentation worker report")
}


fn matching_string(left: &Value, right: &Value, field: &str) -> Result<()> {
    let left_value = left
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("durable state has no {field}"))?;
    let right_value = right
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("retrieval report has no {field}"))?;
    if left_value != right_value {
        bail!("durable state and retrieval report disagree on {field}");
    }
    Ok(())
}
fn hash_file(path: &Path) -> Result<(String, u64)> {
    let mut file = open_regular_read(path, "documentation corpus file")?;
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes = bytes.checked_add(read as u64).context("file byte counter overflow")?;
    }
    Ok((hex::encode(hasher.finalize()), bytes))
}

fn exact_lower_hex(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not a 64-character lowercase SHA-256 digest");
    }
    Ok(())
}

fn safe_component(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{label} is not a safe canonical URI component");
    }
    Ok(())
}

fn validate_completion_timestamp(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| !matches!(index, 4 | 7 | 10 | 13 | 16 | 19) && !byte.is_ascii_digit())
    {
        bail!("completion timestamp is not canonical UTC RFC3339");
    }
    let number = |range: std::ops::Range<usize>| -> Result<u32> {
        Ok(std::str::from_utf8(&bytes[range])?.parse()?)
    };
    let year = number(0..4)?;
    let month = number(5..7)?;
    let day = number(8..10)?;
    let hour = number(11..13)?;
    let minute = number(14..16)?;
    let second = number(17..19)?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0
        || day == 0
        || day > month_days
        || hour > 23
        || minute > 59
        || second > 59
    {
        bail!("completion timestamp is not a valid UTC RFC3339 instant");
    }
    Ok(())
}

fn validate_manifest_coordinates(report: &Value, required_uri: Option<&str>) -> Result<String> {
    let manifest = report
        .get("runtime_manifest")
        .context("retrieval report has no runtime_manifest")?;
    for field in [
        "run_id",
        "record",
        "record_key",
        "attempt_id",
        "source_revision",
        "source_input_sha256",
    ] {
        let report_value = report
            .get(field)
            .and_then(Value::as_str)
            .with_context(|| format!("retrieval report has no {field}"))?;
        let manifest_value = manifest
            .get(field)
            .and_then(Value::as_str)
            .with_context(|| format!("runtime manifest has no {field}"))?;
        if report_value != manifest_value {
            bail!("retrieval report and runtime manifest disagree on {field}");
        }
    }
    let run_id = report["run_id"].as_str().unwrap();
    let catalog = manifest
        .get("catalog")
        .and_then(Value::as_str)
        .context("runtime manifest has no catalog")?;
    let record = report["record"].as_str().unwrap();
    let record_key = report["record_key"].as_str().unwrap();
    let attempt = report
        .get("attempt")
        .and_then(Value::as_u64)
        .filter(|attempt| *attempt > 0)
        .context("retrieval report has no positive attempt")?;
    if manifest.get("attempt").and_then(Value::as_u64) != Some(attempt) {
        bail!("retrieval report and runtime manifest disagree on attempt");
    }
    let attempt_id = report["attempt_id"].as_str().unwrap();
    for (value, label) in [
        (run_id, "run_id"),
        (catalog, "catalog"),
        (record, "record"),
        (attempt_id, "attempt_id"),
    ] {
        safe_component(value, label)?;
    }
    exact_lower_hex(record_key, "record_key")?;
    let base = format!(
        "stado://spis-crawls/{run_id}/{catalog}/{record}/{record_key}/attempts/{attempt}/{attempt_id}"
    );
    let artifact_uri = manifest
        .get("artifact_uri")
        .and_then(Value::as_str)
        .context("runtime manifest has no artifact_uri")?;
    let output_uri = manifest
        .get("output_uri")
        .and_then(Value::as_str)
        .context("runtime manifest has no output_uri")?;
    if artifact_uri != format!("{base}/artifacts.tar.gz")
        || output_uri != format!("{base}/worker-output.log")
    {
        bail!("runtime manifest does not use canonical immutable attempt URIs");
    }
    if required_uri.is_some_and(|required| required != artifact_uri) {
        bail!("imported retrieval artifact does not identify the requested Stado URI");
    }
    Ok(artifact_uri.to_string())
}

fn validate_current_definition(report: &Value) -> Result<()> {
    let slug = report["record"].as_str().unwrap();
    let structure_path = engine_root().join(format!("{slug}.json"));
    let structure_bytes = std::fs::read(&structure_path).with_context(|| {
        format!(
            "current committed documentation definition for {slug} is unavailable at {}",
            structure_path.display()
        )
    })?;
    let structure_sha256 = lib::sha256_hex(&structure_bytes);
    let reported_structure = report
        .get("structure_sha256")
        .and_then(Value::as_str)
        .context("retrieval report has no structure_sha256")?;
    let manifest_structure = report
        .pointer("/runtime_manifest/docs_structure_sha256")
        .and_then(Value::as_str)
        .context("runtime manifest has no docs_structure_sha256")?;
    if structure_sha256 != reported_structure || structure_sha256 != manifest_structure {
        bail!("retrieval corpus is stale relative to the current committed documentation definition");
    }
    let structure: Value = serde_json::from_slice(&structure_bytes)?;
    let declared_source = report
        .get("declared_source_url")
        .or_else(|| report.get("source_url"))
        .and_then(Value::as_str)
        .context("retrieval report has no declared source URL")?;
    if structure.get("source_url").and_then(Value::as_str) != Some(declared_source)
        || report
            .pointer("/runtime_manifest/runtime_product/declared_identifier")
            .and_then(Value::as_str)
            != Some(declared_source)
    {
        bail!("retrieval corpus source URL differs from the current committed definition");
    }
    let definition_path = engine_root().join("full-text-manifest.json");
    let definition_sha256 = lib::sha256_hex(&std::fs::read(&definition_path)?);
    if report.get("definition_sha256").and_then(Value::as_str)
        != Some(definition_sha256.as_str())
    {
        bail!("retrieval corpus is stale relative to the current crawl definition");
    }
    Ok(())
}

fn validate_journal(corpus_dir: &Path, state: &Value) -> Result<()> {
    let path = corpus_dir.join("outcomes.jsonl");
    let mut journal = open_regular_read(&path, "outcome journal")?;
    if journal.metadata()?.len() > MAX_OUTCOME_JOURNAL_BYTES {
        bail!("outcome journal exceeds its durable byte limit");
    }
    let mut bytes = Vec::new();
    journal.read_to_end(&mut bytes)?;
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        bail!("completed outcome journal has an incomplete trailing record");
    }
    let state_outcomes = state
        .get("outcomes")
        .and_then(Value::as_object)
        .context("durable state has no outcomes object")?;
    let mut reconstructed = serde_json::Map::new();
    let mut committed_bytes = 0u64;
    let mut committed_sha256 = lib::sha256_hex(&[]);
    for line in bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let batch: Value = serde_json::from_slice(line)?;
        let entries = batch
            .get("outcomes")
            .and_then(Value::as_array)
            .context("outcome journal batch has no outcomes")?;
        let first = batch
            .get("first_sequence")
            .and_then(Value::as_u64)
            .context("outcome journal batch has no first_sequence")? as usize;
        let last = batch
            .get("last_sequence")
            .and_then(Value::as_u64)
            .context("outcome journal batch has no last_sequence")? as usize;
        if batch.get("schema").and_then(Value::as_str) != Some("wisent.docs-outcome-batch.v1")
            || entries.is_empty()
            || entries.len() > 32
            || first != reconstructed.len()
            || last + 1 != first.saturating_add(entries.len())
        {
            bail!("outcome journal is not canonical and contiguous");
        }
        for entry in entries {
            let key = entry
                .get("key")
                .and_then(Value::as_str)
                .context("outcome journal entry has no key")?;
            let outcome = entry
                .get("outcome")
                .context("outcome journal entry has no outcome")?
                .clone();
            if reconstructed.insert(key.to_string(), outcome).is_some() {
                bail!("outcome journal repeats a target key");
            }
        }
        committed_bytes = batch
            .get("committed_bytes")
            .and_then(Value::as_u64)
            .context("outcome journal batch has no committed_bytes")?;
        committed_sha256 = batch
            .get("committed_sha256")
            .and_then(Value::as_str)
            .context("outcome journal batch has no committed_sha256")?
            .to_string();
        exact_lower_hex(&committed_sha256, "journal committed_sha256")?;
    }
    if &reconstructed != state_outcomes
        || state.get("committed_bytes").and_then(Value::as_u64) != Some(committed_bytes)
        || state.get("committed_sha256").and_then(Value::as_str)
            != Some(committed_sha256.as_str())
    {
        bail!("outcome journal does not reconstruct the completed durable state");
    }
    Ok(())
}

fn validate_outcomes(corpus_dir: &Path, state: &Value, report: &Value) -> Result<()> {
    let targets = state
        .get("targets")
        .and_then(Value::as_array)
        .context("durable state has no targets")?;
    let inventory_downloaded_bytes = state
        .get("inventory_downloaded_bytes")
        .and_then(Value::as_u64)
        .context("durable state has no inventory_downloaded_bytes")?;
    let inventory_descriptor = json!({
        "targets": targets,
        "diagnostics": state
            .get("inventory_diagnostics")
            .context("durable state has no inventory diagnostics")?,
        "robots": state
            .get("robots")
            .context("durable state has no robots policy")?,
        "downloaded_bytes": inventory_downloaded_bytes,
    });
    let inventory_sha256 = lib::sha256_hex(&serde_json::to_vec(&inventory_descriptor)?);
    if state.get("inventory_sha256").and_then(Value::as_str)
        != Some(inventory_sha256.as_str())
    {
        bail!("durable inventory digest differs from its exact contents");
    }
    let outcomes = state
        .get("outcomes")
        .and_then(Value::as_object)
        .context("durable state has no outcomes")?;
    if targets.is_empty() || targets.len() > MAX_PAGE_RECORDS || outcomes.len() != targets.len() {
        bail!("durable target and outcome counts are invalid");
    }
    let declared = url::Url::parse(
        state
            .get("source_url")
            .and_then(Value::as_str)
            .context("durable state has no source_url")?,
    )?;
    let origin = declared.origin();
    let effective = url::Url::parse(
        state
            .get("effective_source_url")
            .and_then(Value::as_str)
            .context("durable state has no effective_source_url")?,
    )?;
    if effective.origin() != origin || effective.as_str() != state["effective_source_url"].as_str().unwrap() {
        bail!("durable effective source URL is noncanonical or cross-origin");
    }
    let mut committed_end = 0u64;
    let mut downloaded_bytes = 0u64;
    let mut retrieved_count = 0usize;
    let mut ok_count = 0usize;
    let mut text_page_count = 0usize;
    let mut outcome_diagnostic_count = 0usize;
    for (sequence, target) in targets.iter().enumerate() {
        let url = target
            .get("url")
            .and_then(Value::as_str)
            .context("durable target has no URL")?;
        let parsed = url::Url::parse(url)?;
        let key = target
            .get("key")
            .and_then(Value::as_str)
            .context("durable target has no key")?;
        if parsed.as_str() != url
            || parsed.origin() != origin
            || target.get("sequence").and_then(Value::as_u64) != Some(sequence as u64)
            || key != lib::sha256_hex(url.as_bytes())
        {
            bail!("durable target inventory is not canonical and ordered");
        }
        let outcome = outcomes
            .get(key)
            .context("durable state is missing a target outcome")?;
        let resolved = url::Url::parse(
            outcome
                .get("resolved_url")
                .and_then(Value::as_str)
                .context("durable outcome has no resolved_url")?,
        )?;
        if outcome.get("sequence").and_then(Value::as_u64) != Some(sequence as u64)
            || outcome.get("url").and_then(Value::as_str) != Some(url)
            || resolved.origin() != origin
            || resolved.as_str() != outcome["resolved_url"].as_str().unwrap()
        {
            bail!("durable outcome identity is not canonical");
        }
        downloaded_bytes = downloaded_bytes
            .checked_add(outcome.get("downloaded_bytes").and_then(Value::as_u64).unwrap_or(0))
            .context("durable download byte counter overflow")?;
        let digest = outcome.get("record_sha256").and_then(Value::as_str);
        let start = outcome.get("corpus_start").and_then(Value::as_u64);
        let end = outcome.get("corpus_end").and_then(Value::as_u64);
        match (digest, start, end) {
            (Some(digest), Some(start), Some(end)) => {
                exact_lower_hex(digest, "page record_sha256")?;
                if start != committed_end || end <= start {
                    bail!("durable page ranges are not contiguous");
                }
                committed_end = end;
                retrieved_count += 1;
            }
            (None, None, None) => {}
            _ => bail!("durable page outcome has incomplete corpus range metadata"),
        }
        let text_bytes = outcome.get("text_bytes").and_then(Value::as_u64);
        if digest.is_some() != text_bytes.is_some() {
            bail!("durable page text metadata is inconsistent");
        }
        if text_bytes.unwrap_or(0) > 0 {
            text_page_count += 1;
        }
        if outcome.get("status").and_then(Value::as_u64) == Some(200) {
            ok_count += 1;
        }
        if !outcome.get("diagnostic").unwrap_or(&Value::Null).is_null() {
            outcome_diagnostic_count += 1;
        }
    }
    if committed_end != state["committed_bytes"].as_u64().unwrap_or(u64::MAX)
        || downloaded_bytes > MAX_TOTAL_PAGE_DOWNLOAD_BYTES
    {
        bail!("durable corpus byte counters are inconsistent or exceed their limits");
    }
    let inventory_downloaded_bytes = state
        .get("inventory_downloaded_bytes")
        .and_then(Value::as_u64)
        .context("durable state has no inventory_downloaded_bytes")?;
    if inventory_downloaded_bytes > MAX_TOTAL_INVENTORY_BYTES {
        bail!("durable inventory download counter exceeds its limit");
    }
    let total_downloaded_bytes = inventory_downloaded_bytes
        .checked_add(downloaded_bytes)
        .filter(|bytes| *bytes <= MAX_TOTAL_DOWNLOAD_BYTES)
        .context("durable total download counter exceeds its limit")?;
    if report.get("inventory_downloaded_bytes").and_then(Value::as_u64)
        != Some(inventory_downloaded_bytes)
        || report.get("page_downloaded_bytes").and_then(Value::as_u64)
            != Some(downloaded_bytes)
        || report.get("downloaded_bytes").and_then(Value::as_u64)
            != Some(total_downloaded_bytes)
        || report
            .pointer("/retrieval/inventory_downloaded_bytes")
            .and_then(Value::as_u64)
            != Some(inventory_downloaded_bytes)
        || report
            .pointer("/retrieval/page_downloaded_bytes")
            .and_then(Value::as_u64)
            != Some(downloaded_bytes)
        || report.pointer("/retrieval/downloaded_bytes").and_then(Value::as_u64)
            != Some(total_downloaded_bytes)
    {
        bail!("retrieval report download byte counters differ from durable state");
    }
    let (pages_sha256, pages_bytes) = hash_file(&corpus_dir.join("pages.jsonl.gz"))?;
    let expected_sha256 = state
        .get("committed_sha256")
        .and_then(Value::as_str)
        .context("durable state has no committed_sha256")?;
    if report.get("pages_sha256").and_then(Value::as_str) != Some(pages_sha256.as_str())
        || report.get("pages_bytes").and_then(Value::as_u64) != Some(pages_bytes)
        || report.pointer("/retrieval/pages_sha256").and_then(Value::as_str)
            != Some(pages_sha256.as_str())
        || report.pointer("/retrieval/pages_bytes").and_then(Value::as_u64) != Some(pages_bytes)
    {
        bail!("retrieval report page digest or length differs from the exact corpus");
    }
    exact_lower_hex(expected_sha256, "state committed_sha256")?;
    if pages_bytes != committed_end || pages_sha256 != expected_sha256 {
        bail!("documentation gzip stream differs from the durable length or SHA-256");
    }
    let inventory_diagnostics = state
        .get("inventory_diagnostics")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let expected_status = if retrieved_count == 0 {
        "retrieval_empty"
    } else if text_page_count == 0 {
        "retrieval_no_text"
    } else if inventory_diagnostics + outcome_diagnostic_count != 0
        || retrieved_count != targets.len()
        || ok_count != targets.len()
        || text_page_count != targets.len()
    {
        "retrieval_partial"
    } else {
        "retrieval_complete"
    };
    if report.get("retrieval_status").and_then(Value::as_str) != Some(expected_status)
        || report.pointer("/retrieval/target_count").and_then(Value::as_u64)
            != Some(targets.len() as u64)
        || report.pointer("/retrieval/outcome_count").and_then(Value::as_u64)
            != Some(outcomes.len() as u64)
        || report.pointer("/retrieval/retrieved_count").and_then(Value::as_u64)
            != Some(retrieved_count as u64)
        || report.pointer("/retrieval/text_page_count").and_then(Value::as_u64)
            != Some(text_page_count as u64)
        || report.pointer("/retrieval/page_downloaded_bytes").and_then(Value::as_u64)
            != Some(downloaded_bytes)
    {
        bail!("retrieval report counts or completion status differ from durable outcomes");
    }
    validate_journal(corpus_dir, state)
}

fn validate_corpus(
    corpus_dir: &Path,
    required_uri: Option<&str>,
    origin: CorpusOrigin,
) -> Result<AttemptCorpus> {
    let mut observed = std::fs::read_dir(corpus_dir)
        .with_context(|| format!("list documentation corpus {}", corpus_dir.display()))?
        .map(|entry| entry.map(|value| value.file_name().to_string_lossy().to_string()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    observed.sort();
    let mut expected = CORPUS_FILES.iter().map(|name| name.to_string()).collect::<Vec<_>>();
    expected.sort();
    if observed != expected {
        bail!(
            "documentation corpus {} does not contain the exact retrieval artifact set",
            corpus_dir.display()
        );
    }

    let state_path = corpus_dir.join("state.json");
    let report_path = corpus_dir.join("docs-retrieval-run.json");
    let state = read_json(&state_path)?;
    let report = read_json(&report_path)?;
    if state.get("schema").and_then(Value::as_str) != Some("wisent.docs-crawl-state.v3") {
        bail!("documentation corpus uses an unsupported durable state schema");
    }
    if report.get("schema").and_then(Value::as_str)
        != Some("wisent.docs-retrieval-run.v2")
    {
        bail!("documentation corpus uses an unsupported retrieval report schema");
    }
    for field in [
        "run_id",
        "record",
        "record_key",
        "attempt_id",
        "source_revision",
        "source_input_sha256",
        "source_url",
        "effective_source_url",
        "completed_at",
    ] {
        matching_string(&state, &report, field)?;
    }
    let state_attempt = state
        .get("attempt")
        .and_then(Value::as_u64)
        .context("durable state has no immutable attempt")?;
    if report.get("attempt").and_then(Value::as_u64) != Some(state_attempt) {
        bail!("durable state and retrieval report disagree on attempt");
    }
    let report_sha256 = state
        .get("report_sha256")
        .and_then(Value::as_str)
        .context("completed durable state has no report_sha256")?;
    exact_lower_hex(report_sha256, "report_sha256")?;
    if hash_file(&report_path)?.0 != report_sha256 {
        bail!("retrieval report differs from the digest in durable state");
    }
    validate_manifest_coordinates(&report, required_uri)?;
    validate_current_definition(&report)?;
    validate_outcomes(corpus_dir, &state, &report)?;
    let inventory_downloaded_bytes = state
        .get("inventory_downloaded_bytes")
        .and_then(Value::as_u64)
        .context("durable state has no inventory_downloaded_bytes")?;
    if inventory_downloaded_bytes > 64 * 1024 * 1024
        || report
            .pointer("/retrieval/inventory_downloaded_bytes")
            .and_then(Value::as_u64)
            != Some(inventory_downloaded_bytes)
    {
        bail!("retrieval inventory byte accounting is inconsistent");
    }
    let slug = report
        .get("record")
        .and_then(Value::as_str)
        .context("retrieval report has no record")?
        .to_string();
    let completed_at = report
        .get("completed_at")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("retrieval report has no completion timestamp")?
        .to_string();
    validate_completion_timestamp(&completed_at)?;
    let attempt_id = report
        .get("attempt_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("retrieval report has no attempt_id")?
        .to_string();
    let retrieval_status = report
        .get("retrieval_status")
        .and_then(Value::as_str)
        .context("retrieval report has no retrieval_status")?
        .to_string();
    if !matches!(
        retrieval_status.as_str(),
        "retrieval_complete" | "retrieval_partial" | "retrieval_no_text" | "retrieval_empty"
    ) {
        bail!("retrieval report has an unsupported retrieval_status");
    }
    Ok(AttemptCorpus {
        slug,
        corpus_dir: corpus_dir.to_path_buf(),
        origin,
        completed_at,
        attempt: state_attempt,
        attempt_id,
        retrieval_status,
        state,
        report,
    })
}

fn visit_corpora(
    directory: &Path,
    depth: usize,
    visited: &mut usize,
    corpora: &mut Vec<AttemptCorpus>,
    origin: CorpusOrigin,
) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    *visited += 1;
    if *visited > MAX_DISCOVERY_DIRECTORIES {
        bail!("durable documentation corpus discovery exceeded its directory limit");
    }
    let metadata = std::fs::symlink_metadata(directory)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Ok(());
    }
    // A corpus is identified by its content, not by its directory name. The
    // durable layout writes the four artifacts straight into the attempt root
    // (`native_attempt_root`), while `import_artifact` stages them under a
    // directory literally named `corpus`; only a content test sees both.
    if CORPUS_FILES.iter().all(|name| directory.join(name).is_file()) {
        corpora.push(validate_corpus(directory, None, origin)?);
        return Ok(());
    }
    if depth == 0 {
        return Ok(());
    }
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            visit_corpora(&entry.path(), depth - 1, visited, corpora, origin)?;
        }
    }
    Ok(())
}

fn selected_corpora() -> Result<HashMap<String, AttemptCorpus>> {
    let mut candidates = Vec::new();
    let mut visited = 0usize;
    visit_corpora(
        &crawl_root()?,
        MAX_DISCOVERY_DEPTH,
        &mut visited,
        &mut candidates,
        CorpusOrigin::Local,
    )?;
    visit_corpora(
        &imports_root()?,
        MAX_DISCOVERY_DEPTH,
        &mut visited,
        &mut candidates,
        CorpusOrigin::Imported,
    )?;
    let mut selected = HashMap::<String, AttemptCorpus>::new();
    for candidate in candidates {
        let replace = selected.get(&candidate.slug).is_none_or(|current| {
            (&candidate.completed_at, candidate.attempt, &candidate.attempt_id)
                > (&current.completed_at, current.attempt, &current.attempt_id)
        });
        if replace {
            selected.insert(candidate.slug.clone(), candidate);
        }
    }
    Ok(selected)
}

fn collect_sites() -> Result<Vec<SiteInfo>> {
    let selected = selected_corpora()?;
    let mut out = Vec::new();
    for entry in std::fs::read_dir(engine_root())? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json")
            || path.file_name().and_then(|name| name.to_str()) == Some("full-text-manifest.json")
        {
            continue;
        }
        let meta = read_json(&path)?;
        let slug = path
            .file_stem()
            .and_then(|value| value.to_str())
            .context("documentation structure file has a non-UTF-8 name")?
            .to_string();
        let corpus = selected.get(&slug);
        let outcomes = corpus
            .and_then(|attempt| attempt.state.get("outcomes"))
            .and_then(Value::as_object);
        let seen = outcomes.map_or(0, serde_json::Map::len);
        let cumulative_ok = outcomes.map_or(0, |values| {
            values
                .values()
                .filter(|outcome| outcome.get("text_bytes").and_then(Value::as_u64).unwrap_or(0) > 0)
                .count()
        });
        let target_count = corpus
            .and_then(|attempt| attempt.report.pointer("/retrieval/target_count"))
            .and_then(Value::as_i64)
            .or_else(|| meta.get("inventory_url_count").and_then(Value::as_i64))
            .unwrap_or(0);
        out.push(SiteInfo {
            slug,
            name: meta.get("name").and_then(Value::as_str).unwrap_or_default().to_string(),
            category: meta
                .get("category")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            source_url: meta
                .get("source_url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            inventory_url_count: target_count,
            seen,
            cumulative_ok,
            noise: seen.saturating_sub(cumulative_ok),
            retrieval_status: corpus.map(|attempt| attempt.retrieval_status.clone()),
            attempt: corpus.map(|attempt| attempt.attempt),
            attempt_id: corpus.map(|attempt| attempt.attempt_id.clone()),
            corpus_dir: corpus.map(|attempt| attempt.corpus_dir.clone()),
        });
    }
    out.sort_by(|left, right| right.inventory_url_count.cmp(&left.inventory_url_count));
    Ok(out)
}

fn open_jsonl(
    corpus_dir: Option<&Path>,
) -> Result<Option<std::io::BufReader<flate2::read::MultiGzDecoder<File>>>> {
    let Some(corpus_dir) = corpus_dir else {
        return Ok(None);
    };
    let path = corpus_dir.join("pages.jsonl.gz");
    let file = open_regular_read(&path, "documentation pages")?;
    Ok(Some(std::io::BufReader::new(
        flate2::read::MultiGzDecoder::new(file),
    )))
}
fn read_corpus_record(
    reader: &mut impl BufRead,
    decompressed_bytes: &mut u64,
    record_count: &mut usize,
) -> Result<Option<Value>> {
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf().context("decompress documentation corpus")?;
        if available.is_empty() {
            if line.is_empty() {
                return Ok(None);
            }
            bail!("documentation corpus ends with an incomplete JSON record");
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(available.len(), |position| position + 1);
        let content = newline.map_or(consumed, |position| position);
        if line.len().saturating_add(content) > MAX_PAGE_RECORD_BYTES {
            bail!("documentation corpus record exceeds its decompressed byte limit");
        }
        *decompressed_bytes = decompressed_bytes
            .checked_add(consumed as u64)
            .filter(|bytes| *bytes <= MAX_DECOMPRESSED_CORPUS_BYTES)
            .context("documentation corpus exceeds its decompressed byte limit")?;
        line.extend_from_slice(&available[..content]);
        reader.consume(consumed);
        if newline.is_some() {
            *record_count = record_count
                .checked_add(1)
                .filter(|count| *count <= MAX_PAGE_RECORDS)
                .context("documentation corpus exceeds its record-count limit")?;
            let record = serde_json::from_slice(&line)
                .context("parse bounded documentation corpus record")?;
            return Ok(Some(record));
        }
    }
}


fn scan_site(
    site: &SiteInfo,
    query: &str,
    limit: usize,
    hits: &mut Vec<Value>,
) -> Result<usize> {
    let Some(mut reader) = open_jsonl(site.corpus_dir.as_deref())? else {
        return Ok(0);
    };
    let query_lower = query.to_lowercase();
    let mut scanned = 0usize;
    let mut decompressed_bytes = 0u64;
    let mut record_count = 0usize;
    while let Some(record) =
        read_corpus_record(&mut reader, &mut decompressed_bytes, &mut record_count)?
    {
        scanned += 1;
        let text = record.get("text").and_then(Value::as_str).unwrap_or("");
        let title = record.get("title").and_then(Value::as_str).unwrap_or("");
        let url = record.get("url").and_then(Value::as_str).unwrap_or("");
        let text_lower = text.to_lowercase();
        if !text_lower.contains(&query_lower)
            && !title.to_lowercase().contains(&query_lower)
            && !url.to_lowercase().contains(&query_lower)
        {
            continue;
        }
        let snippet = text_lower.find(&query_lower).map(|byte_position| {
            let start = text_lower[..byte_position]
                .char_indices()
                .rev()
                .nth(60)
                .map(|(offset, _)| offset)
                .unwrap_or(0);
            let requested_end = byte_position
                .saturating_add(query_lower.len())
                .saturating_add(120);
            let end = text_lower
                .char_indices()
                .find_map(|(offset, _)| (offset >= requested_end).then_some(offset))
                .unwrap_or(text_lower.len());
            text_lower[start..end].replace('\n', " ")
        });
        hits.push(json!({
            "slug": site.slug,
            "site": site.name,
            "url": url,
            "title": (!title.is_empty()).then_some(title),
            "snippet": snippet,
            "attempt": site.attempt,
            "attempt_id": site.attempt_id,
        }));
        if hits.len() >= limit {
            break;
        }
    }
    Ok(scanned)
}

fn archive_member_name(path: &Path) -> Result<String> {
    let components = path
        .components()
        .filter_map(|component| match component {
            Component::CurDir => None,
            Component::Normal(value) => Some(Ok(value.to_string_lossy().to_string())),
            _ => Some(Err(anyhow::anyhow!("retrieval archive contains an unsafe path"))),
        })
        .collect::<Result<Vec<_>>>()?;
    match components.as_slice() {
        [name] => Ok(name.clone()),
        _ => bail!("retrieval archive member is outside its exact corpus directory"),
    }
}

fn extract_corpus_archive(archive_path: &Path, corpus_dir: &Path) -> Result<()> {
    std::fs::create_dir(corpus_dir)?;
    let decoder = flate2::read::GzDecoder::new(open_regular_read(
        archive_path,
        "documentation archive",
    )?);
    let mut archive = tar::Archive::new(decoder);
    let expected = CORPUS_FILES.iter().copied().collect::<HashSet<_>>();
    let mut observed = HashSet::<String>::new();
    let mut total = 0u64;
    for entry in archive.entries().context("read retrieval archive entries")? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            bail!("retrieval archive contains a non-regular member");
        }
        let name = archive_member_name(&entry.path()?)?;
        if !expected.contains(name.as_str()) || !observed.insert(name.clone()) {
            bail!("retrieval archive contains an unexpected or duplicate member {name}");
        }
        total = total
            .checked_add(entry.size())
            .filter(|bytes| *bytes <= MAX_IMPORTED_CORPUS_BYTES)
            .context("retrieval archive exceeds the extracted corpus byte limit")?;
        let destination = corpus_dir.join(&name);
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&destination)?;
        std::io::copy(&mut entry, &mut output)?;
        output.flush()?;
        output.sync_all()?;
    }
    if observed.len() != expected.len() {
        bail!("retrieval archive does not contain the exact corpus artifact set");
    }
    File::open(corpus_dir)?.sync_all()?;
    Ok(())
}

fn validate_installed_import(
    destination: &Path,
    uri: &str,
    expected_archive_sha256: &str,
    expected_archive_bytes: u64,
) -> Result<AttemptCorpus> {
    let archive_path = destination.join("artifact.tar.gz");
    let (archive_sha256, archive_bytes) = hash_file(&archive_path)?;
    if archive_sha256 != expected_archive_sha256 || archive_bytes != expected_archive_bytes {
        bail!("installed immutable documentation artifact differs from the expected digest or length");
    }
    validate_corpus(
        &destination.join("corpus"),
        Some(uri),
        CorpusOrigin::Imported,
    )
}

fn import_artifact(
    uri: &str,
    expected_archive_sha256: &str,
    expected_archive_bytes: u64,
) -> Result<AttemptCorpus> {
    if !uri.starts_with("stado://spis-crawls/") || !uri.ends_with("/artifacts.tar.gz") {
        bail!("--artifact-uri must be an immutable Spis crawl artifact URI");
    }
    exact_lower_hex(expected_archive_sha256, "--archive-sha256")?;
    let root = imports_root()?;
    std::fs::create_dir_all(&root)?;
    let digest = lib::sha256_hex(uri.as_bytes());
    let destination = root.join(&digest);
    if existing_regular_directory(&destination, "installed documentation corpus")? {
        return validate_installed_import(
            &destination,
            uri,
            expected_archive_sha256,
            expected_archive_bytes,
        );
    }
    let staging = super::crawl_docs::staging_directory(&root, "corpus-import-stage")?;
    let archive_path = staging.join("artifact.tar.gz");
    let import_result = (|| -> Result<AttemptCorpus> {
        let mut command = super::crawl::stado_command();
        command.args(["storage", "get", uri]).arg(&archive_path);
        let output = super::crawl::bounded_command_output(
            &mut command,
            "download immutable documentation corpus artifact",
            std::time::Duration::from_secs(30 * 60),
            super::crawl_docs::STADO_OUTPUT_LIMIT,
        )?;
        if !output.status.success() {
            bail!(
                "stado storage get refused documentation corpus artifact: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        let archive = open_regular_read(&archive_path, "downloaded documentation archive")?;
        let archive_length = archive.metadata()?.len();
        if archive_length > MAX_IMPORTED_ARCHIVE_BYTES {
            bail!("documentation corpus artifact exceeds the import byte limit");
        }
        drop(archive);
        let (archive_sha256, archive_bytes) = hash_file(&archive_path)?;
        if archive_sha256 != expected_archive_sha256
            || archive_bytes != expected_archive_bytes
        {
            bail!("downloaded documentation artifact differs from the expected digest or length");
        }
        let staged_corpus = staging.join("corpus");
        extract_corpus_archive(&archive_path, &staged_corpus)?;
        validate_corpus(&staged_corpus, Some(uri), CorpusOrigin::Imported)?;
        open_regular_read(&archive_path, "downloaded documentation archive")?.sync_all()?;
        File::open(&staging)?.sync_all()?;
        match std::fs::rename(&staging, &destination) {
            Ok(()) => {
                File::open(&root)?.sync_all()?;
                validate_installed_import(
                    &destination,
                    uri,
                    expected_archive_sha256,
                    expected_archive_bytes,
                )
            }
            Err(_error)
                if existing_regular_directory(
                    &destination,
                    "concurrently installed documentation corpus",
                )? =>
            {
                std::fs::remove_dir_all(&staging)?;
                let existing = validate_installed_import(
                    &destination,
                    uri,
                    expected_archive_sha256,
                    expected_archive_bytes,
                )
                .context("concurrent import installed different content")?;
                File::open(&root)?.sync_all()?;
                Ok(existing)
            }
            Err(error) => Err(error).context("atomically install imported documentation corpus"),
        }
    })();
    if import_result.is_err() && staging.exists() {
        std::fs::remove_dir_all(&staging)?;
        File::open(&root)?.sync_all()?;
    }
    import_result
}
/// Validate one typed `wisent.docs-worker-report.v1` document in memory.
///
/// This is the single validation of a documentation worker report, shared by both
/// import paths so the same document is never checked twice by two independent
/// rules: `docs-corpus import --attempt-receipt`, which goes on to materialise the
/// corpus bytes, and `crawl.rs::import_record_attempt`'s `docs` arm, which binds
/// the typed corpus summary onto the reference record without installing a
/// readable corpus. Returns the artifact URI and its SHA-256.
pub(crate) fn validate_docs_worker_report(receipt: &Value) -> Result<(String, String)> {
    if receipt.get("schema").and_then(Value::as_str) != Some("wisent.docs-worker-report.v1")
        || receipt.get("engine").and_then(Value::as_str) != Some("docs")
        || receipt.get("state").and_then(Value::as_str) != Some("artifact_published")
        || !receipt.get("failure").unwrap_or(&Value::Null).is_null()
    {
        bail!("attempt receipt is not a successful typed documentation worker report");
    }
    let artifact = receipt
        .get("artifact")
        .and_then(Value::as_object)
        .context("attempt receipt has no artifact object")?;
    let uri = artifact
        .get("uri")
        .and_then(Value::as_str)
        .context("attempt receipt artifact has no URI")?
        .to_string();
    let archive_sha256 = artifact
        .get("sha256")
        .and_then(Value::as_str)
        .context("attempt receipt artifact has no SHA-256")?
        .to_string();
    exact_lower_hex(&archive_sha256, "attempt receipt artifact SHA-256")?;
    let archive_bytes = artifact
        .get("bytes")
        .and_then(Value::as_u64)
        .filter(|bytes| *bytes > 0 && *bytes <= MAX_IMPORTED_ARCHIVE_BYTES)
        .context("attempt receipt artifact has no valid byte length")?;
    if artifact.get("media_type").and_then(Value::as_str) != Some("application/gzip") {
        bail!("attempt receipt artifact media_type is not application/gzip");
    }
    let tree_entries = artifact
        .get("tree_entries")
        .and_then(Value::as_u64)
        .context("attempt receipt artifact has no tree_entries")?;
    let tree_bytes = artifact
        .get("tree_bytes")
        .and_then(Value::as_u64)
        .context("attempt receipt artifact has no tree_bytes")?;
    let corpus = receipt
        .get("corpus")
        .and_then(Value::as_object)
        .context("successful attempt receipt has no corpus summary")?;
    let corpus_files = corpus
        .get("files")
        .and_then(Value::as_u64)
        .filter(|files| *files == CORPUS_FILES.len() as u64)
        .context("attempt receipt corpus does not name the exact file count")?;
    let corpus_bytes = corpus
        .get("bytes")
        .and_then(Value::as_u64)
        .filter(|bytes| *bytes > 0 && *bytes <= MAX_IMPORTED_CORPUS_BYTES)
        .context("attempt receipt corpus has no valid byte count")?;
    corpus
        .get("pages")
        .and_then(Value::as_u64)
        .filter(|pages| *pages <= MAX_PAGE_RECORDS as u64)
        .context("attempt receipt corpus has no valid page count")?;
    if tree_entries != corpus_files || tree_bytes != corpus_bytes {
        bail!("attempt receipt artifact tree and corpus summaries disagree");
    }
    let source_revision = receipt
        .get("source_revision")
        .and_then(Value::as_str)
        .context("attempt receipt has no source_revision")?;
    if source_revision.len() != 40
        || !source_revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("attempt receipt source_revision is not a full lowercase commit SHA");
    }
    for field in [
        "record_key",
        "source_input_sha256",
        "reference_sha256",
        "bindings_file_sha256",
        "bindings_sha256",
        "docs_structure_sha256",
    ] {
        exact_lower_hex(
            receipt
                .get(field)
                .and_then(Value::as_str)
                .with_context(|| format!("attempt receipt has no {field}"))?,
            &format!("attempt receipt {field}"),
        )?;
    }
    receipt
        .get("execution_identity")
        .and_then(Value::as_object)
        .context("attempt receipt has no execution_identity object")?;
    let run_id = receipt
        .get("run_id")
        .and_then(Value::as_str)
        .context("attempt receipt has no run_id")?;
    let catalog = receipt
        .get("catalog")
        .and_then(Value::as_str)
        .context("attempt receipt has no catalog")?;
    let record = receipt
        .get("record")
        .and_then(Value::as_str)
        .context("attempt receipt has no record")?;
    let record_key = receipt
        .get("record_key")
        .and_then(Value::as_str)
        .context("attempt receipt has no record_key")?;
    let attempt = receipt
        .get("attempt")
        .and_then(Value::as_u64)
        .filter(|attempt| *attempt > 0)
        .context("attempt receipt has no positive attempt")?;
    let attempt_id = receipt
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("attempt receipt has no attempt_id")?;
    for (value, label) in [
        (run_id, "receipt run_id"),
        (catalog, "receipt catalog"),
        (record, "receipt record"),
        (attempt_id, "receipt attempt_id"),
    ] {
        safe_component(value, label)?;
    }
    let expected_uri = format!(
        "stado://spis-crawls/{run_id}/{catalog}/{record}/{record_key}/attempts/{attempt}/{attempt_id}/artifacts.tar.gz"
    );
    if uri != expected_uri {
        bail!("attempt receipt artifact URI does not match its immutable coordinates");
    }
    let _ = archive_bytes;
    Ok((uri, archive_sha256))
}

fn read_attempt_receipt(path: &Path) -> Result<(Value, String, String)> {
    let receipt = read_last_worker_report(path)?;
    let (uri, archive_sha256) = validate_docs_worker_report(&receipt)?;
    Ok((receipt, uri, archive_sha256))
}

fn validate_receipt_corpus(receipt: &Value, corpus: &AttemptCorpus) -> Result<()> {
    for field in [
        "run_id",
        "record",
        "record_key",
        "attempt_id",
        "source_revision",
        "source_input_sha256",
    ] {
        if receipt.get(field) != corpus.report.get(field) {
            bail!("attempt receipt and retrieval corpus disagree on {field}");
        }
    }
    for field in [
        "reference_sha256",
        "bindings_file_sha256",
        "bindings_sha256",
        "docs_structure_sha256",
    ] {
        if receipt.get(field) != corpus.report.pointer(&format!("/runtime_manifest/{field}")) {
            bail!("attempt receipt and runtime manifest disagree on {field}");
        }
    }
    if receipt.get("attempt") != corpus.report.get("attempt")
        || receipt.get("catalog") != corpus.report.pointer("/runtime_manifest/catalog")
        || receipt.get("execution_identity") != corpus.report.get("runtime_execution_identity")
        || receipt.get("docs_structure_sha256") != corpus.report.get("structure_sha256")
    {
        bail!("attempt receipt and retrieval corpus identity differ");
    }
    // The sibling `artifact.tar.gz` only exists for corpora that `import_artifact`
    // installed. A locally crawled corpus has no archive next to it, so refuse it
    // here rather than reporting a missing-file error from `hash_file`.
    if corpus.origin != CorpusOrigin::Imported {
        bail!(
            "attempt receipt validation requires an imported documentation corpus; {} was crawled locally and has no immutable artifact archive",
            corpus.corpus_dir.display()
        );
    }
    let archive_path = corpus
        .corpus_dir
        .parent()
        .context("imported corpus has no immutable archive parent")?
        .join("artifact.tar.gz");
    let (archive_sha256, archive_bytes) = hash_file(&archive_path)?;
    let artifact = receipt["artifact"]
        .as_object()
        .context("validated attempt receipt has no artifact")?;
    if artifact.get("sha256").and_then(Value::as_str) != Some(archive_sha256.as_str())
        || artifact.get("bytes").and_then(Value::as_u64) != Some(archive_bytes)
    {
        bail!("attempt receipt archive digest or byte length differs from imported artifact");
    }
    let mut corpus_bytes = 0u64;
    for name in CORPUS_FILES {
        corpus_bytes = corpus_bytes
            .checked_add(hash_file(&corpus.corpus_dir.join(name))?.1)
            .context("imported corpus byte count overflow")?;
    }
    let expected_pages = corpus
        .report
        .pointer("/retrieval/retrieved_count")
        .and_then(Value::as_u64)
        .context("retrieval report has no retrieved_count")?;
    if receipt.pointer("/corpus/files").and_then(Value::as_u64) != Some(CORPUS_FILES.len() as u64)
        || receipt.pointer("/corpus/bytes").and_then(Value::as_u64) != Some(corpus_bytes)
        || receipt.pointer("/corpus/pages").and_then(Value::as_u64) != Some(expected_pages)
    {
        bail!("attempt receipt corpus summary differs from imported corpus");
    }
    Ok(())
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut sub = "";
    let mut query = String::new();
    let mut site: Option<String> = None;
    let mut url_filter = String::new();
    let mut attempt_receipt: Option<PathBuf> = None;
    let mut limit = 20usize;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "status" | "search" | "show" | "import" => sub = rest[index].as_str(),
            "--query" => {
                index += 1;
                query = rest.get(index).context("--query needs a value")?.clone();
            }
            "--site" => {
                index += 1;
                site = Some(rest.get(index).context("--site needs a value")?.clone());
            }
            "--url" => {
                index += 1;
                url_filter = rest.get(index).context("--url needs a value")?.clone();
            }
            "--attempt-receipt" => {
                index += 1;
                attempt_receipt = Some(PathBuf::from(
                    rest.get(index)
                        .context("--attempt-receipt needs a value")?,
                ));
            }
            "--limit" => {
                index += 1;
                limit = rest.get(index).context("--limit needs a value")?.parse()?;
            }
            other => bail!("unknown argument: {other}"),
        }
        index += 1;
    }
    if sub.is_empty() {
        bail!("usage: spis docs-corpus status | search --query T [--site S] [--limit N] | show --site S --url U | import --attempt-receipt FILE");
    }
    if limit == 0 || limit > 10_000 {
        bail!("--limit must be between 1 and 10000");
    }

    match sub {
        "status" => {
            let sites = collect_sites()?;
            let output = sites
                .iter()
                .map(|site| {
                    json!({
                        "slug": site.slug,
                        "name": site.name,
                        "category": site.category,
                        "source_url": site.source_url,
                        "inventory_url_count": site.inventory_url_count,
                        "seen": site.seen,
                        "cumulative_ok": site.cumulative_ok,
                        "noise": site.noise,
                        "done": site.retrieval_status.as_deref() == Some("retrieval_complete"),
                        "retrieval_status": site.retrieval_status,
                        "attempt": site.attempt,
                        "attempt_id": site.attempt_id,
                    })
                })
                .collect::<Vec<_>>();
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
        "search" => {
            if query.is_empty() {
                bail!("--query required");
            }
            let sites = collect_sites()?;
            let mut hits = Vec::new();
            let mut scanned = 0usize;
            for candidate in &sites {
                if site.as_deref().is_some_and(|slug| slug != candidate.slug) {
                    continue;
                }
                scanned += scan_site(candidate, &query, limit, &mut hits)?;
                if hits.len() >= limit {
                    break;
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({"hits": hits, "scanned": scanned, "limit": limit})
                )?
            );
            Ok(())
        }
        "show" => {
            let slug = site.context("show needs --site <slug>")?;
            if url_filter.is_empty() {
                bail!("show needs --url <url>");
            }
            let selected = selected_corpora()?;
            let corpus = selected
                .get(&slug)
                .with_context(|| format!("no completed v2 retrieval corpus for {slug}"))?;
            let mut reader = open_jsonl(Some(&corpus.corpus_dir))?
                .context("selected retrieval corpus has no page stream")?;
            let mut decompressed_bytes = 0u64;
            let mut record_count = 0usize;
            while let Some(record) =
                read_corpus_record(&mut reader, &mut decompressed_bytes, &mut record_count)?
            {
                if record.get("url").and_then(Value::as_str) == Some(url_filter.as_str()) {
                    println!("{record}");
                    return Ok(());
                }
            }
            bail!("url not found in the completed {slug} retrieval corpus")
        }
        "import" => {
            let receipt_path =
                attempt_receipt.context("import needs --attempt-receipt <file>")?;
            let (receipt, uri, expected_sha256) = read_attempt_receipt(&receipt_path)?;
            let expected_bytes = receipt
                .pointer("/artifact/bytes")
                .and_then(Value::as_u64)
                .context("validated attempt receipt artifact has no bytes")?;
            let attempt = import_artifact(&uri, &expected_sha256, expected_bytes)?;
            validate_receipt_corpus(&receipt, &attempt)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "attempt_receipt": receipt_path,
                    "artifact_uri": uri,
                    "archive_sha256": expected_sha256,
                    "record": attempt.slug,
                    "attempt": attempt.attempt,
                    "attempt_id": attempt.attempt_id,
                    "completed_at": attempt.completed_at,
                    "retrieval_status": attempt.retrieval_status,
                    "corpus_dir": attempt.corpus_dir,
                }))?
            );
            Ok(())
        }
        _ => unreachable!(),
    }
}
