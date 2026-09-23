use super::*;

pub(crate) fn worker_report(
    manifest: &super::crawl::RuntimeManifest,
    artifact: Option<Value>,
    failure: Option<Value>,
) -> Result<Value> {
    let value = serde_json::to_value(manifest)?;
    let attempt = value
        .get("attempt")
        .and_then(Value::as_u64)
        .context("TUI runtime manifest has no attempt for worker report")?;
    let attempt_id = value
        .get("attempt_id")
        .and_then(Value::as_str)
        .context("TUI runtime manifest has no attempt_id for worker report")?;
    let bindings_file_sha256 = value
        .get("bindings_file_sha256")
        .and_then(Value::as_str)
        .context("TUI runtime manifest has no bindings_file_sha256")?;
    let bindings_sha256 = value
        .get("bindings_sha256")
        .and_then(Value::as_str)
        .context("TUI runtime manifest has no bindings_sha256")?;
    let execution_identity = value
        .get("execution_identity")
        .filter(|identity| identity.is_object())
        .context("TUI runtime manifest has no typed execution_identity")?
        .clone();
    if let Some(artifact) = artifact.as_ref() {
        if artifact.get("uri").and_then(Value::as_str) != Some(manifest.artifact_uri.as_str()) {
            bail!("published TUI artifact URI does not match the immutable runtime manifest");
        }
    }
    Ok(json!({
        "schema": "wisent.native-worker-report.v1",
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "record_key": manifest.record_key,
        "attempt": attempt,
        "attempt_id": attempt_id,
        "engine": manifest.engine,
        "state": if failure.is_some() { "failed" } else { "artifact_published" },
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "reference_sha256": manifest.reference_sha256,
        "bindings_file_sha256": bindings_file_sha256,
        "bindings_sha256": bindings_sha256,
        "execution_identity": execution_identity,
        "artifact": artifact,
        "failure": failure,
    }))
}

pub(crate) fn submit(
    host: &str,
    selected: &str,
    manifest: &super::crawl::RuntimeManifest,
) -> Result<()> {
    safe_component(host, "--host")?;
    safe_component(selected, "--record")?;
    let _attempt_binding = attempt_root(Path::new("."), manifest)?;
    if revision()? != manifest.source_revision {
        bail!("TUI coordinator revision does not match immutable runtime manifest");
    }
    let artifact = manifest.artifact_uri.clone();
    let output_uri = manifest.output_uri.clone();
    // The absolute path this host executes cargo at, never the bare name.
    // Every worker in this repository is `cargo run --release`, and the job's
    // shell is a non-login `/bin/sh` that reads no profile, so a bare name
    // resolves to nothing however the host installs Rust -- the defect that
    // cost job-545551889f9e88be30daa81f sixteen minutes of a claimed slot in
    // the documentation engine, still open in this one.
    let cargo = super::crawl::resolved_worker_program(host)?;
    // Git the same way, and for the same reason: the worker builds a fixture
    // repository with it, the job's shell finds nothing by bare name, and
    // `/usr/bin/git` must never be the answer because on a host without the
    // Command Line Tools that path opens the installer window rather than
    // running git. The allowlist resolves it to a real installation and the
    // worker is handed that exact path.
    let git = super::crawl::resolved_program(host, &["git", "--version"])?;
    if git.chars().any(char::is_whitespace) || git == "/usr/bin/git" {
        bail!("host {host} resolved git to {git:?}, which this worker must not be handed");
    }
    let worker = format!(
        "{cargo} run --release -- crawl-tui --worker --record {selected} --artifact-uri {artifact} --git-path {git} --runtime-manifest-base64 '{}'",
        manifest.encoded()?
    );
    let mut stado = super::crawl::stado_command();
    stado.args([
        "submit",
        &worker,
        "--run-id",
        &manifest.stado_run_id,
        "--pinned-host",
        host,
        "--repo",
        REPOSITORY,
        "--repo-ref",
        &manifest.source_revision,
        "--repo-workdir",
        super::crawl::STADO_REPO_WORKDIR,
        "--repo-extras",
        "",
        "--output-uri",
        &output_uri,
    ]);
    for (name, reference) in delivery_secret_bindings(manifest)? {
        stado.arg("--secret-env").arg(format!("{name}={reference}"));
    }
    let output = super::crawl::bounded_command_output(
        &mut stado,
        "submit TUI crawl through Stado",
        Duration::from_secs(120),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!("Stado refused TUI crawl: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    super::crawl::print_submission(
        "tui-examples",
        "tui",
        host,
        Some(&artifact),
        &output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut host: Option<String> = None;
    let mut selected: Option<String> = None;
    let mut worker = false;
    let mut artifact_uri: Option<String> = None;
    let mut git_path: Option<String> = None;
    let mut runtime_manifest_base64: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--record" => {
                i += 1;
                selected = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            "--git-path" => {
                i += 1;
                git_path = Some(rest.get(i).context("--git-path needs a value")?.clone());
            }
            "--runtime-manifest-base64" => {
                i += 1;
                runtime_manifest_base64 =
                    Some(rest.get(i).context("--runtime-manifest-base64 needs a value")?.clone());
            }
            "--worker" => worker = true,
            "--help" | "-h" => {
                println!("usage: spis crawl-tui --host TARGET --record SLUG --runtime-manifest-base64 DATA\nworker mode requires the same immutable runtime manifest and exact record.");
                return Ok(());
            }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    let selected = selected.context("--record is required for one exact per-record job")?;
    let manifest = super::crawl::decode_runtime_manifest(
        runtime_manifest_base64.as_deref().context("--runtime-manifest-base64 is required")?,
        "tui-examples",
        "tui",
        Some(&selected),
    )?;
    if !worker {
        return submit(
            &host.context("--host is required; TUI crawls execute as pinned Stado jobs")?,
            &selected,
            &manifest,
        );
    }
    if host.is_some() {
        bail!("--host cannot be used with --worker");
    }
    let artifact_uri = artifact_uri.context("--artifact-uri is required in worker mode")?;
    if artifact_uri != manifest.artifact_uri {
        bail!("worker artifact URI does not match immutable runtime manifest");
    }
    // The path the host itself resolved, handed down by the coordinator. The
    // worker never searches for git: this job's shell reads no profile, and
    // `/usr/bin/git` on macOS is the `xcode-select` shim that opens the
    // Command Line Tools installer window instead of running git, which is
    // not something an unattended fleet host may be made to do.
    let git_path = git_path.context("--git-path is required in worker mode")?;
    if !git_path.starts_with('/')
        || git_path.chars().any(char::is_whitespace)
        || git_path == "/usr/bin/git"
    {
        bail!("--git-path must be an absolute real git, never the /usr/bin shim: {git_path:?}");
    }
    let root = attempt_root(
        &Path::new("target").join("spis-tui-crawls"),
        &manifest,
    )?;
    std::fs::create_dir_all(&root)?;
    let (slug, name) = records(Some(&selected))?.into_iter().next().context("runtime manifest record is absent")?;
    let output = root.join(&slug);
    std::fs::create_dir_all(&output)?;
    let (record_report, failure) = match crawl_one(&slug, &name, &manifest, &output, Path::new(&git_path)) {
        Ok(report) => (report, None),
        Err(error) => {
            let code = failure_code(&error);
            let message = format!("{error:#}");
            // Diagnostics never share stdout with the one worker report line.
            eprintln!("TUI record {slug} failed: {message}");
            (
                json!({
                    "slug": slug,
                    "name": name,
                    "status": "failed",
                    "source_revision": manifest.source_revision,
                    "source_input_sha256": manifest.source_input_sha256,
                    "runtime_manifest": manifest,
                    "error": message,
                }),
                Some((code, message)),
            )
        }
    };
    let summary = json!({
        "schema": "wisent.tui-crawl-batch.v1",
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "records": [record_report],
        "failed": usize::from(failure.is_some()),
    });
    std::fs::write(
        root.join("batch.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    // A failed record still retains a typed failure artifact and still
    // publishes the attempt archive, so every attempt has exactly one archive
    // and exactly one worker report line.
    if let Some((code, message)) = failure.as_ref() {
        std::fs::write(
            root.join("failure.json"),
            serde_json::to_string_pretty(&json!({
                "schema": "wisent.native-worker-failure.v1",
                "code": code,
                "message": message,
                "run_id": manifest.run_id,
                "catalog": manifest.catalog,
                "record": manifest.record,
                "attempt": manifest.attempt,
                "attempt_id": manifest.attempt_id,
                "engine": manifest.engine,
            }))? + "\n",
        )?;
    }
    let artifact = super::crawl::publish_attempt_archive(&root, &artifact_uri)?;
    let failure = failure.map(|(code, message)| json!({"code": code, "message": message}));
    let report = worker_report(&manifest, Some(artifact), failure.clone())?;
    super::crawl::publish_worker_report(&manifest, &report)?;
    println!("{}", serde_json::to_string(&report)?);
    if failure.is_some() {
        bail!("the exact TUI record could not be crawled");
    }
    Ok(())
}
