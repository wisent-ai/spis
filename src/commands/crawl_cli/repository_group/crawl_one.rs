use super::*;

pub(crate) fn crawl_one(
    record: &Record,
    manifest: &super::crawl::RuntimeManifest,
    output: &Path,
) -> Result<Value> {
    // Attempt-clean tree: `pipe-pane` appends, so without this a retried record
    // would blend the previous attempt's raw terminal bytes with this attempt's
    // overwritten state files (finding 11).
    match std::fs::symlink_metadata(output) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            bail!("CLI attempt output {} is a symlink", output.display())
        }
        Ok(_) => std::fs::remove_dir_all(output)
            .with_context(|| format!("clear CLI attempt output {}", output.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let fixture = output.join("fixture");
    std::fs::create_dir_all(output.join("states"))?;
    std::fs::create_dir_all(&fixture)?;
    std::fs::write(
        fixture.join("README.txt"),
        "Spis isolated CLI crawl fixture\n",
    )?;
    let home = fixture.join("home");
    let config = home.join(".config");
    let data = home.join(".local/share");
    let cache = home.join(".cache");
    for directory in [&home, &config, &data, &cache] {
        std::fs::create_dir_all(directory)?;
    }
    let environment = isolated_environment(manifest, &fixture)?;
    let binary = verify_exact_executable(manifest, &record.binary, &environment)?;
    let session_name = format!("spis-cli-{}-{}", std::process::id(), record.slug);
    let socket = fixture.join("tmux.sock");
    // exists() follows symlinks and is false for a dangling one, so a planted
    // link would survive and tmux would place its socket at the link target
    // (finding 18).
    if std::fs::symlink_metadata(&socket).is_ok() {
        std::fs::remove_file(&socket)
            .with_context(|| format!("remove stale private CLI tmux socket {}", socket.display()))?;
    }
    if std::fs::symlink_metadata(&socket).is_ok() {
        bail!(
            "private CLI tmux socket path {} reappeared before launch",
            socket.display()
        );
    }
    let raw = output.join("terminal.raw");
    if std::fs::symlink_metadata(&raw).is_ok() {
        bail!(
            "CLI raw terminal log {} exists before this attempt recorded anything",
            raw.display()
        );
    }
    // Bounded scrollback on this private server (finding 8).
    let configuration = fixture.join("tmux.conf");
    std::fs::write(&configuration, "set -g history-limit 2000\n")?;
    tmux(
        &socket,
        &environment,
        &[
            "-f",
            configuration.to_string_lossy().as_ref(),
            "new-session",
            "-d",
            "-s",
            &session_name,
            "-x",
            "120",
            "-y",
            "40",
            "-c",
            fixture.to_string_lossy().as_ref(),
            "--",
            "/bin/sh",
        ],
        "launch private CLI PTY",
    )?;
    let session = TmuxSession {
        name: session_name,
        socket,
        environment,
    };
    // `>` truncates, so terminal.raw holds this attempt only (finding 11).
    let pipe = format!("cat > {}", shell_quote(raw.to_string_lossy().as_ref()));
    tmux(
        &session.socket,
        &session.environment,
        &["pipe-pane", "-t", &session.name, "-o", &pipe],
        "record CLI terminal bytes",
    )?;
    // A readiness marker replaces the fixed 200ms sleep: on a slow host the
    // first invocation's keystrokes would otherwise race shell startup and be
    // lost (finding 19).
    let ready_nonce = invocation_nonce(&manifest.record_key, 0)?;
    let ready_marker = format!("__SPIS_READY_{ready_nonce}__");
    tmux(
        &session.socket,
        &session.environment,
        &[
            "send-keys",
            "-t",
            &session.name,
            "-l",
            &format!("printf '\\n%s%s\\n' '__SPIS_READY_' '{ready_nonce}__'"),
        ],
        "type CLI shell readiness probe",
    )?;
    tmux(
        &session.socket,
        &session.environment,
        &["send-keys", "-t", &session.name, "Enter"],
        "submit CLI shell readiness probe",
    )?;
    if !await_marker(&session, &ready_marker, Duration::from_secs(15))? {
        bail!("the private CLI shell never acknowledged its readiness probe");
    }

    let mut reports = Vec::new();
    let mut index = 1usize;
    let version = run_in_pty(
        &session,
        &fixture,
        &binary,
        &["--version".to_string()],
        output,
        index,
        &manifest.record_key,
    )?;
    index += 1;
    reports.push(invocation_json(&version, "version"));

    let help = run_in_pty(
        &session,
        &fixture,
        &binary,
        &["--help".to_string()],
        output,
        index,
        &manifest.record_key,
    )?;
    index += 1;
    reports.push(invocation_json(&help, "help"));

    let refusal = run_in_pty(
        &session,
        &fixture,
        &binary,
        &["--spis-invalid-option".to_string()],
        output,
        index,
        &manifest.record_key,
    )?;
    index += 1;
    reports.push(invocation_json(&refusal, "refusal"));

    let recovery = run_in_pty(
        &session,
        &fixture,
        &binary,
        &["--help".to_string()],
        output,
        index,
        &manifest.record_key,
    )?;
    reports.push(invocation_json(&recovery, "recovery"));

    let _ = tmux(
        &session.socket,
        &session.environment,
        &["kill-session", "-t", &session.name],
        "close CLI PTY",
    );
    let variant_events: Vec<Value> = reports.iter().enumerate().filter_map(|(position, invocation)| {
        let kind = invocation.get("kind").and_then(Value::as_str)?;
        let event_kind = if invocation.get("timed_out").and_then(Value::as_bool) == Some(true) {
            "crawler_timeout"
        } else if kind == "refusal" && invocation.get("exit_status").and_then(Value::as_i64).is_some_and(|status| status != 0) {
            "parser_refusal"
        } else if kind == "recovery" && invocation.get("exit_status").and_then(Value::as_i64) == Some(0) {
            "recovery_observation"
        } else {
            return None;
        };
        Some(json!({
            "event_id": format!("invocation-{}", position + 1),
            "event_kind": event_kind,
            "argv": invocation.get("argv"),
            "exit_status": invocation.get("exit_status"),
            "timed_out": invocation.get("timed_out"),
            "state": invocation.get("state"),
            "output_sha256": invocation.get("output_sha256"),
            "linked_interaction_id": Value::Null,
        }))
    }).collect();
    let report = json!({
        "schema": "wisent.cli-crawl-run.v1",
        "slug": record.slug,
        "name": record.name,
        "binary": binary,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "runtime_execution_identity": manifest.execution_identity,
        "terminal": {"columns": 120, "rows": 40, "term": "xterm-256color"},
        "invocations": reports,
        "commands_crawled": 4,
        "evidence_observations": {
            "executed_invocations": reports,
            "variant_events": variant_events,
            "terminal_stream": raw.strip_prefix(output).unwrap_or(&raw),
            "canonical_interactions": [],
            "canonical_journey": Value::Null,
            "canonical_accessibility": Value::Null,
            "canonical_motion_analysis": Value::Null,
            "gaps": [
                "Observed failure/recovery/cancellation events are retained independently; no eight interactions have all required variants linked.",
                "No timed terminal cast or rendered state image was retained.",
                "Terminal keyboard accessibility equivalents and reduced-motion behavior remain unmeasured."
            ]
        },
    });
    std::fs::write(
        output.join("crawl.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}
