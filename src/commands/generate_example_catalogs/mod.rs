//! `spis generate-example-catalogs` — validate measured example catalogs and
//! write the machine-readable cross-catalog index.
//!
//! This generator is the gate. It refuses to index a catalog whose data
//! contradicts the files beside it, and records the measured numbers rather
//! than an intention: how many records are complete, how many are partial, and
//! how the motion evidence was actually obtained (a product we drove, a browser
//! we drove, or media its owner published).
//!
//! Rust port of the validation and JSON indexing behavior from the former
//! `generate-example-catalogs.py` pipeline.

use crate as lib;
use crate::weles_provenance::VerifiedProvenanceSet;
use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod catalog_schema_group;
mod load_catalog_group;

pub use catalog_schema_group::*;
pub use load_catalog_group::*;
