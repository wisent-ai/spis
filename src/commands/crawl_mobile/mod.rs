//! Real iOS/Android application crawler through an Appium device endpoint.
//!
//! The crawler validates the exact returned Appium session binding and a fresh
//! read-only runtime-readiness observation, then retains one genuinely
//! sequential trajectory. Every delivered action is bracketed by alert,
//! accessibility-source, and active-owner observations.

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

mod record_failure_group;
mod write_state_group;

pub use record_failure_group::*;
pub use write_state_group::*;
