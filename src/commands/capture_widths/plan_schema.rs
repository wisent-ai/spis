use super::*;

pub(crate) const PLAN_SCHEMA: &str = "wisent.weles-capture-plan.v1";

pub(crate) const NAMESPACE: &str = "stado://weles-captures/";

pub(crate) const WIDTHS: &[(u32, u32)] = &[(390, 844), (768, 1024), (1440, 1000)];

pub(crate) const DEFAULT_HOST: &str = "charless-mac-mini";

/// Deviation from the Python original, which used ~/.stado/work: this port
/// keeps all generated working files under ~/.spis/work per harness policy.
pub(crate) fn work_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".spis").join("work")
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
    CatalogData {
        directory,
        sources,
        index,
    }
}

/// Returns (example, entry) for the record matching selector (None, NN, slug,
/// or lowercased name), mirroring the Python pick().
pub(crate) fn pick(
    sources: &Value,
    index: &Value,
    selector: Option<&str>,
) -> (Map<String, Value>, Map<String, Value>) {
    let examples = sources["examples"].as_array().cloned().unwrap_or_default();
    let references = index["references"].as_array().cloned().unwrap_or_default();
    for (position, example) in examples.iter().enumerate() {
        let Some(entry) = references.get(position).and_then(Value::as_object) else {
            continue;
        };
        let number = position + 1;
        let slug = entry["path"]
            .as_str()
            .map(Path::new)
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let name_lower = example["name"].as_str().unwrap_or("").to_lowercase();
        let matched = match selector {
            None => true,
            Some(sel) => sel == number.to_string() || sel == slug || sel == name_lower,
        };
        if matched {
            return (
                example.as_object().cloned().unwrap_or_default(),
                entry.clone(),
            );
        }
    }
    fail(&format!("record {:?} not found", selector.unwrap_or("")));
}

pub(crate) fn slug_of(entry: &Map<String, Value>) -> String {
    entry["path"]
        .as_str()
        .map(Path::new)
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Compact UTC stamp like 20260823t123456Z from now_iso_utc().
pub(crate) fn compact_stamp() -> String {
    // Python: strftime("%Y%m%dt%H%M%SZ")
    let iso = crate::now_iso_utc(); // YYYY-MM-DDTHH:MM:SSZ
    format!(
        "{}{}{}t{}{}{}Z",
        &iso[0..4],
        &iso[5..7],
        &iso[8..10],
        &iso[11..13],
        &iso[14..16],
        &iso[17..19]
    )
}
