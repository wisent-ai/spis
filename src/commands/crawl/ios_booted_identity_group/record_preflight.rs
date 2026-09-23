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
        let host = host_report.get("host").and_then(Value::as_str).context("host report has no host")?;
        let (mut identity, mut checks) = match manifest.runtime_product.kind.as_str() {
            "ios-bundle" => ios_booted_identity(host)?,
            "android-package" => android_device_identity(host)?,
            "desktop-display-name" => resolve_desktop_identity(manifest, host)?,
            "cli-binary" | "tui-slug" => resolve_terminal_identity(manifest, host)?,
            "url" => (
                RuntimeExecutionIdentity {
                    host: host.into(),
                    observed_hostname: String::new(),
                    platform: if manifest.engine == "web" { "weles".into() } else { "http".into() },
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
            kind => bail!("unsupported unresolved runtime product kind {kind}"),
        };
        identity.observed_hostname = observed_hostname(host_report)?;
        identity.resolved_product_identifier = manifest.runtime_product.identifier.clone();
        let product = manifest.runtime_product.identifier.as_str();
        let check = match manifest.runtime_product.kind.as_str() {
            "ios-bundle" => {
                let udid = identity.device_id.as_deref().context("iOS identity has no UDID")?;
                host_probe(host, &["xcrun", "simctl", "get_app_container", udid, product, "app"])
            }
            "android-package" => {
                let serial = identity.device_id.as_deref().context("Android identity has no serial")?;
                host_probe(host, &["adb", "-s", serial, "shell", "pm", "path", product])
            }
            "desktop-bundle" => {
                let query = format!("kMDItemCFBundleIdentifier == '{}'", product.replace('\'', "\\'"));
                host_probe(host, &["mdfind", &query])
            }
            "cli-binary" | "tui-binary" => {
                let path = identity.executable_path.as_deref().context("terminal identity has no exact executable path")?;
                host_probe(host, &["shasum", "-a", "256", path])
            }
            "url" => {
                let parsed = url::Url::parse(product)
                    .context("declared URL is invalid")?;
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
            _ => unreachable!(),
        };
        let output = ready_output(&check, "verify exact runtime product")?;
        if matches!(manifest.runtime_product.kind.as_str(), "cli-binary" | "tui-binary") {
            let observed = output.split_whitespace().next().unwrap_or_default();
            if identity.executable_sha256.as_deref() != Some(observed) {
                bail!("terminal executable changed during preflight");
            }
        }
        if manifest.engine == "mobile" {
            checks.extend(resolve_mobile_install_identity(manifest, &mut identity, host, &check)?);
        }
        checks.push(check);
        if matches!(manifest.engine.as_str(), "mobile" | "desktop") {
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

pub(crate) fn aggregate_catalog_entry(entry: &mut Value) {
    let states: Vec<&str> = entry
        .get("records")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|record| record.get("state").and_then(Value::as_str))
        .collect();
    let failure = |state: &&str| {
        matches!(
            *state,
            "unavailable"
                | "preflight_failed"
                | "submission_failed"
                | "lost"
                | "failed"
                | "cancelled"
                | "partial"
        )
    };
    let state = if states.iter().any(|state| *state == "running") {
        "running"
    } else if states.iter().any(|state| *state == "cancel_pending") {
        "cancel_pending"
    } else if states.iter().any(|state| *state == "pending_review") {
        "pending_review"
    } else if states.iter().any(|state| {
        matches!(*state, "queued" | "submitting" | "preflight_passed")
    }) {
        "queued"
    } else if states.iter().any(|state| matches!(*state, "planned" | "preflighting")) {
        "planned"
    } else if states.iter().all(|state| *state == "imported") && !states.is_empty() {
        "imported"
    } else if states
        .iter()
        .all(|state| matches!(*state, "completed" | "uploaded" | "imported"))
        && !states.is_empty()
    {
        "completed"
    } else if states.iter().any(failure) {
        "partial"
    } else {
        "failed"
    };
    // Captured while `states` still borrows `entry`, used after the writes.
    let no_planned_records = states.is_empty();
    let mut failures: BTreeMap<String, usize> = BTreeMap::new();
    for value in states.iter().filter(|state| failure(state)) {
        *failures.entry((*value).to_string()).or_default() += 1;
    }
    entry["state"] = json!(state);
    entry["partial"] = json!(!failures.is_empty());
    entry["failure_counts"] = serde_json::to_value(failures).unwrap_or(Value::Null);
    // A catalog with no records at all reaches the final `else` above and is
    // reported `failed` with `error: null` and no failure counts, which is
    // what the 2026-09-01 documentation catalog looked like after a refresh:
    // a whole family declared failed with nothing anywhere saying why. That
    // state is not a crawl outcome, it is an empty plan — a run written in the
    // retired catalog-level shape, or a checked-out catalog whose references
    // directory is empty — so it says so, in the same typed diagnostic shape
    // every record-level refusal uses.
    if no_planned_records {
        entry["diagnostic"] = json!({
            "code": "no_planned_records",
            "message": "catalog carries no record attempts; nothing was planned, submitted or imported for it",
        });
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
    let recovered: Value =
        crate::read_json(path.to_str().context("receipt path is not UTF-8")?)?;
    if recovered != *receipt {
        bail!("submission receipt read-back differs from accepted content");
    }
    Ok(())
}
