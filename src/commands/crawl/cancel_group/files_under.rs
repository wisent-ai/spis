use super::*;

/// Every file under `root`, sorted. A directory or entry that cannot be read
/// fails with its path: skipped, the retained media and accessibility files
/// under it were left out of the imported record without a word.
pub(crate) fn files_under(root: &Path) -> Result<Vec<PathBuf>> {
    fn walk(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        let entries =
            std::fs::read_dir(root).with_context(|| format!("listing {}", root.display()))?;
        for entry in entries {
            let entry = entry.with_context(|| format!("reading an entry of {}", root.display()))?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    walk(root, &mut files)?;
    files.sort();
    Ok(files)
}

pub(crate) fn media_kind(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("state"),
        "jpg" => Some("state"),
        "jpeg" => Some("state"),
        "webp" => Some("state"),
        "gif" => Some("motion"),
        "mp4" => Some("motion"),
        "webm" => Some("motion"),
        "cast" => Some("motion"),
        _ => None,
    }
}

/// The motion kind a retained file's extension declares, or none for an
/// extension that declares no motion kind (never "unknown").
pub(crate) fn declared_motion_kind(path: &Path) -> Option<&'static str> {
    Some(
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
            _ => return None,
        },
    )
}

pub(crate) fn capture_method(engine: &str) -> &'static str {
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
pub(crate) fn attempt_media(
    engine: &str,
    attempt_dir: &Path,
    record_dir: &Path,
    source_url: &str,
) -> Result<(Vec<Value>, Vec<Value>)> {
    let mut motion = Vec::new();
    let mut states = Vec::new();
    let files = files_under(attempt_dir)?;
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

/// Whether a retained file is an accessibility or DOM source: by extension,
/// or by the name an engine gives its tree.
fn accessibility_source(path: &Path) -> bool {
    let by_extension = match path.extension().and_then(|value| value.to_str()) {
        Some("xml") => true,
        Some("html") => true,
        Some("txt") => true,
        _ => false,
    };
    let by_name = match path.file_name().and_then(|value| value.to_str()) {
        Some("snapshot.json") => true,
        Some("source.json") => true,
        Some("axe.json") => true,
        _ => false,
    };
    by_extension || by_name
}

pub(crate) fn accessibility_gap(engine: &str, attempt_dir: &Path) -> Result<Value> {
    let trees: Vec<PathBuf> = files_under(attempt_dir)?
        .into_iter()
        .filter(|path| accessibility_source(path))
        .collect();
    let sizes = trees
        .iter()
        .map(|path| {
            std::fs::metadata(path)
                .map(|metadata| metadata.len())
                .with_context(|| format!("reading the size of {}", path.display()))
        })
        .collect::<Result<Vec<u64>>>()?;
    let bytes: u64 = sizes.iter().sum();
    Ok(json!({
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
    }))
}

/// One durable `crawl_runs` entry, keyed by the immutable attempt id.
pub(crate) fn crawl_run_entry(
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
pub(crate) fn write_immutable_file(path: &Path, bytes: &[u8]) -> Result<()> {
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
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&staged, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    crate::discarding(result, &staged)
}

/// Merge one immutable retained subtree into the record without disturbing the
/// objects earlier attempts still reference.
pub(crate) fn merge_immutable_tree(source: &Path, destination: &Path) -> Result<()> {
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        let metadata = std::fs::symlink_metadata(&from)?;
        if metadata.file_type().is_symlink() {
            bail!(
                "retained crawl content {} is a symbolic link",
                from.display()
            );
        }
        if metadata.is_dir() {
            merge_immutable_tree(&from, &to)?;
        } else if metadata.is_file() {
            write_immutable_file(&to, &std::fs::read(&from)?)?;
        } else {
            bail!(
                "retained crawl content {} is not a regular file",
                from.display()
            );
        }
    }
    Ok(())
}
