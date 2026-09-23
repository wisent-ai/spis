use super::*;

/// Walk `root` and prove it is a self-contained tree of ordinary directories and
/// regular files. A symlink, device, socket, FIFO, hard-link fan-out or an
/// out-of-bound entry count/byte total is refused before anything is archived,
/// so a compromised or racing worker cannot smuggle host content into a
/// published crawl artifact.
pub(crate) fn audit_attempt_tree(root: &Path) -> Result<(usize, u64)> {
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

pub(crate) fn hash_regular_file(path: &Path, maximum: u64) -> Result<(String, u64)> {
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
