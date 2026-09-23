use super::*;

pub(crate) fn run_worker(
    catalog: &str,
    record: &str,
    manifest: &super::crawl::RuntimeManifest,
    wait_seconds: u64,
) -> Result<()> {
    if catalog != manifest.catalog || record != manifest.record {
        bail!("worker catalog/record differ from the immutable runtime manifest");
    }
    let base = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".spis")
        .join("crawls");
    let attempt_root = super::crawl::native_attempt_root(&base, manifest)?;
    std::fs::create_dir_all(&attempt_root)?;
    prune_stale_attempt_temporaries(&attempt_root)?;
    let private = PrivateBridge::open(manifest)?;
    let mut collected = Collected::default();
    let outcome = capture(
        manifest,
        wait_seconds,
        &attempt_root,
        &private,
        &mut collected,
    );
    private.discard();
    let failure = match outcome {
        // A publication failure is itself a typed attempt failure, so the single mandatory
        // report line is emitted on every path.
        Ok(()) => {
            match super::crawl::publish_attempt_archive(&attempt_root, &manifest.artifact_uri) {
                Ok(artifact) => {
                    let report = worker_report(
                        manifest,
                        "artifact_published",
                        Some(artifact),
                        &collected,
                        None,
                    );
                    super::crawl::publish_worker_report(manifest, &report)?;
                    println!("{}", serde_json::to_string(&report)?);
                    return Ok(());
                }
                Err(error) => WorkerFailure::new(
                    "attempt_archive_publication_failed",
                    format!("{error:#}"),
                ),
            }
        }
        Err(failure) => failure,
    };
    let document = json!({
        "schema": FAILURE_SCHEMA,
        "code": failure.code,
        "message": failure.message,
        "run_id": manifest.run_id,
        "catalog": manifest.catalog,
        "record": manifest.record,
        "attempt": u64::from(manifest.attempt),
        "attempt_id": manifest.attempt_id,
    });
    if let Err(retention) = retain_attempt_document(&attempt_root, "failure.json", &document) {
        eprintln!(
            "web worker failure artifact could not be retained ({}): {}",
            retention.code, retention.message
        );
    }
    let artifact = super::crawl::publish_attempt_archive(&attempt_root, &manifest.artifact_uri);
    let report = worker_report(
        manifest,
        "failed",
        artifact.as_ref().ok().cloned(),
        &collected,
        Some(&failure),
    );
    super::crawl::publish_worker_report(manifest, &report)?;
    println!("{}", serde_json::to_string(&report)?);
    match artifact {
        Ok(_) => bail!("web worker failed ({}): {}", failure.code, failure.message),
        Err(error) => bail!(
            "web worker failed ({}): {}; the attempt archive could not be published either: {error:#}",
            failure.code,
            failure.message
        ),
    }
}
