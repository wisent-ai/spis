use super::*;

pub(crate) fn registry_placements() -> Result<(BTreeMap<String, String>, Option<RuntimeServiceIdentity>)> {
    let mut command = stado_command();
    command.args(["registry", "pull"]);
    let output = bounded_command_output(
        &mut command,
        "Stado registry pull",
        Duration::from_secs(60),
        4 * 1024 * 1024,
    )?;
    if !output.status.success() {
        bail!(
            "Stado registry could not select crawler hosts: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let registry: Value = serde_json::from_slice(&output.stdout)?;
    let targets = registry
        .get("targets")
        .and_then(Value::as_array)
        .context("Stado registry has no targets")?;
    let service = registry.pointer("/service_directory/services/weles-admission");
    let service_identity = service.and_then(|service| {
        let generation = registry
            .pointer("/service_directory/generation")
            .and_then(Value::as_u64)?;
        let capabilities = service
            .pointer("/consumers/spis/capabilities")
            .and_then(Value::as_array)?;
        if !capabilities
            .iter()
            .any(|capability| capability.as_str() == Some("browser-evidence"))
        {
            return None;
        }
        let active_host = service.get("active_host").and_then(Value::as_str)?;
        let target = targets.iter().find(|target| {
            target.get("name").and_then(Value::as_str) == Some(active_host)
        })?;
        let actions = target.pointer("/weles/actions").and_then(Value::as_array)?;
        if !actions
            .iter()
            .any(|action| action.as_str() == Some("generic_browser_task"))
        {
            return None;
        }
        let endpoint = service
            .pointer(&format!("/endpoints/{active_host}/url"))
            .and_then(Value::as_str)
            .filter(|value| canonical_api_endpoint(value))?;
        let release_id = service
            .pointer(&format!("/endpoints/{active_host}/release_id"))
            .or_else(|| service.get("release_id"))
            .and_then(Value::as_str)
            .filter(|value| value.starts_with("weles-worker@") && *value != "weles-worker@")?;
        let source_revision = service
            .pointer(&format!("/endpoints/{active_host}/source_revision"))
            .or_else(|| service.get("source_revision"))
            .and_then(Value::as_str)
            .filter(|value| is_git_revision(value))?;
        Some(RuntimeServiceIdentity {
            name: "weles-admission".into(),
            generation,
            consumer: "spis".into(),
            capability: "browser-evidence".into(),
            active_host: active_host.into(),
            endpoint: endpoint.into(),
            action: "generic_browser_task".into(),
            release_id: release_id.into(),
            source_revision: source_revision.into(),
        })
    });
    let always_on = targets
        .iter()
        .find(|target| {
            target.get("role").and_then(Value::as_str) == Some("always-on")
                && target.pointer("/weles/enabled").and_then(Value::as_bool) == Some(true)
        })
        .and_then(|target| target.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let cpu = targets
        .iter()
        .find(|target| target.get("role").and_then(Value::as_str) == Some("always-on"))
        .and_then(|target| target.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    // Mobile placement is declared by `targets[].mobile_runtime`, and the
    // required Appium driver is the family route: XCUITest for iOS,
    // UiAutomator2 for Android. Looking for a service whose name happened to
    // contain "appium" ignored the registry's typed runtime declaration and
    // reported no placement even while both fleet hosts declared one.
    let mobile = [
        ("ios-app-examples", "xcuitest"),
        ("android-app-examples", "uiautomator2"),
    ]
    .into_iter()
    .filter_map(|(catalog, driver)| {
        targets
            .iter()
            .find(|target| {
                target
                    .pointer("/mobile_runtime/drivers")
                    .and_then(Value::as_array)
                    .is_some_and(|drivers| {
                        drivers
                            .iter()
                            .any(|declared| declared.as_str() == Some(driver))
                    })
            })
            .and_then(|target| target.get("name"))
            .and_then(Value::as_str)
            .map(|host| (catalog.to_string(), host.to_string()))
    })
    .collect::<BTreeMap<_, _>>();
    let mut placements = BTreeMap::new();
    if let Some(service) = &service_identity {
        placements.insert("web".into(), service.active_host.clone());
    }
    if let Some(host) = always_on {
        placements.insert("desktop".into(), host);
    }
    placements.extend(mobile);
    if let Some(host) = cpu {
        placements.insert("cli".into(), host.clone());
        placements.insert("tui".into(), host.clone());
        placements.insert("docs".into(), host);
    }
    Ok((placements, service_identity))
}

pub(crate) fn host_for(
    catalog: &str,
    engine: &str,
    explicit: &BTreeMap<String, String>,
    discovered: &BTreeMap<String, String>,
) -> Result<String> {
    explicit
        .get(catalog)
        .or_else(|| explicit.get(engine))
        .or_else(|| explicit.get("*"))
        .or_else(|| discovered.get(engine))
        .cloned()
        .ok_or_else(|| anyhow!(
            "no Stado host advertises the {engine} execution boundary for {catalog}; pass --host {engine}=TARGET after registering that capability"
        ))
}
