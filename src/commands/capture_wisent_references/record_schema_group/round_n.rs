use super::*;

pub(crate) fn round_n(x: f64, places: u32) -> f64 {
    let f = 10f64.powi(places as i32);
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
        // PTYs terminate ordinary lines with CRLF. A trailing CR is dropped;
        // internal CR still means an in-place repaint, and a terminal shows the
        // final segment in that case.
        let line = line.strip_suffix('\r').unwrap_or(line);
        let line = match line.rfind('\r') {
            Some(i) => &line[i + 1..],
            None => line,
        };
        out.push(line.replace('\t', "    "));
    }
    out
}

/// Collapse whitespace and truncate with an ellipsis, like the Python `quote`.
pub(crate) fn quote(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = collapsed.chars().collect();
    if chars.len() > limit {
        let head: String = chars[..limit - 1].iter().collect();
        format!("{head}\u{2026}")
    } else {
        collapsed
    }
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
    format!("{}+00:00", iso_utc_body(now_unix_secs()))
}

pub(crate) fn iso_utc_body(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}")
}

pub(crate) fn today_utc() -> String {
    iso_utc_body(now_unix_secs())[..10].to_string()
}

// ---------------------------------------------------------------------- host

pub(crate) fn sw_vers(arg: &str) -> String {
    Command::new("sw_vers")
        .arg(arg)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

pub(crate) fn uname_field(field: fn(&libc::utsname) -> &[libc::c_char]) -> String {
    unsafe {
        let mut uts: libc::utsname = std::mem::zeroed();
        if libc::uname(&mut uts) != 0 {
            return String::new();
        }
        std::ffi::CStr::from_ptr(field(&uts).as_ptr())
            .to_string_lossy()
            .into_owned()
    }
}

pub(crate) struct HostFacts {
    pub(crate) host: Value,
    pub(crate) sentence: String,
}

pub(crate) fn host_facts() -> &'static HostFacts {
    static HOST: LazyLock<HostFacts> = LazyLock::new(|| {
        let host = json!({
            "os": "macOS",
            "os_version": sw_vers("-productVersion"),
            "os_build": sw_vers("-buildVersion"),
            "arch": uname_field(|u| &u.machine),
            "kernel": format!(
                "{} {}",
                uname_field(|u| &u.sysname),
                uname_field(|u| &u.release)
            ),
            "shell": SHELL,
            "terminal": format!("pseudo-terminal, {COLS}x{ROWS}, TERM=xterm-256color"),
        });
        let sentence = format!(
            "macOS {} ({}) {}",
            host["os_version"].as_str().unwrap_or_default(),
            host["os_build"].as_str().unwrap_or_default(),
            host["arch"].as_str().unwrap_or_default(),
        );
        HostFacts { host, sentence }
    });
    &HOST
}

pub(crate) fn host_sentence() -> &'static str {
    &host_facts().sentence
}

// ---------------------------------------------------------------- resolution

pub(crate) fn resolve(product: &Product) -> Option<String> {
    let path_var = std::env::var("PATH").ok()?;
    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }
        let candidate = Path::new(dir).join(product.binary);
        if is_executable(&candidate) {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

pub(crate) fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(md) => md.is_file() && md.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

pub(crate) enum QuickOutcome {
    Ran(i32, String), // exit status, first line
    Failed(String),   // exception type name, like the Python returned
    Missing,
}

pub(crate) fn quick_version(product: &Product) -> (Option<String>, QuickOutcome) {
    let Some(path) = resolve(product) else {
        return (None, QuickOutcome::Missing);
    };
    let args: Vec<&str> = product.version_cmd.split_whitespace().collect();
    match run_with_timeout(&args, Duration::from_secs(120)) {
        QuickRun::SpawnError => (Some(path), QuickOutcome::Failed("OSError".into())),
        QuickRun::TimedOut => (Some(path), QuickOutcome::Failed("TimeoutExpired".into())),
        QuickRun::Done(status, out, err) => {
            let text = strip_ansi(&(out + &err)).trim().to_string();
            let first = text.split('\n').next().unwrap_or("").to_string();
            (Some(path), QuickOutcome::Ran(status, first))
        }
    }
}

pub(crate) enum QuickRun {
    Done(i32, String, String),
    TimedOut,
    SpawnError,
}

pub(crate) fn run_with_timeout(argv: &[&str], limit: Duration) -> QuickRun {
    let mut child = match Command::new(argv[0])
        .args(&argv[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return QuickRun::SpawnError,
    };
    let mut stdout_pipe = child.stdout.take().expect("stdout piped");
    let mut stderr_pipe = child.stderr.take().expect("stderr piped");
    let t_out = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        buf
    });
    let t_err = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        buf
    });
    let deadline = Instant::now() + limit;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let out = String::from_utf8_lossy(&t_out.join().unwrap_or_default()).into_owned();
                let err = String::from_utf8_lossy(&t_err.join().unwrap_or_default()).into_owned();
                return QuickRun::Done(status.code().unwrap_or(-1), out, err);
            }
            Ok(None) => {}
            Err(_) => return QuickRun::SpawnError,
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return QuickRun::TimedOut;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
