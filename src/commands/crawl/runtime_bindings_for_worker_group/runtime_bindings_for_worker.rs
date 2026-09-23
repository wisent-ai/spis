use super::*;

pub(crate) fn runtime_bindings_for_worker(manifest: &RuntimeManifest) -> Result<RuntimeBindings> {
    let stado_path = std::env::var_os("SPIS_STADO_BIN")
        .map(PathBuf::from)
        .context("worker_stado_binary_undeclared: the coordinator passed no Stado binary")?;
    let token_path = std::env::var_os("SPIS_CRAWL_OBJECT_TOKEN_FILE")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| host_home_crawl_token_path(&home))
        })
        .context("worker_crawl_token_undeclared: HOME and SPIS_CRAWL_OBJECT_TOKEN_FILE are absent")?;
    validate_worker_runtime_files(
        &stado_path,
        stado_path.is_file(),
        &token_path,
        token_path.is_file(),
    )?;
    let home = std::env::var_os("HOME").context("HOME is required for private Stado work cache")?;
    let directory = PathBuf::from(home)
        .join(".stado")
        .join("work")
        .join("spis")
        .join("runtime-bindings");
    std::fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    }
    let cache = directory.join(format!("{}.json", manifest.bindings_file_sha256));
    if !cache.is_file() {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let temporary = directory.join(format!(
            ".{}.{}.{}.tmp",
            manifest.bindings_file_sha256,
            std::process::id(),
            nonce
        ));
        let output = crawl_storage_command()
            .args(["storage", "get", &manifest.bindings_uri])
            .arg(&temporary)
            .output()
            .context("download immutable runtime bindings")?;
        if !output.status.success() {
            let _ = std::fs::remove_file(&temporary);
            bail!(
                "download runtime bindings {}: {}",
                manifest.bindings_uri,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        let downloaded = std::fs::read(&temporary)?;
        if crate::sha256_hex(&downloaded) != manifest.bindings_file_sha256 {
            let _ = std::fs::remove_file(&temporary);
            bail!("downloaded runtime bindings digest does not match manifest");
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
        }
        if cache.is_file() {
            let _ = std::fs::remove_file(&temporary);
        } else {
            std::fs::rename(&temporary, &cache)?;
            File::open(&directory)?.sync_all()?;
        }
    }
    let bytes = std::fs::read(&cache)?;
    if crate::sha256_hex(&bytes) != manifest.bindings_file_sha256 {
        bail!("private runtime bindings cache conflicts with manifest digest");
    }
    let document: Value = serde_json::from_slice(&bytes)?;
    validate_runtime_bindings_document(&document)?;
    Ok(RuntimeBindings {
        source: manifest.bindings_source.clone(),
        local_path: None,
        uri: manifest.bindings_uri.clone(),
        sha256: manifest.bindings_file_sha256.clone(),
        document,
    })
}

pub(crate) fn publish_runtime_bindings(bindings: &RuntimeBindings) -> Result<()> {
    let source = bindings
        .local_path
        .as_ref()
        .context("runtime binding input has no local immutable source")?;
    let bytes = std::fs::read(source)?;
    if crate::sha256_hex(&bytes) != bindings.sha256 {
        bail!("runtime binding input changed after planning");
    }
    let output = crawl_storage_command()
        .args([
            "storage",
            "put",
            "--if-absent",
            "--content-type",
            "application/json",
            bindings.uri.as_str(),
        ])
        .arg(source)
        .output()
        .context("publish immutable runtime bindings")?;
    if !output.status.success() {
        bail!(
            "Stado refused immutable runtime bindings: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let home = std::env::var_os("HOME").context("HOME is required for private Stado work cache")?;
    let directory = PathBuf::from(home)
        .join(".stado")
        .join("work")
        .join("spis")
        .join("runtime-bindings");
    std::fs::create_dir_all(&directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let target = directory.join(format!(
        ".{}.{}.{}.readback",
        bindings.sha256,
        std::process::id(),
        nonce
    ));
    let downloaded = crawl_storage_command()
        .args(["storage", "get", bindings.uri.as_str()])
        .arg(&target)
        .output()
        .context("read back immutable runtime bindings")?;
    if !downloaded.status.success() {
        let _ = std::fs::remove_file(&target);
        bail!(
            "runtime bindings read-back failed: {}",
            String::from_utf8_lossy(&downloaded.stderr).trim()
        );
    }
    let stored = std::fs::read(&target)?;
    let _ = std::fs::remove_file(&target);
    if crate::sha256_hex(&stored) != bindings.sha256 || stored != bytes {
        bail!("immutable runtime bindings read-back differs from the planned exact input");
    }
    Ok(())
}

pub(crate) fn valid_secret_reference(reference: &str) -> bool {
    reference
        .split_once('#')
        .is_some_and(|(item, field)| {
            !item.is_empty()
                && !field.is_empty()
                && item
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
                && field
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        })
}
