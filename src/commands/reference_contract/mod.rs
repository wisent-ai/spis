//! `spis` reference-evidence contract — Rust port of `reference_contract.py`.
//!
//! One definition of the reference-evidence vocabulary, shared by every tool
//! here. Change a rule here and both consumers change with it.

use anyhow::Result;

mod catalog_schema;
mod canonical_timing_class;

pub use catalog_schema::*;
pub use canonical_timing_class::*;
