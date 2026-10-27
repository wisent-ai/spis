//! Generate product documentation from Spis's own contracts.
//!
//! `docs-site/docs.json` is consumed by the canonical landing site's `DocPage`
//! loader. `--site-content PATH` writes that payload directly into a selected
//! site checkout; `--check` refuses output that differs from the product.
//!
//! Generated claims come from the subcommand and catalog tables, each crawler's
//! declared preconditions and the selected corpus. Editorial purpose and parity
//! prose stay authored in [`WHAT_SPIS_IS`] and [`PARITY`].

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
// The split submodules reach their sibling commands and the subcommand table
// through `super::`, which resolves here by way of their `use super::*`.
use super::SUBCOMMANDS;
use crate::commands::{crawl, docs_corpus};

mod docs_content;
mod plan;
mod run;

pub use docs_content::*;
pub use plan::*;
pub use run::*;
