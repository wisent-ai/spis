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
//! Ported 1:1 from the former check-upstream-drift.py.

use crate as lib;
use anyhow::{Context, Result};
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

mod report;
mod check_sources;

pub use report::*;
pub use check_sources::*;
