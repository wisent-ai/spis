use super::*;

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut host: Option<String> = None;
    let mut record: Option<String> = None;
    let mut runtime_manifest_base64: Option<String> = None;
    let mut artifact_uri: Option<String> = None;
    let mut worker = false;
    let mut arguments = rest.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--host" => {
                host = Some(
                    crate::commands::required(arguments.next(), "--host needs a value")?.clone(),
                );
            }
            "--record" => {
                record = Some(
                    crate::commands::required(arguments.next(), "--record needs a value")?.clone(),
                );
            }
            "--runtime-manifest-base64" => {
                runtime_manifest_base64 = Some(
                    crate::commands::required(
                        arguments.next(),
                        "--runtime-manifest-base64 needs a value",
                    )?
                    .clone(),
                );
            }
            "--artifact-uri" => {
                artifact_uri = Some(
                    crate::commands::required(arguments.next(), "--artifact-uri needs a value")?
                        .clone(),
                );
            }
            "--worker" => worker = true,
            "--help" | "-h" => {
                println!("usage: spis crawler run web <catalog> --host TARGET --record SLUG --runtime-manifest-base64 DATA\nworker mode: spis crawler run web <catalog> --worker --record SLUG --artifact-uri URI --runtime-manifest-base64 DATA");
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
    let web_families = web_catalogs();
    if !web_families.contains(&catalog) {
        bail!("crawler run web accepts {}", web_families.join(", "));
    }
    let record = record.context("--record is required for one exact per-record job")?;
    let manifest = crate::commands::crawl::decode_runtime_manifest(
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
        let host = host.context("--host is required; web crawls execute as pinned Stado jobs")?;
        return submit_worker(&host, &catalog, &record, &manifest);
    }
    if host.is_some() {
        bail!("--host is coordinator-only");
    }
    let artifact_uri = artifact_uri.context("--artifact-uri is required in worker mode")?;
    if artifact_uri != manifest.artifact_uri {
        bail!("worker artifact URI does not match immutable runtime manifest");
    }
    run_worker(&catalog, &record, &manifest)
}
