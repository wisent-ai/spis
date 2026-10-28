use super::*;

/// The base decimal places count in: one place is one power of ten.
/// https://en.wikipedia.org/wiki/Decimal
const DECIMAL_BASE: f64 = 10.0;

pub(crate) fn round_n(x: f64, places: u32) -> f64 {
    let f = DECIMAL_BASE.powi(places as i32);
    (x * f).round() / f
}

pub(crate) fn ansi_re() -> &'static Regex {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-Z\\-_]")
            .expect("ansi regex")
    });
    &RE
}

pub(crate) fn sgr_re() -> &'static Regex {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").expect("sgr regex"));
    &RE
}

pub(crate) fn next_action_re() -> &'static Regex {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"(try\s+'[^']+'|try\s+"[^"]+"|\(run:\s*[^)]+\)|see\s+'[^']+'|usage:|USAGE:|Usage:|--help|<command>)"#,
        )
        .expect("next-action regex")
    });
    &RE
}

pub(crate) fn exit_status_re() -> &'static Regex {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"exit-status=(\d+)").expect("exit-status regex"));
    &RE
}

pub(crate) fn cursor_re() -> &'static Regex {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\x1b\[\d*;\d*H").expect("cursor regex"));
    &RE
}

pub(crate) fn strip_ansi(text: &str) -> String {
    ansi_re().replace_all(text, "").replace('\u{7}', "")
}

/// Replay plain text the way the recorded terminal showed it.
pub(crate) fn visible_lines(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in strip_ansi(text).split('\n') {
        // PTYs terminate ordinary lines with CRLF. Trailing CRs are dropped;
        // internal CR still means an in-place repaint, and a terminal shows the
        // final segment in that case.
        let line = line.trim_end_matches('\r');
        let line = match line.rsplit_once('\r') {
            Some((_, last)) => last,
            None => line,
        };
        out.push(line.replace('\t', "    "));
    }
    out
}

/// The text with its whitespace collapsed to single spaces, whole.
pub(crate) fn quote(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn json_str(s: &str) -> String {
    serde_json::to_string(s).expect("string serialization")
}

pub(crate) fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs()
}

/// ISO-8601 UTC timestamp with the `+00:00` offset the former script produced.
pub(crate) fn captured_at_now() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S+00:00")
        .to_string()
}

pub(crate) fn today_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

// ---------------------------------------------------------------------- host

/// A system call's status: these calls report failure as a negative status
/// and leave the cause in errno.
fn os_status(status: libc::c_int, what: &str) -> std::result::Result<(), String> {
    if status.is_negative() {
        return Err(format!("{what}: {}", std::io::Error::last_os_error()));
    }
    Ok(())
}

/// One text value the kernel reports through sysctl, such as the macOS
/// product version; an unreadable or empty value is refused, never written
/// into a record as a blank host fact.
pub(crate) fn sysctl_text(name: &str) -> std::result::Result<String, String> {
    let key =
        std::ffi::CString::new(name).map_err(|error| format!("sysctl name {name:?}: {error}"))?;
    let mut buffer: Vec<u8> = Vec::new();
    let mut length = buffer.len();
    let what = format!("sysctl {name}");
    unsafe {
        os_status(
            libc::sysctlbyname(
                key.as_ptr(),
                std::ptr::null_mut(),
                &mut length,
                std::ptr::null_mut(),
                buffer.len(),
            ),
            &what,
        )?;
        buffer.resize(length, b'\0');
        os_status(
            libc::sysctlbyname(
                key.as_ptr(),
                buffer.as_mut_ptr().cast(),
                &mut length,
                std::ptr::null_mut(),
                Vec::<u8>::new().len(),
            ),
            &what,
        )?;
    }
    let text = std::ffi::CStr::from_bytes_until_nul(&buffer)
        .map_err(|error| format!("{what} returned no terminated text: {error}"))?
        .to_string_lossy()
        .trim()
        .to_string();
    if text.is_empty() {
        return Err(format!("{what} reported nothing"));
    }
    Ok(text)
}

pub(crate) fn uname_field(
    name: &str,
    field: fn(&libc::utsname) -> &[libc::c_char],
) -> std::result::Result<String, String> {
    unsafe {
        let mut uts: libc::utsname = std::mem::zeroed();
        os_status(
            libc::uname(&mut uts),
            &format!("uname cannot report the {name}"),
        )?;
        Ok(std::ffi::CStr::from_ptr(field(&uts).as_ptr())
            .to_string_lossy()
            .into_owned())
    }
}

pub(crate) struct HostFacts {
    pub(crate) host: Value,
    pub(crate) sentence: String,
}

/// The capture host as every record states it, read once; a host that cannot
/// be read refuses the capture rather than recording blank host facts.
pub(crate) fn host_facts() -> Result<&'static HostFacts> {
    static HOST: LazyLock<std::result::Result<HostFacts, String>> = LazyLock::new(|| {
        let os_version = sysctl_text("kern.osproductversion")?;
        let os_build = sysctl_text("kern.osversion")?;
        let arch = uname_field("machine", |u| &u.machine)?;
        let kernel = format!(
            "{} {}",
            uname_field("system name", |u| &u.sysname)?,
            uname_field("release", |u| &u.release)?
        );
        let sentence = format!("macOS {os_version} ({os_build}) {arch}");
        let host = json!({
            "os": "macOS",
            "os_version": os_version,
            "os_build": os_build,
            "arch": arch,
            "kernel": kernel,
            "shell": SHELL,
            "terminal": format!("pseudo-terminal, {}x{}, TERM=xterm-256color", terminal().columns, terminal().rows),
        });
        Ok(HostFacts { host, sentence })
    });
    HOST.as_ref()
        .map_err(|cause| anyhow::anyhow!("cannot read the capture host: {cause}"))
}

pub(crate) fn host_sentence() -> Result<&'static str> {
    Ok(&host_facts()?.sentence)
}

// ---------------------------------------------------------------- resolution

pub(crate) fn resolve(product: &Product) -> Option<String> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.join(&product.binary);
        if is_executable(&candidate) {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

/// A regular file someone may execute: its owner, group or others execute
/// bit is set. A path that does not exist is not one; a path that cannot be
/// read is not taken as one either, and standard error says why.
pub(crate) fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(md) => {
            let mode = md.permissions().mode();
            md.is_file()
                && [libc::S_IXUSR, libc::S_IXGRP, libc::S_IXOTH]
                    .into_iter()
                    .map(u32::from)
                    .any(|bit| mode & bit == bit)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            eprintln!(
                "spis: cannot read {}: {error}; it is not taken as the program",
                path.display()
            );
            false
        }
    }
}

pub(crate) enum QuickOutcome {
    Ran(std::process::ExitStatus, String), // how it ended, first line
    Failed(String),                        // why the version command could not be started
    Missing,
}

pub(crate) fn quick_version(product: &Product) -> (Option<String>, QuickOutcome) {
    let Some(path) = resolve(product) else {
        return (None, QuickOutcome::Missing);
    };
    let args: Vec<&str> = product.version_cmd.split_whitespace().collect();
    match run_to_exit(&args) {
        QuickRun::SpawnError(cause) => (Some(path), QuickOutcome::Failed(cause)),
        QuickRun::Done(status, out, err) => {
            let text = strip_ansi(&(out + &err)).trim().to_string();
            let first = match text.split_once('\n') {
                Some((first, _)) => first.to_string(),
                None => text,
            };
            (Some(path), QuickOutcome::Ran(status, first))
        }
    }
}

pub(crate) enum QuickRun {
    Done(std::process::ExitStatus, String, String),
    SpawnError(String),
}

/// Runs `argv` to its own exit and captures both streams. There is no
/// deadline: the program's exit or its own error is the answer (cli.md rule 8).
/// The wait is announced, as every wait of this product is.
pub(crate) fn run_to_exit(argv: &[&str]) -> QuickRun {
    let Some((program, args)) = argv.split_first() else {
        return QuickRun::SpawnError("an empty command names no program to start".into());
    };
    match stado_wait::output(
        Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    ) {
        Ok(output) => QuickRun::Done(
            output.status,
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ),
        Err(error) => {
            QuickRun::SpawnError(format!("`{}` could not start: {error}", argv.join(" ")))
        }
    }
}
