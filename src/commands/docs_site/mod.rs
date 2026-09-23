//! `spis docs-site` — generate the product documentation from the product.
//!
//! ## Why this exists
//!
//! `docs-site/docs.json` is the generated product-document payload consumed by
//! the canonical `wisent-ai/spis-landing` site's existing `DocPage` loader.
//! Older releases wrote `docs-site/docs.ts`, but no deployed site imported
//! that module. The JSON output can also be written directly into the landing
//! checkout with `--site-content PATH`, so publishing does not require a
//! hand-maintained copy or a second documentation renderer.
//! The earlier generated inputs also drifted from the shipped product:
//!
//! * `docs-brief.json` declares the CLI surface by probing `bin/spis`, the
//!   Python-era shim, and lists nine commands — `analyze-readmes`, `capture`,
//!   `catalogs`, `drift`, `sync-readmes`, `verify` and three more — NOT ONE of
//!   which exists in the shipped binary. The real surface is `crawl`,
//!   `crawl-docs`, `crawl-cli`, `crawl-desktop`, `crawl-mobile`, `crawl-web`,
//!   `docs-corpus`, and the rest of [`super::SUBCOMMANDS`].
//! * it declares `13` interface families; there are fifteen in
//!   [`super::crawl::CATALOGS`].
//! * it declares no graphical surface at all, and the published prose calls
//!   Spis "the command-line machinery that maintains it" — while
//!   `wisent-ai/spis-desktop` ships a macOS application that starts crawls,
//!   tracks runs and searches the corpus. A product whose documentation does
//!   not mention its own graphical surface cannot be checked for the parity
//!   this fleet requires.
//!
//! Patching those sentences would have left the process that produced them
//! exactly as broken. So this module IS the process: every page is derived
//! from the running product — the subcommand table, the catalog table, each
//! engine's declared preconditions, the corpus bound — and `--check` fails
//! when the checked-in output no longer matches what the product would
//! generate. A documentation claim that drifts from the code is then a failing
//! command rather than a sentence nobody re-read.
//!
//! Content that is genuinely editorial (what Spis is for, the words it uses)
//! stays authored, in [`AUTHORED_SECTIONS`], and is carried through verbatim.
//! The distinction is deliberate: prose about intent is not derivable, and
//! pretending to derive it would be the same lie one level up.

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

mod docs_content;
mod plan;
mod run;

pub use docs_content::*;
pub use plan::*;
pub use run::*;
