use super::*;

pub(crate) fn start(rest: &[String]) -> Result<()> {
    let mut hosts: BTreeMap<String, String> = BTreeMap::new();
    let mut catalogs = Vec::new();
    let mut record = None;
    let mut requested_run_id = None;
    let mut bindings_path = None;
    let mut arguments = rest.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--host" => {
                let value = crate::commands::required(arguments.next(), "--host needs a value")?;
                if let Some((scope, target)) = value.split_once('=') {
                    if scope.is_empty() || target.is_empty() {
                        return Err(crate::commands::usage(
                            "--host mapping must be ENGINE=TARGET or CATALOG=TARGET",
                        ));
                    }
                    hosts.insert(scope.to_string(), target.to_string());
                } else {
                    hosts.insert("*".into(), value.clone());
                }
            }
            "--catalog" => {
                catalogs.push(
                    crate::commands::required(arguments.next(), "--catalog needs a value")?.clone(),
                );
            }
            "--record" => {
                record = Some(
                    crate::commands::required(arguments.next(), "--record needs a value")?.clone(),
                );
            }
            "--run-id" => {
                requested_run_id = Some(
                    crate::commands::required(arguments.next(), "--run-id needs a value")?.clone(),
                );
            }
            "--bindings" => {
                bindings_path = Some(
                    crate::commands::required(arguments.next(), "--bindings needs a value")?
                        .clone(),
                );
            }
            value => return Err(crate::commands::usage(format!("unknown argument: {value}"))),
        }
    }
    let specs = selected_specs(&catalogs)?;
    if record.is_some() && !matches!(specs.as_slice(), [_]) {
        bail!("--record requires exactly one --catalog");
    }
    let source_revision = source_snapshot_revision()?;
    let bindings = load_runtime_bindings(bindings_path.as_deref())?;
    let (discovered_hosts, service_identity, driver_urls, registry_diagnostic) =
        match registry_placements() {
            Ok(placements) => (
                placements.hosts,
                placements.service_identity,
                placements.driver_urls,
                None,
            ),
            Err(error) => (
                BTreeMap::new(),
                None,
                BTreeMap::new(),
                Some(format!(
                    "Stado registry placement discovery failed: {error}"
                )),
            ),
        };
    // Without --run-id the run is named for the moment it starts.
    let run_id = match requested_run_id {
        Some(run_id) => run_id,
        None => format!(
            "crawl-{}",
            crate::now_iso_utc().replace(':', "-").replace('T', "-")
        ),
    };
    let request_identity = json!({
        "source_revision": source_revision,
        "catalogs": specs,
        "record": record,
        "hosts": hosts,
        "service_identity": service_identity,
        "bindings_source": bindings.source,
        "bindings_sha256": bindings.sha256,
        "bindings_uri": bindings.uri,
    });
    let request_digest = crate::sha256_hex(&serde_json::to_vec(&request_identity)?);
    if run_path(&run_id)?.is_file() {
        {
            let _guard = RunMutationGuard::acquire(&run_id)?;
            let run = load(Some(&run_id))?;
            if run.get("request_digest").and_then(Value::as_str) != Some(&request_digest) {
                bail!("run id {run_id} already belongs to a different exact crawl request");
            }
        }
        publish_runtime_bindings(&bindings)?;
        let run = continue_start(&run_id)?;
        print_operation("start", &run, None)?;
        if has_failures(&run)? {
            bail!("one or more records remain unavailable or failed");
        }
        return Ok(());
    }
    let mut entries = Vec::new();
    for (catalog, engine) in &specs {
        let (catalog, engine) = (catalog.as_str(), engine.as_str());
        let paths = record_directories(catalog, record.as_deref())?;
        let host = host_for(catalog, engine, &hosts, &discovered_hosts);
        let unavailable = match &host {
            Err(error) => Some(error.to_string()),
            // Discovery that failed says why; discovery that answered without
            // a service identity means the directory grants none.
            Ok(_) if engine == "web" && service_identity.is_none() => Some(match &registry_diagnostic {
                Some(diagnostic) => diagnostic.clone(),
                None => "Stado service directory does not authorize consumer spis for weles-admission browser-evidence on a host advertising generic_browser_task".into(),
            }),
            Ok(host) if engine == "mobile" && !driver_urls.contains_key(host) => Some(format!("Stado registry host {host} declares no mobile_runtime.address, so no Appium server is known for it; declare the address the Appium server listens on for that host")),
            Ok(_) => None,
        };
        if let Some(message) = unavailable {
            let records = paths
                .iter()
                .map(|path| {
                    let name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .with_context(|| {
                            format!("record directory {} has no UTF-8 name", path.display())
                        })?;
                    Ok(unavailable_record(
                        &run_id,
                        catalog,
                        name,
                        "runtime_placement_unavailable",
                        message.clone(),
                        json!({"engine": engine}),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            entries.push(json!({
                "catalog": catalog,
                "engine": engine,
                "host": Value::Null,
                "state": "unavailable",
                "host_preflight": {
                    "ready": false,
                    "diagnostic": {
                        "code": "runtime_placement_unavailable",
                        "message": message,
                    }
                },
                "records": records,
            }));
            continue;
        }
        let host = host?;
        let service = (engine == "web")
            .then_some(service_identity.as_ref())
            .flatten();
        let driver_url = (engine == "mobile")
            .then(|| driver_urls.get(&host).map(String::as_str))
            .flatten();
        let records = paths
            .iter()
            .map(|path| {
                planned_record(
                    &run_id,
                    &source_revision,
                    catalog,
                    engine,
                    &host,
                    path,
                    &bindings,
                    service,
                    driver_url,
                )
            })
            .collect::<Vec<_>>();
        entries.push(json!({
            "catalog": catalog,
            "engine": engine,
            "host": host,
            "state": "planned",
            "host_preflight": Value::Null,
            "records": records,
        }));
    }
    let mut run = json!({
        "schema": RUN_SCHEMA,
        "run_id": run_id,
        "source_revision": source_revision,
        "request_digest": request_digest,
        "request": request_identity,
        "created_at": crate::now_iso_utc(),
        "updated_at": crate::now_iso_utc(),
        // The first persist writes the first mutation revision.
        "hosts": hosts,
        "bindings_source": bindings.source,
        "bindings_sha256": bindings.sha256,
        "bindings_uri": bindings.uri,
        "state": "planned",
        "catalogs": entries,
    });
    {
        let _guard = RunMutationGuard::acquire(&run_id)?;
        if run_path(&run_id)?.is_file() {
            bail!("run id {run_id} was concurrently created");
        }
        persist(&mut run)?;
    }
    publish_runtime_bindings(&bindings)?;
    run = continue_start(&run_id)?;
    print_operation("start", &run, None)?;
    if has_failures(&run)? {
        bail!("one or more records remain unavailable or failed");
    }
    Ok(())
}
