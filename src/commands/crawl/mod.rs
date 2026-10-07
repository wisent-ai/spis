//! Durable coordinator for every Spis crawler.
//!
//! The six surface-specific commands remain the execution engines. This command
//! is the single operator and desktop contract for planning, submission, status,
//! resumption, artifact retrieval and idempotent record import.

use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod apply_web_attempt_group;
mod cancel_group;
mod continue_record_group;
mod ios_booted_identity_group;
mod op_schema_group;
mod publish_worker_report_group;
mod registry_placements_group;
mod runtime_bindings_for_worker_group;
mod usage_group;

pub use apply_web_attempt_group::*;
pub use cancel_group::*;
pub use continue_record_group::*;
pub use ios_booted_identity_group::*;
pub use op_schema_group::*;
pub use publish_worker_report_group::*;
pub use registry_placements_group::*;
pub use runtime_bindings_for_worker_group::*;
pub use usage_group::*;
