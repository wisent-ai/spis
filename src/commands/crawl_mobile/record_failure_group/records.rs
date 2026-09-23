use super::*;

pub(crate) fn records(catalog: &str, selected: Option<&str>) -> Result<Vec<Record>> {
    let directory = super::corpus::data_root()
        .join(catalog)
        .join("references");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&directory)
        .with_context(|| format!("read {}", directory.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    entries.sort();
    let mut found = Vec::new();
    for path in entries {
        let slug = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        if selected.is_some_and(|value| {
            value != slug && value != slug.split_once('-').map(|(_, tail)| tail).unwrap_or(&slug)
        }) {
            continue;
        }
        let record_path = path.join("reference.json");
        let document: Value = serde_json::from_slice(&std::fs::read(&record_path)?)?;
        found.push(Record {
            slug,
            name: document
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            path: record_path,
        });
    }
    if found.is_empty() {
        bail!("no matching records in {catalog}");
    }
    Ok(found)
}

pub(crate) fn ios_bundle_id_for(product_url: &str) -> Result<(String, String)> {
    let track = product_url.rsplit("/id").next()
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .ok_or_else(|| anyhow!("{product_url} has no exact /idNNN App Store identity"))?;
    let track_id: u64 = track.parse().context("parse App Store track id")?;
    let url = format!("https://itunes.apple.com/lookup?id={track_id}");
    let cache = Path::new(".wisent-output/product-resolution/ios").join(format!("{track_id}.json"));
    let response: Value = if cache.is_file() {
        serde_json::from_slice(&std::fs::read(&cache)?)?
    } else {
        // Same bounded, redirect-free treatment as every Appium read: no
        // unbounded into_json and no cross-origin redirect (findings 12/12b).
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(20))
            .redirects(0)
            .build();
        let response = agent
            .get(&url)
            .call()
            .context("resolve exact iOS bundle id through Apple's lookup API")?;
        if (300..400).contains(&response.status()) {
            bail!(
                "Apple lookup returned redirect HTTP {}; this agent follows no redirects",
                response.status()
            );
        }
        let limit = response_limit(&url);
        let value: Value = serde_json::from_reader(response.into_reader().take(limit))
            .with_context(|| {
                format!("Apple lookup response exceeded its {limit}-byte bound or was invalid JSON")
            })?;
        super::crawl::atomic_json_write(&cache, &value)?;
        value
    };
    let results = response.get("results").and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Apple lookup response has no results array"))?;
    if response.get("resultCount").and_then(Value::as_u64) != Some(1) || results.len() != 1 {
        bail!("Apple lookup for track id {track_id} did not return exactly one result");
    }
    let candidate = &results[0];
    if candidate.get("trackId").and_then(Value::as_u64) != Some(track_id) {
        bail!("Apple lookup result does not match requested track id {track_id}");
    }
    let bundle = candidate.get("bundleId").and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Apple lookup result has no bundleId"))?;
    Ok((bundle.to_string(), url))
}

pub(crate) fn attribute(tag: &str, name: &str) -> Option<String> {
    let pattern = Regex::new(&format!(r#"\b{}="([^"]*)""#, regex::escape(name))).ok()?;
    pattern
        .captures(tag)
        .and_then(|capture| capture.get(1))
        .map(|value| value.as_str().replace("&quot;", "\"").replace("&amp;", "&"))
}

pub(crate) fn xpath_literal(value: &str) -> Option<String> {
    if value.is_empty() || value.contains('\'') || value.contains('\n') || value.len() > 200 {
        return None;
    }
    Some(format!("'{value}'"))
}

pub(crate) fn system_owner(platform: Platform, identity: &str) -> bool {
    match platform {
        Platform::Android => [
            "com.android.permissioncontroller",
            "com.google.android.permissioncontroller",
            "com.android.systemui",
            "com.android.packageinstaller",
        ]
        .iter()
        .any(|owner| identity == *owner || identity.starts_with(&format!("{owner}."))),
        Platform::Ios => [
            "com.apple.springboard",
            "com.apple.Preferences",
            "com.apple.UserNotificationsUI",
            "com.apple.CoreAuthUI",
        ]
        .iter()
        .any(|owner| identity == *owner || identity.starts_with(&format!("{owner}."))),
    }
}

#[derive(serde::Serialize)]
pub(crate) struct SurfaceObservation {
    pub(crate) active_owner_before: String,
    pub(crate) source: String,
    pub(crate) alert_text: Option<String>,
    pub(crate) active_owner_after: String,
}

impl SurfaceObservation {
    pub(crate) fn refusal_reason(&self, platform: Platform, expected: &str) -> Option<String> {
        for observed in [&self.active_owner_before, &self.active_owner_after] {
            if observed != expected {
                return Some(if system_owner(platform, observed) {
                    format!(
                        "system-owned surface {observed} replaced exact app {expected}; further input withheld"
                    )
                } else {
                    format!(
                        "other-owner surface {observed} replaced exact app {expected}; further input withheld"
                    )
                });
            }
        }
        self.alert_text.as_ref().map(|text| {
            format!(
                "Appium reported an alert surface with exact text {text:?}; it was not classified or clicked"
            )
        })
    }
}

pub(crate) fn inspect_surface(appium: &Appium, session: &str) -> Result<SurfaceObservation> {
    let active_owner_before = appium.active_app_identity(session)?;
    let source = appium.source(session)?;
    let alert_text = appium.alert_text(session)?;
    let active_owner_after = appium.active_app_identity(session)?;
    Ok(SurfaceObservation {
        active_owner_before,
        source,
        alert_text,
        active_owner_after,
    })
}

pub(crate) fn exact_surface_screenshot(
    appium: &Appium,
    session: &str,
    platform: Platform,
    expected_owner: &str,
) -> Result<(Vec<u8>, String, String)> {
    // The screenshot is only taken from a settled surface (finding 19).
    appium.settle(session)?;
    let active_owner_before = appium.active_app_identity(session)?;
    if active_owner_before != expected_owner {
        let owner_class = if system_owner(platform, &active_owner_before) {
            "system-owned"
        } else {
            "other-owner"
        };
        bail!(
            "{owner_class} surface {active_owner_before:?} was active immediately before the state screenshot, not exact app {expected_owner:?}; screenshot withheld"
        );
    }
    let screenshot = appium.screenshot(session)?;
    let active_owner_after = appium.active_app_identity(session)?;
    if active_owner_after != expected_owner {
        let owner_class = if system_owner(platform, &active_owner_after) {
            "system-owned"
        } else {
            "other-owner"
        };
        bail!(
            "{owner_class} surface {active_owner_after:?} was active immediately after the state screenshot, not exact app {expected_owner:?}; screenshot rejected"
        );
    }
    Ok((screenshot, active_owner_before, active_owner_after))
}

pub(crate) fn returned_capability<'a>(
    capabilities: &'a Value,
    prefixed: &str,
    alias: &str,
) -> Result<&'a str> {
    let prefixed_value = capabilities.get(prefixed).and_then(Value::as_str);
    let alias_value = capabilities.get(alias).and_then(Value::as_str);
    if let (Some(left), Some(right)) = (prefixed_value, alias_value) {
        if left != right {
            bail!(
                "Appium returned conflicting scalar capabilities {prefixed}={left:?} and {alias}={right:?}"
            );
        }
    }
    prefixed_value
        .or(alias_value)
        .ok_or_else(|| anyhow!("Appium returned neither {prefixed} nor {alias}: {capabilities}"))
}

pub(crate) fn verify_session_capabilities(
    capabilities: &Value,
    platform: Platform,
    app_id: &str,
    execution: &super::crawl::RuntimeExecutionIdentity,
) -> Result<()> {
    let platform_name = capabilities
        .get("platformName")
        .and_then(Value::as_str)
        .context("Appium returned no scalar platformName capability")?;
    if platform_name != platform.appium_name() {
        bail!(
            "Appium session platform differs: expected {:?}, observed {platform_name:?}",
            platform.appium_name()
        );
    }
    let expected_udid = execution
        .device_id
        .as_deref()
        .context("mobile execution identity has no exact UDID")?;
    let observed_udid = returned_capability(capabilities, "appium:udid", "udid")?;
    if observed_udid != expected_udid {
        bail!(
            "Appium session UDID differs: expected {expected_udid:?}, observed {observed_udid:?}"
        );
    }
    let (prefixed, alias) = match platform {
        Platform::Ios => ("appium:bundleId", "bundleId"),
        Platform::Android => ("appium:appPackage", "appPackage"),
    };
    let observed_app = returned_capability(capabilities, prefixed, alias)?;
    if observed_app != app_id {
        bail!(
            "Appium session app identity differs: expected {app_id:?}, observed {observed_app:?}"
        );
    }
    Ok(())
}

pub(crate) fn hash_file(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("open {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("hash {}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex::encode(digest.finalize()))
}
