pub mod commands;
pub mod onboarding;
pub mod weles_provenance;

use anyhow::{bail, Context, Result};
use std::time::Duration;

mod staging;
mod user_agent_group;
mod web_scheme;

pub use staging::{discard, discarding};
pub use user_agent_group::*;
pub use web_scheme::WebScheme;
