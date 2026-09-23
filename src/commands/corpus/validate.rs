use super::*;

pub(crate) fn validate(root: &Path) -> Result<CorpusSummary> {
    if !root.is_dir() {
        bail!("corpus input is not a directory: {}", root.display());
    }
    let index_path = contained_file(root, "example-catalogs.json", "canonical corpus index")?;
    let index_bytes = fs::read(&index_path)
        .with_context(|| format!("read canonical corpus index {}", index_path.display()))?;
    let index: Value = serde_json::from_slice(&index_bytes)
        .with_context(|| format!("parse canonical corpus index {}", index_path.display()))?;
    if index.get("schema").and_then(Value::as_str) != Some(INDEX_SCHEMA) {
        bail!("{} is not a canonical {} corpus index", index_path.display(), INDEX_SCHEMA);
    }
    let catalogs = index
        .get("catalogs")
        .and_then(Value::as_array)
        .context("canonical corpus index has no catalogs array")?;
    if catalogs.is_empty() {
        bail!("canonical corpus index contains no catalogs");
    }

    let mut slugs = HashSet::new();
    let mut files = HashSet::from([index_path.clone()]);
    let mut total_references = 0usize;
    for catalog in catalogs {
        let slug = required_text(catalog, "slug", "catalog index entry has no slug")?;
        safe_component(slug, "catalog slug")?;
        if !slugs.insert(slug.to_string()) {
            bail!("canonical corpus index repeats catalog {slug}");
        }
        for key in ["title", "description"] {
            required_text(catalog, key, &format!("catalog {slug} has no {key}"))?;
        }
        let declared_count = catalog
            .get("count")
            .and_then(Value::as_u64)
            .with_context(|| format!("catalog {slug} has no numeric count"))?;
        for key in [
            "image_count",
            "structure_count",
            "complete_record_count",
            "partial_record_count",
        ] {
            if catalog.get(key).and_then(Value::as_u64).is_none() {
                bail!("catalog {slug} has no numeric {key}");
            }
        }
        if catalog.get("measured_provenance").and_then(Value::as_object).is_none() {
            bail!("catalog {slug} has no measured_provenance object");
        }

        let source_relative = required_text(
            catalog,
            "source",
            &format!("catalog {slug} has no source path"),
        )?;
        let references_relative = required_text(
            catalog,
            "full_reference_source",
            &format!("catalog {slug} has no reference index path"),
        )?;
        let source_path = contained_file(root, source_relative, "catalog source")?;
        let references_path = contained_file(root, references_relative, "reference index")?;
        files.insert(source_path.clone());
        files.insert(references_path.clone());
        let source: Value = read_json(&source_path)?;
        let references: Value = read_json(&references_path)?;
        if source.get("schema").and_then(Value::as_str) != Some(SOURCES_SCHEMA)
            || source.get("catalog").and_then(Value::as_str) != Some(slug)
        {
            bail!("{} does not identify canonical catalog {slug}", source_path.display());
        }
        if references.get("schema").and_then(Value::as_str) != Some(REFERENCES_SCHEMA)
            || references.get("catalog").and_then(Value::as_str) != Some(slug)
        {
            bail!("{} does not identify canonical reference catalog {slug}", references_path.display());
        }

        let examples = source
            .get("examples")
            .and_then(Value::as_array)
            .with_context(|| format!("canonical catalog {slug} has no examples array"))?;
        if source.get("count").and_then(Value::as_u64) != Some(examples.len() as u64)
            || declared_count != examples.len() as u64
        {
            bail!("canonical catalog {slug} counts do not match its source examples");
        }
        validate_local_paths(
            &source,
            source_path.parent().context("catalog source has no parent")?,
            &format!("catalog {slug} evidence"),
            &mut files,
        )?;

        let entries = references
            .get("references")
            .and_then(Value::as_array)
            .with_context(|| format!("reference catalog {slug} has no references array"))?;
        if references.get("reference_count").and_then(Value::as_u64) != Some(entries.len() as u64)
            || declared_count != entries.len() as u64
        {
            bail!("reference catalog {slug} counts do not match its references array");
        }
        let catalog_root = references_path.parent().context("reference index has no parent")?;
        let mut record_paths = HashSet::new();
        for entry in entries {
            let relative = required_text(
                entry,
                "path",
                &format!("reference catalog {slug} contains an entry with no path"),
            )?;
            let record_path = contained_file(catalog_root, relative, "reference record")?;
            if !record_paths.insert(record_path.clone()) {
                bail!("reference catalog {slug} repeats record path {relative}");
            }
            files.insert(record_path.clone());
            let record: Value = read_json(&record_path)?;
            if record.get("schema").and_then(Value::as_str) != Some(RECORD_SCHEMA) {
                bail!("{} is not a canonical {} record", record_path.display(), RECORD_SCHEMA);
            }
            let evidence_status = record
                .get("evidence_status")
                .and_then(Value::as_str)
                .with_context(|| format!("{} has no evidence status", record_path.display()))?;
            let evidence_gaps = record
                .get("evidence_gaps")
                .and_then(Value::as_array)
                .with_context(|| format!("{} has no evidence gaps", record_path.display()))?;
            if entry.get("evidence_status").and_then(Value::as_str) != Some(evidence_status)
                || entry.get("evidence_gap_count").and_then(Value::as_u64)
                    != Some(evidence_gaps.len() as u64)
            {
                bail!("reference index facts do not match {}", record_path.display());
            }
            validate_local_paths(
                &record,
                record_path.parent().context("reference record has no parent")?,
                &format!("reference record {}", record_path.display()),
                &mut files,
            )?;
        }
        total_references += entries.len();
    }
    if index.get("catalog_count").and_then(Value::as_u64) != Some(catalogs.len() as u64) {
        bail!("canonical corpus catalog_count does not match its catalogs array");
    }
    if index.get("record_count").and_then(Value::as_u64) != Some(total_references as u64) {
        bail!("canonical corpus record_count does not match its reference indexes");
    }

    Ok(CorpusSummary {
        root: root.to_path_buf(),
        catalog_count: catalogs.len(),
        reference_count: total_references,
        file_count: files.len(),
        index_sha256: Sha256::digest(index_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    })
}

pub(crate) fn required_text<'a>(document: &'a Value, key: &str, message: &str) -> Result<&'a str> {
    document
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .with_context(|| message.to_string())
}

pub(crate) fn safe_component(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
    {
        bail!("{label} is not a safe path component: {value:?}");
    }
    Ok(())
}

pub(crate) fn contained_file(root: &Path, relative: &str, label: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("{label} path is not a contained relative path: {relative}");
    }
    let joined = root.join(path);
    let canonical = joined
        .canonicalize()
        .with_context(|| format!("open {label} {}", joined.display()))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        bail!("{label} escapes the adopted corpus or is not a file: {}", joined.display());
    }
    Ok(canonical)
}

pub(crate) fn read_json(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn validate_local_paths(
    document: &Value,
    root: &Path,
    label: &str,
    files: &mut HashSet<PathBuf>,
) -> Result<()> {
    match document {
        Value::Object(fields) => {
            for (key, value) in fields {
                if matches!(key.as_str(), "local_path" | "motion_path") {
                    let relative = value
                        .as_str()
                        .with_context(|| format!("{label} has a non-string {key}"))?;
                    files.insert(contained_file(root, relative, label)?);
                } else {
                    validate_local_paths(value, root, label, files)?;
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_local_paths(value, root, label, files)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn write_config(summary: &CorpusSummary) -> Result<()> {
    let path = config_path();
    let parent = path.parent().context("corpus location path has no parent")?;

    fs::create_dir_all(parent)
        .with_context(|| format!("create corpus configuration directory {}", parent.display()))?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = path.with_extension(format!("json.{}.{}.tmp", std::process::id(), nonce));
    let body = serde_json::to_vec_pretty(&json!({
        "schema": CONFIG_SCHEMA,
        "root": summary.root,
        "catalog_count": summary.catalog_count,
        "reference_count": summary.reference_count,
        "index_sha256": summary.index_sha256,
    }))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)
        .with_context(|| format!("create corpus location {}", temporary.display()))?;
    file.write_all(&body)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(&temporary, &path)
        .with_context(|| format!("replace corpus location {}", path.display()))?;
    Ok(())
}

pub(crate) fn config_path() -> PathBuf {
    if let Some(root) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(root).join("spis/corpus.json");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config/spis/corpus.json");
    }
    PathBuf::from(".spis/corpus.json")
}
