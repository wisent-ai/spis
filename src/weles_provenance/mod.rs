//! Fail-closed Weles receipt provenance for retained Spis evidence.
//!
//! The Rust verifier never treats a JSON boolean, a verifier label, a receipt-provided
//! key, or a caller-chosen correlation ID as trust. It re-runs the checked-in Node bridge,
//! which loads the exact pinned official `@wisent-ai/weles-client`, then independently
//! rechecks the returned claims, receipt identity, and retained artifact digest here.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(unix)]
use std::os::unix::process::CommandExt;

mod bridge_command_schema_group;
mod validate_request_and_evidence_manifest_group;

pub use bridge_command_schema_group::*;
pub use validate_request_and_evidence_manifest_group::*;
