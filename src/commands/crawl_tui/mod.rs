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
use std::time::{Duration, Instant};

mod repository;
mod verify_exact_executable;
mod launch;
mod worker_report;

pub use repository::*;
pub use verify_exact_executable::*;
pub use launch::*;
pub use worker_report::*;
