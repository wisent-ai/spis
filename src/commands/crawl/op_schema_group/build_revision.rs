use super::*;

pub(crate) fn build_revision() -> Result<String> {
    let revision = env!("SPIS_GIT_REVISION");
    let dirty = env!("SPIS_GIT_DIRTY");
    if dirty != "false" {
        bail!("this Spis binary was built from a dirty source tree; exact-revision crawl submission is refused");
    }
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("this Spis binary has no identifiable build-time Git revision");
    }
    Ok(revision.to_string())
}

pub(crate) fn bounded_git(arguments: &[&str], operation: &str) -> Result<Output> {
    let mut command = Command::new("git");
    command.arg("-C").arg(source_root()).args(arguments);
    bounded_command_output(
        &mut command,
        operation,
        Duration::from_secs(30),
        4 * 1024 * 1024,
    )
}

pub(crate) fn source_snapshot_revision() -> Result<String> {
    let embedded = build_revision()?;
    let head = bounded_git(
        &["rev-parse", "--verify", "HEAD"],
        "read current Spis source revision",
    )?;
    if !head.status.success() {
        bail!(
            "cannot identify current Spis source revision: {}",
            String::from_utf8_lossy(&head.stderr).trim()
        );
    }
    let runtime = String::from_utf8(head.stdout)
        .context("current source revision is not UTF-8")?
        .trim()
        .to_string();
    if runtime != embedded {
        bail!(
            "stale Spis binary: embedded revision {embedded} differs from runtime source revision {runtime}"
        );
    }
    let status = bounded_git(
        &["status", "--porcelain=v1", "--untracked-files=all"],
        "verify current Spis source snapshot",
    )?;
    if !status.status.success() {
        bail!(
            "cannot verify current Spis source snapshot: {}",
            String::from_utf8_lossy(&status.stderr).trim()
        );
    }
    if !status.stdout.is_empty() {
        bail!("current Spis source snapshot is dirty; crawl planning is refused");
    }
    Ok(runtime)
}

/// Environment Stado itself reads to find its configuration, authenticate, and
/// select the backend a `stado://` URI resolves against. Nothing else reaches a
/// crawl subprocess: the worker environment carries unrelated credentials that
/// Stado has no use for, and a child that never receives them cannot leak them.
///
/// Every name here is a documented Stado binding, not a guess:
/// `STADO_CONFIG` is `config_file::FILE_ENV`, the explicit config-file override
/// that precedes `./stado.config.json` and `~/.config/stado/config.json`;
/// `STADO_API_TOKEN` and `STADO_API_TOKEN_FILE` are the object API bearer, read
/// directly by the storage client before it falls back to
/// `storage.stado.token_file`; `WC_STORAGE_BACKEND` selects the storage adapter;
/// `WC_STADO_STORAGE_URL`, `WC_STADO_STORAGE_TOKEN_FILE`,
/// `WC_STADO_STORAGE_NAMESPACE` and `WC_STADO_STORAGE_CA_FILE` are that
/// adapter's catalog fields (endpoint, token file, namespace, private CA root);
/// `WC_LOCAL_STORAGE_PATH` is the same field for a device-local backend; and
/// `STADO_RESOLVER_SSH_KEY_FILE` relocates the resolver key `stado host exec`
/// authenticates with. Attribution-only variables (`USER`, `LOGNAME`,
/// `HOSTNAME`) are deliberately excluded: they feed a registry actor string that
/// already defaults to empty, and they are not needed to reach any backend.
pub(crate) const STADO_PASSTHROUGH_ENV: [&str; 12] = [
    "PATH",
    "HOME",
    "STADO_CONFIG",
    "STADO_API_TOKEN",
    "STADO_API_TOKEN_FILE",
    "STADO_RESOLVER_SSH_KEY_FILE",
    "WC_STORAGE_BACKEND",
    "WC_STADO_STORAGE_URL",
    "WC_STADO_STORAGE_TOKEN_FILE",
    "WC_STADO_STORAGE_NAMESPACE",
    "WC_STADO_STORAGE_CA_FILE",
    "WC_LOCAL_STORAGE_PATH",
];

pub(crate) fn stado_command() -> Command {
    let mut command =
        Command::new(std::env::var_os("SPIS_STADO_BIN").unwrap_or_else(|| "stado".into()));
    command.env_clear();
    for name in STADO_PASSTHROUGH_ENV {
        // Only when set: an empty value is a configured value to Stado, and
        // would mask the config file entry the operator actually wrote.
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}

#[doc(hidden)]
pub fn host_home_crawl_token_path(home: &Path) -> PathBuf {
    home.join(".stado").join("spis-crawls-object-api-token")
}

#[doc(hidden)]
pub fn validate_worker_runtime_files(
    stado_path: &Path,
    stado_present: bool,
    token_path: &Path,
    token_present: bool,
) -> Result<()> {
    if !stado_present {
        bail!(
            "worker_stado_binary_missing: Stado's declared worker binary is missing at {}",
            stado_path.display()
        );
    }
    if !token_present {
        bail!(
            "worker_crawl_token_missing: the spis-crawls bearer file is missing at {}",
            token_path.display()
        );
    }
    Ok(())
}

/// Stado invocation for objects Spis owns, i.e. everything under
/// [`crate::CRAWL_NAMESPACE`].
///
/// One object request carries exactly one bearer, and Stado compares it against
/// the credential item of the namespace being addressed. The coordinator
/// therefore needs two: its own for `spis-crawls`, and whatever the host
/// already uses for the queue plane the job submission reads and writes
/// (`probierz/...`). Forcing one bearer for both made `crawl start` choose
/// which half to break: with the crawl bearer the queue write was refused,
/// without it the runtime-bindings read-back was.
///
/// `SPIS_CRAWL_OBJECT_TOKEN_FILE` explicitly names the owner-only file holding
/// the crawl namespace bearer. When it is unset, a worker uses the canonical
/// file under its own host `HOME` when present. It is injected as
/// `STADO_API_TOKEN_FILE` on exactly the invocations that address Spis's own
/// objects and on no other, so the queue plane keeps the host's configured
/// bearer.
pub(crate) fn crawl_storage_command() -> Command {
    let mut command = stado_command();
    let configured = std::env::var_os("SPIS_CRAWL_OBJECT_TOKEN_FILE");
    let host_home_token = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| host_home_crawl_token_path(&home))
        .filter(|path| path.is_file());
    if let Some(token_file) = configured.or_else(|| host_home_token.map(Into::into)) {
        command.env("STADO_API_TOKEN_FILE", token_file);
        command.env_remove("STADO_API_TOKEN");
    }
    command
}

pub(crate) fn read_bounded<R: Read>(mut reader: R, maximum: usize) -> std::io::Result<(Vec<u8>, bool)> {
    let mut retained = Vec::with_capacity(maximum.min(64 * 1024));
    let mut buffer = [0_u8; 16 * 1024];
    let mut overflow = false;
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let remaining = maximum.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..count.min(remaining)]);
        overflow |= count > remaining;
    }
    Ok((retained, overflow))
}

pub(crate) fn bounded_command_output(
    command: &mut Command,
    operation: &str,
    timeout: Duration,
    maximum_stream_bytes: usize,
) -> Result<Output> {
    use std::process::Stdio;
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("start {operation}"))?;
    let stdout = child.stdout.take().context("capture bounded child stdout")?;
    let stderr = child.stderr.take().context("capture bounded child stderr")?;
    let stdout_reader = std::thread::spawn(move || read_bounded(stdout, maximum_stream_bytes));
    let stderr_reader = std::thread::spawn(move || read_bounded(stderr, maximum_stream_bytes));
    let deadline = Instant::now() + timeout;
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait()? {
            break (status, false);
        }
        if Instant::now() >= deadline {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            break (child.wait()?, true);
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let (stdout, stdout_overflow) = stdout_reader
        .join()
        .map_err(|_| anyhow!("{operation} stdout reader panicked"))??;
    let (stderr, stderr_overflow) = stderr_reader
        .join()
        .map_err(|_| anyhow!("{operation} stderr reader panicked"))??;
    if timed_out {
        return Err(CommandTimedOut {
            operation: operation.to_string(),
            timeout,
            stdout,
            stderr,
        }
        .into());
    }
    if stdout_overflow || stderr_overflow {
        bail!("{operation} exceeded the {maximum_stream_bytes}-byte stdout/stderr bound");
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

pub(crate) fn atomic_json_write(path: &Path, value: &Value) -> Result<()> {
    let parent = path.parent().context("JSON path has no parent")?;
    std::fs::create_dir_all(parent)?;
    let file_name = path.file_name().and_then(|name| name.to_str()).context("JSON filename is not UTF-8")?;
    let lock_path = parent.join(format!(".{file_name}.lock"));
    let lock = OpenOptions::new().read(true).write(true).create(true).open(&lock_path)?;
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        bail!("another process is updating {}", path.display());
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<()> {
        let mut output = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        output.write_all((serde_json::to_string_pretty(value)? + "\n").as_bytes())?;
        output.sync_all()?;
        std::fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    let _ = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) };
    drop(lock);
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

pub(crate) const MAX_ATTEMPT_TREE_ENTRIES: usize = 20_000;

pub(crate) const MAX_ATTEMPT_TREE_BYTES: u64 = 512 * 1024 * 1024;

pub(crate) const MAX_ATTEMPT_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;
