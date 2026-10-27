use super::*;

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut record: Option<String> = None;
    // The Appium address is the placement host's `mobile_runtime.address` in
    // the Stado registry, bound into the runtime manifest at planning.
    // `--driver-url` overrides it for a coordinator submission; nothing is
    // assumed.
    let mut driver_url: Option<String> = None;
    let mut host: Option<String> = None;
    let mut worker = false;
    let mut artifact_uri: Option<String> = None;
    let mut runtime_manifest_base64: Option<String> = None;
    let mut max_states = None;
    let mut max_depth = None;
    // `--output`, else `~/.spis/crawls`, resolved only when it is needed so an
    // unset HOME refuses only a run that has no `--output`.
    let mut output: Option<PathBuf> = None;
    let mut arguments = rest.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--record" => {
                record = Some(crate::commands::required(arguments.next(), "--record needs a value")?.clone());
            }
            "--driver-url" => {
                driver_url = Some(crate::commands::required(arguments.next(), "--driver-url needs a value")?.clone());
            }
            "--host" => {
                host = Some(crate::commands::required(arguments.next(), "--host needs a value")?.clone());
            }
            "--worker" => worker = true,
            "--artifact-uri" => {
                artifact_uri = Some(crate::commands::required(arguments.next(), "--artifact-uri needs a value")?.clone());
            }
            "--runtime-manifest-base64" => {
                runtime_manifest_base64 = Some(crate::commands::required(
                    arguments.next(), "--runtime-manifest-base64 needs a value"
                )?.clone());
            }
            "--max-states" => {
                max_states = Some(crate::commands::parsed(arguments.next(), "--max-states")?);
            }
            "--max-depth" => {
                max_depth = Some(crate::commands::parsed(arguments.next(), "--max-depth")?);
            }
            "--output" => {
                output = Some(PathBuf::from(crate::commands::required(
                    arguments.next(), "--output needs a value"
                )?));
            }
            "--help" | "-h" => {
                println!("usage: spis crawler run mobile <ios-app-examples|android-app-examples> --host TARGET --record SLUG --runtime-manifest-base64 DATA [--driver-url URL] [--max-states N] [--max-depth N]\nTraversal bounds must be supplied by flags or SPIS_CRAWL_MAX_STATES and SPIS_CRAWL_MAX_DEPTH in the coordinator environment.\n--driver-url is the Appium server on that host; absent, the runtime manifest's bound address (the host's mobile_runtime.address in the Stado registry) is used.\nworker mode requires the same immutable runtime manifest and exact record.");
                return Ok(());
            }
            value if value.starts_with('-') => {
                return Err(crate::commands::usage(format!("unknown argument: {value}")))
            }
            value if catalog.is_none() => catalog = Some(value.to_string()),
            value => {
                return Err(crate::commands::usage(format!(
                    "unexpected argument: {value}"
                )))
            }
        }
    }
    let catalog = catalog.context("catalog is required")?;
    let platform = Platform::from_catalog(&catalog)?;
    let driver_url = driver_url
        .as_deref()
        .map(canonical_driver_url)
        .transpose()?;
    let (max_states, max_depth) = crate::commands::crawler::traversal_limits(max_states, max_depth)?;
    let record = record.context("--record is required for one exact per-record job")?;
    let encoded_manifest = runtime_manifest_base64
        .as_deref()
        .context("--runtime-manifest-base64 is required")?;
    let manifest = crate::commands::crawl::decode_runtime_manifest(
        encoded_manifest,
        &catalog,
        "mobile",
        Some(&record),
    )?;
    if !worker {
        let host =
            host.context("--host is required; mobile crawls execute as pinned Stado jobs")?;
        return submit_worker(MobileSubmission {
            host: &host,
            catalog: &catalog,
            record: &record,
            driver_url: driver_url.as_deref(),
            max_states,
            max_depth,
            manifest: &manifest,
        });
    }
    if host.is_some() {
        bail!("--host is coordinator-only");
    }
    let artifact_uri = artifact_uri.context("--artifact-uri is required in worker mode")?;
    if artifact_uri != manifest.artifact_uri {
        bail!("worker artifact URI does not match immutable runtime manifest");
    }
    let driver_url = match driver_url {
        Some(url) => url,
        None => {
            let bound = manifest.driver_url.as_deref().context(
                "the runtime manifest binds no Appium address and --driver-url was not given",
            )?;
            canonical_driver_url(bound).context("runtime manifest driver_url")?
        }
    };
    let appium = Appium::new(&driver_url)?;
    let output = match output {
        Some(output) => output,
        None => crate::commands::docs_corpus::crawl_root()?,
    };
    let run_root = attempt_root(&output, &manifest)?;
    std::fs::create_dir_all(&run_root)?;
    let entry = records(&catalog, Some(&record))?
        .into_iter()
        .next()
        .context("runtime manifest record is absent from catalog")?;
    let (record_report, failure) = match crawl_record(
        &appium, platform, &entry, &manifest, &run_root, max_states, max_depth,
    ) {
        Ok(report) => (report, None),
        Err(error) => {
            let code = failure_code(&error);
            let message = format!("{error:#}");
            // Diagnostics never share stdout with the one worker report line.
            eprintln!("mobile record {} failed: {message}", entry.slug);
            (
                json!({
                    "record": entry.slug,
                    "name": entry.name,
                    "status": "failed",
                    "source_revision": manifest.source_revision,
                    "source_input_sha256": manifest.source_input_sha256,
                    "runtime_manifest": manifest,
                    "no_consent_diagnostic": message,
                    "error": message,
                }),
                Some((code, message)),
            )
        }
    };
    let summary = json!({
        "schema": "wisent.mobile-crawl-batch.v1",
        "catalog": catalog,
        "driver_url": driver_url,
        "source_revision": manifest.source_revision,
        "source_input_sha256": manifest.source_input_sha256,
        "runtime_manifest": manifest,
        "records": [record_report],
        "failed": usize::from(failure.is_some()),
    });
    std::fs::write(
        run_root.join("batch.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    // A failed record still retains a typed failure artifact and still
    // publishes the attempt archive, so every attempt has exactly one archive
    // and exactly one worker report line.
    if let Some((code, message)) = failure.as_ref() {
        std::fs::write(
            run_root.join("failure.json"),
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
    let artifact = crate::commands::crawl::publish_attempt_archive(&run_root, &artifact_uri)?;
    let failure = failure.map(|(code, message)| json!({"code": code, "message": message}));
    let report = worker_report(&manifest, Some(artifact), failure.clone())?;
    crate::commands::crawl::publish_worker_report(&manifest, &report)?;
    println!("{}", serde_json::to_string(&report)?);
    if failure.is_some() {
        bail!("the exact mobile record could not be crawled");
    }
    Ok(())
}
