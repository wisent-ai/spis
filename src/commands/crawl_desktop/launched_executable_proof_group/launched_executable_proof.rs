use super::*;

/// Prove the process now running is the exact executable the manifest bound:
/// the path comes from an anchored field of the launch response, must be an
/// absolute canonical non-symlink file, and is re-hashed against the manifest
/// digest (finding 5b).
pub(crate) fn launched_executable_proof(
    launched: &LaunchedApp,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<Value> {
    let identity = manifest
        .execution_identity
        .as_ref()
        .context("desktop runtime manifest has no resolved execution identity")?;
    let expected_path = identity
        .executable_path
        .as_deref()
        .context("desktop execution identity has no exact executable path")?;
    let observed = launched.executable_path.as_deref().context(
        "cua-driver launch response has no anchored executable path for the launched pid",
    )?;
    if observed != expected_path {
        return Err(anyhow::Error::new(RecordFailure {
            code: "desktop_executable_identity_mismatch",
            message: format!(
                "launched pid {} runs {observed}, not the manifest-bound executable {expected_path}",
                launched.pid
            ),
        }));
    }
    let path = Path::new(observed);
    if !path.is_absolute() {
        bail!("launched executable path {observed} is not absolute");
    }
    if std::fs::symlink_metadata(path)?.file_type().is_symlink() {
        bail!("launched executable path {observed} is a symlink");
    }
    let canonical = std::fs::canonicalize(path)?;
    if canonical != path {
        bail!(
            "launched executable path is not canonical: declared {observed}, canonical {}",
            canonical.display()
        );
    }
    let expected_sha = identity
        .executable_sha256
        .as_deref()
        .context("desktop execution identity has no executable SHA-256")?;
    let observed_sha = hash_file(path)?;
    if !observed_sha.eq_ignore_ascii_case(expected_sha) {
        bail!(
            "launched executable SHA-256 differs from the manifest: expected {expected_sha}, observed {observed_sha}"
        );
    }
    Ok(json!({
        "pid": launched.pid,
        "executable_path": canonical,
        "executable_sha256": observed_sha,
        "matches_manifest_execution_identity": true,
    }))
}
