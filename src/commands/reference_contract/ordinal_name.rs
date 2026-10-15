//! How a catalog names the item at a position: its one-based ordinal,
//! zero-padded to two digits, a hyphen, then its slug (`NN-<slug>`). Record
//! directories and stored example images are named this way, and every
//! reader of such a name takes it apart here.

use anyhow::{bail, Result};

/// The name of the catalog item at one-based `ordinal` whose slug is `slug`.
pub fn ordinal_name(ordinal: usize, slug: &str) -> String {
    format!("{ordinal:02}-{slug}")
}

/// The slug part of an `NN-<slug>` name; a name with no all-digit ordinal
/// before its first hyphen, or nothing after it, is refused by name.
pub fn ordinal_slug(name: &str) -> Result<&str> {
    match name.split_once('-') {
        Some((ordinal, slug))
            if !ordinal.is_empty()
                && ordinal.bytes().all(|byte| byte.is_ascii_digit())
                && !slug.is_empty() =>
        {
            Ok(slug)
        }
        _ => bail!("{name} is not named NN-<slug>: no ordinal, a hyphen and a slug"),
    }
}
