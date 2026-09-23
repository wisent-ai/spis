use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod product_id;
mod save_state;

pub use product_id::*;
pub use save_state::*;
