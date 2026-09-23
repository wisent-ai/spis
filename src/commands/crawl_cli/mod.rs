//! Real command-line application crawler.
//!
//! The worker executes only exact top-level version, help, refusal, and recovery
//! observations. Raw terminal bytes, screen states, argv, and exit statuses are
//! kept.

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod repository_group;
mod worker_report_group;

pub use repository_group::*;
pub use worker_report_group::*;
