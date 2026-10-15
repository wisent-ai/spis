use super::*;

pub(crate) const CONFIG_SCHEMA: &str = "spis.corpus-location.v1";

pub(crate) use crate::commands::reference_contract::{
    CATALOG_SCHEMA as INDEX_SCHEMA, CATALOG_SCHEMA as SOURCES_SCHEMA,
    INDEX_SCHEMA as REFERENCES_SCHEMA, RECORD_SCHEMA,
};

#[derive(Debug)]
pub(crate) struct CorpusSummary {
    pub(crate) root: PathBuf,
    pub(crate) catalog_count: usize,
    pub(crate) reference_count: usize,
    pub(crate) file_count: usize,
    pub(crate) index_sha256: String,
}

pub fn run(rest: &[String]) -> Result<()> {
    let action = rest.split_first();
    match action.map(|(name, args)| (name.as_str(), args)) {
        Some(("adopt", args)) => adopt(args),
        Some(("status", args)) => status(args),
        Some(("--help", _)) | Some(("-h", _)) | None => {
            println!("usage: spis corpus <adopt PATH|status>\n\n  adopt PATH  validate and remember an unpacked canonical Spis corpus\n  status      print the currently adopted corpus and its measured counts\n\nZIP, tar, gzip, bzip2, xz, zstd, 7z and rar archives are not accepted; unpack them first so every provenance and evidence file can be validated.");
            Ok(())
        }
        Some((other, _)) => Err(crate::commands::usage(format!(
            "unknown corpus action: {other} (expected adopt or status)"
        ))),
    }
}

pub fn activate_configured_root() -> Result<()> {
    let Some(root) = configured_root()? else {
        return Ok(());
    };
    if !root.is_dir() {
        bail!("the adopted Spis corpus is unavailable at {}; run `spis corpus adopt PATH` with an existing unpacked corpus", root.display());
    }
    std::env::set_current_dir(&root)
        .with_context(|| format!("open adopted Spis corpus {}", root.display()))
}

/// The adopted corpus, or the source checkout when none was adopted. A
/// corpus location that exists but cannot be read or parsed stops Spis with
/// its cause: answered with the source checkout, every reader silently
/// worked on a different corpus than the one the operator adopted.
pub fn data_root() -> PathBuf {
    match configured_root() {
        Ok(Some(root)) => root,
        Ok(None) => PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        Err(error) => panic!("the adopted Spis corpus cannot be located: {error:#}"),
    }
}

pub fn configured_root() -> Result<Option<PathBuf>> {
    let path = config_path()?;
    // `exists` answers false when the directory cannot be searched, which
    // read an adopted corpus as never adopted.
    if !path
        .try_exists()
        .with_context(|| format!("check corpus location {}", path.display()))?
    {
        return Ok(None);
    }
    let document: Value = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("read corpus location {}", path.display()))?,
    )
    .with_context(|| {
        format!(
            "parse corpus location {}; run `spis corpus adopt PATH` to replace it",
            path.display()
        )
    })?;
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
    let [supplied] = rest else {
        return Err(crate::commands::usage("usage: spis corpus adopt PATH"));
    };
    let supplied = PathBuf::from(supplied);
    if supplied.is_file() {
        let archive = supplied
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| {
                ["zip", "tar", "tgz", "gz", "bz2", "xz", "zst", "7z", "rar"]
                    .contains(&extension.to_ascii_lowercase().as_str())
            });
        if archive {
            bail!("archives are not supported corpus inputs: {}; unpack the archive and adopt its corpus directory", supplied.display());
        }
        bail!(
            "unsupported corpus input at {}; choose a directory containing example-catalogs.json",
            supplied.display()
        );
    }
    let root = supplied
        .canonicalize()
        .with_context(|| format!("open corpus directory {}", supplied.display()))?;
    let summary = validate(&root)?;
    // A location that cannot be read is replaced by this adoption, and the
    // cause it could not be read for is reported rather than dropped.
    let previous = match configured_root() {
        Ok(previous) => previous,
        Err(error) => {
            eprintln!(
                "spis: the previous corpus location could not be read and is replaced: {error:#}"
            );
            None
        }
    };
    let unchanged = previous.as_deref() == Some(summary.root.as_path());
    let n = summary.reference_count;
    // Every reference was imported now or already adopted (https://spis.wisent.com/docs/corpus-adoption).
    let (imported, kept) = if unchanged { (0, n) } else { (n, 0) };
    // validate() refused any conflicting or rejected catalog, so both are zero (https://spis.wisent.com/docs/corpus-adoption).
    let (conflicting, rejected) = (0, 0);
    write_config(&summary)?;
    let onboarding_warning = crate::onboarding::record_first_success()
        .err()
        .map(|error| {
            format!("The corpus is active, but first-use completion could not be saved: {error:#}")
        });
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
            "imported": imported,
            "unchanged": kept,
            "conflicting": conflicting,
            "rejected": rejected,
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
        return Err(crate::commands::usage("usage: spis corpus status"));
    }
    let root =
        configured_root()?.context("no corpus has been adopted; run `spis corpus adopt PATH`")?;
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
