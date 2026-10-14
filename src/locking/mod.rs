//! One exclusive lock on a lock file, taken without waiting and held for as
//! long as its file stays open: closing the file releases it, so a step that
//! fails or panics never leaves the lock behind. Another holder is said apart
//! from a lock that could not be taken at all, which is an error naming the
//! file and its cause.

use anyhow::{Context, Result};
use std::fs::{File, OpenOptions, TryLockError};
use std::path::Path;

/// The lock file at `path`, created when absent and held exclusively; `None`
/// while another process holds it.
pub fn exclusive(path: &Path) -> Result<Option<File>> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .with_context(|| format!("opening the lock file {}", path.display()))?;
    Ok(try_exclusive(&file, path)?.then_some(file))
}

/// Take `file`'s exclusive lock without waiting: `true` once held, `false`
/// while another process holds it. `path` names the file in an error.
pub fn try_exclusive(file: &File, path: &Path) -> Result<bool> {
    match file.try_lock() {
        Ok(()) => Ok(true),
        Err(TryLockError::WouldBlock) => Ok(false),
        Err(TryLockError::Error(error)) => {
            Err(error).with_context(|| format!("locking {}", path.display()))
        }
    }
}
