//! `spis crawl-docs` — parallel full-text crawler for the documentation set.
//!
//! Every immutable runtime manifest owns one durable corpus beneath
//! `~/.stado/work`. Fetches may run concurrently, but a single ordered writer
//! commits complete gzip members before atomically checkpointing their full
//! URL hashes. A resumed worker therefore truncates only an uncommitted tail
//! and never borrows pages from another crawl run.

use crate as lib;
use anyhow::{bail, Context, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::os::unix::fs::OpenOptionsExt;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use url::Url;

#[cfg(test)]
mod tests;

mod override_group;
mod resolve_urls_group;
mod inventory_sha256_group;
mod load_or_create_state_group;

pub use override_group::*;
pub use resolve_urls_group::*;
pub use inventory_sha256_group::*;
pub use load_or_create_state_group::*;
