//! `spis reference-record` — Rust port of `reference-record.py`.
//!
//! Add, get, or remove a single reference record inside a product-type catalog.
//!
//! A record is one numbered product reference: an overview image plus
//! `references/<NN-slug>/reference.json`. Adding scaffolds the record honestly —
//! motion, states, journey, and accessibility start empty and are named in
//! `evidence_gaps`, so the record is `partial` until the pipeline measures it.
//! The generated index is refreshed after every mutation.

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

mod catalog_dir;
mod add;
mod parse_flags;

pub use catalog_dir::*;
pub use add::*;
pub use parse_flags::*;
