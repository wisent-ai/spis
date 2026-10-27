use super::*;

pub(crate) fn invoke_engine(args: &[String], host: &str, host_report: &Value) -> Result<Output> {
    let executable = std::env::current_exe().context("locate running spis binary")?;
    let worker_program = resolved_program_from_host_preflight(host, &WORKER_PROGRAM, host_report)?;
    let mut command = Command::new(executable);
    command
        .args(args)
        .env("SPIS_PREFLIGHT_WORKER_PROGRAM", worker_program);
    command_output(&mut command, "crawler coordinator")
}

/// The Spis command line that crawls one record on `host`, built from the
/// engine the manifest declares; a manifest bound to another host is refused.
pub(crate) fn engine_command(
    manifest: &RuntimeManifest,
    host: &str,
    retained_command: Option<&[String]>,
) -> Result<Vec<String>> {
    let execution = manifest
        .execution_identity
        .as_ref()
        .context("cannot build worker command without execution identity")?;
    if execution.host != host {
        bail!("worker command host differs from the bound execution identity");
    }
    let engine = Engine::of(&manifest.engine)?;
    if engine == Engine::Web
        && manifest
            .service_identity
            .as_ref()
            .is_none_or(|service| service.active_host != host)
    {
        bail!("web worker command host differs from the bound Weles service placement");
    }
    let mut args = vec!["crawler".into(), "run".into(), engine.name().into()];
    if engine.takes_catalog() {
        args.push(manifest.catalog.clone());
    }
    if matches!(engine, Engine::Mobile | Engine::Desktop) {
        let (max_states, max_depth) = native_limits(retained_command)?;
        args.extend([
            "--max-states".into(), max_states.to_string(),
            "--max-depth".into(), max_depth.to_string(),
        ]);
    }
    args.push("--host".into());
    args.push(host.into());
    if engine == Engine::Mobile {
        let driver_url = manifest
            .driver_url
            .as_deref()
            .context("mobile runtime manifest binds no Appium address")?;
        args.push("--driver-url".into());
        args.push(driver_url.into());
    }
    args.push("--record".into());
    args.push(manifest.record.clone());
    args.push("--runtime-manifest-base64".into());
    args.push(manifest.encoded()?);
    Ok(args)
}

/// A prepared attempt keeps its declared bounds in its retained argv. Resuming
/// it must neither require the original process environment nor adopt new bounds.
fn native_limits(retained: Option<&[String]>) -> Result<(usize, usize)> {
    let Some(retained) = retained else {
        return crate::commands::crawler::traversal_limits(None, None);
    };
    let mut states = None;
    let mut depth = None;
    let mut arguments = retained.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--max-states" => {
                states = Some(crate::commands::parsed(arguments.next(), "retained --max-states")?);
            }
            "--max-depth" => {
                depth = Some(crate::commands::parsed(arguments.next(), "retained --max-depth")?);
            }
            _ => {}
        }
        // Both bounds precede opaque record and manifest values.
        if states.is_some() && depth.is_some() {
            break;
        }
    }
    let states = states.context("retained native worker command has no --max-states")?;
    let depth = depth.context("retained native worker command has no --max-depth")?;
    crate::commands::crawler::traversal_limits(Some(states), Some(depth))
}

/// The selected families with the engine that crawls each, every family the
/// reference contract declares when none is selected.
pub(crate) fn selected_specs(selected: &[String]) -> Result<Vec<(String, String)>> {
    let all = crawl_catalogs();
    if selected.is_empty() {
        return Ok(all);
    }
    let mut out = Vec::new();
    for wanted in selected {
        let spec = all
            .iter()
            .find(|(catalog, _)| catalog == wanted)
            .cloned()
            .ok_or_else(|| anyhow!("unknown crawl catalog {wanted}"))?;
        if !out.contains(&spec) {
            out.push(spec);
        }
    }
    Ok(out)
}

/// Enumerate the committed record directories of one catalog.
///
/// A record directory must be a real directory with a portable UTF-8 name. A
/// symbolic link, a non-UTF-8 name, an unreadable entry or a non-directory is an
/// error rather than a silently dropped record, so a planted link can never
/// redirect a crawl and a broken checkout can never shrink the plan in silence.
pub(crate) fn record_directories(catalog: &str, selected: Option<&str>) -> Result<Vec<PathBuf>> {
    let root = catalog_root(catalog)?.join("references");
    let require_real_directory = |path: &Path| -> Result<()> {
        let metadata = std::fs::symlink_metadata(path)
            .with_context(|| format!("read crawl record {}", path.display()))?;
        if metadata.file_type().is_symlink() {
            bail!("crawl record {} is a symbolic link", path.display());
        }
        if !metadata.is_dir() {
            bail!("crawl record {} is not a directory", path.display());
        }
        Ok(())
    };
    if let Some(record) = selected {
        safe_component(record, "record")?;
        let path = root.join(record);
        if !path.exists() {
            bail!("record {record} does not exist in catalog {catalog}");
        }
        require_real_directory(&path)?;
        return Ok(vec![path]);
    }
    let mut records = Vec::new();
    for entry in std::fs::read_dir(&root)
        .with_context(|| format!("read crawl catalog {}", root.display()))?
    {
        let entry = entry.with_context(|| format!("read crawl catalog {}", root.display()))?;
        let name = entry.file_name();
        let name = name.to_str().with_context(|| {
            format!(
                "crawl catalog {} contains a non-UTF-8 record name",
                root.display()
            )
        })?;
        if name.starts_with('.') {
            continue;
        }
        safe_component(name, "record")?;
        let path = entry.path();
        require_real_directory(&path)?;
        records.push(path);
    }
    records.sort();
    Ok(records)
}

/// The whole runtime bindings document: its schema and one object of record
/// bindings per family.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingsDocument {
    schema: String,
    records: serde_json::Map<String, Value>,
}

/// A record binding declared unconfigured, with the reason.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct UnconfiguredBinding {
    configured: bool,
    diagnostic: Value,
}

/// A configured record binding: account, constraints and delivery are
/// required, a prepared proof and a surface are optional.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct ConfiguredBinding {
    configured: bool,
    account: Value,
    constraints: Value,
    delivery: Value,
    #[serde(default)]
    prepared_proof: Option<Value>,
    #[serde(default)]
    surface: Option<Value>,
}

pub(crate) fn validate_runtime_bindings_document(document: &Value) -> Result<()> {
    let document: BindingsDocument = serde_json::from_value(document.clone())
        .context("runtime bindings must hold exactly a schema and records")?;
    if document.schema != "wisent.crawl-runtime-bindings.v1" {
        bail!("runtime bindings must declare wisent.crawl-runtime-bindings.v1");
    }
    let catalogs = document.records;
    let families = crawl_catalogs();
    let mut actual_catalogs = catalogs.keys().map(String::as_str).collect::<Vec<_>>();
    let mut expected_catalogs = families
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    actual_catalogs.sort_unstable();
    expected_catalogs.sort_unstable();
    if actual_catalogs != expected_catalogs {
        bail!("runtime bindings must contain every and only the checked-in crawl catalogs");
    }
    for (catalog, _) in &families {
        let records = catalogs
            .get(catalog)
            .and_then(Value::as_object)
            .with_context(|| format!("runtime bindings {catalog} must be a record object"))?;
        let mut actual_records = records.keys().map(String::as_str).collect::<Vec<_>>();
        let expected_paths = record_directories(catalog, None)?;
        let mut expected_records = expected_paths
            .iter()
            .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
            .collect::<Vec<_>>();
        actual_records.sort_unstable();
        expected_records.sort_unstable();
        if actual_records != expected_records {
            bail!("{catalog}: runtime bindings must contain every and only checked-in record slug");
        }
        for (slug, binding) in records {
            match binding.get("configured").and_then(Value::as_bool) {
                Some(false) => {
                    serde_json::from_value::<UnconfiguredBinding>(binding.clone()).with_context(
                        || format!("{catalog}/{slug}: an unconfigured binding holds only its diagnostic"),
                    )?;
                }
                Some(true) => {
                    serde_json::from_value::<ConfiguredBinding>(binding.clone()).with_context(
                        || format!("{catalog}/{slug}: configured binding fields are not the declared ones"),
                    )?;
                }
                _ => bail!("{catalog}/{slug}: configured must be an explicit boolean"),
            }
        }
    }
    Ok(())
}

pub(crate) fn safe_unconfigured_binding(engine: &str) -> Value {
    json!({
        "configured": false,
        "diagnostic": format!(
            "{engine} requires an explicit record binding and independently observed authorization proof"
        ),
    })
}

pub(crate) fn anonymous_probe_account() -> Value {
    let mode = crate::commands::reference_contract::spelling(&AccountMode::AnonymousReadOnlyProbe);
    json!({
        "mode": mode,
        "account_id": mode,
        "credential_refs": [],
    })
}

/// Every crawl refuses first-run consent, system permission prompts,
/// notifications, purchases and final destructive actions.
///
/// `headless` is the one engine-specific control: only the Weles browser engine
/// executes headless. `planned_record` requires `headless == (engine == "web")`,
/// so a generator that hardcoded `true` made every documentation, terminal and
/// native record permanently unavailable.
pub(crate) fn prohibited_action_constraints(engine: &str) -> Value {
    json!({
        "no_first_run_consent": true,
        "no_system_permission_prompts": true,
        "no_notifications": true,
        "no_purchase": true,
        "no_final_destructive_action": true,
        "headless": engine == "web",
    })
}
