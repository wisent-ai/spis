use super::*;

pub(crate) const WIDTHS: &[(u32, u32)] = &[(390, 844), (768, 1024), (1440, 1000)];

/// Deviation from the Python original, which used ~/.stado/work: this port
/// keeps all generated working files under ~/.spis/work per harness policy.
/// An unset HOME is refused rather than read as the working directory.
pub(crate) fn work_root() -> anyhow::Result<PathBuf> {
    Ok(crate::commands::docs_corpus::home_dir()?
        .join(".spis")
        .join("work"))
}

pub(crate) fn fail(message: &str) -> ! {
    eprintln!("capture-widths: {message}");
    std::process::exit(1);
}

pub(crate) struct CatalogData {
    pub(crate) directory: PathBuf,
    pub(crate) sources: Value,
    pub(crate) index: Value,
}

/// The catalog, found and its two lists checked by the catalog module that
/// reference-record also reads it through.
pub(crate) fn load_catalog(catalog: &str) -> CatalogData {
    let loaded = (|| -> Result<CatalogData> {
        let directory = crate::commands::reference_record::catalog_dir(catalog)?;
        let sources_file = format!("{}/sources.json", directory.display());
        let references_file = format!("{}/references.json", directory.display());
        let sources: Value =
            crate::read_json(&sources_file).with_context(|| format!("read {sources_file}"))?;
        let index: Value = crate::read_json(&references_file)
            .with_context(|| format!("read {references_file}"))?;
        catalog_list(&sources, "examples", &sources_file)?;
        catalog_list(&index, "references", &references_file)?;
        Ok(CatalogData {
            directory,
            sources,
            index,
        })
    })();
    match loaded {
        Ok(data) => data,
        Err(error) => fail(&format!("{error:#}")),
    }
}

/// Returns (example, entry) for the record matching selector (None, NN, slug,
/// or lowercased name), mirroring the Python pick().
pub(crate) fn pick(
    sources: &Value,
    index: &Value,
    selector: Option<&str>,
) -> Result<(Map<String, Value>, Map<String, Value>)> {
    let examples = catalog_list(sources, "examples", "sources.json")?;
    let references = catalog_list(index, "references", "references.json")?;
    for (position, example) in examples.iter().enumerate() {
        let entry = match references.get(position) {
            Some(reference) => catalog_object(reference, "references.json", "references", position),
            None => bail!(
                "references.json: references has no entry at {position} for examples[{position}] of sources.json"
            ),
        }?;
        let entry = &entry;
        let number = position + 1;
        let slug = slug_of(entry)?;
        let name_lower = example
            .get("name")
            .and_then(Value::as_str)
            .with_context(|| format!("sources.json: examples[{position}] has no \"name\""))?
            .to_lowercase();
        let matched = match selector {
            None => true,
            Some(sel) => sel == number.to_string() || sel == slug || sel == name_lower,
        };
        if matched {
            return Ok((
                catalog_object(example, "sources.json", "examples", position)?,
                entry.clone(),
            ));
        }
    }
    match selector {
        Some(selector) => fail(&format!("record {selector:?} not found")),
        None => fail("sources.json lists no examples, so there is no record to capture"),
    }
}

/// The record directory a reference's `path` lies in; a reference without
/// one is refused, never captured under an empty slug that every such
/// reference would share.
pub(crate) fn slug_of(entry: &Map<String, Value>) -> Result<String> {
    let path = entry
        .get("path")
        .and_then(Value::as_str)
        .with_context(|| format!("reference {} has no \"path\"", Value::Object(entry.clone())))?;
    Path::new(path)
        .parent()
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().to_string())
        .with_context(|| format!("reference path {path:?} is not inside a record directory"))
}

/// Compact UTC stamp like 20260823t123456Z: now_iso_utc() without its date
/// and time separators, the date/time mark lower-cased.
pub(crate) fn compact_stamp() -> String {
    crate::now_iso_utc()
        .chars()
        .filter(|character| !matches!(character, '-' | ':'))
        .map(|character| if character == 'T' { 't' } else { character })
        .collect()
}
