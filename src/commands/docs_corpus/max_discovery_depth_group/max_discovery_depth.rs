use super::*;

pub(crate) const MAX_DISCOVERY_DEPTH: usize = 16;

pub(crate) const MAX_DISCOVERY_DIRECTORIES: usize = 100_000;

pub(crate) const MAX_IMPORTED_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub(crate) const MAX_IMPORTED_CORPUS_BYTES: u64 = 1536 * 1024 * 1024;

pub(crate) const MAX_OUTCOME_JOURNAL_BYTES: u64 = 256 * 1024 * 1024;

/// The per-corpus page bound. One site is one corpus, so this is also the
/// most pages one documentation record can ever hold.
///
/// Public because the generated documentation states it: `docs_site` reads it
/// here rather than repeating the number in prose, so a documented bound
/// cannot drift from the enforced one.
pub(crate) const MAX_PAGE_RECORDS: usize = 50_000;

pub(crate) const MAX_PAGE_RECORD_BYTES: usize = 128 * 1024 * 1024;

pub(crate) const MAX_DECOMPRESSED_CORPUS_BYTES: u64 = 3 * 1024 * 1024 * 1024;

pub(crate) const MAX_TOTAL_PAGE_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub(crate) const MAX_TOTAL_INVENTORY_BYTES: u64 = 64 * 1024 * 1024;

pub(crate) const MAX_TOTAL_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub(crate) const MAX_METADATA_BYTES: u64 = 256 * 1024 * 1024;

pub(crate) const CORPUS_FILES: [&str; 4] = [
    "docs-retrieval-run.json",
    "outcomes.jsonl",
    "pages.jsonl.gz",
    "state.json",
];

pub(crate) fn engine_root() -> PathBuf {
    super::corpus::data_root().join("documentation-site-examples/content-structure")
}

pub(crate) fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .context("HOME is not set; cannot locate the durable Stado work root")
}

pub(crate) fn crawl_root() -> Result<PathBuf> {
    Ok(home_dir()?.join(".spis/crawls"))
}

pub(crate) fn imports_root() -> Result<PathBuf> {
    Ok(home_dir()?.join(".stado/work/spis/docs-corpus-imports"))
}

/// Where a discovered corpus came from. `Imported` corpora were installed by
/// `import_artifact` and therefore have a sibling `artifact.tar.gz` receipt
/// archive; `Local` corpora were written in place by `crawl-docs --worker` and
/// have no archive next to them.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CorpusOrigin {
    Local,
    Imported,
}

#[derive(Clone)]
pub(crate) struct AttemptCorpus {
    pub(crate) slug: String,
    pub(crate) corpus_dir: PathBuf,
    pub(crate) origin: CorpusOrigin,
    pub(crate) completed_at: String,
    pub(crate) attempt: u64,
    pub(crate) attempt_id: String,
    pub(crate) retrieval_status: String,
    pub(crate) state: Value,
    pub(crate) report: Value,
}

pub(crate) struct SiteInfo {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) category: String,
    pub(crate) source_url: String,
    pub(crate) inventory_url_count: i64,
    pub(crate) seen: usize,
    pub(crate) cumulative_ok: usize,
    pub(crate) noise: usize,
    pub(crate) retrieval_status: Option<String>,
    /// How many in-scope pages this site declares that no attempt of this
    /// record can hold, and whether that number is exact. `0` for every site
    /// that fits, which is 47 of the 51 in this family.
    pub(crate) pages_outside_corpus: u64,
    pub(crate) pages_outside_corpus_exact: bool,
    pub(crate) attempt: Option<u64>,
    pub(crate) attempt_id: Option<String>,
    pub(crate) corpus_dir: Option<PathBuf>,
}

pub(crate) fn open_regular_read(path: &Path, label: &str) -> Result<File> {
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

pub(crate) fn existing_regular_directory(path: &Path, label: &str) -> Result<bool> {
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

pub(crate) fn read_json(path: &Path) -> Result<Value> {
    let mut file = open_regular_read(path, "documentation corpus metadata")?;
    if file.metadata()?.len() > MAX_METADATA_BYTES {
        bail!("documentation corpus metadata exceeds its byte limit");
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn read_last_worker_report(path: &Path) -> Result<Value> {
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

pub(crate) fn matching_string(left: &Value, right: &Value, field: &str) -> Result<()> {
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

pub(crate) fn hash_file(path: &Path) -> Result<(String, u64)> {
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

pub(crate) fn exact_lower_hex(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not a 64-character lowercase SHA-256 digest");
    }
    Ok(())
}

pub(crate) fn safe_component(value: &str, label: &str) -> Result<()> {
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

pub(crate) fn validate_completion_timestamp(value: &str) -> Result<()> {
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
