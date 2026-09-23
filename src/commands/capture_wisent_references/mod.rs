//! `spis capture-wisent-references` — capture the reference catalog for the Wisent
//! products installed on this workstation.
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
//! Port of the former `capture-wisent-references.py` (deleted in 1672030).
//! One deliberate environment difference: the transient scratch tree lives under
//! `~/.spis/work/wisent-capture/` instead of `~/.stado/work/wisent-capture/`.

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

macro_rules! product {
    ($slug:expr, $name:expr, $binary:expr, $repo:expr, $url:expr, $cat:expr, $one:expr,
     $note:expr, $ver:expr, $help:expr, $sub:expr, $subnote:expr) => {
        Product {
            slug: $slug,
            name: $name,
            binary: $binary,
            repository: $repo,
            product_url: $url,
            category: $cat,
            one_line: $one,
            selection_note: $note,
            version_cmd: $ver,
            help_cmd: $help,
            sub_cmd: $sub,
            sub_note: $subnote,
        }
    };
}

mod record_schema_group;
mod build_record_group;

pub use record_schema_group::*;
pub use build_record_group::*;
