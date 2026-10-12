//! Taking back the temporary file or directory a step staged. A temporary
//! that is already gone is no failure; one that stays is named, with its
//! cause, in the error the step answers, so a refused or finished step never
//! leaves a staged path beside its destination that nobody was told about.

use anyhow::Result;
use std::path::Path;

/// `result` unchanged when it succeeded; when it failed, the same error after
/// removing `temporary`, carrying the sentence of a temporary that stays.
pub fn discarding<T>(result: Result<T>, temporary: &Path) -> Result<T> {
    let Err(error) = result else {
        return result;
    };
    match left_behind(temporary) {
        Some(left) => Err(anyhow::anyhow!("{error:#}; {left}")),
        None => Err(error),
    }
}

/// Remove a temporary whatever the step that used it answered: one already
/// gone is no failure, one that stays is an error naming it and its cause.
pub fn discard(temporary: &Path) -> Result<()> {
    match left_behind(temporary) {
        Some(left) => Err(anyhow::anyhow!(left)),
        None => Ok(()),
    }
}

/// Remove a temporary file or directory; the sentence of one that stays.
fn left_behind(temporary: &Path) -> Option<String> {
    let removed = if temporary.is_dir() {
        std::fs::remove_dir_all(temporary)
    } else {
        std::fs::remove_file(temporary)
    };
    match removed {
        Err(left) if left.kind() != std::io::ErrorKind::NotFound => {
            Some(format!("{} is left behind: {left}", temporary.display()))
        }
        _ => None,
    }
}
