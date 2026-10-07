//! `spis` reference-evidence contract — Rust port of `reference_contract.py`.
//!
//! One definition of the reference-evidence vocabulary, shared by every tool
//! here. Change a rule here and both consumers change with it.

use anyhow::Result;
// The split submodules reach their sibling commands through `super::`, which
// resolves here by way of their `use super::*`.
use crate::commands::generate_example_catalogs;

mod canonical_timing_class;
mod catalog_schema;

pub use canonical_timing_class::*;
pub use catalog_schema::*;
