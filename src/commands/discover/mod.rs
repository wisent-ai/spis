//! `spis discover` — Rust port of `discover.py`.
//!
//! Discover the important pages behind a start URL and turn them into records.
//!
//! 1. Fetch the start page and extract every same-origin link with its text.
//! 2. Ask Brama which pages matter for a reference corpus (pricing, docs,
//!    sign-in, about…). If Brama is unreachable or unauthenticated, fall back to
//!    deterministic keyword classification — discovery never blocks on a model.
//! 3. Download an overview screenshot per selected page and scaffold a numbered
//!    record through the same contract as `reference-record add`.

use crate as lib;
use crate::commands::reference_record::{self, AddArgs};
use anyhow::{bail, Context, Result};
use serde_json::json;
use std::io::Read;
use std::time::Duration;

mod ua;
mod keywords;

pub use ua::*;
pub use keywords::*;
