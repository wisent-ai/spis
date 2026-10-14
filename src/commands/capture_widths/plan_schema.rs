use super::*;

pub(crate) const PLAN_SCHEMA: &str = "wisent.weles-capture-plan.v1";

pub(crate) const NAMESPACE: &str = "stado://weles-captures/";

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

pub(crate) fn load_catalog(catalog: &str) -> CatalogData {
    let name = if catalog.ends_with("-examples") {
        catalog.to_string()
    } else {
        format!("{catalog}-examples")
    };
    let directory = PathBuf::from(&name);
    if !directory.is_dir() {
        fail(&format!("{name} does not exist"));
    }
    let sources = match crate::read_json(directory.join("sources.json").to_str().unwrap()) {
        Ok(v) => v,
        Err(e) => fail(&format!("read {name}/sources.json: {e:#}")),
    };
    let index = match crate::read_json(directory.join("references.json").to_str().unwrap()) {
        Ok(v) => v,
        Err(e) => fail(&format!("read {name}/references.json: {e:#}")),
    };
    list_field(&sources, "examples", &format!("{name}/sources.json"));
    list_field(&index, "references", &format!("{name}/references.json"));
    CatalogData {
        directory,
        sources,
        index,
    }
}

/// The list under `key` of a catalog file. A file without it, or holding
/// something else there, is refused naming the file and what it holds: a
/// catalog read as empty would answer "nothing to capture" for a file
/// that is damaged, not empty.
pub(crate) fn list_field<'a>(document: &'a Value, key: &str, file: &str) -> &'a [Value] {
    match document.get(key) {
        Some(Value::Array(items)) => items,
        Some(other) => fail(&format!(
            "{file}: {key} holds {other} where a list is expected"
        )),
        None => fail(&format!("{file}: {key} is missing")),
    }
}

/// The object at `position` of the list under `key` of a catalog file; any
/// other value there is refused naming the file, the list and the position.
pub(crate) fn object_entry(
    value: &Value,
    file: &str,
    key: &str,
    position: usize,
) -> Result<Map<String, Value>> {
    match value.as_object() {
        Some(object) => Ok(object.clone()),
        None => bail!("{file}: {key}[{position}] holds {value} where a record object is expected"),
    }
}

/// Returns (example, entry) for the record matching selector (None, NN, slug,
/// or lowercased name), mirroring the Python pick().
pub(crate) fn pick(
    sources: &Value,
    index: &Value,
    selector: Option<&str>,
) -> Result<(Map<String, Value>, Map<String, Value>)> {
    let examples = list_field(sources, "examples", "sources.json");
    let references = list_field(index, "references", "references.json");
    for (position, example) in examples.iter().enumerate() {
        let entry = match references.get(position) {
            Some(reference) => object_entry(reference, "references.json", "references", position),
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
                object_entry(example, "sources.json", "examples", position)?,
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
