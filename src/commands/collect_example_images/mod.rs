//! `spis collect-example-images` — collect one attributable official interface
//! image per catalog entry.
//!
//! Static HTTP reads only. Every readable raster image on the official page is
//! listed with its address, the page's own description of it and its
//! dimensions, and Brama chooses the one that shows the product's interface;
//! a page without one is refused with that cause. Format and dimensions are
//! read from the image header (PNG IHDR, JPEG SOF, WebP VP8/VP8L/VP8X, GIF
//! logical screen), and the original bytes are stored verbatim beside the
//! image's source address; nothing is decoded, resampled or re-encoded.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};
use std::io::Read;
use std::path::{Path, PathBuf};

mod user_agent_group;

pub use user_agent_group::*;
