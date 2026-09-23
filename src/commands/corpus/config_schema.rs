use super::*;

pub(crate) const CONFIG_SCHEMA: &str = "spis.corpus-location.v1";

pub(crate) const INDEX_SCHEMA: &str = "wisent.example-catalog.v2";

pub(crate) const SOURCES_SCHEMA: &str = "wisent.example-catalog.v2";

pub(crate) const REFERENCES_SCHEMA: &str = "wisent.full-reference-catalog.v2";

pub(crate) const RECORD_SCHEMA: &str = "wisent.full-product-reference.v2";

#[derive(Debug)]
pub(crate) struct CorpusSummary {
    pub(crate) root: PathBuf,
    pub(crate) catalog_count: usize,
    pub(crate) reference_count: usize,
    pub(crate) file_count: usize,
    pub(crate) index_sha256: String,
}

pub fn run(rest: &[String]) -> Result<()> {
    match rest.first().map(String::as_str) {
        Some("adopt") => adopt(&rest[1..]),
        Some("status") => status(&rest[1..]),
        Some("--help") | Some("-h") | None => {
            println!("usage: spis corpus <adopt PATH|status>\n\n  adopt PATH  validate and remember an unpacked canonical Spis corpus\n  status      print the currently adopted corpus and its measured counts\n\nZIP, tar, gzip, bzip2, xz, zstd, 7z and rar archives are not accepted; unpack them first so every provenance and evidence file can be validated.");
            Ok(())
        }
        Some(other) => bail!("unknown corpus action: {other} (expected adopt or status)"),
    }
}

pub fn activate_configured_root() -> Result<()> {
    let Some(root) = configured_root()? else { return Ok(()) };
    if !root.is_dir() {
        bail!("the adopted Spis corpus is unavailable at {}; run `spis corpus adopt PATH` with an existing unpacked corpus", root.display());
    }
    std::env::set_current_dir(&root)
        .with_context(|| format!("open adopted Spis corpus {}", root.display()))
}

pub fn data_root() -> PathBuf {
    configured_root()
        .ok()
        .flatten()
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

pub fn configured_root() -> Result<Option<PathBuf>> {
    let path = config_path();
    if !path.exists() {
        return Ok(None);
    }
    let document: Value = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("read corpus location {}", path.display()))?,
    )
    .with_context(|| format!("parse corpus location {}; run `spis corpus adopt PATH` to replace it", path.display()))?;
    if document.get("schema").and_then(Value::as_str) != Some(CONFIG_SCHEMA) {
        bail!("corpus location {} has an unsupported schema; run `spis corpus adopt PATH` to replace it", path.display());
    }
    let root = document
        .get("root")
        .and_then(Value::as_str)
        .context("corpus location has no root; run `spis corpus adopt PATH` to replace it")?;
    Ok(Some(PathBuf::from(root)))
}

pub(crate) fn adopt(rest: &[String]) -> Result<()> {
    if rest.len() != 1 {
        bail!("usage: spis corpus adopt PATH");
    }
    let supplied = PathBuf::from(&rest[0]);
    if supplied.is_file() {
        let extension = supplied.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
        if ["zip", "tar", "tgz", "gz", "bz2", "xz", "zst", "7z", "rar"].contains(&extension.as_str()) {
            bail!("archives are not supported corpus inputs: {}; unpack the archive and adopt its corpus directory", supplied.display());
        }
        bail!("unsupported corpus input at {}; choose a directory containing example-catalogs.json", supplied.display());
    }
    let root = supplied
        .canonicalize()
        .with_context(|| format!("open corpus directory {}", supplied.display()))?;
    let summary = validate(&root)?;
    let previous = configured_root().ok().flatten();
    let unchanged = previous.as_deref() == Some(summary.root.as_path());
    write_config(&summary)?;
    let onboarding_warning = crate::onboarding::record_first_success()
        .err()
        .map(|error| format!("The corpus is active, but first-use completion could not be saved: {error:#}"));
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "spis.corpus-adoption.v1",
            "state": if unchanged { "unchanged" } else { "adopted" },
            "root": summary.root,
            "catalogs": summary.catalog_count,
            "references": summary.reference_count,
            "files": summary.file_count,
            "index_sha256": summary.index_sha256,
            "imported": if unchanged { 0 } else { summary.reference_count },
            "unchanged": if unchanged { summary.reference_count } else { 0 },
            "conflicting": 0,
            "rejected": 0,
            "message": if unchanged {
                "This exact corpus was already adopted; no catalog or reference was duplicated."
            } else {
                "The corpus was accepted in place. Spis will use its original provenance and evidence files for subsequent commands."
            },
            "onboarding_warning": onboarding_warning,
        }))?
    );
    Ok(())
}

pub(crate) fn status(rest: &[String]) -> Result<()> {
    if !rest.is_empty() {
        bail!("usage: spis corpus status");
    }
    let root = configured_root()?.context("no corpus has been adopted; run `spis corpus adopt PATH`")?;
    let summary = validate(&root)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": CONFIG_SCHEMA,
            "root": summary.root,
            "catalogs": summary.catalog_count,
            "references": summary.reference_count,
            "files": summary.file_count,
            "index_sha256": summary.index_sha256,
        }))?
    );
    Ok(())
}
