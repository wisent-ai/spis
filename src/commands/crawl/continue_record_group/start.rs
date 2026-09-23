use super::*;

pub(crate) fn start(rest: &[String]) -> Result<()> {
    let mut hosts: BTreeMap<String, String> = BTreeMap::new();
    let mut catalogs = Vec::new();
    let mut record = None;
    let mut requested_run_id = None;
    let mut bindings_path = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                let value = rest.get(i).context("--host needs a value")?;
                if let Some((scope, target)) = value.split_once('=') {
                    if scope.is_empty() || target.is_empty() {
                        bail!("--host mapping must be ENGINE=TARGET or CATALOG=TARGET");
                    }
                    hosts.insert(scope.to_string(), target.to_string());
                } else {
                    hosts.insert("*".into(), value.clone());
                }
            }
            "--catalog" => {
                i += 1;
                catalogs.push(rest.get(i).context("--catalog needs a value")?.clone());
            }
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--run-id" => {
                i += 1;
                requested_run_id = Some(rest.get(i).context("--run-id needs a value")?.clone());
            }
            "--bindings" => {
                i += 1;
                bindings_path = Some(rest.get(i).context("--bindings needs a value")?.clone());
            }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    let specs = selected_specs(&catalogs)?;
    if record.is_some() && specs.len() != 1 {
        bail!("--record requires exactly one --catalog");
    }
    let source_revision = source_snapshot_revision()?;
    let bindings = load_runtime_bindings(bindings_path.as_deref())?;
    let (discovered_hosts, service_identity, registry_diagnostic) =
        match registry_placements() {
            Ok((hosts, service)) => (hosts, service, None),
            Err(error) => (
                BTreeMap::new(),
                None,
                Some(format!("Stado registry placement discovery failed: {error}")),
            ),
        };
    let run_id = requested_run_id.unwrap_or_else(|| {
        format!("crawl-{}", crate::now_iso_utc().replace(':', "-").replace('T', "-"))
    });
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
        if has_failures(&run) {
            bail!("one or more records remain unavailable or failed");
        }
        return Ok(());
    }
    let mut entries = Vec::new();
    for (catalog, engine) in specs {
        let paths = record_directories(catalog, record.as_deref())?;
        let host = host_for(catalog, engine, &hosts, &discovered_hosts);
        let unavailable = host
            .as_ref()
            .err()
            .map(ToString::to_string)
            .or_else(|| {
                (engine == "web" && service_identity.is_none()).then(|| {
                    registry_diagnostic.clone().unwrap_or_else(|| {
                        "Stado service directory does not authorize consumer spis for weles-admission browser-evidence on a host advertising generic_browser_task".into()
                    })
                })
            });
        if let Some(message) = unavailable {
            let records = paths
                .iter()
                .map(|path| {
                    unavailable_record(
                        &run_id,
                        catalog,
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("invalid-record"),
                        "runtime_placement_unavailable",
                        message.clone(),
                        json!({"engine": engine}),
                    )
                })
                .collect::<Vec<_>>();
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
        let host = host.expect("placement error handled");
        let service = (engine == "web")
            .then_some(service_identity.as_ref())
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
        "mutation_revision": 0,
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
    if has_failures(&run) {
        bail!("one or more records remain unavailable or failed");
    }
    Ok(())
}

#[derive(Debug)]
pub(crate) struct LookupFailure {
    pub(crate) diagnostic: Value,
    pub(crate) not_found: bool,
}

pub(crate) fn machine_status(job_id: &str) -> std::result::Result<Value, LookupFailure> {
    let output = super::crawl::stado_command().args(["machine", "status", job_id]).output()
        .map_err(|error| LookupFailure {
            diagnostic: json!({"code": "transport_error", "retryable": true, "message": error.to_string()}),
            not_found: false,
        })?;
    let document: Value = serde_json::from_slice(&output.stdout).map_err(|error| LookupFailure {
        diagnostic: json!({"code": "invalid_response", "retryable": true, "message": error.to_string(), "stderr": String::from_utf8_lossy(&output.stderr).trim()}),
        not_found: false,
    })?;
    if !output.status.success() || document.get("ok").and_then(Value::as_bool) != Some(true) {
        let error = document.get("error").cloned().unwrap_or_else(|| json!({
            "code": "status_failed",
            "retryable": true,
            "message": String::from_utf8_lossy(&output.stderr).trim(),
        }));
        let code = error
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_uppercase();
        let not_found = matches!(code.as_str(), "NOT_FOUND" | "JOB_NOT_FOUND");
        return Err(LookupFailure { diagnostic: error, not_found });
    }
    document.pointer("/result/job").cloned().ok_or_else(|| LookupFailure {
        diagnostic: json!({"code": "invalid_response", "retryable": true, "message": "Stado status has no result.job"}),
        not_found: false,
    })
}
