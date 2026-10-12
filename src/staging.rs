//! Taking back the temporary file a failed write staged. A temporary that is
//! already gone is no failure; one that stays is named, with its cause, in
//! the error the write answers, so a refused write never leaves a file
//! beside its destination that nobody was told about.

use anyhow::Result;
use std::path::Path;

/// `result` unchanged when it succeeded; when it failed, the same error after
/// removing `temporary`, carrying the sentence of a temporary that stays.
pub fn discarding<T>(result: Result<T>, temporary: &Path) -> Result<T> {
    let Err(error) = result else {
        return result;
    };
    match std::fs::remove_file(temporary) {
        Err(left) if left.kind() != std::io::ErrorKind::NotFound => {
            Err(anyhow::anyhow!("{error:#}; {} is left behind: {left}", temporary.display()))
        }
        _ => Err(error),
    }
}

/// Remove a temporary whatever the step that used it answered: one already
/// gone is no failure, one that stays is an error naming it and its cause.
pub fn discard(temporary: &Path) -> Result<()> {
    match std::fs::remove_file(temporary) {
        Err(left) if left.kind() != std::io::ErrorKind::NotFound => {
            Err(anyhow::anyhow!("{} is left behind: {left}", temporary.display()))
        }
        _ => Ok(()),
    }
}
