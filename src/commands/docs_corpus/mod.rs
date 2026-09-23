//! `spis docs-corpus` — JSON views and immutable Stado artifact import for
//! documentation retrieval attempts. stdout carries exactly one JSON document.

use crate as lib;
use anyhow::{bail, Context as _, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::io::{BufRead, Read, Write};
use std::path::{Component, Path, PathBuf};

mod max_discovery_depth_group;
mod collect_sites_group;

pub use max_discovery_depth_group::*;
pub use collect_sites_group::*;
