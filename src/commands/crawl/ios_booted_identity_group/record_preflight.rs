use super::*;

pub(crate) fn record_preflight(manifest: &mut RuntimeManifest, host_report: &Value) -> Value {
    let host_ready = host_report.get("ready").and_then(Value::as_bool) == Some(true);
    if !host_ready {
        return json!({
            "schema": "wisent.crawl-record-preflight.v2",
            "record": manifest.record,
            "ready": false,
            "diagnostic": {"code": "host_unavailable", "message": "host capability preflight failed"},
            "checks": [],
        });
    }
    if manifest.engine == "web" && manifest.service_identity.is_none() {
        return json!({
            "schema": "wisent.crawl-record-preflight.v2",
            "record": manifest.record,
            "ready": false,
            "diagnostic": {
                "code": "weles_service_identity_unbound",
                "message": "web execution is unavailable until the exact authorized Weles service directory generation, active host, endpoint, consumer capability and action are bound"
            },
            "checks": [],
        });
    }
    let original_manifest = manifest.clone();
    let result = (|| -> Result<Vec<Value>> {
        let host = host_report
            .get("host")
            .and_then(Value::as_str)
            .context("host report has no host")?;
        let declared_kind = ProductKind::of(&manifest.runtime_product.kind)?;
        let (mut identity, mut checks) = match declared_kind {
            ProductKind::IosBundle => ios_booted_identity(host)?,
            ProductKind::AndroidPackage => android_device_identity(host)?,
            ProductKind::DesktopDisplayName => resolve_desktop_identity(manifest, host)?,
            ProductKind::CliBinary | ProductKind::TuiSlug => {
                resolve_terminal_identity(manifest, host)?
            }
            ProductKind::Url => (
                RuntimeExecutionIdentity {
                    host: host.into(),
                    observed_hostname: String::new(),
                    platform: if manifest.engine == "web" {
                        "weles".into()
                    } else {
                        "http".into()
                    },
                    device_id: None,
                    resolved_product_identifier: String::new(),
                    device_name: None,
                    executable_path: None,
                    product_version: None,
                    executable_sha256: None,
                    effective_url: None,
                },
                Vec::new(),
            ),
            kind => bail!(
                "unsupported unresolved runtime product kind {}",
                kind.name()
            ),
        };
        identity.observed_hostname = observed_hostname(host_report)?;
        identity.resolved_product_identifier = manifest.runtime_product.identifier.clone();
        let product = manifest.runtime_product.identifier.as_str();
        let resolved_kind = ProductKind::of(&manifest.runtime_product.kind)?;
        let check = match resolved_kind {
            ProductKind::IosBundle => {
                let udid = identity
                    .device_id
                    .as_deref()
                    .context("iOS identity has no UDID")?;
                host_probe(
                    host,
                    &["xcrun", "simctl", "get_app_container", udid, product, "app"],
                )
            }
            ProductKind::AndroidPackage => {
                let serial = identity
                    .device_id
                    .as_deref()
                    .context("Android identity has no serial")?;
                host_probe(host, &["adb", "-s", serial, "shell", "pm", "path", product])
            }
            ProductKind::DesktopBundle => {
                let query = format!(
                    "kMDItemCFBundleIdentifier == '{}'",
                    product.replace('\'', "\\'")
                );
                host_probe(host, &["mdfind", &query])
            }
            ProductKind::CliBinary | ProductKind::TuiBinary => {
                let path = identity
                    .executable_path
                    .as_deref()
                    .context("terminal identity has no exact executable path")?;
                host_probe(host, &["shasum", "-a", "256", path])
            }
            ProductKind::Url => {
                let parsed = url::Url::parse(product).context("declared URL is invalid")?;
                if parsed.scheme() != "https"
                    || parsed.username() != ""
                    || parsed.password().is_some()
                    || parsed.host_str().is_none()
                {
                    bail!("URL identity must be an exact credential-free HTTPS URL");
                }
                // A URL product has no host command to run: the identity IS
                // the exact committed URL, already parsed and refused above if
                // it carried credentials or a non-HTTPS scheme. It is reported
                // as this check's observed output because the caller proves
                // readiness through `ready_output`, and a synthetic check with
                // empty stdout made every documentation and browser record
                // refuse with "verify exact runtime product: command returned
                // no identity".
                json!({
                    "command": [],
                    "ready": true,
                    "stdout": product,
                    "network_policy_owner": manifest.engine,
                    "declared_url": product,
                })
            }
            kind => bail!(
                "runtime product kind {} was not resolved before its readiness check",
                kind.name()
            ),
        };
        let output = ready_output(&check, "verify exact runtime product")?;
        if resolved_kind.executable() {
            let observed = output.split_whitespace().next().unwrap_or_default();
            if identity.executable_sha256.as_deref() != Some(observed) {
                bail!("terminal executable changed during preflight");
            }
        }
        if manifest.engine == "mobile" {
            checks.extend(resolve_mobile_install_identity(
                manifest,
                &mut identity,
                host,
                &check,
            )?);
        }
        checks.push(check);
        if Engine::of(&manifest.engine)?.prepares_runtime() {
            checks.push(prepared_runtime_check(manifest, &identity, host)?);
        }
        manifest.execution_identity = Some(identity.clone());
        if manifest.resource_lease.is_some() {
            manifest.resource_lease = Some(format!(
                "stado-exclusive://{}/{}",
                host,
                identity.device_id.as_deref().unwrap_or(product)
            ));
        }
        Ok(checks)
    })();
    match result {
        Ok(checks) => json!({
            "schema": "wisent.crawl-record-preflight.v2",
            "record": manifest.record,
            "ready": true,
            "runtime_product": manifest.runtime_product,
            "account": manifest.account,
            "execution_identity": manifest.execution_identity,
            "resource_lease": manifest.resource_lease,
            "prepared_runtime_proof": manifest.prepared_proof,
            "checks": checks,
        }),
        Err(error) => {
            *manifest = original_manifest;
            json!({
                "schema": "wisent.crawl-record-preflight.v2",
                "record": manifest.record,
                "ready": false,
                "runtime_product": manifest.runtime_product,
                "account": manifest.account,
                "diagnostic": {"code": "runtime_identity_or_readiness_unavailable", "message": error.to_string()},
                "checks": [],
            })
        }
    }
}

pub(crate) fn submission_receipt_path(
    run_id: &str,
    catalog: &str,
    record: &str,
    attempt_id: &str,
) -> Result<PathBuf> {
    safe_component(run_id, "run id")?;
    safe_component(catalog, "catalog")?;
    safe_component(record, "record")?;
    safe_component(attempt_id, "attempt id")?;
    Ok(run_root()?
        .join(run_id)
        .join("receipts")
        .join(catalog)
        .join(record)
        .join(attempt_id)
        .join("receipt.json"))
}

pub(crate) fn load_submission_receipt(
    run_id: &str,
    catalog: &str,
    record: &str,
    attempt_id: &str,
) -> Result<Option<Value>> {
    let path = submission_receipt_path(run_id, catalog, record, attempt_id)?;
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(crate::read_json(
        path.to_str().context("receipt path is not UTF-8")?,
    )?))
}

pub(crate) fn persist_submission_receipt(
    run_id: &str,
    catalog: &str,
    record: &str,
    attempt_id: &str,
    receipt: &Value,
) -> Result<()> {
    let path = submission_receipt_path(run_id, catalog, record, attempt_id)?;
    if path.is_file() {
        let existing: Value =
            crate::read_json(path.to_str().context("receipt path is not UTF-8")?)?;
        if existing == *receipt {
            return Ok(());
        }
        bail!("immutable submission receipt already exists with different content");
    }
    atomic_json_write(&path, receipt)?;
    let recovered: Value = crate::read_json(path.to_str().context("receipt path is not UTF-8")?)?;
    if recovered != *receipt {
        bail!("submission receipt read-back differs from accepted content");
    }
    Ok(())
}
