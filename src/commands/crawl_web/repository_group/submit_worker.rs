use super::*;

pub(crate) fn submit_worker(
    host: &str,
    catalog: &str,
    record: &str,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<()> {
    safe_job_value(host, "--host")?;
    safe_job_value(record, "--record")?;
    if crate::commands::crawl::build_revision()? != manifest.source_revision {
        bail!("web coordinator revision does not match immutable runtime manifest");
    }
    let service = manifest
        .service_identity
        .as_ref()
        .context("web crawls require the exact Weles service identity in the runtime manifest")?;
    if service.active_host != host {
        bail!(
            "runtime manifest Weles service identity is bound to {}, not {host}",
            service.active_host
        );
    }
    if manifest.delivery.kind != "weles-service-env" {
        bail!("web credential delivery must be weles-service-env");
    }
    if !manifest.delivery.secret_env.keys().map(String::as_str)
        .eq(["WELES_TOKEN", "WISENT_ORGANIZATION_ID"])
    {
        bail!("web delivery must carry exactly the WELES_TOKEN and WISENT_ORGANIZATION_ID secret references");
    }
    // Stado's non-login shell requires the resolved absolute cargo path.
    let cargo = crate::commands::crawl::resolved_worker_program(host)?;
    let command = format!(
        "{cargo} run --release -- crawler run web {catalog} --worker --record {record} --artifact-uri {} --runtime-manifest-base64 '{}'",
        manifest.artifact_uri,
        manifest.encoded()?,
    );
    let mut arguments = vec![
        "submit".to_string(),
        command,
        "--run-id".to_string(),
        manifest.stado_run_id.clone(),
        "--pinned-host".to_string(),
        host.to_string(),
        "--repo".to_string(),
        REPOSITORY.to_string(),
        "--repo-ref".to_string(),
        manifest.source_revision.clone(),
        "--repo-workdir".to_string(),
        super::crawl::STADO_REPO_WORKDIR.to_string(),
        "--repo-extras".to_string(),
        String::new(),
        "--output-uri".to_string(),
        manifest.output_uri.clone(),
    ];
    // `secret_env` is a BTreeMap, so the injected references are already in sorted key
    // order. The values are opaque `role#field` references and are never logged.
    for (name, reference) in &manifest.delivery.secret_env {
        arguments.push("--secret-env".to_string());
        arguments.push(format!("{name}={reference}"));
    }
    let mut stado = crate::commands::crawl::stado_command();
    stado.args(arguments);
    let output =
        crate::commands::crawl::command_output(&mut stado, "submit web crawl through Stado")?;
    if !output.status.success() {
        bail!(
            "Stado refused web crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    crate::commands::crawl::print_submission(
        catalog,
        "web",
        host,
        Some(&manifest.artifact_uri),
        &manifest.output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

/// The private, never-published, never-logged bridge work area. It holds exactly one
/// file: the protected config carrying the delivered bearer. Bridge commands travel on
/// the child's stdin, so no command is ever written to the host.
pub(crate) struct PrivateBridge {
    pub(crate) config: PathBuf,
}

impl PrivateBridge {
    pub(crate) fn open(manifest: &super::crawl::RuntimeManifest) -> Result<Self> {
        let directory = crate::commands::docs_corpus::work_area("weles-bridge")?;
        std::fs::create_dir_all(&directory)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(libc::S_IRWXU.into()))?;
        }
        prune_orphaned_configs(&directory)?;
        let config = directory.join(format!(
            "config-{}-{}.json",
            manifest.attempt_id,
            std::process::id()
        ));
        Ok(Self { config })
    }

    /// Removes the delivered bearer token from the host. `Drop` guarantees this runs on
    /// every exit this process controls, including a panic unwind; the explicit call in
    /// `run_worker` only makes it prompt, so the second call finds nothing to remove. A
    /// signal kill runs neither, which is why `open` also prunes the configs earlier runs
    /// orphaned. A config that is still there and cannot be removed holds a live bearer,
    /// so that failure is said on stderr with the path and cause.
    pub(crate) fn discard(&self) {
        match std::fs::remove_file(&self.config) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => eprintln!(
                "spis: the Weles bridge config {} still holds the delivered bearer and could not be removed: {error}",
                self.config.display()
            ),
        }
    }

    pub(crate) fn write_config(
        &self,
        endpoint: &str,
        bearer: &str,
        organization_id: &str,
    ) -> Outcome<()> {
        let document = json!({
            "schema": weles::BRIDGE_CONFIG_SCHEMA,
            "endpoint": endpoint,
            "bearer": bearer,
            "organizationId": organization_id,
        });
        write_private(&self.config, &serde_json::to_vec(&document)?)
    }
}

impl Drop for PrivateBridge {
    fn drop(&mut self) {
        self.discard();
    }
}

/// `attempt-{attempt}-{fingerprint}`: the exact identifier
/// `crawl::finalize_manifest_identity` derives and the Weles public admission runtime
/// re-derives, a decimal attempt number and the first 16 hex characters of the attempt
/// digest.
pub(crate) fn is_attempt_id(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("attempt-") else {
        return false;
    };
    let Some((attempt, fingerprint)) = rest.split_once('-') else {
        return false;
    };
    !attempt.is_empty()
        && attempt.bytes().all(|byte| byte.is_ascii_digit())
        && is_lowercase_hex(fingerprint, 16) // https://github.com/wisent-ai/weles/blob/main/src/worker/public-task-service/admission/identity.mjs
}

/// Is `file_name` one of this worker's own config names, `config-{attempt_id}-{pid}.json`,
/// whose process is gone?
///
/// Only the exact attempt identity and a positive `pid_t` identify this worker's
/// files. A live process always retains its config; similar names are not ours.
pub(crate) fn is_orphaned_config(file_name: &str) -> bool {
    let Some(body) = file_name
        .strip_prefix("config-")
        .and_then(|rest| rest.strip_suffix(".json"))
    else {
        return false;
    };
    let Some((attempt_id, pid)) = body.rsplit_once('-') else {
        return false;
    };
    if !is_attempt_id(attempt_id) {
        return false;
    }
    // Parsed as the signed `pid_t` the kernel takes: a value that does not fit is not a
    // pid this worker ever wrote, and a wrapped negative would address a process group.
    let Ok(pid) = pid.parse::<i32>() else {
        return false;
    };
    pid.is_positive() && !process_is_live(pid)
}

/// A worker killed by a signal — which is exactly how a job that outruns its wait budget
/// ends — runs no destructor, so its config file survives on the host with the delivered
/// bearer in it. Nothing else sweeps this directory, so every worker sweeps it on the way
/// in, mirroring `crawl_docs::prune_stale_temporaries`.
pub(crate) fn prune_orphaned_configs(directory: &Path) -> Result<()> {
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "list the private Weles bridge directory {}",
                    directory.display()
                )
            })
        }
    };
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if is_orphaned_config(file_name) && entry.file_type()?.is_file() {
            let path = entry.path();
            std::fs::remove_file(&path)
                .with_context(|| format!("remove the orphaned Weles config {}", path.display()))?;
        }
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn process_is_live(pid: i32) -> bool {
    if unsafe { libc::kill(pid, 0) } == 0 { // POSIX null signal and success: https://pubs.opengroup.org/onlinepubs/9799919799/functions/kill.html
        return true;
    }
    // Only a definite `ESRCH` proves the process is gone. `EPERM` means it exists under
    // another user, and any other errno is unexplained, so neither authorizes deleting a
    // file that still names a live bearer.
    std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

#[cfg(not(unix))]
pub(crate) fn process_is_live(_pid: i32) -> bool {
    true
}

pub(crate) fn write_private(path: &Path, bytes: &[u8]) -> Outcome<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(libc::S_IRWXU.into()))?;
        }
    }
    // A previous file that cannot be cleared is refused by its own cause; left
    // in place it would surface only as create_new's "already exists".
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            return Err(WorkerFailure::new(
                "web_worker_io_failed",
                format!(
                    "the previous private file {} could not be cleared: {error}",
                    path.display()
                ),
            ));
        }
        _ => {}
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    // The bridge refuses a config that any other user could read, so the bearer is never
    // written through a mode that has to be tightened afterwards.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode((libc::S_IRUSR | libc::S_IWUSR).into());
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

/// Content-addressed byte retention: an identical file is proof, never a rewrite.
pub(crate) fn write_exact(path: &Path, bytes: &[u8]) -> Outcome<()> {
    match std::fs::read(path) {
        Ok(existing) if existing == bytes => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(WorkerFailure::new(
            "web_worker_io_failed",
            format!("read content-addressed file {}: {error}", path.display()),
        )),
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}
