//! `spis discover` — Rust port of `discover.py`.
//!
//! Discover the important pages behind a start URL and turn them into records.
//!
//! 1. Fetch the start page and extract every same-origin link with its text.
//! 2. Ask Brama which pages matter for a reference corpus (pricing, docs,
//!    sign-in, about…). Without a configured, reachable Brama discovery stops
//!    naming the missing setting; no word list decides a page's family.
//! 3. Download an overview screenshot per selected page and scaffold a numbered
//!    record through the same contract as `reference-record add`.

use crate as lib;
use crate::commands::reference_record::{self, AddArgs};
use anyhow::{bail, Context, Result};
use serde_json::json;
use std::io::Read;
// The split submodules reach their sibling commands through `super::`, which
// resolves here by way of their `use super::*`.
use crate::commands::reference_contract;

mod ua;
mod rank;

pub use rank::*;
pub use ua::*;
