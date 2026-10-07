//! Real terminal-application crawler.
//!
//! The worker retains only the initial exact terminal state and applies a
//! default-deny policy to all keyboard, mouse, pointer, paste, and text input.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
// The split submodules reach their sibling commands through `super::`, which
// resolves here by way of their `use super::*`.
use crate::commands::crawl;

mod launch;
mod repository;
mod verify_exact_executable;
mod worker_report;

pub use launch::*;
pub use repository::*;
pub use verify_exact_executable::*;
pub use worker_report::*;
