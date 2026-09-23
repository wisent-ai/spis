//! One-record browser-evidence worker driven exclusively through the official Weles client.
//!
//! The coordinator never launches a browser and never speaks HTTP to Weles. It pins one
//! immutable runtime manifest into one exact-revision Stado job for exactly one catalog
//! record. The worker submits exactly one `generic_browser_task` through the checked-in
//! Node bridge (`weles-bridge/spis-weles-bridge.mjs`), which owns every Weles request,
//! loads the pinned official `@wisent-ai/weles-client`, and verifies every receipt.
//!
//! Everything this worker retains is re-proved locally before it is published: the live
//! service release, the signed Spis binding, the canonical official request, the
//! receipt-bound evidence manifest bytes and every retained evidence file. The documents
//! written here are the exact inputs of `crate::weles_provenance`, so each non-obvious
//! check below names the verifier invariant it satisfies.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::weles_provenance as weles;

mod repository_group;
mod ensure_manifest_identity_group;
mod run_group;

pub use repository_group::*;
pub use ensure_manifest_identity_group::*;
pub use run_group::*;
