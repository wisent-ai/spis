//! `spis collect-example-images` — collect one attributable official interface
//! image per catalog entry (port of collect-example-images.py).
//!
//! Static HTTP reads only; prefers large images whose URL, alt text, or
//! surrounding metadata identifies a screenshot or product interface, then
//! stores a bounded derivative while retaining the original image URL.
//!
//! PIL replacement note: the Python original decoded pixels with Pillow to
//! resample a 1400×1000 WebP derivative. This port performs **header-only**
//! parsing (PNG IHDR, JPEG SOF scan, WebP VP8/VP8L/VP8X, GIF logical screen)
//! for format and dimensions, and stores the ORIGINAL image bytes verbatim
//! instead of re-encoding. No pixel decode happens here; see the module
//! report for the resulting metadata-shape gaps.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod html_tests;

#[cfg(test)]
mod real_image_tests;

mod user_agent_group;

pub use user_agent_group::*;
