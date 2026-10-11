//! `spis check-upstream-drift` — report where stored reference evidence no
//! longer matches its upstream. Read-only checks:
//!
//! 1. README drift — current blob SHA of each snapshotted README via the
//!    GitHub API (`gh` CLI, preserved external call) against the recorded SHA.
//! 2. Source reachability — HTTP HEAD falling back to a ranged GET for every
//!    catalog `source_url`, `source_image_url`, and motion `source_url`.
//! 3. Local integrity — every recorded local media path resolves and matches
//!    its recorded SHA-256.
//!
//! Optional operational reports stay in the active corpus's ignored `.build/`.

use crate as lib;
use anyhow::{Context, Result};
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

mod catalogs;
mod check_sources;
mod report;

pub use check_sources::*;
pub use report::*;

/// `spis check-upstream-drift`. Every catalog file the checks read is read
/// strictly first, so a catalog whose files cannot be read or parsed is
/// refused by name instead of being checked as a catalog with nothing
/// upstream; this definition takes the name over the re-exported one.
pub fn run(rest: &[String]) -> Result<()> {
    catalogs::verify_readable()?;
    check_sources::run(rest)
}
