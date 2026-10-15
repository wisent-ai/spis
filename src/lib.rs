pub mod commands;
pub mod onboarding;
pub mod weles_provenance;

use anyhow::{bail, Context, Result};
use std::time::Duration;

mod locking;
mod staging;
mod user_agent_group;
mod web_scheme;

pub use locking::{exclusive, try_exclusive};
pub use staging::{discard, discarding};
pub use user_agent_group::*;
pub use web_scheme::WebScheme;

/// What a panicked thread said: the message `panic!` was given, which is a
/// `&str` or a `String`. A payload of any other type carries no text, and
/// that is what is said.
pub fn panic_reason(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "its panic payload is not text".to_string()
}

/// Today's date in UTC (`YYYY-MM-DD`): the date part of [`now_iso_utc`], what
/// a record says it was measured or scaffolded on.
pub fn today_utc() -> String {
    match now_iso_utc().split_once('T') {
        Some((date, _)) => date.to_string(),
        None => unreachable!("now_iso_utc writes a date, 'T' and a time"),
    }
}
