use super::*;

pub(crate) fn verify_exact_executable(
    manifest: &super::crawl::RuntimeManifest,
    expected_filename: &str,
    environment: &BTreeMap<OsString, OsString>,
) -> Result<PathBuf> {
    let identity = manifest
        .execution_identity
        .as_ref()
        .context("CLI runtime manifest has no resolved execution identity")?;
    if identity.platform != "terminal" {
        bail!(
            "CLI execution identity platform differs: expected \"terminal\", observed {:?}",
            identity.platform
        );
    }
    if manifest.runtime_product.identifier != expected_filename {
        bail!(
            "CLI manifest product identifier differs: expected {expected_filename:?}, observed {:?}",
            manifest.runtime_product.identifier
        );
    }
    if identity.host.is_empty() {
        bail!("CLI execution identity has no registry host alias");
    }
    let identity_value = serde_json::to_value(identity)?;
    let expected_hostname = identity_value
        .get("observed_hostname")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("CLI execution identity has no typed observed_hostname")?;
    let mut hostname_command = Command::new("hostname");
    hostname_command.env_clear().env("PATH", "/usr/bin:/bin");
    let hostname = super::crawl::bounded_command_output(
        &mut hostname_command,
        "read CLI worker hostname",
        Duration::from_secs(10),
        64 * 1024,
    )?;
    if !hostname.status.success() {
        bail!(
            "CLI worker hostname command failed: status={}; stdout={:?}; stderr={:?}",
            hostname.status,
            String::from_utf8_lossy(&hostname.stdout),
            String::from_utf8_lossy(&hostname.stderr)
        );
    }
    let observed_hostname = String::from_utf8_lossy(&hostname.stdout).trim().to_string();
    if observed_hostname != expected_hostname {
        bail!(
            "CLI observed hostname differs: expected {expected_hostname:?}, observed {observed_hostname:?}"
        );
    }
    let configured = identity
        .executable_path
        .as_deref()
        .context("CLI execution identity has no exact executable path")?;
    let path = PathBuf::from(configured);
    if !path.is_absolute() || !path.is_file() {
        bail!("CLI execution identity path is not an absolute executable file: {configured}");
    }
    let canonical = std::fs::canonicalize(&path)
        .with_context(|| format!("canonicalize exact CLI executable {}", path.display()))?;
    if canonical != path {
        bail!(
            "CLI execution identity path is not canonical: declared {}, canonical {}",
            path.display(),
            canonical.display()
        );
    }
    if path.file_name().and_then(|value| value.to_str()) != Some(expected_filename) {
        bail!(
            "CLI execution identity filename differs: expected {expected_filename}, observed {}",
            path.display()
        );
    }
    let expected_sha = identity
        .executable_sha256
        .as_deref()
        .context("CLI execution identity has no executable SHA-256")?;
    let mut file = std::fs::File::open(&path)
        .with_context(|| format!("open exact CLI executable {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash exact CLI executable {}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let observed_sha = hex::encode(digest.finalize());
    if !observed_sha.eq_ignore_ascii_case(expected_sha) {
        bail!(
            "CLI executable SHA-256 changed immediately before use: expected {expected_sha}, observed {observed_sha}"
        );
    }
    let expected_version = identity
        .product_version
        .as_deref()
        .context("CLI execution identity has no exact product version")?;
    let mut version_command = Command::new(&path);
    version_command
        .arg("--version")
        .env_clear()
        .envs(environment);
    let version = super::crawl::bounded_command_output(
        &mut version_command,
        "read exact CLI version",
        Duration::from_secs(30),
        1024 * 1024,
    )
    .with_context(|| format!("read exact CLI version from {}", path.display()))?;
    if !version.status.success() {
        bail!(
            "exact CLI version command failed immediately before use: status={}; stdout={:?}; stderr={:?}",
            version.status,
            String::from_utf8_lossy(&version.stdout),
            String::from_utf8_lossy(&version.stderr)
        );
    }
    let observed_version = String::from_utf8_lossy(&version.stdout).trim().to_string();
    if observed_version != expected_version {
        bail!(
            "CLI product version changed immediately before use: expected {expected_version:?}, observed {observed_version:?}"
        );
    }
    Ok(path)
}

pub(crate) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn tmux(
    socket: &Path,
    environment: &BTreeMap<OsString, OsString>,
    args: &[&str],
    context: &str,
) -> Result<String> {
    let mut command = Command::new("tmux");
    command
        .env_clear()
        .envs(environment)
        .args(["-S", socket.to_string_lossy().as_ref()])
        .args(args);
    let output = super::crawl::bounded_command_output(
        &mut command,
        context,
        Duration::from_secs(30),
        MAXIMUM_CAPTURE_BYTES,
    )?;
    if !output.status.success() {
        bail!(
            "{context}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(crate) struct TmuxSession {
    pub(crate) name: String,
    pub(crate) socket: PathBuf,
    pub(crate) environment: BTreeMap<OsString, OsString>,
}

impl Drop for TmuxSession {
    fn drop(&mut self) {
        // Short deadline: cleanup must never block the worker (finding 4).
        let mut command = Command::new("tmux");
        command
            .env_clear()
            .env("PATH", "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin")
            .args(["-S", self.socket.to_string_lossy().as_ref()])
            .args(["kill-session", "-t", &self.name]);
        let _ = super::crawl::bounded_command_output(
            &mut command,
            "close private CLI PTY",
            Duration::from_secs(5),
            64 * 1024,
        );
    }
}

/// A capture larger than this is refused rather than digested as evidence.
pub(crate) const MAXIMUM_CAPTURE_BYTES: usize = 4 * 1024 * 1024;

pub(crate) fn capture_range(session: &TmuxSession, start: &str, context: &'static str) -> Result<String> {
    let screen = tmux(
        &session.socket,
        &session.environment,
        &["capture-pane", "-t", &session.name, "-p", "-e", "-S", start],
        context,
    )?;
    if screen.len() > MAXIMUM_CAPTURE_BYTES {
        bail!(
            "{context} returned {} bytes, beyond the {MAXIMUM_CAPTURE_BYTES}-byte bound",
            screen.len()
        );
    }
    Ok(screen)
}

/// Marker polling reads only the visible tail; `-S -` would re-read the whole
/// history ten times a second for the entire deadline (finding 8).
pub(crate) fn capture_tail(session: &TmuxSession) -> Result<String> {
    capture_range(session, "-50", "poll CLI PTY tail")
}

/// Exactly one bounded full capture per invocation (finding 8).
pub(crate) fn capture_history(session: &TmuxSession) -> Result<String> {
    capture_range(session, "-2000", "capture CLI PTY")
}

pub(crate) fn await_marker(session: &TmuxSession, marker: &str, timeout: Duration) -> Result<bool> {
    let deadline = Instant::now() + timeout;
    loop {
        if capture_tail(session)?.contains(marker) {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub(crate) fn clean_terminal(value: &str) -> String {
    static ANSI: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]")
            .expect("static ANSI regex")
    });
    ANSI.replace_all(value, "").replace('\r', "")
}

/// A fresh 128-bit value per invocation. The program under test never observes
/// the wall-clock nanosecond, the worker pid, the invocation index or the
/// record key, so it cannot predict the markers or the exit-status path and
/// therefore cannot forge either (finding 7).
pub(crate) fn invocation_nonce(record_key: &str, index: usize) -> Result<String> {
    let nanoseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("read the wall clock for a CLI invocation nonce")?
        .as_nanos();
    let mut nonce = crate::sha256_hex(
        format!("{nanoseconds}|{}|{index}|{record_key}", std::process::id()).as_bytes(),
    );
    nonce.truncate(32);
    Ok(nonce)
}
