use super::*;

/// One helper binary, pinned exactly once per record.
#[derive(Clone, Debug)]
pub(crate) struct PinnedHelper {
    pub(crate) path: PathBuf,
    pub(crate) sha256: String,
    pub(crate) version: String,
}

/// Resolve the readiness helper from absolute directories only. The inherited
/// PATH never participates, the path must be canonical and must not be a
/// symlink, and its digest and version are retained, so no writable earlier
/// PATH entry can substitute the binary that observes device readiness
/// (finding 10).
pub(crate) fn pinned_readiness_helper() -> Result<PinnedHelper> {
    const PROGRAM: &str = "stado-runtime-readiness";
    let path = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"]
        .iter()
        .map(|directory| Path::new(directory).join(PROGRAM))
        .find(|candidate| candidate.is_file())
        .with_context(|| {
            format!("{PROGRAM} is absent from the pinned absolute helper directories")
        })?;
    if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
        bail!(
            "pinned mobile readiness helper {} is a symlink",
            path.display()
        );
    }
    let canonical = std::fs::canonicalize(&path)?;
    if canonical != path {
        bail!(
            "pinned mobile readiness helper is not canonical: declared {}, canonical {}",
            path.display(),
            canonical.display()
        );
    }
    let sha256 = crate::sha256_file(&path)?;
    let mut version_command = Command::new(&path);
    version_command
        .arg("--version")
        .env_clear()
        .env("PATH", "/usr/bin:/bin");
    let output = crate::commands::crawl::command_output(
        &mut version_command,
        "read pinned mobile readiness helper version",
    )?;
    if !output.status.success() {
        bail!(
            "pinned mobile readiness helper {} refused --version: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(PinnedHelper {
        path,
        sha256,
        version: String::from_utf8_lossy(&output.stdout).trim().to_string(),
    })
}

pub(crate) fn readiness_observation(
    manifest: &super::crawl::RuntimeManifest,
    app_id: &str,
    helper: &PinnedHelper,
) -> Result<Value> {
    let execution = manifest
        .execution_identity
        .as_ref()
        .context("mobile runtime manifest has no exact execution identity")?;
    let bound = crate::commands::crawl::bound_proof(manifest, execution, app_id)?;
    let mut readiness = Command::new(&helper.path);
    readiness.args(bound.verify_args());
    let output = crate::commands::crawl::command_output(
        &mut readiness,
        "run fresh mobile runtime-readiness verification",
    )?;
    if !output.status.success() {
        bail!(
            "fresh mobile runtime-readiness verification failed: status={}; stdout={:?}; stderr={:?}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let observation: Value = serde_json::from_slice(&output.stdout)
        .context("fresh mobile runtime-readiness output is not JSON")?;
    bound.attested(&observation)?;
    Ok(observation)
}

/// Why a control is not pressed, from what Brama judged pressing it does.
pub(crate) fn action_block_reason(action: &Action) -> Option<&'static str> {
    action.safety.withheld()
}

/// The attribute `field` of `element` when it names the element.
fn named_by(element: &crate::html::Element, field: &'static str) -> Option<(&'static str, String)> {
    element
        .attribute(field)
        .filter(|value| !value.trim().is_empty())
        .map(|value| (field, value.to_string()))
}

/// Every named element of `source`, each with what pressing it does as
/// Brama judged it (`control_safety`). A screen nobody could judge is
/// refused, never pressed. The page source is read through the one reader
/// every Spis command uses (`crate::html`), so a quoted '>', a single-quoted
/// value and every character reference read as the driver wrote them.
pub(crate) fn actions(source: &str, platform: Platform) -> Result<Vec<Action>> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    for element in crate::html::elements(source, "*")? {
        // iOS states an element's type in `type`, Android its widget class in
        // `class`; the parser lowers the element name's case, so the name is
        // read only when neither is stated.
        let kind = element
            .attribute("type")
            .or_else(|| element.attribute("class"))
            .unwrap_or(&element.name)
            .to_string();
        if element.attribute("enabled") == Some("false") {
            continue;
        }
        // Android says which elements take a tap; an input takes text.
        if matches!(platform, Platform::Android)
            && element.attribute("clickable") != Some("true")
            && !kind.ends_with("EditText")
        {
            continue;
        }
        // The attribute that names an element on each platform, in the
        // order its accessibility tree states them.
        let named = match platform {
            Platform::Android => named_by(&element, "resource-id")
                .or_else(|| named_by(&element, "content-desc"))
                .or_else(|| named_by(&element, "hint"))
                .or_else(|| named_by(&element, "text")),
            Platform::Ios => named_by(&element, "name")
                .or_else(|| named_by(&element, "label"))
                .or_else(|| named_by(&element, "placeholder"))
                .or_else(|| named_by(&element, "value")),
        };
        let Some((field, label)) = named else {
            continue;
        };
        let Some(literal) = xpath_literal(&label) else {
            continue;
        };
        let selector = format!("//*[@{field}={literal}]");
        if seen.insert(selector.clone()) {
            found.push((selector, label, kind));
        }
    }
    let named: Vec<(String, String)> = found
        .iter()
        .map(|(_, label, kind)| (kind.clone(), label.clone()))
        .collect();
    let classes = crate::commands::control_safety::judge("crawl-mobile", &named)?;
    Ok(found
        .into_iter()
        .zip(classes)
        .filter(|(_, safety)| *safety != crate::commands::control_safety::Safety::Inert)
        .map(|((selector, label, kind), safety)| Action {
            selector,
            label,
            safety,
            kind,
        })
        .collect())
}
