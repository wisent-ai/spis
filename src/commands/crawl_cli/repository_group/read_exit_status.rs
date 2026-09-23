use super::*;

/// The exit status is read out of band. A missing or unparsable file is a typed
/// failure; it is never silently reported as `Some(0)` (finding 7).
pub(crate) fn read_exit_status(fixture: &Path, exit_file: &Path) -> Result<i32> {
    let metadata = std::fs::symlink_metadata(exit_file).with_context(|| {
        format!(
            "CLI invocation wrote no out-of-band exit status at {}",
            exit_file.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() || metadata.len() > 16 {
        bail!(
            "CLI out-of-band exit status {} is not a small regular file",
            exit_file.display()
        );
    }
    let parent = exit_file
        .parent()
        .context("CLI out-of-band exit status has no parent directory")?;
    if std::fs::canonicalize(parent)? != std::fs::canonicalize(fixture)? {
        bail!(
            "CLI out-of-band exit status {} is outside this record's fixture directory",
            exit_file.display()
        );
    }
    std::fs::read_to_string(exit_file)?
        .trim()
        .parse::<i32>()
        .with_context(|| {
            format!(
                "CLI out-of-band exit status {} is not an integer",
                exit_file.display()
            )
        })
}

pub(crate) fn run_in_pty(
    session: &TmuxSession,
    fixture: &Path,
    binary: &Path,
    argv: &[String],
    output: &Path,
    index: usize,
    record_key: &str,
) -> Result<Invocation> {
    let nonce = invocation_nonce(record_key, index)?;
    let start_marker = format!("__SPIS_START_{nonce}__");
    let end_marker = format!("__SPIS_END_{nonce}__");
    let exit_file = fixture.join(format!("exit-{nonce}"));
    if std::fs::symlink_metadata(&exit_file).is_ok() {
        bail!(
            "CLI out-of-band exit path {} already exists",
            exit_file.display()
        );
    }
    let mut invocation = shell_quote(binary.to_string_lossy().as_ref());
    for argument in argv {
        invocation.push(' ');
        invocation.push_str(&shell_quote(argument));
    }
    // Both markers are assembled from two shell words, so the shell's own echo
    // of this command line cannot satisfy the poll before the program has run.
    let command = format!(
        "printf '\\n%s%s\\n' '__SPIS_START_' '{nonce}__'; {invocation}; printf '%s' \"$?\" > {}; printf '\\n%s%s\\n' '__SPIS_END_' '{nonce}__'",
        shell_quote(exit_file.to_string_lossy().as_ref())
    );
    tmux(
        &session.socket,
        &session.environment,
        &["send-keys", "-t", &session.name, "-l", &command],
        "type CLI invocation",
    )?;
    tmux(
        &session.socket,
        &session.environment,
        &["send-keys", "-t", &session.name, "Enter"],
        "submit CLI invocation",
    )?;
    let mut timed_out = false;
    if !await_marker(session, &end_marker, Duration::from_secs(30))? {
        let _ = tmux(
            &session.socket,
            &session.environment,
            &["send-keys", "-t", &session.name, "C-c"],
            "interrupt CLI timeout",
        );
        // A program that survives the interrupt would receive the next
        // invocation's keystrokes as stdin and its output would be digested
        // under the wrong argv, so the record is abandoned here instead of
        // continuing on the shared session (finding 6).
        if !await_marker(session, &end_marker, Duration::from_secs(5))? {
            let _ = tmux(
                &session.socket,
                &session.environment,
                &["kill-session", "-t", &session.name],
                "kill hung CLI PTY",
            );
            return Err(anyhow::Error::new(RecordFailure {
                code: "cli_invocation_timeout",
                message: format!(
                    "CLI invocation {argv:?} did not terminate after an interrupt; no further invocation was attempted on the shared session"
                ),
            }));
        }
        timed_out = true;
    }
    let screen = capture_history(session)?;
    let state_path = format!("states/state-{index:04}.ansi");
    std::fs::write(output.join(&state_path), &screen)?;
    let cleaned_screen = clean_terminal(&screen);
    // The timeout path has no exit status at all.
    let exit_status = if timed_out {
        None
    } else {
        Some(read_exit_status(fixture, &exit_file)?)
    };
    let after_start = cleaned_screen
        .rsplit_once(&start_marker)
        .map(|(_, tail)| tail)
        .unwrap_or(&cleaned_screen);
    let invocation_output = after_start
        .split_once(&end_marker)
        .map(|(body, _)| body)
        .unwrap_or(after_start)
        .trim()
        .to_string();
    Ok(Invocation {
        argv: argv.to_vec(),
        output: invocation_output,
        exit_status,
        timed_out,
        state_path,
    })
}

pub(crate) fn invocation_json(invocation: &Invocation, kind: &str) -> Value {
    let mut digest = Sha256::new();
    digest.update(invocation.output.as_bytes());
    let output_sha256 = hex::encode(digest.finalize());
    json!({
        "kind": kind,
        "delivered_input": {
            "argv": invocation.argv,
        },
        "observed_state": {
            "exit_status": invocation.exit_status,
            "timed_out": invocation.timed_out,
            "raw_terminal_state": invocation.state_path,
            "rendered_output": invocation.output,
            "rendered_output_sha256": output_sha256,
        },
        "argv": invocation.argv,
        "exit_status": invocation.exit_status,
        "timed_out": invocation.timed_out,
        "state": invocation.state_path,
        "output_sha256": output_sha256,
    })
}
