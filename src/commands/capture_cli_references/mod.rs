//! `spis capture-cli-references --plan <file.json>` — capture a reference
//! catalog of command-line products installed on this workstation. Which
//! products, under which catalog, and which binaries are deliberately left out
//! is the capture plan (`spis.cli-capture-plan.v1`, e.g.
//! `capture-plans/cli-products.json`); nothing about a product is compiled in.
//!
//! For each product the command opens a PTY, drives one `/bin/bash --norc
//! --noprofile -i` session, and issues exactly seven read-only commands: the
//! version form, the top-level help, one subcommand help surface, one deliberately
//! invalid flag, Ctrl-C on an unsubmitted line, the help that recovers from the
//! refusal, and the same help with NO_COLOR=1. The session becomes
//! `media/session.cast` (asciinema v2); five PNGs are rendered from that cast's
//! text with Pillow at named points in the sequence. Afterwards the JSON catalog
//! files (`sources.json` and `references.json`) are rebuilt from every record on disk.
//!
//! The transient scratch tree lives under `~/.spis/work/cli-capture/`.

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod record_schema_group;
mod build_record_group;

pub use record_schema_group::*;
pub use build_record_group::*;
