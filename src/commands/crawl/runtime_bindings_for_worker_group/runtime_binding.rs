use super::*;

pub(crate) fn runtime_binding(
    bindings: &RuntimeBindings,
    catalog: &str,
    engine: &str,
    slug: &str,
) -> Result<RuntimeBinding> {
    let binding = bindings
        .document
        .get("records")
        .and_then(Value::as_object)
        .and_then(|catalogs| catalogs.get(catalog))
        .and_then(Value::as_object)
        .and_then(|records| records.get(slug))
        .and_then(Value::as_object)
        .ok_or_else(|| {
            anyhow!("{catalog}/{slug}: runtime bindings have no exact catalog and record key")
        })?;
    if binding.get("configured").and_then(Value::as_bool) != Some(true) {
        bail!(
            "{catalog}/{slug}: {}",
            binding
                .get("diagnostic")
                .and_then(Value::as_str)
                .unwrap_or("runtime binding is explicitly unconfigured")
        );
    }
    let account: RuntimeAccount = serde_json::from_value(
        binding.get("account").cloned().context("record binding has no account declaration")?,
    )
    .context("record account declaration is invalid")?;
    match account.mode.as_str() {
        "anonymous-read-only-probe" => {
            if account.account_id.as_deref() != Some("anonymous-read-only-probe")
                || !account.credential_refs.is_empty()
            {
                bail!("{catalog}/{slug}: anonymous read-only probe mode must be explicit and cannot carry credentials or an account claim");
            }
        }
        "none" => {
            if account.account_id.is_some() || !account.credential_refs.is_empty() {
                bail!("{catalog}/{slug}: explicit none account mode cannot carry identity or credentials");
            }
        }
        mode => bail!("{catalog}/{slug}: unsupported account mode {mode}"),
    }
    if account
        .credential_refs
        .iter()
        .any(|reference| reference.is_empty() || reference.chars().any(char::is_whitespace))
    {
        bail!("{catalog}/{slug}: credentialRefs must be nonempty opaque identifiers");
    }
    let constraints: RuntimeConstraints = serde_json::from_value(
        binding
            .get("constraints")
            .cloned()
            .context("record binding has no constraint declaration")?,
    )
    .context("record constraint declaration is invalid")?;
    if !constraints.no_first_run_consent
        || !constraints.no_system_permission_prompts
        || !constraints.no_notifications
        || !constraints.no_purchase
        || !constraints.no_final_destructive_action
    {
        bail!("{catalog}/{slug}: runtime constraints would permit a prohibited crawl action");
    }
    let delivery: RuntimeDelivery = serde_json::from_value(
        binding
            .get("delivery")
            .cloned()
            .context("record binding has no typed credential delivery")?,
    )
    .context("record credential delivery is invalid")?;
    if delivery.secret_env.values().any(|reference| !valid_secret_reference(reference)) {
        bail!("{catalog}/{slug}: secret_env must contain exact NAME=item#field references");
    }
    let prepared_proof = binding
        .get("prepared_proof")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .context("prepared proof declaration is invalid")?;
    match engine {
        "web" => {
            let expected = ["WELES_TOKEN", "WISENT_ORGANIZATION_ID"];
            if delivery.kind != "weles-service-env"
                || delivery.secret_env.len() != expected.len()
                || expected.iter().any(|name| !delivery.secret_env.contains_key(*name))
            {
                bail!("{catalog}/{slug}: web binding needs only exact bearer and organization secret references; public receipt trust is checked in");
            }
        }
        "mobile" | "desktop" => {
            if delivery.kind != "preauthenticated-device"
                || !delivery.secret_env.is_empty()
                || prepared_proof.is_none()
            {
                bail!("{catalog}/{slug}: native binding needs preauthenticated-device delivery, no secret injection, and prepared proof");
            }
        }
        "cli" | "tui" => {
            let expected = if delivery.secret_env.is_empty() { "none" } else { "stado-secret-env" };
            let delivered: std::collections::BTreeSet<&str> =
                delivery.secret_env.values().map(String::as_str).collect();
            let declared: std::collections::BTreeSet<&str> =
                account.credential_refs.iter().map(String::as_str).collect();
            if delivery.kind != expected || delivered != declared {
                bail!("{catalog}/{slug}: terminal delivery must exactly bind account refs through Stado secret-env");
            }
        }
        "docs" => {
            if delivery.kind != "none" || !delivery.secret_env.is_empty() {
                bail!("{catalog}/{slug}: documentation public delivery must be explicit none");
            }
        }
        _ => bail!("{catalog}/{slug}: unsupported engine delivery"),
    }
    let surface = binding
        .get("surface")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .context("surface identity declaration is invalid")?;
    Ok(RuntimeBinding {
        account,
        constraints,
        prepared_proof,
        delivery,
        surface,
    })
}

pub(crate) fn runtime_product(
    catalog: &str,
    engine: &str,
    slug: &str,
    record: &Value,
    surface: Option<RuntimeSurfaceIdentity>,
) -> Result<RuntimeProduct> {
    let product_url = record
        .get("product_url")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("record has no product_url")?
        .to_string();
    let (kind, identifier, identity_source) = match (engine, catalog) {
        ("mobile", "ios-app-examples") => {
            let (bundle, source) = super::crawl_mobile::ios_bundle_id_for(&product_url)?;
            ("ios-bundle", bundle, source)
        }
        ("mobile", "android-app-examples") => {
            let parsed = url::Url::parse(&product_url)?;
            let package = parsed
                .query_pairs()
                .find(|(key, _)| key == "id")
                .map(|(_, value)| value.into_owned())
                .filter(|value| !value.is_empty())
                .context("Play product_url has no exact package id")?;
            ("android-package", package, product_url.clone())
        }
        ("desktop", _) => {
            let display_name = record
                .get("name")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .context("reference has no exact product display name")?;
            (
                "desktop-display-name",
                display_name.to_string(),
                "pending unique typed host resolution".into(),
            )
        }
        ("cli", _) => (
            "cli-binary",
            super::crawl_cli::binary_for(slug),
            "Spis exact CLI catalog mapping".into(),
        ),
        ("tui", _) => {
            let display_name = record
                .get("name")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .context("reference has no exact product display name")?;
            (
                "tui-slug",
                display_name.to_string(),
                "pending unique typed host executable/version resolution".into(),
            )
        }
        ("web" | "docs", _) => ("url", product_url.clone(), "reference.json product_url".into()),
        _ => bail!("{catalog}/{slug}: unsupported runtime identity for {engine}"),
    };
    let surface = if engine == "web" {
        let surface = surface.context("web record has no exact typed surface identity")?;
        let expected_family = catalog
            .strip_suffix("-examples")
            .context("web catalog has no canonical family suffix")?;
        let parsed = url::Url::parse(&product_url).context("web product URL is invalid")?;
        if surface.family != expected_family
            || surface.exact_url != product_url
            || surface.origin != parsed.origin().ascii_serialization()
            || surface.path != parsed.path()
            || surface.allowed_origins.is_empty()
            || !surface.allowed_origins.iter().any(|origin| origin == &surface.origin)
            || surface.allowed_origins.iter().any(|origin| url::Url::parse(origin).is_err())
            || surface.allowed_actions.is_empty()
            || surface.allowed_actions.iter().any(|action| action.is_empty() || action.chars().any(char::is_whitespace))
            || surface.terminal_outcomes.is_empty()
            || surface.terminal_outcomes.iter().any(|outcome| {
                !matches!(outcome.as_str(), "completed" | "failed" | "blocked")
            })
        {
            bail!("{catalog}/{slug}: typed web surface family, origin, path, URL, allowed actions or terminal outcomes are not exact");
        }
        Some(surface)
    } else {
        surface
    };
    if matches!(engine, "desktop" | "cli" | "tui") && !is_host_query_literal(&identifier) {
        bail!(
            "{catalog}/{slug}: {engine} product identity {identifier:?} contains characters that cannot be embedded in a host resolution query"
        );
    }
    Ok(RuntimeProduct {
        kind: kind.into(),
        declared_identifier: identifier.clone(),
        identifier,
        product_url,
        identity_source,
        surface,
    })
}

/// A display name, bundle id or binary name that is safe to embed verbatim in a
/// Spotlight predicate or an argv element.
///
/// Escaping a single quote with a backslash is not how the Spotlight query
/// language quotes literals, so the only safe posture is to refuse any identity
/// that could close a literal or append a predicate.
pub(crate) fn is_host_query_literal(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, ' ' | '.' | '_' | '-'))
        && !value.starts_with(' ')
        && !value.ends_with(' ')
}

pub(crate) fn docs_structure_sha256(catalog: &str, record: &str, engine: &str) -> Result<Option<String>> {
    if engine != "docs" {
        return Ok(None);
    }
    safe_component(record, "record")?;
    let path = catalog_root(catalog)?
        .join("content-structure")
        .join(format!("{record}.json"));
    let bytes = std::fs::read(&path)
        .with_context(|| format!("read docs crawl definition {}", path.display()))?;
    Ok(Some(crate::sha256_hex(&bytes)))
}
