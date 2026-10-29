//! The secret a `ROLE#FIELD` reference names: read with
//! `stado credentials get --role`, or, on a machine without Stado, from the
//! owner-only JSON file `SPIS_CREDENTIALS_FILE` names. Only the vault role the
//! secret's item plays travels in the environment, never an item name.

use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::commands::usage;

/// The variable naming the owner-only file of role -> field -> value that
/// answers a reference in Stado's place.
pub(super) const CREDENTIALS_FILE: &str = "SPIS_CREDENTIALS_FILE";

/// The secret a `ROLE#FIELD` reference in `variable` names, `None` when the
/// variable is unset. With `SPIS_CREDENTIALS_FILE` set the file answers and
/// Stado is not asked; otherwise `STADO_BIN`, or the `stado` the PATH finds,
/// is asked.
pub(super) fn role_secret(variable: &str) -> Result<Option<String>> {
    // A reference that is not text is named, not read as absent: read as
    // absent, a declared secret was silently replaced by none.
    let reference = match std::env::var(variable) {
        Ok(value) if !value.trim().is_empty() => value,
        Ok(_) | Err(std::env::VarError::NotPresent) => return Ok(None),
        Err(error) => return Err(usage(format!("{variable} cannot be read: {error}"))),
    };
    let reference = reference.trim();
    let Some((role, field)) = reference
        .rsplit_once('#')
        .filter(|(role, field)| !role.is_empty() && !field.is_empty())
    else {
        return Err(usage(format!(
            "{variable} must be a role reference ROLE#FIELD, not {reference:?}"
        )));
    };
    if let Some(file) = std::env::var_os(CREDENTIALS_FILE) {
        if file.is_empty() {
            return Err(usage(format!("{CREDENTIALS_FILE} is present but empty")));
        }
        return file_secret(variable, Path::new(&file), role, field).map(Some);
    }
    stado_secret(variable, role, field).map(Some)
}

/// `role`'s `field` as Stado reads it.
fn stado_secret(variable: &str, role: &str, field: &str) -> Result<String> {
    let program = match std::env::var_os("STADO_BIN").filter(|path| !path.is_empty()) {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("stado"),
    };
    let binary = program.display();
    let output = stado_wait::output(std::process::Command::new(&program).args([
        "credentials",
        "get",
        "--role",
        role,
        "--field",
        field,
    ]))
    .with_context(|| {
        format!(
            "{variable}: {binary} could not be run; without Stado set {CREDENTIALS_FILE} to an \
             owner-only JSON file of role -> field -> value"
        )
    })?;
    if !output.status.success() {
        anyhow::bail!(
            "{variable}: {binary} credentials get --role {role} --field {field} failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let secret = String::from_utf8(output.stdout)
        .map_err(|error| {
            anyhow::anyhow!(
                "{variable}: {binary} returned a value that is not UTF-8: {}",
                error.utf8_error()
            )
        })?
        .trim_end_matches('\n')
        .to_string();
    if secret.is_empty() {
        anyhow::bail!("{variable}: the item playing role {role} holds no value in field {field}");
    }
    Ok(secret)
}

/// `role`'s `field` from the owner-only credentials file. A file another user
/// owns, or one other users can read, is refused as Skarbiec's own vault file
/// would be.
fn file_secret(variable: &str, file: &Path, role: &str, field: &str) -> Result<String> {
    let named = format!("{variable}: {CREDENTIALS_FILE} {}", file.display());
    // Opened without blocking so a FIFO is inspected, not waited on.
    let mut opened = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(file)
        .with_context(|| format!("{named} cannot be opened"))?;
    let metadata = opened
        .metadata()
        .with_context(|| format!("{named} cannot be inspected"))?;
    if !metadata.is_file() {
        anyhow::bail!("{named} is not a regular file");
    }
    // SAFETY: geteuid takes no arguments and cannot fail.
    let process_owner = unsafe { libc::geteuid() };
    if metadata.uid() != process_owner {
        anyhow::bail!(
            "{named} is owned by uid {}, not the process uid {process_owner}",
            metadata.uid()
        );
    }
    let mode = metadata.permissions().mode();
    let others = u32::from(libc::S_IRWXG | libc::S_IRWXO);
    // Clearing the group and other bits changes the mode exactly when some
    // of them were set.
    if mode & !others != mode {
        anyhow::bail!(
            "{named} must be readable by its owner only (mode {:o})",
            mode & (others | u32::from(libc::S_IRWXU))
        );
    }
    let mut text = String::new();
    opened
        .read_to_string(&mut text)
        .with_context(|| format!("{named} cannot be read"))?;
    let roles: Value = serde_json::from_str(&text)
        .with_context(|| format!("{named} is not a JSON object of role -> field -> value"))?;
    let fields = roles
        .get(role)
        .with_context(|| format!("{named} has no role {role}"))?
        .as_object()
        .with_context(|| format!("{named} role {role} must be an object of fields"))?;
    let value = fields
        .get(field)
        .with_context(|| format!("{named} role {role} has no field {field}"))?
        .as_str()
        .with_context(|| format!("{named} field {role}#{field} must be text"))?;
    if value.is_empty() {
        anyhow::bail!("{named} field {role}#{field} is empty");
    }
    Ok(value.to_owned())
}
