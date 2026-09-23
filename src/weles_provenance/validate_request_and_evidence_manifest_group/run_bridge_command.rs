use super::*;

/// Runs one bridge operation and returns its bounded stdout, which is empty when the
/// document was persisted to `output` instead.
///
/// The script is read, digest-pinned against the build-time embedded source digest, and
/// executed as verified bytes through a data URL, so no on-disk module is loaded by path.
/// The child runs in its own process group with a cleared environment: exactly `PATH`,
/// the canonical trust path, the verified trust bytes, the verified bridge directory and,
/// on the network path, the protected config path. The command document travels on stdin,
/// so no bridge command is ever left on disk.
pub fn run_bridge_command(invocation: &BridgeInvocation<'_>) -> Result<Vec<u8>, BridgeFailure> {
    let absent = |message: &str| BridgeFailure::new("absent", message);
    let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("weles-bridge")
        .join("spis-weles-bridge.mjs");
    let bridge_directory = fs::canonicalize(
        script_path
            .parent()
            .ok_or_else(|| absent("checked-in Weles bridge has no resource directory"))?,
    )
    .map_err(|_| absent("checked-in Weles bridge resource directory could not be resolved"))?;
    let script_metadata = fs::symlink_metadata(&script_path)
        .map_err(|_| absent("checked-in Weles bridge is absent"))?;
    if script_metadata.file_type().is_symlink() || !script_metadata.is_file() {
        return Err(absent("checked-in Weles bridge is not a regular non-symlink file"));
    }
    let script = fs::canonicalize(&script_path)
        .map_err(|_| absent("checked-in Weles bridge could not be resolved"))?;
    if script.parent() != Some(bridge_directory.as_path()) {
        return Err(absent(
            "checked-in Weles bridge escaped its canonical resource directory",
        ));
    }
    let script_file =
        fs::File::open(&script).map_err(|_| absent("checked-in Weles bridge could not be opened"))?;
    let script_bytes = read_stream_limited(
        script_file,
        MAX_BRIDGE_SCRIPT_BYTES as usize,
        "checked-in Weles bridge",
    )
    .map_err(|message| BridgeFailure::new("absent", message))?;
    if sha256_bytes(&script_bytes) != BRIDGE_SCRIPT_SHA256 {
        return Err(BridgeFailure::new(
            "unpinned",
            "checked-in Weles bridge differs from the embedded source pin",
        ));
    }
    let bridge_module = format!(
        "await import('data:text/javascript;base64,{}')",
        STANDARD.encode(&script_bytes)
    );
    let input_bytes = serde_json::to_vec(invocation.command)
        .map_err(|_| BridgeFailure::new("io-failed", "could not serialize the bridge command"))?;
    let mut command = Command::new("node");
    command
        .arg("--input-type=module")
        .arg("--eval")
        .arg(bridge_module)
        .arg("--")
        .arg("spis-weles-bridge.mjs")
        .arg("--input")
        .arg("-")
        .arg("--output");
    match invocation.output {
        Some(path) => command.arg(path),
        None => command.arg("-"),
    };
    command
        .current_dir(invocation.working_dir)
        .env_clear()
        .env("PATH", BRIDGE_PATH)
        .env("SPIS_WELES_TRUST_FILE", &invocation.trust.path)
        .env(
            "SPIS_WELES_VERIFIED_TRUST_BASE64",
            STANDARD.encode(&invocation.trust.bytes),
        )
        .env("SPIS_WELES_VERIFIED_BRIDGE_DIR", bridge_directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(config) = invocation.config {
        command.env("SPIS_WELES_CONFIG_FILE", config);
    }
    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    let mut child = command.spawn().map_err(|_| {
        BridgeFailure::new(
            "spawn-failed",
            "could not start Node for the checked-in Weles bridge",
        )
    })?;
    let started = std::time::Instant::now();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| BridgeFailure::new("io-failed", "Weles bridge stdout was unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| BridgeFailure::new("io-failed", "Weles bridge stderr was unavailable"))?;
    let stdout_reader = std::thread::spawn(move || {
        read_stream_limited(stdout, MAX_DOCUMENT_BYTES as usize, "stdout")
    });
    let stderr_reader = std::thread::spawn(move || {
        read_stream_limited(stderr, MAX_BRIDGE_ERROR_BYTES, "stderr")
    });
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| BridgeFailure::new("io-failed", "Weles bridge stdin was unavailable"))?;
    let stdin_writer = std::thread::spawn(move || {
        stdin
            .write_all(&input_bytes)
            .map_err(|_| "could not send the command document to the Weles bridge".to_string())
    });
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < invocation.timeout => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Ok(None) => {
                // The whole group: the official client may itself be waiting on a socket.
                terminate_bridge_process_group(&mut child);
                let _ = stdin_writer.join();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(BridgeFailure::new(
                    "timeout",
                    format!(
                        "official Weles bridge exceeded the {}-second deadline",
                        invocation.timeout.as_secs()
                    ),
                ));
            }
            Err(_) => {
                terminate_bridge_process_group(&mut child);
                let _ = stdin_writer.join();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(BridgeFailure::new(
                    "io-failed",
                    "could not collect the Weles bridge result",
                ));
            }
        }
    };
    let stdin_result = stdin_writer.join();
    let stdout_result = stdout_reader.join();
    let stderr_result = stderr_reader.join();
    if !status.success() {
        let stderr = match stderr_result {
            Ok(Ok(stderr)) => stderr,
            _ => Vec::new(),
        };
        let code = bridge_error_code(&stderr);
        return Err(BridgeFailure {
            message: format!("official Weles bridge failed closed ({code})"),
            code,
        });
    }
    let io_failed = |message: String| BridgeFailure::new("io-failed", message);
    stdin_result
        .map_err(|_| io_failed("official Weles bridge stdin writer failed".to_string()))?
        .map_err(io_failed)?;
    let stdout = stdout_result
        .map_err(|_| io_failed("official Weles bridge stdout reader failed".to_string()))?
        .map_err(io_failed)?;
    stderr_result
        .map_err(|_| io_failed("official Weles bridge stderr reader failed".to_string()))?
        .map_err(io_failed)?;
    Ok(stdout)
}

pub(crate) fn invoke_bridge(
    persisted: &WelesProvenanceDocument,
    record_dir: &Path,
    trust: &CanonicalTrust,
) -> Result<WelesProvenanceDocument, String> {
    let command = serde_json::json!({
        "schema": BRIDGE_COMMAND_SCHEMA,
        "operation": "verify",
        "receipt": persisted.receipt,
        "expectedClaims": persisted.expected_claims,
        "artifact": persisted.artifact,
    });
    let stdout = run_bridge_command(&BridgeInvocation {
        command: &command,
        trust,
        working_dir: record_dir,
        output: None,
        // Re-verification is secretless: it re-reads retained bytes and the public trust
        // document, and must never be able to reach the network.
        config: None,
        timeout: VERIFY_BRIDGE_TIMEOUT,
    })
    .map_err(|failure| failure.message)?;
    serde_json::from_slice(&stdout)
        .map_err(|_| "official Weles bridge returned a malformed verification document".to_string())
}

pub(crate) fn terminate_bridge_process_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    unsafe {
        if libc::killpg(child.id() as libc::pid_t, libc::SIGKILL) == -1 {
            let _ = child.kill();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}

pub(crate) fn validate_fresh_document(
    persisted: &WelesProvenanceDocument,
    fresh: &WelesProvenanceDocument,
    record_dir: &Path,
) -> Result<(), String> {
    validate_document_shape(fresh)?;
    if fresh != persisted {
        return Err(
            "fresh official verification differs from the persisted verification document"
                .to_string(),
        );
    }
    if !claims_match_expected(&fresh.claims, &fresh.expected_claims)
        || fresh.claims.key_id != fresh.receipt.key_id
    {
        return Err("fresh verified claims do not exactly match caller expectations".to_string());
    }
    let artifact_path = resolve_retained_file(record_dir, &fresh.artifact.path)?;
    let metadata = fs::metadata(&artifact_path)
        .map_err(|_| "retained artifact metadata is unavailable".to_string())?;
    if metadata.len() != fresh.artifact.bytes {
        return Err("retained artifact byte count changed".to_string());
    }
    let actual_artifact_digest = sha256_file(&artifact_path)?;
    if actual_artifact_digest != fresh.artifact.sha256
        || fresh.claims.evidence_digest != actual_artifact_digest
    {
        return Err("fresh receipt is not bound to the retained artifact bytes".to_string());
    }
    let expected_id = provenance_id(
        &fresh.receipt,
        &fresh.client.key_set_version,
        &fresh.artifact,
    )?;
    if fresh.id != expected_id || !is_sha256_id(&fresh.id) {
        return Err("verification document ID is not derived from verified receipt material".to_string());
    }
    Ok(())
}

pub(crate) fn claims_match_expected(
    claims: &VerifiedReceiptClaims,
    expected: &ExpectedReceiptClaims,
) -> bool {
    claims.task_id == expected.task_id
        && claims.organization_id == expected.organization_id
        && claims.origin == expected.origin
        && claims.action == expected.action
        && claims.outcome == expected.outcome
        && claims.evidence_digest == expected.evidence_digest
        && claims.request_digest == expected.request_digest
        && claims.result_digest == expected.result_digest
        && claims.spis_binding == expected.spis_binding
}
