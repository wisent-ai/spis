//! Where crawl state lives: the checkout, a catalog's records, the run
//! documents, and the move of runs out of the legacy in-checkout location.
//! Every name that becomes a path component is checked here first.

use super::*;

pub(crate) fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn safe_component(value: &str, name: &str) -> Result<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        bail!("{name} must be one strict ASCII path component");
    }
    Ok(())
}

pub(crate) fn catalog_root(catalog: &str) -> Result<PathBuf> {
    safe_component(catalog, "catalog")?;
    if crate::commands::reference_contract::named::<crate::commands::reference_contract::Family>(
        catalog,
    )
    .is_none()
    {
        bail!("unknown crawl catalog {catalog}");
    }
    Ok(crate::commands::corpus::data_root().join(catalog))
}

pub(crate) fn reference_path(catalog: &str, record: &str) -> Result<PathBuf> {
    safe_component(record, "record")?;
    Ok(catalog_root(catalog)?
        .join("references")
        .join(record)
        .join("reference.json"))
}

pub(crate) fn run_root() -> Result<PathBuf> {
    crate::commands::docs_corpus::work_area("crawl-runs")
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
        // An entry that cannot be read is refused: skipped, its run was left
        // unmigrated and could not be loaded afterwards.
        std::fs::read_dir(&legacy)
            .with_context(|| format!("list legacy crawl runs under {}", legacy.display()))?
            .collect::<std::io::Result<Vec<_>>>()
            .with_context(|| format!("read an entry of {}", legacy.display()))?
            .into_iter()
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
        let document: Value = crate::read_json(&source)?;
        atomic_json_write(&destination, &document)?;
        let recovered: Value = crate::read_json(&destination)?;
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
