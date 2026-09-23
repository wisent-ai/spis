use super::*;

/// Catalog slug -> directory, appending `-examples` when missing.
pub(crate) fn catalog_dir(slug: &str) -> Result<PathBuf> {
    let named = if slug.ends_with("-examples") {
        slug.to_string()
    } else {
        format!("{slug}-examples")
    };
    let directory = PathBuf::from(&named);
    if !directory.is_dir() {
        bail!("reference: {named} does not exist");
    }
    Ok(directory)
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

pub(crate) fn sha256_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(lib::sha256_hex(&bytes))
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
    let examples = sources["examples"].as_array().map(Vec::len).unwrap_or(0);
    sources["count"] = json!(examples);
    sources["visual_count"] = json!(examples);
    sources["structure_count"] = json!(examples);
    lib::write_pretty_json(&directory.join("sources.json").to_string_lossy(), sources)?;

    let references = index["references"].as_array().cloned().unwrap_or_default();
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

/// Locate a record by 1-based number or normalized name; returns its position.
pub(crate) fn find_record(directory: &Path, index: &Value, identifier: &str) -> Result<usize> {
    let number: Option<usize> =
        if !identifier.is_empty() && identifier.chars().all(|c| c.is_ascii_digit()) {
            identifier.parse().ok()
        } else {
            None
        };
    let wanted = identifier.to_lowercase();
    let references = index["references"].as_array().cloned().unwrap_or_default();
    for (position, entry) in references.iter().enumerate() {
        let name_key = entry["name"]
            .as_str()
            .unwrap_or_default()
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
