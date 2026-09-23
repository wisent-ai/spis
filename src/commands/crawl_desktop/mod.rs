//! Real native desktop application crawler through Cua Driver.
//!
//! Runs one strictly window-scoped, genuinely sequential trajectory on the
//! Stado-selected host. Each action uses a token from a fresh target-window
//! snapshot and retains the exact driver response plus a fresh observed state.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

mod repository_group;
mod launched_executable_proof_group;

pub use repository_group::*;
pub use launched_executable_proof_group::*;
