use super::*;

pub(crate) const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";

#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub(crate) slug: String,
    pub(crate) name: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Step {
    pub(crate) role: String,
    pub(crate) label: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Action {
    pub(crate) role: String,
    pub(crate) label: String,
    pub(crate) token: String,
    pub(crate) destructive: bool,
}

/// Only the absolute bundle paths the coordinator itself admits. Inherited
/// PATH entries are deliberately absent: any writable earlier PATH entry would
/// be arbitrary code execution with accessibility privileges (finding 10).
pub(crate) fn cua_driver_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications/CuaDriver.app/Contents/MacOS/cua-driver"),
        PathBuf::from("/Applications/CuaDriver.app/Contents/MacOS/CuaDriver"),
    ]
}

/// The driver binary pinned for the whole record: path, digest and version are
/// observed once and every later call runs this exact file.
#[derive(Clone, Debug)]
pub(crate) struct CuaDriver {
    pub(crate) path: PathBuf,
    pub(crate) sha256: String,
    pub(crate) version: String,
}

pub(crate) fn hash_file(path: &Path) -> Result<String> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash {}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex::encode(digest.finalize()))
}

/// Resolve, canonicalize, hash and version-stamp the driver exactly once per
/// record. Re-resolving inside every call left the binary unstable even within
/// one record, and following a symlink out of the bundle defeated the pin
/// entirely (finding 10).
pub(crate) fn pin_cua_driver() -> Result<CuaDriver> {
    let path = cua_driver_candidates()
        .into_iter()
        .find(|candidate| candidate.is_file())
        .context("Cua Driver executable is absent from the admitted /Applications bundle paths")?;
    if !path.is_absolute() {
        bail!("Cua Driver candidate {} is not absolute", path.display());
    }
    if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
        bail!("Cua Driver executable {} is a symlink", path.display());
    }
    let canonical = std::fs::canonicalize(&path)?;
    if canonical != path {
        bail!(
            "Cua Driver executable is not canonical: declared {}, canonical {}",
            path.display(),
            canonical.display()
        );
    }
    let sha256 = hash_file(&path)?;
    let mut version_command = Command::new(&path);
    version_command.arg("--version");
    let output = super::crawl::bounded_command_output(
        &mut version_command,
        "read pinned Cua Driver version",
        Duration::from_secs(15),
        64 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "pinned Cua Driver {} refused --version: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(CuaDriver {
        path,
        sha256,
        version: String::from_utf8_lossy(&output.stdout).trim().to_string(),
    })
}

/// One helper binary, pinned exactly once per record.
#[derive(Clone, Debug)]
pub(crate) struct PinnedHelper {
    pub(crate) path: PathBuf,
    pub(crate) sha256: String,
    pub(crate) version: String,
}

/// The readiness helper is resolved from absolute directories only, never from
/// the inherited PATH, and its digest and version are retained (finding 10).
pub(crate) fn pinned_readiness_helper() -> Result<PinnedHelper> {
    const PROGRAM: &str = "stado-runtime-readiness";
    let path = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"]
        .iter()
        .map(|directory| Path::new(directory).join(PROGRAM))
        .find(|candidate| candidate.is_file())
        .with_context(|| {
            format!("{PROGRAM} is absent from the pinned absolute helper directories")
        })?;
    if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
        bail!(
            "pinned desktop readiness helper {} is a symlink",
            path.display()
        );
    }
    let canonical = std::fs::canonicalize(&path)?;
    if canonical != path {
        bail!(
            "pinned desktop readiness helper is not canonical: declared {}, canonical {}",
            path.display(),
            canonical.display()
        );
    }
    let sha256 = hash_file(&path)?;
    let mut version_command = Command::new(&path);
    version_command
        .arg("--version")
        .env_clear()
        .env("PATH", "/usr/bin:/bin");
    let output = super::crawl::bounded_command_output(
        &mut version_command,
        "read pinned desktop readiness helper version",
        Duration::from_secs(15),
        64 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "pinned desktop readiness helper {} refused --version: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(PinnedHelper {
        path,
        sha256,
        version: String::from_utf8_lossy(&output.stdout).trim().to_string(),
    })
}

pub(crate) fn call(driver: &CuaDriver, tool: &str, payload: &Value) -> Result<Value> {
    call_with_cli_options(driver, tool, payload, &[], Duration::from_secs(30))
}

/// Cleanup deadline. A guard must never block the worker (finding 4).
pub(crate) fn call_briefly(driver: &CuaDriver, tool: &str, payload: &Value) -> Result<Value> {
    call_with_cli_options(driver, tool, payload, &[], Duration::from_secs(5))
}

pub(crate) fn call_with_cli_options(
    driver: &CuaDriver,
    tool: &str,
    payload: &Value,
    options: &[&std::ffi::OsStr],
    timeout: Duration,
) -> Result<Value> {
    let mut command = Command::new(&driver.path);
    command.arg(tool).arg(serde_json::to_string(payload)?);
    command.args(options);
    let output = super::crawl::bounded_command_output(
        &mut command,
        &format!("cua-driver {tool}"),
        timeout,
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "cua-driver {tool} failed: status={}; stdout={:?}; stderr={:?}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("cua-driver {tool} returned no exact JSON document"))
}

pub(crate) struct SessionGuard {
    pub(crate) driver: CuaDriver,
    pub(crate) session: String,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        let _ = call_briefly(&self.driver, "end_session", &json!({"session": self.session}));
    }
}

/// The launched application is terminated with the record. Without this the app
/// outlives the crawl, and across a catalog every record leaves another live GUI
/// app contending for focus (finding 5).
pub(crate) struct AppGuard {
    pub(crate) driver: CuaDriver,
    pub(crate) session: String,
    pub(crate) pid: i64,
}

impl Drop for AppGuard {
    fn drop(&mut self) {
        let _ = call_briefly(
            &self.driver,
            "terminate_app",
            &json!({"session": self.session, "pid": self.pid}),
        );
    }
}

pub(crate) struct RecordingGuard {
    pub(crate) driver: CuaDriver,
    pub(crate) session: String,
    pub(crate) active: bool,
}

impl RecordingGuard {
    pub(crate) fn stop(&mut self) -> Option<Result<Value>> {
        if !self.active {
            return None;
        }
        self.active = false;
        Some(call(
            &self.driver,
            "stop_recording",
            &json!({"session": self.session}),
        ))
    }
}

impl Drop for RecordingGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = call_briefly(
                &self.driver,
                "stop_recording",
                &json!({"session": self.session}),
            );
        }
    }
}

/// Typed record failure so the worker report can name a stable machine code
/// instead of a free-text diagnostic.
#[derive(Debug)]
pub(crate) struct RecordFailure {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

impl std::fmt::Display for RecordFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for RecordFailure {}
