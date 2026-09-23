use super::*;

pub(crate) fn invoke_engine(args: &[String], host: &str, host_report: &Value) -> Result<Output> {
    let executable = std::env::current_exe().context("locate running spis binary")?;
    let worker_program = resolved_program_from_host_preflight(host, &WORKER_PROGRAM, host_report)?;
    let mut command = Command::new(executable);
    command
        .args(args)
        .env("SPIS_PREFLIGHT_WORKER_PROGRAM", worker_program);
    bounded_command_output(
        &mut command,
        "crawler coordinator",
        Duration::from_secs(180),
        8 * 1024 * 1024,
    )
}

pub(crate) fn selected_specs(selected: &[String]) -> Result<Vec<(&'static str, &'static str)>> {
    if selected.is_empty() {
        return Ok(CATALOGS.to_vec());
    }
    let mut out = Vec::new();
    for wanted in selected {
        let spec = CATALOGS.iter().find(|(catalog, _)| catalog == wanted).copied()
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
            format!("crawl catalog {} contains a non-UTF-8 record name", root.display())
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

pub(crate) fn require_exact_object_keys(value: &Value, expected: &[&str], context: &str) -> Result<()> {
    let object = value
        .as_object()
        .with_context(|| format!("{context} must be an object"))?;
    let mut actual = object.keys().map(String::as_str).collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    actual.sort_unstable();
    expected.sort_unstable();
    if actual != expected {
        bail!(
            "{context} fields differ: expected {}, found {}",
            expected.join(", "),
            actual.join(", ")
        );
    }
    Ok(())
}

pub(crate) fn validate_runtime_bindings_document(document: &Value) -> Result<()> {
    require_exact_object_keys(document, &["schema", "records"], "runtime bindings")?;
    if document.get("schema").and_then(Value::as_str)
        != Some("wisent.crawl-runtime-bindings.v1")
    {
        bail!("runtime bindings must declare wisent.crawl-runtime-bindings.v1");
    }
    let catalogs = document
        .get("records")
        .and_then(Value::as_object)
        .context("runtime bindings records must be an object")?;
    let mut actual_catalogs = catalogs.keys().map(String::as_str).collect::<Vec<_>>();
    let mut expected_catalogs = CATALOGS.iter().map(|(name, _)| *name).collect::<Vec<_>>();
    actual_catalogs.sort_unstable();
    expected_catalogs.sort_unstable();
    if actual_catalogs != expected_catalogs {
        bail!("runtime bindings must contain every and only the checked-in crawl catalogs");
    }
    for (catalog, _) in CATALOGS {
        let records = catalogs
            .get(*catalog)
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
                Some(false) => require_exact_object_keys(
                    binding,
                    &["configured", "diagnostic"],
                    &format!("{catalog}/{slug} binding"),
                )?,
                Some(true) => {
                    let object = binding
                        .as_object()
                        .with_context(|| format!("{catalog}/{slug} binding must be an object"))?;
                    let allowed = [
                        "configured",
                        "account",
                        "constraints",
                        "delivery",
                        "prepared_proof",
                        "surface",
                    ];
                    let unknown = object
                        .keys()
                        .filter(|key| !allowed.contains(&key.as_str()))
                        .cloned()
                        .collect::<Vec<_>>();
                    if !unknown.is_empty() {
                        bail!("{catalog}/{slug}: unknown runtime binding fields: {}", unknown.join(", "));
                    }
                    for required in ["account", "constraints", "delivery"] {
                        if !object.contains_key(required) {
                            bail!("{catalog}/{slug}: configured binding is missing {required}");
                        }
                    }
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
    json!({
        "mode": "anonymous-read-only-probe",
        "account_id": "anonymous-read-only-probe",
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
