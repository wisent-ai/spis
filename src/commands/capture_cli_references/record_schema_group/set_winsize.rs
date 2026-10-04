use super::*;

// --------------------------------------------------------------- pty session

pub(crate) fn set_winsize(fd: i32, rows: u32, cols: u32) {
    let ws = libc::winsize {
        ws_row: rows as u16,
        ws_col: cols as u16,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        libc::ioctl(fd, libc::TIOCSWINSZ, &ws);
    }
}

/// One real pseudo-terminal running one interactive shell, recorded as it runs.
pub(crate) struct Session {
    pub(crate) fd: i32,
    pub(crate) pid: i32,
    pub(crate) events: Vec<(f64, String)>,
    pub(crate) start: Instant,
}

impl Session {
    pub(crate) fn new(cwd: &Path) -> Result<Session> {
        let mut env_pairs: Vec<(String, String)> = std::env::vars()
            .filter(|(k, _)| {
                k != "NO_COLOR"
                    && k != "CLICOLOR_FORCE"
                    && k != "TERM"
                    && k != "PS1"
                    && k != "PAGER"
                    && k != "LESS"
                    && k != "COLUMNS"
                    && k != "LINES"
                    && k != "SHELL"
            })
            .collect();
        for (k, v) in [
            ("TERM", "xterm-256color"),
            ("PS1", PROMPT),
            ("PAGER", "cat"),
            ("LESS", "-FRX"),
            ("COLUMNS", &terminal().columns.to_string()),
            ("LINES", &terminal().rows.to_string()),
            ("SHELL", SHELL),
        ] {
            env_pairs.retain(|(ek, _)| ek != k);
            env_pairs.push((k.to_string(), v.to_string()));
        }

        let cwd_c = std::ffi::CString::new(cwd.as_os_str().as_encoded_bytes())
            .context("scratch cwd path")?;
        let shell_c = std::ffi::CString::new(SHELL).unwrap();
        let argv: Vec<std::ffi::CString> = ["--norc", "--noprofile", "-i"]
            .iter()
            .map(|a| std::ffi::CString::new(*a).unwrap())
            .collect();
        let env_c: Vec<std::ffi::CString> = env_pairs
            .iter()
            .map(|(k, v)| std::ffi::CString::new(format!("{k}={v}")).map_err(|e| anyhow!("{e}")))
            .collect::<Result<_>>()?;

        let mut master: libc::c_int = -1;
        let pid = unsafe {
            libc::forkpty(
                &mut master,
                std::ptr::null_mut::<libc::c_char>(),
                std::ptr::null_mut::<libc::termios>(),
                std::ptr::null_mut::<libc::winsize>(),
            )
        };
        if pid < 0 {
            bail!("forkpty failed: {}", std::io::Error::last_os_error());
        }
        if pid == 0 {
            // Child: become the recorded shell.
            unsafe {
                if libc::chdir(cwd_c.as_ptr()) != 0 {
                    libc::_exit(127);
                }
                let mut argp: Vec<*const libc::c_char> = vec![shell_c.as_ptr()];
                argp.extend(argv.iter().map(|a| a.as_ptr()));
                argp.push(std::ptr::null());
                let mut envp: Vec<*const libc::c_char> = env_c.iter().map(|e| e.as_ptr()).collect();
                envp.push(std::ptr::null());
                libc::execve(shell_c.as_ptr(), argp.as_ptr(), envp.as_ptr());
                libc::_exit(127);
            }
        }
        set_winsize(master, terminal().rows as u32, terminal().columns as u32);
        let mut session = Session {
            fd: master,
            pid,
            events: Vec::new(),
            start: Instant::now(),
        };
        session.write_text(&format!("PS1='{PROMPT}'\n"));
        let (_, ready) = session.read_until_prompt();
        if !ready {
            bail!("the recording shell closed its terminal before printing the prompt");
        }
        // Everything recorded from here is the product's own session.
        session.events.clear();
        session.start = Instant::now();
        Ok(session)
    }

    pub(crate) fn elapsed(&self) -> f64 {
        round_n(self.start.elapsed().as_secs_f64(), 6)
    }

    pub(crate) fn write_text(&self, text: &str) {
        let bytes = text.as_bytes();
        let mut written = 0;
        while written < bytes.len() {
            let n = unsafe {
                libc::write(
                    self.fd,
                    bytes[written..].as_ptr() as *const _,
                    bytes.len() - written,
                )
            };
            if n <= 0 {
                break;
            }
            written += n as usize;
        }
    }

    /// Blocks until the terminal has output, then returns it. `None` means the
    /// shell closed the terminal; there is no deadline (cli.md rule 8).
    pub(crate) fn read_chunk(&mut self) -> Option<String> {
        let mut pfd = libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN,
            revents: 0,
        };
        if unsafe { libc::poll(&mut pfd, 1, -1) } <= 0 {
            return None;
        }
        let mut buf = vec![0u8; 1 << 16];
        let n = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        if n <= 0 {
            return None;
        }
        let text = String::from_utf8_lossy(&buf[..n as usize]).into_owned();
        self.events.push((self.elapsed(), text.clone()));
        Some(text)
    }

    /// Read until the shell reprints its prompt (`true`) or closes the
    /// terminal (`false`).
    pub(crate) fn read_until_prompt(&mut self) -> (String, bool) {
        let mut buf = String::new();
        while let Some(chunk) = self.read_chunk() {
            buf.push_str(&chunk);
            if buf.ends_with(PROMPT) {
                return (buf, true);
            }
        }
        (buf, false)
    }

    pub(crate) fn command(&mut self, command: &str) -> Step {
        let started_at = self.elapsed();
        self.write_text(&format!("{command}\n"));
        let (raw, ok) = self.read_until_prompt();
        let ended_at = self.elapsed();
        let exit_status = if ok {
            self.write_text("printf \"exit-status=%s\\n\" \"$?\"\n");
            let (status_raw, _) = self.read_until_prompt();
            exit_status_re()
                .captures(&status_raw)
                .and_then(|c| c[1].parse::<i64>().ok())
        } else {
            None
        };
        let status_reported_at = self.elapsed();
        Step {
            command: command.to_string(),
            raw,
            started_at,
            ended_at,
            status_reported_at,
            exit_status,
            prompt_returned: ok,
            event_index: 0,
            kind: String::new(),
        }
    }

    /// Type a command and press Ctrl-C instead of Enter. Nothing is submitted.
    pub(crate) fn cancel(&mut self, pending: &str) -> Step {
        let started_at = self.elapsed();
        self.write_text(pending);
        self.write_text("\u{3}");
        let (raw, ok) = self.read_until_prompt();
        let ended_at = self.elapsed();
        Step {
            command: pending.to_string(),
            raw,
            started_at,
            ended_at,
            status_reported_at: ended_at,
            exit_status: None,
            prompt_returned: ok,
            event_index: 0,
            kind: String::new(),
        }
    }

    pub(crate) fn close(&mut self) {
        self.write_text("exit\n");
        while self.read_chunk().is_some() {}
        unsafe {
            libc::close(self.fd);
            libc::waitpid(self.pid, std::ptr::null_mut(), 0);
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Clone)]
pub(crate) struct Step {
    pub(crate) command: String,
    pub(crate) raw: String,
    pub(crate) started_at: f64,
    pub(crate) ended_at: f64,
    pub(crate) status_reported_at: f64,
    pub(crate) exit_status: Option<i64>,
    #[allow(dead_code)]
    pub(crate) prompt_returned: bool,
    pub(crate) event_index: usize,
    pub(crate) kind: String,
}

pub(crate) struct Run {
    pub(crate) index: usize,
    pub(crate) product: &'static Product,
    pub(crate) binary_path: String,
    pub(crate) workdir: String,
    pub(crate) events: Vec<(f64, String)>,
    pub(crate) wall_start: u64,
    pub(crate) steps: BTreeMap<String, Step>,
}

pub(crate) const STEP_PLAN: &[(&str, &str)] = &[
    ("version", "version identity"),
    ("help", "top-level help"),
    ("subcommand-help", "subcommand help surface"),
    ("invalid-flag", "invalid flag refusal"),
    ("cancellation", "Ctrl-C on an unsubmitted line"),
    ("recovery-help", "recovery help"),
    ("no-color-help", "help with NO_COLOR=1"),
];
