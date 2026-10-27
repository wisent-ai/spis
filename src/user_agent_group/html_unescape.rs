use super::*;

/// A sitemap's children or pages, and a page's text and title, read by the
/// one HTML reader every command uses.
pub use crate::html::{extract_text, parse_sitemap};

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

/// The SHA-256 of one file's bytes, streamed. Every executable, helper and
/// visual digest Spis records or checks is read through this; a failure
/// names the file.
pub fn sha256_file(path: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut file =
        std::fs::File::open(path).with_context(|| format!("open {} to hash it", path.display()))?;
    let mut digest = Sha256::new();
    std::io::copy(&mut file, &mut digest).with_context(|| format!("hash {}", path.display()))?;
    Ok(hex::encode(digest.finalize()))
}

/// Now, in UTC, to the second: `YYYY-MM-DDTHH:MM:SSZ`.
pub fn now_iso_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// The JSON document at `path`; a file that cannot be read or parsed is
/// refused naming the path and the cause.
pub fn read_json<T: serde::de::DeserializeOwned>(path: impl AsRef<std::path::Path>) -> Result<T> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

/// Write `value` as pretty JSON to `path`; a failure names the path.
pub fn write_pretty_json(
    path: impl AsRef<std::path::Path>,
    value: &serde_json::Value,
) -> Result<()> {
    let path = path.as_ref();
    std::fs::write(path, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}
