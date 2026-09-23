use super::*;

pub(crate) fn failure_code(error: &anyhow::Error) -> &'static str {
    error
        .chain()
        .find_map(|cause| {
            cause
                .downcast_ref::<RecordFailure>()
                .map(|failure| failure.code)
        })
        .unwrap_or("desktop_record_failed")
}

pub(crate) fn find_i64(value: &Value, key: &str) -> Option<i64> {
    let object = value.as_object()?;
    let direct = object.get(key).and_then(Value::as_i64);
    let result = object
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get(key))
        .and_then(Value::as_i64);
    match (direct, result) {
        (Some(left), Some(right)) if left != right => None,
        (Some(value), _) | (_, Some(value)) => Some(value),
        _ => None,
    }
}

pub(crate) fn find_array<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    let object = value.as_object()?;
    let direct = object.get(key).and_then(Value::as_array);
    let result = object
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get(key))
        .and_then(Value::as_array);
    match (direct, result) {
        (Some(left), Some(right)) if left != right => None,
        (Some(value), None) | (None, Some(value)) | (Some(value), Some(_)) => Some(value),
        _ => None,
    }
}

/// Anchored, typed view of one cua-driver response. Every read consults only
/// the root object and its `result` child, so a child element, a nested
/// accessibility node or a sibling window list can never answer an identity or
/// ownership question by serde_json map order (finding 13).
pub(crate) struct DriverResponse<'a>(pub(crate) &'a Value);

impl<'a> DriverResponse<'a> {
    pub(crate) fn integer(&self, key: &str) -> Option<i64> {
        find_i64(self.0, key)
    }

    pub(crate) fn text(&self, key: &str) -> Option<&'a str> {
        find_string(self.0, key)
    }

    pub(crate) fn bundle(&self) -> Option<&'a str> {
        ["owner_bundle_id", "bundle_id", "bundle_identifier"]
            .iter()
            .find_map(|key| self.text(key))
    }

    pub(crate) fn elements(&self) -> Option<&'a Vec<Value>> {
        find_array(self.0, "elements")
    }

    pub(crate) fn apps(&self) -> Option<&'a Vec<Value>> {
        find_array(self.0, "apps")
    }
}

/// The window ownership triple, read only from anchored positions.
pub(crate) struct WindowOwnership {
    pub(crate) pid: i64,
    pub(crate) window_id: i64,
    pub(crate) owner_bundle_id: String,
}

impl WindowOwnership {
    pub(crate) fn read(snapshot: &Value) -> Result<Self> {
        let response = DriverResponse(snapshot);
        Ok(Self {
            pid: response
                .integer("pid")
                .context("Cua window snapshot has no anchored owner pid")?,
            window_id: response
                .integer("window_id")
                .context("Cua window snapshot has no anchored window id")?,
            owner_bundle_id: response
                .bundle()
                .context("Cua window snapshot has no anchored owner bundle identifier")?
                .to_string(),
        })
    }
}

/// One entry of an anchored `list_apps` response.
pub(crate) struct DriverApp<'a>(pub(crate) &'a Value);

impl DriverApp<'_> {
    pub(crate) fn pid(&self) -> Option<i64> {
        self.0.get("pid").and_then(Value::as_i64)
    }

    pub(crate) fn bundle(&self) -> Option<&str> {
        ["bundle_id", "bundle_identifier", "owner_bundle_id"]
            .iter()
            .find_map(|key| self.0.get(*key).and_then(Value::as_str))
    }

    pub(crate) fn frontmost(&self) -> bool {
        ["frontmost", "is_frontmost", "active", "is_active"]
            .iter()
            .any(|key| self.0.get(*key).and_then(Value::as_bool) == Some(true))
    }
}

pub(crate) fn find_bool(value: &Value, key: &str) -> Option<bool> {
    let object = value.as_object()?;
    let direct = object.get(key).and_then(Value::as_bool);
    let result = object
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get(key))
        .and_then(Value::as_bool);
    match (direct, result) {
        (Some(left), Some(right)) if left != right => None,
        (Some(value), _) | (_, Some(value)) => Some(value),
        _ => None,
    }
}

pub(crate) fn find_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    let object = value.as_object()?;
    let direct = object.get(key).and_then(Value::as_str);
    let result = object
        .get("result")
        .and_then(Value::as_object)
        .and_then(|result| result.get(key))
        .and_then(Value::as_str);
    match (direct, result) {
        (Some(left), Some(right)) if left != right => None,
        (Some(value), _) | (_, Some(value)) => Some(value),
        _ => None,
    }
}

pub(crate) fn preflight(driver: &CuaDriver) -> Result<()> {
    if std::env::consts::OS != "macos" {
        return Ok(());
    }
    let mut command = Command::new(&driver.path);
    command.args(["permissions", "status", "--json"]);
    let output = super::crawl::bounded_command_output(
        &mut command,
        "Cua Driver permission status",
        Duration::from_secs(15),
        1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "cua-driver permission status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let status: Value =
        serde_json::from_slice(&output.stdout).context("parse Cua Driver permission status")?;
    let accessibility =
        find_bool(&status, "accessibility").or_else(|| find_bool(&status, "accessibility_granted"));
    let screen = find_bool(&status, "screen_recording")
        .or_else(|| find_bool(&status, "screen_recording_granted"));
    if accessibility != Some(true) {
        bail!("Cua Driver accessibility permission is unavailable on the selected host");
    }
    if screen != Some(true) {
        bail!("Cua Driver screen-recording permission is unavailable on the selected host");
    }
    Ok(())
}

pub(crate) fn records(catalog: &str, selected: Option<&str>) -> Result<Vec<Record>> {
    if !matches!(catalog, "macos-app-examples" | "desktop-app-examples") {
        bail!("crawl-desktop accepts macos-app-examples or desktop-app-examples");
    }
    let directory = super::corpus::data_root()
        .join(catalog)
        .join("references");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    paths.sort();
    let mut records = Vec::new();
    for path in paths {
        let slug = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if selected.is_some_and(|selected| {
            selected != slug
                && selected != slug.split_once('-').map(|(_, tail)| tail).unwrap_or(slug)
        }) {
            continue;
        }
        let record: Value = serde_json::from_slice(&std::fs::read(path.join("reference.json"))?)?;
        records.push(Record {
            slug: slug.to_string(),
            name: record
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        });
    }
    if records.is_empty() {
        bail!("no matching records in {catalog}");
    }
    Ok(records)
}
