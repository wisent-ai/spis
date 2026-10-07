//! `spis audit-reference-accessibility` measures reference accessibility with
//! axe-core through Weles on a Stado-selected host.
//!
//! Plans a `wisent.weles-capture-plan.v1` batch of generic_accessibility_audit
//! actions, runs it through `stado workload run weles-capture`, reads terminal
//! status, retrieves and validates axe artifacts, and installs measured evidence
//! under each record's media/accessibility/. The current executable then runs
//! `verify-reference-evidence` for each completed catalog.
//!
//! Operational plans, staging files and the audit index stay in the corpus's
//! ignored `.build/accessibility-audit/` directory. JSON inputs reject duplicate
//! keys rather than silently accepting conflicting observations.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

mod enqueue_group;
mod index_group;

pub use enqueue_group::*;
pub use index_group::*;
