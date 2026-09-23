use super::*;

pub(crate) fn decode_runtime_manifest(
    encoded: &str,
    catalog: &str,
    engine: &str,
    record: Option<&str>,
) -> Result<RuntimeManifest> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let manifest: RuntimeManifest = serde_json::from_slice(
        &STANDARD.decode(encoded).context("runtime manifest is not valid base64")?,
    )?;
    if manifest.schema != RUNTIME_MANIFEST_SCHEMA
        || manifest.catalog != catalog
        || manifest.engine != engine
        || record.is_some_and(|value| value != manifest.record)
        || manifest.source_revision != build_revision()?
    {
        bail!("runtime manifest does not match this exact worker, record, engine and source revision");
    }
    safe_component(&manifest.run_id, "run id")?;
    safe_component(&manifest.catalog, "catalog")?;
    safe_component(&manifest.record, "record")?;
    safe_component(&manifest.attempt_id, "attempt id")?;
    if manifest.runtime_product.identifier.is_empty()
        || manifest.source_input_sha256.len() != 64
        || manifest.correlation_id.is_empty()
        || manifest.stado_run_id.is_empty()
        || manifest.execution_identity.is_none()
    {
        bail!("runtime manifest is incomplete and cannot authorize a worker");
    }
    if manifest.attempt == 0 || manifest.attempt_id.is_empty() {
        bail!("runtime manifest has no immutable execution attempt");
    }
    let execution_identity = manifest
        .execution_identity
        .as_ref()
        .context("runtime manifest has no execution identity")?;
    if execution_identity.host.is_empty() || execution_identity.observed_hostname.is_empty() {
        bail!("runtime manifest execution identity lacks registry host alias or observed hostname");
    }
    if manifest.engine == "web" {
        let service = manifest
            .service_identity
            .as_ref()
            .context("web runtime manifest has no exact Weles service identity")?;
        if service.name != "weles-admission"
            || service.consumer != "spis"
            || service.capability != "browser-evidence"
            || service.action != "generic_browser_task"
            || service.active_host != execution_identity.host
        {
            bail!("web runtime manifest Weles service identity is invalid");
        }
        let (_, current) = registry_placements()?;
        if serde_json::to_value(current)? != serde_json::to_value(&manifest.service_identity)? {
            bail!("Weles service directory generation or exact placement changed after planning");
        }
    } else if manifest.service_identity.is_some() {
        bail!("non-web runtime manifest cannot bind a Weles service identity");
    }
    let bindings = runtime_bindings_for_worker(&manifest)?;
    if bindings.sha256 != manifest.bindings_file_sha256 {
        bail!("runtime bindings whole-file digest differs from the immutable manifest");
    }
    let authoritative = runtime_binding(
        &bindings,
        &manifest.catalog,
        &manifest.engine,
        &manifest.record,
    )?;
    let authoritative_sha256 =
        crate::sha256_hex(&serde_json::to_vec(&authoritative)?);
    if authoritative_sha256 != manifest.bindings_sha256 {
        bail!("normalized catalog+record binding digest differs from the immutable manifest");
    }
    if serde_json::to_value(&authoritative.account)? != serde_json::to_value(&manifest.account)?
        || serde_json::to_value(&authoritative.constraints)?
            != serde_json::to_value(&manifest.constraints)?
        || serde_json::to_value(&authoritative.delivery)?
            != serde_json::to_value(&manifest.delivery)?
        || serde_json::to_value(&authoritative.prepared_proof)?
            != serde_json::to_value(&manifest.prepared_proof)?
        || serde_json::to_value(&authoritative.surface)?
            != serde_json::to_value(&manifest.runtime_product.surface)?
    {
        bail!("runtime manifest differs from the exact committed catalog+record binding");
    }
    let reference_path = reference_path(&manifest.catalog, &manifest.record)?;
    let reference_bytes = std::fs::read(&reference_path)
        .with_context(|| format!("read committed worker record {}", reference_path.display()))?;
    let reference: Value = serde_json::from_slice(&reference_bytes)?;
    let expected_product = runtime_product(
        &manifest.catalog,
        &manifest.engine,
        &manifest.record,
        &reference,
        manifest.runtime_product.surface.clone(),
    )?;
    let expected_docs_structure =
        docs_structure_sha256(&manifest.catalog, &manifest.record, &manifest.engine)?;
    if expected_docs_structure != manifest.docs_structure_sha256 {
        bail!("docs crawl definition digest differs from the committed exact structure");
    }
    let execution = manifest.execution_identity.as_ref().expect("checked above");
    let resolving_engine = matches!(manifest.engine.as_str(), "desktop" | "cli" | "tui");
    // The declared identity is never rewritten, so this comparison covers every
    // engine — including desktop, cli and tui, whose resolved `identifier` is a
    // bundle id or binary name that legitimately differs from the record.
    if expected_product.declared_identifier != manifest.runtime_product.declared_identifier
        || execution.resolved_product_identifier != manifest.runtime_product.identifier
        || expected_product.product_url != manifest.runtime_product.product_url
        || expected_product.surface != manifest.runtime_product.surface
    {
        bail!(
            "runtime product declared identity, URL, surface or resolved execution identity differs from the committed record"
        );
    }
    if resolving_engine && !is_host_query_literal(&manifest.runtime_product.declared_identifier) {
        bail!("declared runtime identity is not a safe host resolution literal");
    }
    if !resolving_engine && expected_product.kind != manifest.runtime_product.kind {
        bail!("runtime product kind differs from the committed record");
    }
    match manifest.engine.as_str() {
        "mobile" | "web" | "docs" => {
            if serde_json::to_value(&expected_product)?
                != serde_json::to_value(&manifest.runtime_product)?
            {
                bail!("canonical runtime product differs from the committed record");
            }
        }
        "cli" => {
            let path = execution.executable_path.as_deref().context("CLI identity has no path")?;
            let digest = execution
                .executable_sha256
                .as_deref()
                .context("CLI identity has no executable digest")?;
            if expected_product.identifier != manifest.runtime_product.identifier
                || manifest.runtime_product.kind != "cli-binary"
                || manifest.runtime_product.identity_source
                    != format!("typed isolated host path resolution: {path}; sha256={digest}")
            {
                bail!("canonical CLI runtime product is invalid");
            }
        }
        "tui" => {
            let path = execution.executable_path.as_deref().context("TUI identity has no path")?;
            let digest = execution
                .executable_sha256
                .as_deref()
                .context("TUI identity has no executable digest")?;
            if execution.device_name.as_deref() != Some(expected_product.identifier.as_str())
                || manifest.runtime_product.kind != "tui-binary"
                || manifest.runtime_product.identity_source
                    != format!("typed isolated host path resolution: {path}; sha256={digest}")
            {
                bail!("canonical TUI runtime product is invalid");
            }
        }
        "desktop" => {
            let path = execution
                .executable_path
                .as_deref()
                .context("desktop identity has no executable path")?;
            let version = execution
                .product_version
                .as_deref()
                .context("desktop identity has no product version")?;
            let digest = execution
                .executable_sha256
                .as_deref()
                .context("desktop identity has no executable digest")?;
            let app_path = path
                .split_once("/Contents/MacOS/")
                .map(|(value, _)| value)
                .context("desktop executable is outside a canonical app bundle")?;
            if execution.device_name.as_deref() != Some(expected_product.identifier.as_str())
                || manifest.runtime_product.kind != "desktop-bundle"
                || manifest.runtime_product.identity_source
                    != format!("typed host display-name resolution: bundle={app_path}; executable={path}; version={version}; sha256={digest}")
            {
                bail!("canonical desktop runtime product is invalid");
            }
        }
        _ => bail!("unsupported runtime engine"),
    }
    let mut recomputed = manifest.clone();
    finalize_manifest_identity(&mut recomputed, &reference_bytes)?;
    if serde_json::to_value(&recomputed)? != serde_json::to_value(&manifest)? {
        bail!("runtime manifest identity, input digest, keys or artifact URIs are not canonical");
    }
    Ok(manifest)
}

pub(crate) fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn safe_component(value: &str, name: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || matches!(value, "." | "..")
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
        })
    {
        bail!("{name} must be one strict ASCII path component of at most 128 bytes");
    }
    Ok(())
}

pub(crate) fn catalog_root(catalog: &str) -> Result<PathBuf> {
    safe_component(catalog, "catalog")?;
    if !CATALOGS.iter().any(|(known, _)| *known == catalog) {
        bail!("unknown crawl catalog {catalog}");
    }
    Ok(super::corpus::data_root().join(catalog))
}

pub(crate) fn reference_path(catalog: &str, record: &str) -> Result<PathBuf> {
    safe_component(record, "record")?;
    Ok(catalog_root(catalog)?
        .join("references")
        .join(record)
        .join("reference.json"))
}

pub(crate) fn run_root() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").context("HOME is required for durable crawl run state")?;
    Ok(PathBuf::from(home)
        .join(".stado")
        .join("work")
        .join("spis")
        .join("crawl-runs"))
}

pub(crate) fn legacy_run_root() -> PathBuf {
    source_root().join(".wisent-output").join("crawl-runs")
}

pub(crate) fn migrate_run_state(run_id: Option<&str>) -> Result<()> {
    let legacy = legacy_run_root();
    if !legacy.is_dir() {
        return Ok(());
    }
    let selected = if let Some(run_id) = run_id {
        safe_component(run_id, "run id")?;
        vec![run_id.to_string()]
    } else {
        std::fs::read_dir(&legacy)?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().join("run.json").is_file())
            .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
            .collect()
    };
    for id in selected {
        safe_component(&id, "run id")?;
        let source = legacy.join(&id).join("run.json");
        let destination = run_root()?.join(&id).join("run.json");
        if !source.is_file() || destination.is_file() {
            continue;
        }
        let document: Value =
            crate::read_json(source.to_str().context("legacy run path is not UTF-8")?)?;
        atomic_json_write(&destination, &document)?;
        let recovered: Value =
            crate::read_json(destination.to_str().context("migrated run path is not UTF-8")?)?;
        if recovered != document {
            bail!("legacy crawl run {id} migration read-back differs");
        }
    }
    Ok(())
}

pub(crate) fn run_path(run_id: &str) -> Result<PathBuf> {
    safe_component(run_id, "run id")?;
    Ok(run_root()?.join(run_id).join("run.json"))
}
