use super::*;

/// Catalog slug -> directory, appending `-examples` when missing. Every
/// command that reads a catalog (reference-record, capture-widths) finds it
/// through this.
pub(crate) fn catalog_dir(slug: &str) -> Result<PathBuf> {
    let named = if slug.ends_with("-examples") {
        slug.to_string()
    } else {
        format!("{slug}-examples")
    };
    let directory = PathBuf::from(&named);
    if !directory.is_dir() {
        bail!("catalog {named} does not exist");
    }
    Ok(directory)
}

/// The list under `key` of a catalog file. A file without it, or holding
/// something else there, is refused naming the file and what it holds: a
/// catalog read as empty would answer "not found" or "nothing to capture"
/// for a file that is damaged, not empty.
pub(crate) fn catalog_list<'a>(document: &'a Value, key: &str, file: &str) -> Result<&'a [Value]> {
    match document.get(key) {
        Some(Value::Array(items)) => Ok(items),
        Some(other) => bail!("{file}: {key} holds {other} where a list is expected"),
        None => bail!("{file}: {key} is missing"),
    }
}

/// The object at `position` of the list under `key` of a catalog file; any
/// other value there is refused naming the file, the list and the position.
pub(crate) fn catalog_object(
    value: &Value,
    file: &str,
    key: &str,
    position: usize,
) -> Result<serde_json::Map<String, Value>> {
    match value.as_object() {
        Some(object) => Ok(object.clone()),
        None => bail!("{file}: {key}[{position}] holds {value} where a record object is expected"),
    }
}

pub(crate) fn kebab(name: &str) -> Result<String> {
    let mut slug = String::new();
    let mut prev_sep = true;
    for c in name.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            slug.push(c);
            prev_sep = false;
        } else if !prev_sep && !slug.is_empty() {
            slug.push('-');
            prev_sep = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        bail!("reference: --name {name:?} produces an empty slug");
    }
    Ok(slug)
}

/// Image dimensions via macOS `sips`, exactly like the original script.
pub(crate) fn image_dimensions(path: &Path) -> Result<(i64, i64)> {
    let output = std::process::Command::new("sips")
        .args(["-g", "pixelWidth", "-g", "pixelHeight"])
        .arg(path)
        .output()
        .context("run sips")?;
    if !output.status.success() {
        bail!(
            "reference: cannot read image dimensions: {}",
            path.display()
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut width = None;
    let mut height = None;
    for line in stdout.lines() {
        if line.contains("pixelWidth") {
            width = line.rsplit(':').next().and_then(|v| v.trim().parse().ok());
        }
        if line.contains("pixelHeight") {
            height = line.rsplit(':').next().and_then(|v| v.trim().parse().ok());
        }
    }
    match (width, height) {
        (Some(w), Some(h)) => Ok((w, h)),
        _ => bail!(
            "reference: cannot read image dimensions: {}",
            path.display()
        ),
    }
}

pub(crate) fn today_iso() -> String {
    lib::now_iso_utc()[..10].to_string()
}

/// Refresh the counters on both catalog files and regenerate the rendered indexes.
pub(crate) fn save_all(directory: &Path, sources: &mut Value, index: &mut Value) -> Result<()> {
    // A catalog file without its list is refused, never written back with
    // its counters set to nothing.
    let sources_file = format!("{}/sources.json", directory.display());
    let examples = catalog_list(sources, "examples", &sources_file)?.len();
    sources["count"] = json!(examples);
    sources["visual_count"] = json!(examples);
    sources["structure_count"] = json!(examples);
    lib::write_pretty_json(&directory.join("sources.json").to_string_lossy(), sources)?;

    let references_file = format!("{}/references.json", directory.display());
    let references = catalog_list(index, "references", &references_file)?.to_vec();
    index["reference_count"] = json!(references.len());
    index["complete_count"] = json!(references
        .iter()
        .filter(|r| r["evidence_status"] == "complete")
        .count());
    index["partial_count"] = json!(references
        .iter()
        .filter(|r| r["evidence_status"] == "partial")
        .count());
    let now = lib::now_iso_utc();
    index["generated_at"] = json!(now.clone());
    index["measured_at"] = json!(now);
    lib::write_pretty_json(&directory.join("references.json").to_string_lossy(), index)?;

    super::reference_contract::regenerate_index()
}

/// The record file of the reference at `position` in `index`; an entry
/// without its path is refused naming the catalog, never read as the
/// catalog directory itself.
pub(crate) fn record_path(directory: &Path, index: &Value, position: usize) -> Result<PathBuf> {
    let path = index["references"][position]["path"]
        .as_str()
        .with_context(|| {
            format!(
                "reference: references[{position}].path is missing in {}",
                directory.display()
            )
        })?;
    Ok(directory.join(path))
}

/// Locate a record by 1-based number or normalized name; returns its position.
pub(crate) fn find_record(directory: &Path, index: &Value, identifier: &str) -> Result<usize> {
    let number: Option<usize> =
        if !identifier.is_empty() && identifier.chars().all(|c| c.is_ascii_digit()) {
            identifier.parse().ok()
        } else {
            None
        };
    let wanted = identifier.to_lowercase();
    let references_file = format!("{}/references.json", directory.display());
    let references = catalog_list(index, "references", &references_file)?;
    for (position, entry) in references.iter().enumerate() {
        let name_key = entry["name"]
            .as_str()
            .with_context(|| {
                format!(
                    "{}/references.json entry {position} names no record",
                    directory.display()
                )
            })?
            .to_lowercase()
            .replace('_', "-")
            .replace(' ', "-");
        if number == Some(position + 1) || name_key == wanted {
            return Ok(position);
        }
    }
    bail!(
        "reference: record {:?} not found in {}",
        identifier,
        directory.display()
    )
}

#[derive(Debug, Clone)]
pub struct AddArgs {
    pub catalog: String,
    pub name: String,
    pub source_url: String,
    pub category: String,
    pub selection_note: String,
    pub visual: String,
    pub owner: Option<String>,
}

/// The sources.json example of the record at `position`; a sources.json
/// without one there is refused naming the file, never read as null or
/// removed past its end.
pub(crate) fn catalog_example<'a>(
    directory: &Path,
    sources: &'a Value,
    position: usize,
) -> Result<&'a Value> {
    let file = format!("{}/sources.json", directory.display());
    catalog_list(sources, "examples", &file)?
        .get(position)
        .with_context(|| {
            format!("{file} has no example at position {position} though references.json does")
        })
}

/// One reference record of a catalog: its directory slug, the name the record
/// states and its `reference.json`.
#[derive(Clone, Debug)]
pub(crate) struct CatalogRecord {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
}

/// The records under the adopted corpus's `<catalog>/references`, by slug,
/// narrowed to `selected` (a whole slug or the slug after its number). Every
/// crawl lists its records here. A directory or record that cannot be read, a
/// directory name that is not UTF-8 and a record that states no name are
/// refused naming the path; none is listed under an empty slug or name.
pub(crate) fn catalog_records(catalog: &str, selected: Option<&str>) -> Result<Vec<CatalogRecord>> {
    let directory = crate::commands::corpus::data_root()
        .join(catalog)
        .join("references");
    let mut paths = std::fs::read_dir(&directory)
        .with_context(|| format!("read {}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("list {}", directory.display()))?;
    paths.retain(|path| path.is_dir());
    paths.sort();
    let mut records = Vec::new();
    for path in paths {
        let slug = path
            .file_name()
            .and_then(|name| name.to_str())
            .with_context(|| format!("{} is not a UTF-8 record directory", path.display()))?
            .to_string();
        let tail = slug.split_once('-').map(|(_, tail)| tail);
        if selected.is_some_and(|value| value != slug && Some(value) != tail) {
            continue;
        }
        let path = path.join("reference.json");
        let document: Value = lib::read_json(&path)?;
        let name = document
            .get("name")
            .and_then(Value::as_str)
            .with_context(|| format!("{} states no name", path.display()))?
            .to_string();
        records.push(CatalogRecord { slug, name, path });
    }
    Ok(records)
}
