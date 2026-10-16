//! `spis capture-widths` — capture a landing record at all three review
//! widths through Weles.
//!
//! Builds one `wisent.weles-capture-plan.v1` batch with a composition-axis
//! capture per width (390 × 844, 768 × 1024, 1440 × 1000) and enqueues it
//! through `stado workload run weles-capture`. Weles stores the screenshot **and**
//! the rendered DOM (`*_dom_*.html`) for every width.
//!
//! Usage: spis capture-widths <catalog> [--record <NN|slug>] [--host <target>] [--dry-run]

use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

// The Weles capture plan's schema and artifact namespace are declared once,
// beside the enqueue both capture commands use.
use crate::commands::audit_reference_accessibility::{NAMESPACE, PLAN_SCHEMA};
// A catalog is found and its lists read by the catalog module reference-record
// also reads it through.
use crate::commands::reference_record::{catalog_list, catalog_object};

mod plan_schema;
mod run;

pub use plan_schema::*;
pub use run::*;
