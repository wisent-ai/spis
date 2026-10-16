//! `spis discover`.
//!
//! Discover the important pages behind a start URL and turn them into records.
//!
//! 1. Fetch the start page and extract every same-origin link with its text.
//! 2. Ask Brama which pages matter for a reference corpus (pricing, docs,
//!    sign-in, about…). Without a configured, reachable Brama discovery stops
//!    naming the missing setting; no word list decides a page's family.
//! 3. Take each selected page's own image of the product's interface, chosen
//!    as `collect-example-images` chooses it, and scaffold a numbered record
//!    through the same contract as `reference-record add`. A page without such
//!    an image is refused with that cause.

use crate as lib;
use crate::commands::reference_record::{self, AddArgs};
use anyhow::{bail, Context, Result};
use serde_json::json;
use std::io::Read;
// The split submodules reach their sibling commands through `super::`, which
// resolves here by way of their `use super::*`.
use crate::commands::reference_contract;

mod links;
mod rank;
mod ua;

pub(crate) use links::*;
pub use rank::*;
pub use ua::*;
