use super::*;

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut host: Option<String> = None;
    let mut record: Option<String> = None;
    let mut runtime_manifest_base64: Option<String> = None;
    let mut artifact_uri: Option<String> = None;
    let mut worker = false;
    let mut wait_seconds = 7_200u64;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--runtime-manifest-base64" => {
                i += 1;
                runtime_manifest_base64 = Some(
                    rest.get(i)
                        .context("--runtime-manifest-base64 needs a value")?
                        .clone(),
                );
            }
            "--artifact-uri" => {
                i += 1;
                artifact_uri = Some(rest.get(i).context("--artifact-uri needs a value")?.clone());
            }
            "--wait-seconds" => {
                i += 1;
                wait_seconds = rest
                    .get(i)
                    .context("--wait-seconds needs a value")?
                    .parse()
                    .context("--wait-seconds must be a whole number of seconds")?;
            }
            "--worker" => worker = true,
            "--help" | "-h" => {
                println!("usage: spis crawl-web <catalog> --host TARGET --record SLUG --runtime-manifest-base64 DATA [--wait-seconds N]\nworker mode: spis crawl-web <catalog> --worker --record SLUG --artifact-uri URI --runtime-manifest-base64 DATA [--wait-seconds N]");
                return Ok(());
            }
            value if value.starts_with('-') => bail!("unknown argument: {value}"),
            value if catalog.is_none() => catalog = Some(value.to_string()),
            value => bail!("unexpected argument: {value}"),
        }
        i += 1;
    }
    let catalog = catalog.context("catalog is required")?;
    if !CATALOGS.contains(&catalog.as_str()) {
        bail!("crawl-web accepts {}", CATALOGS.join(", "));
    }
    if !(30..=86_400).contains(&wait_seconds) {
        bail!("--wait-seconds must be 30..86400");
    }
    let record = record.context("--record is required for one exact per-record job")?;
    let manifest = super::crawl::decode_runtime_manifest(
        runtime_manifest_base64
            .as_deref()
            .context("--runtime-manifest-base64 is required")?,
        &catalog,
        "web",
        Some(&record),
    )?;
    if !worker {
        if artifact_uri.is_some() {
            bail!("--artifact-uri is worker-only");
        }
        let host = host
            .context("--host is required; web crawls execute as pinned Stado jobs")?;
        return submit_worker(&host, &catalog, &record, &manifest, wait_seconds);
    }
    if host.is_some() {
        bail!("--host is coordinator-only");
    }
    let artifact_uri = artifact_uri.context("--artifact-uri is required in worker mode")?;
    if artifact_uri != manifest.artifact_uri {
        bail!("worker artifact URI does not match immutable runtime manifest");
    }
    run_worker(&catalog, &record, &manifest, wait_seconds)
}
