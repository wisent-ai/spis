use super::*;

pub(crate) fn generate_runtime_bindings(rest: &[String]) -> Result<()> {
    let mut output_path: Option<PathBuf> = None;
    let mut weles_token_ref: Option<String> = None;
    let mut organization_ref: Option<String> = None;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--output" => {
                index += 1;
                output_path = Some(PathBuf::from(
                    rest.get(index).context("--output needs a path")?,
                ));
            }
            "--weles-token-ref" => {
                index += 1;
                weles_token_ref = Some(
                    rest.get(index)
                        .context("--weles-token-ref needs ITEM#FIELD")?
                        .clone(),
                );
            }
            "--organization-ref" => {
                index += 1;
                organization_ref = Some(
                    rest.get(index)
                        .context("--organization-ref needs ITEM#FIELD")?
                        .clone(),
                );
            }
            other => bail!("unknown crawl bindings generate option: {other}"),
        }
        index += 1;
    }
    // The two references authorize exactly one engine: the Weles browser
    // engine. Requiring them for the whole document made every other family
    // hostage to a credential coordinate it never reads — a documentation
    // crawl is bounded HTTP on a Stado host and its delivery is
    // `{"kind": "none"}` — so a fleet without a provisioned Weles bearer could
    // not plan a docs record at all. Supply them and the web families are
    // configured exactly as before; omit them and the web records become
    // explicitly unconfigured with a typed diagnostic, which is the same
    // treatment a native record without an authorization proof already gets
    // and which `planned_record` already turns into one `unavailable` attempt
    // rather than a silent crawl. Both must still be supplied together: half a
    // browser credential is a misconfiguration, not a narrower plan.
    if weles_token_ref.is_some() != organization_ref.is_some() {
        bail!(
            "--weles-token-ref and --organization-ref are supplied together or not at all; \
             one without the other cannot authorize a browser record"
        );
    }
    if weles_token_ref
        .as_deref()
        .into_iter()
        .chain(organization_ref.as_deref())
        .any(|value| !valid_secret_reference(value))
    {
        bail!("binding secret references must use exact ITEM#FIELD syntax");
    }

    let mut catalogs = serde_json::Map::new();
    for (catalog, engine) in CATALOGS {
        let mut records = serde_json::Map::new();
        for directory in record_directories(catalog, None)? {
            let slug = directory
                .file_name()
                .and_then(|value| value.to_str())
                .context("record directory name is not UTF-8")?
                .to_string();
            let binding = match *engine {
                // A web record without the pair of references has no
                // authorized delivery, so it is declared unconfigured here
                // rather than written as a configured record whose secret
                // environment names nothing.
                "web" if weles_token_ref.is_none() => safe_unconfigured_binding(engine),
                "web" | "docs" => {
                    let reference: Value = crate::read_json(
                        directory
                            .join("reference.json")
                            .to_str()
                            .context("record path is not UTF-8")?,
                    )?;
                    let exact_url = reference
                        .get("product_url")
                        .and_then(Value::as_str)
                        .filter(|value| !value.is_empty())
                        .with_context(|| format!("{catalog}/{slug}: reference has no product_url"))?;
                    let parsed = url::Url::parse(exact_url)
                        .with_context(|| format!("{catalog}/{slug}: product_url is invalid"))?;
                    let delivery = if *engine == "web" {
                        json!({
                            "kind": "weles-service-env",
                            "secret_env": {
                                "WELES_TOKEN": weles_token_ref,
                                "WISENT_ORGANIZATION_ID": organization_ref,
                            },
                        })
                    } else {
                        json!({"kind": "none", "secret_env": {}})
                    };
                    let mut object = serde_json::Map::new();
                    object.insert("configured".into(), json!(true));
                    object.insert("account".into(), anonymous_probe_account());
                    object.insert("constraints".into(), prohibited_action_constraints(engine));
                    object.insert("delivery".into(), delivery);
                    if *engine == "web" {
                        object.insert(
                            "surface".into(),
                            json!({
                                "family": catalog.strip_suffix("-examples")
                                    .context("web catalog has no canonical family")?,
                                "exact_url": exact_url,
                                "origin": parsed.origin().ascii_serialization(),
                                "path": parsed.path(),
                                "allowed_origins": [parsed.origin().ascii_serialization()],
                                "allowed_actions": ["generic_browser_task"],
                                "terminal_outcomes": ["blocked", "completed", "failed"],
                            }),
                        );
                    }
                    Value::Object(object)
                }
                other => safe_unconfigured_binding(other),
            };
            records.insert(slug, binding);
        }
        catalogs.insert((*catalog).to_string(), Value::Object(records));
    }
    let document = json!({
        "schema": "wisent.crawl-runtime-bindings.v1",
        "records": catalogs,
    });
    validate_runtime_bindings_document(&document)?;
    let bytes = serde_json::to_vec_pretty(&document)?;
    let mut with_newline = bytes;
    with_newline.push(b'\n');
    if let Some(path) = output_path {
        let outcome = write_generated_bindings(&path, &with_newline)?;
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema": OP_SCHEMA,
                "operation": "bindings_generate",
                "path": path,
                "sha256": crate::sha256_hex(&with_newline),
                "outcome": outcome,
            }))?
        );
    } else {
        print!("{}", String::from_utf8(with_newline)?);
    }
    Ok(())
}

/// Install one freshly validated generated bindings document at `path`.
///
/// The document has already passed `validate_runtime_bindings_document`, so an
/// existing file that differs is stale generated output, not a reason to refuse
/// forever: stage the new bytes beside it, fsync, rename atomically, fsync the
/// directory, then read the installed file back and prove the exact bytes. A
/// crash therefore leaves either the previous or the new complete document, and
/// regeneration after a catalog change is idempotent rather than blocked.
pub(crate) fn write_generated_bindings(path: &Path, bytes: &[u8]) -> Result<&'static str> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&parent)?;
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            bail!(
                "refusing to replace {}: generated runtime bindings must be a regular file",
                path.display()
            );
        }
        Ok(_) if std::fs::read(path)? == bytes => return Ok("unchanged"),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let existed = path.exists();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .context("generated runtime bindings path has no UTF-8 file name")?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let staged = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&staged)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&staged, path)?;
        File::open(&parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result?;
    let installed = std::fs::read(path)?;
    if installed != bytes {
        bail!(
            "generated runtime bindings read-back differs from the validated document at {}",
            path.display()
        );
    }
    Ok(if existed { "replaced" } else { "created" })
}

pub(crate) struct RuntimeBindings {
    pub(crate) source: String,
    pub(crate) local_path: Option<PathBuf>,
    pub(crate) uri: String,
    pub(crate) sha256: String,
    pub(crate) document: Value,
}

pub(crate) fn load_runtime_bindings(explicit: Option<&str>) -> Result<RuntimeBindings> {
    let project_template = source_root().join("crawl-runtime-bindings.json");
    let (source, selected) = if let Some(path) = explicit {
        ("explicit".to_string(), PathBuf::from(path))
    } else if let Some(path) = std::env::var_os("SPIS_RUNTIME_BINDINGS") {
        ("env:SPIS_RUNTIME_BINDINGS".to_string(), PathBuf::from(path))
    } else if let Some(path) = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/spis/crawl-runtime-bindings.json"))
        .filter(|path| path.is_file())
    {
        ("default:user-config".to_string(), path)
    } else if project_template.is_file() {
        ("template:project".to_string(), project_template)
    } else {
        bail!("no runtime bindings: pass --bindings PATH, set SPIS_RUNTIME_BINDINGS, generate ~/.config/spis/crawl-runtime-bindings.json, or provide the project template");
    };
    let bytes = std::fs::read(&selected)
        .with_context(|| format!("read runtime bindings {}", selected.display()))?;
    let document: Value = serde_json::from_slice(&bytes)?;
    validate_runtime_bindings_document(&document)?;
    let sha256 = crate::sha256_hex(&bytes);
    Ok(RuntimeBindings {
        source,
        local_path: Some(selected),
        uri: format!("{}/{sha256}.json", crate::CRAWL_INPUT_ROOT),
        sha256,
        document,
    })
}
