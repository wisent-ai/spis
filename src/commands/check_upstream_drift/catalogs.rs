//! Every catalog file the drift check reads, read strictly before it runs.
//!
//! The collectors the checks use skip a `sources.json` or `reference.json`
//! they cannot read or parse, and a `references` directory or entry they
//! cannot list, so such a catalog contributed no URLs and no records and the
//! check (and `--strict`) passed as if it had nothing upstream. Reading the
//! same files here first turns each of those into a refusal naming the file
//! and the cause. A catalog without a `sources.json` or a `references`
//! directory is still a catalog with nothing of that kind to check.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// The entries of `directory`, refusing one it cannot list or an entry in it
/// it cannot read.
fn listed(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let entries = std::fs::read_dir(directory)
        .with_context(|| format!("cannot list {}", directory.display()))?;
    for entry in entries {
        let entry = entry.with_context(|| format!("cannot list {}", directory.display()))?;
        paths.push(entry.path());
    }
    Ok(paths)
}

/// Whether `path` is there, refusing a path whose presence cannot be read.
fn present(path: &Path) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("cannot read {}", path.display())),
    }
}

/// `path` read and parsed as JSON, or the reason it could not be.
fn parsed(path: &Path) -> Result<()> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    serde_json::from_str::<serde_json::Value>(&text)
        .with_context(|| format!("{} is not JSON", path.display()))?;
    Ok(())
}

/// Refuse the first catalog file under the working directory that the drift
/// check would otherwise read as absent or empty.
pub(super) fn verify_readable() -> Result<()> {
    for catalog in listed(Path::new("."))? {
        let named_catalog = catalog
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with("-examples"));
        if !named_catalog || !catalog.is_dir() {
            continue;
        }
        let sources = catalog.join("sources.json");
        if present(&sources)? {
            parsed(&sources)?;
        }
        let references = catalog.join("references");
        if !present(&references)? {
            continue;
        }
        for reference in listed(&references)? {
            if !reference.is_dir() {
                continue;
            }
            let record = reference.join("reference.json");
            if present(&record)? {
                parsed(&record)?;
            }
        }
    }
    Ok(())
}
