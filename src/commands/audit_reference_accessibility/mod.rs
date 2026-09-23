//! `spis audit-reference-accessibility` — measure reference accessibility with
//! axe-core through Weles on a Stado host (port of audit-reference-accessibility.py).
//!
//! Plans a `wisent.weles-capture-plan.v1` batch of generic_accessibility_audit
//! actions, enqueues it via `stado host weles-capture`, polls status, retrieves
//! axe artifacts through `stado storage get`, validates them, installs them
//! under each record's media/accessibility/, updates reference.json, and runs
//! the `verify-reference-evidence` subcommand per completed catalog.
//!
//! Deviations from the Python original (reported, deliberate):
//! * Plan/staging directories live under ~/.spis/work instead of ~/.stado/work.
//! * The verifier is invoked as a spis subcommand (`spis
//!   verify-reference-evidence`) on the current executable rather than
//!   `python3 verify-reference-evidence.py`.
//! * The strict JSON reader is a hand-rolled parser with duplicate-key
//!   detection (serde_json alone accepts duplicates silently).

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

mod index_group;
mod enqueue_group;

pub use index_group::*;
pub use enqueue_group::*;
