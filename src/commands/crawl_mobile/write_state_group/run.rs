use super::*;

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut record: Option<String> = None;
    let mut driver_url = "http://127.0.0.1:4723".to_string();
    let mut host: Option<String> = None;
    let mut worker = false;
    let mut artifact_uri: Option<String> = None;
    let mut runtime_manifest_base64: Option<String> = None;
    let mut max_states = 200usize;
    let mut max_depth = 8usize;
    let mut output = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".spis")
        .join("crawls");
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--driver-url" => {
                i += 1;
                driver_url = rest.get(i).context("--driver-url needs a value")?.clone();
            }
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--worker" => worker = true,
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            "--runtime-manifest-base64" => {
                i += 1;
                runtime_manifest_base64 =
                    Some(rest.get(i).context("--runtime-manifest-base64 needs a value")?.clone());
            }
            "--max-states" => {
                i += 1;
                max_states = rest.get(i).context("--max-states needs a value")?.parse()?;
            }
            "--max-depth" => {
                i += 1;
                max_depth = rest.get(i).context("--max-depth needs a value")?.parse()?;
            }
            "--output" => {
                i += 1;
                output = PathBuf::from(rest.get(i).context("--output needs a value")?);
            }
            "--help" | "-h" => {
                println!("usage: spis crawl-mobile <ios-app-examples|android-app-examples> --host TARGET --record SLUG --runtime-manifest-base64 DATA [--driver-url URL] [--max-states N] [--max-depth N]\nworker mode requires the same immutable runtime manifest and exact record.");
                return Ok(());
            }
            value if value.starts_with('-') => bail!("unknown argument: {value}"),
            value if catalog.is_none() => catalog = Some(value.to_string()),
            value => bail!("unexpected argument: {value}"),
        }
        i += 1;
    }
    let catalog = catalog.context("catalog is required")?;
    let platform = Platform::from_catalog(&catalog)?;
    driver_url = canonical_driver_url(&driver_url)?;
    if max_states == 0 || max_states > 10_000 || max_depth > 32 {
        bail!("--max-states must be 1..10000 and --max-depth must be 0..32");
    }
    let record = record.context("--record is required for one exact per-record job")?;
    let encoded_manifest = runtime_manifest_base64
        .as_deref()
        .context("--runtime-manifest-base64 is required")?;
    let manifest = super::crawl::decode_runtime_manifest(
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
            driver_url: &driver_url,
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
    let appium = Appium::new(&driver_url)?;
    let run_root = attempt_root(
        &output,
        &manifest,
    )?;
    std::fs::create_dir_all(&run_root)?;
    let entry = records(&catalog, Some(&record))?
        .into_iter()
        .next()
        .context("runtime manifest record is absent from catalog")?;
    let (record_report, failure) = match crawl_record(
        &appium,
        platform,
        &entry,
        &manifest,
        &run_root,
        max_states,
        max_depth,
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
    let artifact = super::crawl::publish_attempt_archive(&run_root, &artifact_uri)?;
    let failure = failure.map(|(code, message)| json!({"code": code, "message": message}));
    let report = worker_report(&manifest, Some(artifact), failure.clone())?;
    super::crawl::publish_worker_report(&manifest, &report)?;
    println!("{}", serde_json::to_string(&report)?);
    if failure.is_some() {
        bail!("the exact mobile record could not be crawled");
    }
    Ok(())
}
