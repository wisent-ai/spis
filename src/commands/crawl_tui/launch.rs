use super::*;

pub(crate) fn launch(
    record_slug: &str,
    binary: &Path,
    fixture: &Path,
    raw: &Path,
    attempt: usize,
    environment: BTreeMap<OsString, OsString>,
) -> Result<TmuxSession> {
    let name = format!("spis-tui-{}-{attempt}-{record_slug}", std::process::id());
    let socket = fixture.join("tmux.sock");
    // exists() follows symlinks and is false for a dangling one, so a planted
    // link would survive and tmux would place its socket at the link target
    // (finding 18).
    if std::fs::symlink_metadata(&socket).is_ok() {
        std::fs::remove_file(&socket)
            .with_context(|| format!("remove stale private TUI tmux socket {}", socket.display()))?;
    }
    if std::fs::symlink_metadata(&socket).is_ok() {
        bail!(
            "private TUI tmux socket path {} reappeared before launch",
            socket.display()
        );
    }
    if std::fs::symlink_metadata(raw).is_ok() {
        bail!(
            "TUI raw terminal log {} exists before this attempt recorded anything",
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
            &name,
            "-x",
            "120",
            "-y",
            "40",
            "-c",
            fixture.to_string_lossy().as_ref(),
            "--",
            binary.to_string_lossy().as_ref(),
        ],
        "launch TUI in private tmux PTY",
    )?;
    let session = TmuxSession {
        name,
        socket,
        environment,
    };
    // `>` truncates, so terminal.raw holds this attempt only (finding 11).
    let pipe = format!("cat > {}", shell_quote(raw.to_string_lossy().as_ref()));
    tmux(
        &session.socket,
        &session.environment,
        &["pipe-pane", "-t", &session.name, "-o", &pipe],
        "record TUI byte stream",
    )?;
    // Bounded readiness poll instead of trusting a fixed sleep: the exact
    // initial state is only captured once the program has actually drawn
    // something (finding 19). A dynamic TUI keeps repainting, so this waits for
    // first paint rather than for a stable screen.
    let floor = Duration::from_secs(1);
    let started = Instant::now();
    loop {
        let blank = capture_tail(&session)?.trim().is_empty();
        if !blank && started.elapsed() >= floor {
            break;
        }
        if started.elapsed() >= Duration::from_secs(20) {
            // The hung program is killed with the session by TmuxSession::drop;
            // nothing downstream may reuse this PTY (finding 6).
            return Err(anyhow::Error::new(RecordFailure {
                code: "tui_launch_not_ready",
                message: format!(
                    "the exact TUI executable drew nothing in the private PTY within 20s for record {record_slug}"
                ),
            }));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(session)
}

pub(crate) fn crawl_one(
    slug: &str,
    name: &str,
    manifest: &super::crawl::RuntimeManifest,
    output: &Path,
    git: &Path,
) -> Result<Value> {
    let configured_path = manifest
        .execution_identity
        .as_ref()
        .and_then(|identity| identity.executable_path.as_deref())
        .context("TUI runtime manifest has no exact executable path")?;
    let expected_filename = Path::new(configured_path)
        .file_name()
        .and_then(|value| value.to_str())
        .context("TUI executable path has no UTF-8 filename")?
        .to_string();
    // Attempt-clean tree: `pipe-pane` appends, so without this a retried record
    // would blend the previous attempt's raw terminal bytes with this attempt's
    // overwritten state files (finding 11).
    match std::fs::symlink_metadata(output) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            bail!("TUI attempt output {} is a symlink", output.display())
        }
        Ok(_) => std::fs::remove_dir_all(output)
            .with_context(|| format!("clear TUI attempt output {}", output.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let states = output.join("states");
    let trajectory_root = output.join("trajectories/trajectory-00001");
    let fixture = trajectory_root.join("fixture");
    let raw = trajectory_root.join("terminal.raw");
    std::fs::create_dir_all(&states)?;
    std::fs::create_dir_all(&fixture)?;
    prepare_fixture(&fixture, git)?;
    let environment = isolated_environment(manifest, &fixture)?;
    let binary = verify_exact_executable(manifest, &expected_filename, &environment)?;
    let session = launch(slug, &binary, &fixture, &raw, 1, environment)?;
    let screen = capture(&session)?;
    let digest = hash(&screen);
    let state_path = states.join("state-00001.ansi");
    std::fs::write(&state_path, &screen)?;
    let graph = vec![json!({
        "state": digest,
        "index": 1,
        "trajectory_depth": 0,
        "delivered_inputs": [],
        "observed_state": {
            "changed_from_unobserved_initial_state": Value::Null,
            "exact_terminal_state": state_path.strip_prefix(output).unwrap_or(&state_path),
            "raw_terminal_stream": raw.strip_prefix(output).unwrap_or(&raw),
            "isolated_fixture": fixture.strip_prefix(output).unwrap_or(&fixture),
        },
    })];
    let blocked = REPRESENTATIVE_INPUT_CLASSES
        .iter()
        .map(|(class, reason)| {
            json!({
                "state": digest,
                "delivered_input": Value::Null,
                "observed_state_change": Value::Null,
                "representative_class": class,
                "reason": reason,
                "representative_not_exhaustive": true,
            })
        })
        .collect::<Vec<_>>();
    let representative_classes = REPRESENTATIVE_INPUT_CLASSES
        .iter()
        .map(|(class, reason)| json!({"class": class, "reason": reason}))
        .collect::<Vec<_>>();
    let report = json!({
        "schema": "wisent.tui-crawl-run.v1",
        "slug": slug,
        "name": name,
        "binary": configured_path,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "runtime_execution_identity": manifest.execution_identity,
        "terminal": {"columns": 120, "rows": 40, "term": "xterm-256color"},
        "states": graph,
        "states_seen": 1,
        "blocked_paths": blocked,
        "input_policy": {
            "default_decision": "withhold",
            "scope": "all keyboard, mouse, pointer, paste, text, and other input",
            "authorized_exceptions": [],
            "authorized_exception_count": 0,
            "delivered_input_count": 0,
            "representative_classes": representative_classes,
        },
        "evidence_observations": {
            "executed_trajectories": [[]],
            "terminal_streams": graph.iter().filter_map(|state| state.pointer("/observed_state/raw_terminal_stream").cloned()).collect::<Vec<_>>(),
            "canonical_interactions": [],
            "canonical_journey": Value::Null,
            "canonical_accessibility": Value::Null,
            "canonical_motion_analysis": Value::Null,
            "gaps": [
                "A default-deny policy withheld all keyboard, mouse, pointer, paste, text, and other input.",
                "No input class or key had an authorized exception.",
                "Representative input classes are reported without claiming an exhaustive key list."
            ]
        },
    });
    std::fs::write(
        output.join("crawl.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}

pub(crate) fn records(selected: Option<&str>) -> Result<Vec<(String, String)>> {
    let directory = super::corpus::data_root()
        .join("tui-examples")
        .join("references");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    paths.sort();
    let mut records = Vec::new();
    for path in paths {
        let slug = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if selected.is_some_and(|value| {
            value != slug && value != slug.split_once('-').map(|(_, tail)| tail).unwrap_or(slug)
        }) {
            continue;
        }
        let record: Value = serde_json::from_slice(&std::fs::read(path.join("reference.json"))?)?;
        let name = record
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        records.push((slug.to_string(), name.to_string()));
    }
    if records.is_empty() {
        bail!("no matching TUI records");
    }
    Ok(records)
}
