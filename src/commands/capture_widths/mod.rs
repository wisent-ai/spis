//! `spis capture-widths` — capture a landing record at all three review
//! widths through Weles (1:1 port of capture-widths.py).
//!
//! Builds one `wisent.weles-capture-plan.v1` batch with a composition-axis
//! capture per width (390 × 844, 768 × 1024, 1440 × 1000) and enqueues it
//! through `stado host weles-capture`. Weles stores the screenshot **and**
//! the rendered DOM (`*_dom_*.html`) for every width.
//!
//! Usage: spis capture-widths <catalog> [--record <NN|slug>] [--host <target>] [--dry-run]

use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

mod plan_schema;
mod run;

pub use plan_schema::*;
pub use run::*;
