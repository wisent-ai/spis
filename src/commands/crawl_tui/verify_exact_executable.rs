use super::*;

pub(crate) fn verify_exact_executable(
    manifest: &super::crawl::RuntimeManifest,
    expected_filename: &str,
    environment: &BTreeMap<OsString, OsString>,
) -> Result<PathBuf> {
    let identity = manifest
        .execution_identity
        .as_ref()
        .context("TUI runtime manifest has no resolved execution identity")?;
    if identity.platform != "terminal" {
        bail!(
            "TUI execution identity platform differs: expected \"terminal\", observed {:?}",
            identity.platform
        );
    }
    if manifest.runtime_product.identifier != expected_filename {
        bail!(
            "TUI manifest product identifier differs: expected {expected_filename:?}, observed {:?}",
            manifest.runtime_product.identifier
        );
    }
    if identity.host.is_empty() {
        bail!("TUI execution identity has no registry host alias");
    }
    let identity_value = serde_json::to_value(identity)?;
    let expected_hostname = identity_value
        .get("observed_hostname")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("TUI execution identity has no typed observed_hostname")?;
    let mut hostname_command = Command::new("hostname");
    hostname_command.env_clear().env("PATH", "/usr/bin:/bin");
    let hostname = super::crawl::bounded_command_output(
        &mut hostname_command,
        "read TUI worker hostname",
        Duration::from_secs(10),
        64 * 1024,
    )?;
    if !hostname.status.success() {
        bail!(
            "TUI worker hostname command failed: status={}; stdout={:?}; stderr={:?}",
            hostname.status,
            String::from_utf8_lossy(&hostname.stdout),
            String::from_utf8_lossy(&hostname.stderr)
        );
    }
    let observed_hostname = String::from_utf8_lossy(&hostname.stdout).trim().to_string();
    if observed_hostname != expected_hostname {
        bail!(
            "TUI observed hostname differs: expected {expected_hostname:?}, observed {observed_hostname:?}"
        );
    }
    let configured = identity
        .executable_path
        .as_deref()
        .context("TUI execution identity has no exact executable path")?;
    let path = PathBuf::from(configured);
    if !path.is_absolute() || !path.is_file() {
        bail!("TUI execution identity path is not an absolute executable file: {configured}");
    }
    let canonical = std::fs::canonicalize(&path)
        .with_context(|| format!("canonicalize exact TUI executable {}", path.display()))?;
    if canonical != path {
        bail!(
            "TUI execution identity path is not canonical: declared {}, canonical {}",
            path.display(),
            canonical.display()
        );
    }
    if path.file_name().and_then(|value| value.to_str()) != Some(expected_filename) {
        bail!(
            "TUI execution identity filename differs: expected {expected_filename}, observed {}",
            path.display()
        );
    }
    let expected_sha = identity
        .executable_sha256
        .as_deref()
        .context("TUI execution identity has no executable SHA-256")?;
    let mut file = std::fs::File::open(&path)
        .with_context(|| format!("open exact TUI executable {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash exact TUI executable {}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let observed_sha = hex::encode(digest.finalize());
    if !observed_sha.eq_ignore_ascii_case(expected_sha) {
        bail!(
            "TUI executable SHA-256 changed immediately before trajectory launch: expected {expected_sha}, observed {observed_sha}"
        );
    }
    let expected_version = identity
        .product_version
        .as_deref()
        .context("TUI execution identity has no exact product version")?;
    let mut version_command = Command::new(&path);
    version_command
        .arg("--version")
        .env_clear()
        .envs(environment);
    let version = super::crawl::bounded_command_output(
        &mut version_command,
        "read exact TUI version",
        Duration::from_secs(30),
        1024 * 1024,
    )
    .with_context(|| format!("read exact TUI version from {}", path.display()))?;
    if !version.status.success() {
        bail!(
            "exact TUI version command failed immediately before launch: status={}; stdout={:?}; stderr={:?}",
            version.status,
            String::from_utf8_lossy(&version.stdout),
            String::from_utf8_lossy(&version.stderr)
        );
    }
    let observed_version = String::from_utf8_lossy(&version.stdout).trim().to_string();
    if observed_version != expected_version {
        bail!(
            "TUI product version changed immediately before trajectory launch: expected {expected_version:?}, observed {observed_version:?}"
        );
    }
    Ok(path)
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

/// Readiness polling reads only the visible tail; `-S -` would re-read the
/// entire scrollback on every poll (finding 8).
pub(crate) fn capture_tail(session: &TmuxSession) -> Result<String> {
    capture_range(session, "-50", "poll TUI pane tail")
}

/// Exactly one bounded full capture per record (finding 8).
pub(crate) fn capture(session: &TmuxSession) -> Result<String> {
    capture_range(session, "-2000", "capture TUI pane")
}

pub(crate) fn hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    hex::encode(digest.finalize())
}

pub(crate) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn prepare_fixture(fixture: &Path, git: &Path) -> Result<()> {
    let home = fixture.join("home");
    std::fs::create_dir_all(&home)?;
    for directory in [
        home.join(".config"),
        home.join(".local/share"),
        home.join(".cache"),
        home.join(".local/state"),
        fixture.join("runtime"),
    ] {
        std::fs::create_dir_all(directory)?;
    }
    std::fs::write(fixture.join("seed.txt"), "Spis TUI crawl fixture\n")?;
    std::fs::write(fixture.join("tracked.txt"), "committed fixture state\n")?;
    let run_git = |arguments: &[&str]| -> Result<()> {
        // The absolute git the host resolved, never a name this PATH would
        // have to search. `/usr/bin/git` is refused upstream because on macOS
        // it is the `xcode-select` shim, which opens the Command Line Tools
        // installer window on a host that has none.
        let mut command = Command::new(git);
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .args(arguments)
            .current_dir(fixture)
            .env("HOME", &home)
            .env("GIT_CONFIG_GLOBAL", fixture.join("gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1");
        let output = super::crawl::bounded_command_output(
            &mut command,
            "prepare TUI fixture with git",
            Duration::from_secs(60),
            1024 * 1024,
        )
        .with_context(|| format!("prepare TUI fixture: git {}", arguments.join(" ")))?;
        if !output.status.success() {
            bail!(
                "prepare TUI fixture: git {}: {}",
                arguments.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(())
    };
    run_git(&["init", "--quiet"])?;
    run_git(&["config", "user.name", "Spis crawler"])?;
    run_git(&["config", "user.email", "spis-crawler@invalid"])?;
    run_git(&["add", "seed.txt", "tracked.txt"])?;
    run_git(&["commit", "--quiet", "-m", "Seed isolated crawl fixture"])?;
    std::fs::write(
        fixture.join("tracked.txt"),
        "committed fixture state\nmodified fixture state\n",
    )?;
    std::fs::write(fixture.join("untracked.txt"), "untracked fixture state\n")?;
    Ok(())
}
