//! `spis analyze-example-structures` — measure panel geometry and record
//! interface anatomy for example images.
//!
//! For every example in a catalog's sources.json this decodes the overview
//! image, detects dominant separators on a grayscale downscale, classifies a
//! layout model with semantic hints from the example metadata, and stores an
//! `interface_structure` record back into sources.json. Failures land in
//! structure-analysis-failures.json. Rust port of the former
//! `analyze-example-structures.py`; the separator statistics mirror the numpy
//! pipeline (zero-padded moving average, linear-interpolated percentiles,
//! population standard deviation, banker's rounding).
//!
//! Requires the `image` crate (reported to the integrator for Cargo.toml).

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::Path;

mod catalogs;
mod semantic_hints;
mod classify_layout;
mod analyze;

pub use catalogs::*;
pub use semantic_hints::*;
pub use classify_layout::*;
pub use analyze::*;
