use super::*;

pub(crate) fn host_probe(host: &str, arguments: &[&str]) -> Value {
    let mut command = stado_command();
    command
        .args(["host", "exec", host, "--json", "--"])
        .args(arguments);
    let result = (|| -> Result<Value> {
        let output = bounded_command_output(
            &mut command,
            "Stado host probe",
            HOST_PROBE_TIMEOUT,
            1024 * 1024,
        )?;
        // A non-zero exit is an answer, not a missing one: `stado host exec`
        // prints its typed receipt on stdout either way, and that receipt is
        // where the host records which executable it resolved for the command.
        // Bailing on the exit status first threw that away and left the caller
        // with an opaque "probe failed" - on `lukasz-macbook` it discarded the
        // one receipt that names the host's own Stado binary, because
        // `stado registry doctor` exits 1 while reporting 24 registry
        // divergences. Readiness below is still exactly `status == ok` and
        // `exit_code == 0`; only the evidence survives the failure now.
        let receipt: Value = serde_json::from_slice(&output.stdout).with_context(|| {
            format!(
                "host probe receipt is not JSON (exit {}): {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr).trim()
            )
        })?;
        let object = receipt
            .as_object()
            .context("host probe receipt must be an object")?;
        let allowed = [
            "schema",
            "target",
            "ssh",
            "ssh_fallbacks",
            "command",
            "argv",
            "stdout",
            "stderr",
            "exit_code",
            "status",
            "program_candidates",
            // Stado reports which of an entry's candidate paths the host
            // actually execed. It is retained evidence, not a surprise: a
            // strict allowlist means the field can only ever name a path the
            // entry itself declares.
            "resolved_executable",
            "error",
        ];
        let receipt_target = receipt
            .get("target")
            .and_then(Value::as_str)
            .context("host probe receipt has no target identity")?;
        if receipt_target != host {
            bail!(
                "host probe receipt target {receipt_target:?} does not match placement host {host:?}"
            );
        }
        if object.keys().any(|key| !allowed.contains(&key.as_str()))
            || receipt.get("schema").and_then(Value::as_str)
                != Some("stado.host-exec-receipt.v1")
            || !receipt.get("target").is_some_and(Value::is_string)
            || !receipt.get("ssh").is_some_and(|value| value.is_null() || value.is_string())
            || !receipt.get("ssh_fallbacks").is_some_and(Value::is_array)
            || !receipt.get("command").is_some_and(Value::is_string)
            || !receipt.get("stdout").is_some_and(Value::is_string)
            || !receipt.get("stderr").is_some_and(Value::is_string)
            || !receipt.get("exit_code").is_some_and(Value::is_i64)
        {
            bail!("host probe receipt does not match the exact typed Stado contract");
        }
        // `argv` is the entry's approved absolute spelling followed by its
        // fixed arguments. `resolved_executable` separately records the
        // candidate the host selected. The receipt must still answer exactly
        // the requested command and must not add, remove, or rewrite arguments.
        if receipt.get("command").and_then(Value::as_str) != Some(arguments.join(" ").as_str()) {
            bail!("host probe receipt answers a different command than the one requested");
        }
        let argv = receipt
            .get("argv")
            .and_then(Value::as_array)
            .context("host probe receipt has no argv")?;
        let (program, rest) = argv
            .split_first()
            .context("host probe receipt argv is empty")?;
        let program = program
            .as_str()
            .context("host probe receipt argv program is not a string")?;
        let requested_program = arguments
            .first()
            .context("host probe requires at least one word")?;
        let program_matches = program == *requested_program
            || program
                .rsplit('/')
                .next()
                .is_some_and(|name| name == *requested_program);
        let arguments_match = rest.len() == arguments.len() - 1
            && rest
                .iter()
                .zip(arguments.iter().skip(1))
                .all(|(observed, expected)| observed.as_str() == Some(*expected));
        if !program_matches || !arguments_match {
            bail!("host probe receipt argv differs from the approved exact command");
        }
        Ok(receipt)
    })();
    match result {
        Ok(receipt) => json!({
            "command": arguments,
            "outcome": if receipt.get("status").and_then(Value::as_str) == Some("ok")
                && receipt.get("exit_code").and_then(Value::as_i64) == Some(0)
            {
                "ready"
            } else {
                "unavailable"
            },
            "ready": receipt.get("status").and_then(Value::as_str) == Some("ok")
                && receipt.get("exit_code").and_then(Value::as_i64) == Some(0),
            "stdout": receipt.get("stdout").and_then(Value::as_str).unwrap_or_default(),
            "stderr": receipt.get("stderr").and_then(Value::as_str).unwrap_or_default(),
            "stado_receipt": receipt,
        }),
        Err(error) => {
            if let Some(timeout) = error.downcast_ref::<CommandTimedOut>() {
                return host_probe_timeout_report(arguments, timeout.timeout.as_secs());
            }
            json!({
                "command": arguments,
                "outcome": "failed",
                "ready": false,
                "diagnostic": {
                    "code": "host_probe_failed",
                    "retryable": false,
                    "message": error.to_string(),
                },
                "error": error.to_string(),
            })
        }
    }
}

/// Typed evidence that a host probe gave no answer before its hard deadline.
#[doc(hidden)]
pub fn host_probe_timeout_report(arguments: &[&str], timeout_seconds: u64) -> Value {
    let command = arguments.join(" ");
    let message =
        format!("host did not answer probe `{command}` within {timeout_seconds} seconds");
    json!({
        "command": arguments,
        "outcome": "timed_out",
        "ready": false,
        "diagnostic": {
            "code": "host_probe_timed_out",
            "retryable": true,
            "message": message,
            "probe": arguments,
            "timeout_seconds": timeout_seconds,
        },
        "error": message,
    })
}

pub(crate) fn host_preflight_is_retryable(report: &Value) -> bool {
    report
        .get("checks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|check| {
            check.pointer("/diagnostic/retryable").and_then(Value::as_bool) == Some(true)
        })
}

/// Record state selected when a host preflight has not passed.
#[doc(hidden)]
pub fn failed_host_preflight_record_state(report: &Value) -> &'static str {
    if host_preflight_is_retryable(report) {
        "planned"
    } else {
        "unavailable"
    }
}

/// Turn the program candidate selected by the placement host into one shell word.
///
/// Stado keeps the coordinator-approved command spelling in `argv` and reports
/// the candidate the placement host actually selected in `resolved_executable`.
/// Reading `argv[0]` put the coordinator's `/opt/homebrew/bin/cargo` into a
/// Charless job even though that host selected `~/.cargo/bin/cargo`; the worker
/// then failed before crawling a page because the baked-in path did not exist.
/// A home-relative selected candidate stays home-relative until the submitted
/// shell runs on that host: the coordinator must never expand another machine's
/// `~`.
pub fn executable_word_from_host_receipt(host: &str, receipt: &Value) -> Result<String> {
    let receipt_target = receipt
        .get("target")
        .and_then(Value::as_str)
        .context("host probe receipt has no target identity")?;
    if receipt_target != host {
        bail!(
            "host probe receipt target {receipt_target:?} does not match placement host {host:?}"
        );
    }
    // `resolved_executable` is what a probe of a bare program word reports:
    // Stado searched the host's own PATH and says which file it picked. An
    // allowlisted command that Stado itself rewrites - `stado registry doctor`
    // becomes `~/.stado/bin/stado registry doctor` - reports the same fact in
    // the receipt's own `argv[0]` and carries no separate field, so both are
    // read here. The exit code is irrelevant to either: which file the host
    // resolved is not a claim about what that file then did.
    let selected = receipt
        .get("resolved_executable")
        .and_then(Value::as_str)
        .or_else(|| {
            receipt
                .get("argv")
                .and_then(Value::as_array)
                .and_then(|argv| argv.first())
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .with_context(|| format!("host {host} probe reported no selected executable"))?;
    if selected.contains('\0') || selected.chars().any(|character| character.is_whitespace()) {
        bail!("host {host} probe reported unsafe selected executable {selected:?}");
    }
    if let Some(rest) = selected.strip_prefix("~/") {
        if rest.is_empty() {
            bail!("host {host} probe reported incomplete selected executable {selected:?}");
        }
        return Ok(format!(
            "\"$HOME\"/'{}'",
            rest.replace('\'', "'\\''")
        ));
    }
    if selected.starts_with('/') {
        return Ok(selected.to_string());
    }
    bail!("host {host} probe reported non-absolute selected executable {selected:?}")
}

/// The shell word for the program that the placement host actually selected.
pub(crate) fn resolved_program(host: &str, arguments: &[&str]) -> Result<String> {
    let check = host_probe(host, arguments);
    if check.get("ready").and_then(Value::as_bool) != Some(true) {
        bail!(
            "host {host} cannot run `{}`: {}",
            arguments.join(" "),
            check
                .get("error")
                .and_then(Value::as_str)
                .or_else(|| check.get("stderr").and_then(Value::as_str))
                .unwrap_or("probe was not ready")
                .trim()
        );
    }
    let receipt = check
        .get("stado_receipt")
        .context("ready host probe retained no Stado receipt")?;
    executable_word_from_host_receipt(host, receipt)
}
