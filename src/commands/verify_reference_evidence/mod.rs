//! `spis verify-reference-evidence` — measure the stored reference evidence and
//! rewrite every record to what the files prove.
//!
//! Reads every `<catalog>/references/*/reference.json`, probes the real media
//! with ffprobe (or the asciinema cast header), verifies bytes and SHA-256,
//! derives the media kind and provenance class from observable facts, locates
//! each state frame inside its motion source by pixel comparison, and recomputes
//! `evidence_status` from the measured floor.
//!
//! Nothing here invents evidence. A field that cannot be measured is null and
//! named in `evidence_gaps`; the record is not called complete.
//!
//! Ported 1:1 from verify-reference-evidence.py + reference_contract.py.

use crate as lib;
use crate::weles_provenance::VerifiedProvenanceSet;
use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

mod record_schema_group;
mod catalogs_group;

pub use record_schema_group::*;
pub use catalogs_group::*;
