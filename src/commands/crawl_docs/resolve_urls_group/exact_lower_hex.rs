use super::*;

pub(crate) fn exact_lower_hex(value: &str, length: usize, label: &str) -> Result<()> {
    if value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not an exact {length}-character lowercase hexadecimal digest");
    }
    Ok(())
}

pub(crate) fn manifest_attempt(manifest: &super::crawl::RuntimeManifest) -> Result<(u32, String)> {
    let encoded = serde_json::to_value(manifest)?;
    let attempt = encoded
        .get("attempt")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .context("runtime manifest has no positive immutable attempt")?;
    let attempt_id = encoded
        .get("attempt_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("runtime manifest has no immutable attempt_id")?
        .to_string();
    for (value, label) in [
        (manifest.run_id.as_str(), "runtime manifest run_id"),
        (manifest.catalog.as_str(), "runtime manifest catalog"),
        (manifest.record.as_str(), "runtime manifest record"),
        (attempt_id.as_str(), "runtime manifest attempt_id"),
    ] {
        safe_path_component(value, label)?;
    }
    exact_lower_hex(&manifest.record_key, 64, "runtime manifest record_key")?;
    let base = crate::crawl_attempt_base_uri(
        &manifest.run_id,
        &manifest.catalog,
        &manifest.record,
        &manifest.record_key,
        attempt,
        &attempt_id,
    );
    if manifest.artifact_uri != format!("{base}/artifacts.tar.gz")
        || manifest.output_uri != format!("{base}/worker-output.log")
    {
        bail!(
            "runtime manifest artifact/output URIs do not exactly match canonical run/catalog/record/record_key/attempt/attempt_id coordinates"
        );
    }
    Ok((attempt, attempt_id))
}

pub(crate) fn manifest_structure_sha256(manifest: &super::crawl::RuntimeManifest) -> Result<String> {
    let encoded = serde_json::to_value(manifest)?;
    let digest = encoded
        .get("docs_structure_sha256")
        .and_then(Value::as_str)
        .context("documentation runtime manifest has no docs_structure_sha256")?;
    exact_lower_hex(
        digest,
        64,
        "runtime manifest docs_structure_sha256",
    )?;
    Ok(digest.to_string())
}

pub(crate) fn work_layout(manifest: &super::crawl::RuntimeManifest) -> Result<WorkLayout> {
    safe_path_component(&manifest.run_id, "runtime manifest run_id")?;
    safe_path_component(&manifest.catalog, "runtime manifest catalog")?;
    safe_path_component(&manifest.record, "runtime manifest record")?;
    exact_lower_hex(&manifest.source_revision, 40, "runtime manifest source_revision")?;
    exact_lower_hex(
        &manifest.source_input_sha256,
        64,
        "runtime manifest source_input_sha256",
    )?;
    exact_lower_hex(&manifest.record_key, 64, "runtime manifest record_key")?;
    let (_, attempt_id) = manifest_attempt(manifest)?;
    safe_path_component(&attempt_id, "runtime manifest attempt_id")?;
    let home = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .context("HOME is not set; cannot locate the durable crawl work root")?;
    let base = PathBuf::from(home).join(".spis/crawls");
    let root = super::crawl::native_attempt_root(&base, manifest)?;
    let corpus = root.clone();
    Ok(WorkLayout {
        state: corpus.join("state.json"),
        pages: corpus.join("pages.jsonl.gz"),
        journal: corpus.join("outcomes.jsonl"),
        report: corpus.join("docs-retrieval-run.json"),
        root,
        corpus,
    })
}

pub(crate) fn regular_file_exists(path: &Path, label: &str) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                bail!("{label} is not a regular non-symlink file: {}", path.display());
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}

pub(crate) fn open_regular_file(
    path: &Path,
    read: bool,
    write: bool,
    append: bool,
    create: bool,
    label: &str,
) -> Result<File> {
    let exists = match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                bail!("{label} is not a regular non-symlink file: {}", path.display());
            }
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error).with_context(|| format!("inspect {label}")),
    };
    let mut options = OpenOptions::new();
    options
        .read(read)
        .write(write)
        .append(append)
        .custom_flags(libc::O_NOFOLLOW);
    if create && !exists {
        options.create_new(true);
    }
    let file = options
        .open(path)
        .with_context(|| format!("open {label} {}", path.display()))?;
    if !file.metadata()?.is_file() {
        bail!("{label} opened as a non-regular file: {}", path.display());
    }
    Ok(file)
}

pub(crate) fn staging_directory(root: &Path, label: &str) -> Result<PathBuf> {
    for _ in 0..128 {
        let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::SeqCst);
        let directory = root.join(format!(".{label}-{}-{sequence}", std::process::id()));
        match std::fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("create staging directory {}", directory.display()))
            }
        }
    }
    bail!("could not allocate a unique {label} staging directory")
}

/// The exact artifact set an attempt root may contain. `corpus_summary` enforces
/// it and `audit_attempt_tree` archives it, so nothing else may ever be staged
/// or left inside `layout.root`.
pub(crate) const CORPUS_ARTIFACTS: [&str; 4] = [
    "docs-retrieval-run.json",
    "outcomes.jsonl",
    "pages.jsonl.gz",
    "state.json",
];

/// Name for a staged temp file, scoped by the owning attempt directory so two
/// workers on sibling attempt_ids under the same `attempts/<n>` parent never
/// share a temp namespace. This mirrors the `.{attempt}.crawl.lock` and
/// `.{attempt}.archive.lock` names that already live in that directory.
pub(crate) fn temporary_name(owner: &str, name: &str, sequence: u64) -> String {
    format!(".{owner}.{name}.{}-{sequence}.tmp", std::process::id())
}

/// Does `file_name` denote a staged temp file for one of this attempt's own
/// artifacts? Recognises the current attempt-scoped shape
/// `.<owner>.<artifact>.<pid>-<sequence>.tmp` and the legacy in-root shape
/// `.<artifact>.<pid>-<sequence>.tmp` that earlier workers left behind. Anything
/// that does not match one of those exact shapes is never touched, so a sibling
/// attempt's temp and every lock, archive and read-back file are left alone.
pub(crate) fn is_own_temporary(file_name: &str, owner: &str) -> bool {
    let Some(body) = file_name
        .strip_prefix('.')
        .and_then(|rest| rest.strip_suffix(".tmp"))
    else {
        return false;
    };
    let body = body
        .strip_prefix(owner)
        .and_then(|rest| rest.strip_prefix('.'))
        .unwrap_or(body);
    let Some(suffix) = CORPUS_ARTIFACTS
        .iter()
        .find_map(|artifact| body.strip_prefix(*artifact))
    else {
        return false;
    };
    let Some((pid, sequence)) = suffix
        .strip_prefix('.')
        .and_then(|rest| rest.split_once('-'))
    else {
        return false;
    };
    !pid.is_empty()
        && !sequence.is_empty()
        && pid.bytes().all(|byte| byte.is_ascii_digit())
        && sequence.bytes().all(|byte| byte.is_ascii_digit())
}

/// Drop temp files this attempt orphaned in an earlier, killed run. Called at
/// resume while the exclusive work lock is held, so no live writer can own a
/// matching name. Without this, a SIGKILL inside the old in-root `write_all` +
/// `sync_all` window left a `.state.json.<pid>-<n>.tmp` that made
/// `corpus_summary` reject the attempt for good.
pub(crate) fn prune_stale_temporaries(layout: &WorkLayout) -> Result<()> {
    let owner = layout
        .root
        .file_name()
        .and_then(|value| value.to_str())
        .context("durable work directory has no UTF-8 name")?;
    let staging_parent = layout
        .root
        .parent()
        .context("durable work directory has no staging parent")?;
    for directory in [staging_parent, layout.root.as_path()] {
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("list staging directory {}", directory.display()))
            }
        };
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            if !is_own_temporary(file_name, owner) || !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            std::fs::remove_file(&path)
                .with_context(|| format!("remove stale staged file {}", path.display()))?;
        }
    }
    Ok(())
}

/// Path of the retained worker failure diagnostic. It sits beside the attempt
/// root, alongside `<attempt_id>.tar.gz`, so it is neither audited by
/// `corpus_summary` nor archived by `publish_attempt_archive`.
pub(crate) fn failure_diagnostic_path(layout: &WorkLayout) -> Result<PathBuf> {
    let owner = layout
        .root
        .file_name()
        .and_then(|value| value.to_str())
        .context("durable work directory has no UTF-8 name")?;
    let parent = layout
        .root
        .parent()
        .context("durable work directory has no parent")?;
    Ok(parent.join(format!("{owner}.failure.json")))
}
