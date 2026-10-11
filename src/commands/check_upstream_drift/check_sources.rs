use super::*;

pub(crate) fn check_sources(drift: &Mutex<Drift>) -> Result<()> {
    let pairs = collect_urls()?;
    pool(&pairs, 12, |pair| {
        let (url, where_, expected) = pair;
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            drift.lock().sources_skipped += 1;
            return;
        }
        let client = agent();
        let (mut state, detail) = url_state(&client, url);
        if state == "reachable" {
            drift.lock().sources_ok += 1;
            return;
        }
        // A private repository or authenticated application may deliberately
        // answer anonymous HTTP with 400/404. Its recorded classification wins
        // over the transport code.
        if expected.as_deref() == Some("guarded") && state == "gone" {
            state = "guarded";
        }
        let item = json!({
            "url": url,
            "where": where_,
            "result": detail,
            "expected": expected.clone().map(Value::String).unwrap_or(Value::Null),
        });
        let mut guard = drift.lock();
        match state {
            "gone" => guard.sources_gone.push(item),
            "guarded" => guard.sources_guarded.push(item),
            _ => guard.sources_unresolved.push(item),
        }
    });
    Ok(())
}

pub(crate) fn stream_sha256(path: &std::path::Path) -> Result<String> {
    use sha2::Digest;
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut digest = sha2::Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

pub(crate) fn check_local_media(drift: &mut Drift) -> Result<()> {
    for record in reference_records()? {
        let data: Value = lib::read_json(record.to_str().context("non-UTF8 path")?)?;
        let base = record.parent().context("record without parent")?;
        for key in ["motion", "states"] {
            for entry in data
                .get(key)
                .and_then(|v| v.as_array())
                .map(|a| a.as_slice())
                .unwrap_or(&[])
            {
                let local = base.join(
                    entry
                        .get("local_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                );
                let rel = local.to_string_lossy().to_string();
                if !local.exists() {
                    drift.media_missing.push(rel);
                    continue;
                }
                let Some(recorded) = entry.get("sha256").and_then(|v| v.as_str()) else {
                    continue;
                };
                if stream_sha256(&local)? != recorded {
                    drift.media_hash_mismatch.push(rel);
                } else {
                    drift.media_ok += 1;
                }
            }
        }
    }
    Ok(())
}

/// Sorted `*-examples/references/*/reference.json` under the working
/// directory. A catalog without a `references` directory has none; a
/// directory or entry that cannot be read fails with its path. Skipped, it
/// made the drift report check fewer references while reading as complete.
pub(crate) fn reference_records() -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for catalog in sorted_example_dirs()? {
        let refs_dir = catalog.join("references");
        let entries = match std::fs::read_dir(&refs_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("listing {}", refs_dir.display()))
            }
        };
        for entry in entries {
            let entry =
                entry.with_context(|| format!("reading an entry of {}", refs_dir.display()))?;
            let candidate = entry.path().join("reference.json");
            if candidate.is_file() {
                out.push(candidate);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Sorted `*-examples` directories of the working directory, or the listing
/// failure that would otherwise have dropped catalogs from the check.
pub(crate) fn sorted_example_dirs() -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(".").context("listing the working directory")? {
        let entry = entry.context("reading an entry of the working directory")?;
        let path = PathBuf::from(entry.file_name());
        let named = path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with("-examples"));
        if named && path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    Ok(dirs)
}

pub(crate) fn sorted_glob_examples_files(name: &str) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = sorted_example_dirs()?
        .into_iter()
        .map(|d| d.join(name))
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    Ok(files)
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut skip_network = false;
    let mut skip_readme = false;
    let mut write_report = false;
    let mut strict = false;
    for arg in rest {
        match arg.as_str() {
            "--skip-network" => skip_network = true,
            "--skip-readme" => skip_readme = true,
            "--write-report" => write_report = true,
            "--strict" => strict = true,
            other => return Err(crate::commands::usage(format!("unknown argument: {other}"))),
        }
    }

    let mut drift = Drift::default();
    check_local_media(&mut drift)?;
    if !skip_network {
        let shared = Mutex::new(Drift::default());
        if !skip_readme {
            check_readmes(&shared)?;
        }
        check_sources(&shared)?;
        let net = shared.into_inner();
        drift.readme_changed = net.readme_changed;
        drift.readme_unreachable = net.readme_unreachable;
        drift.readme_unchanged = net.readme_unchanged;
        drift.sources_gone = net.sources_gone;
        drift.sources_guarded = net.sources_guarded;
        drift.sources_unresolved = net.sources_unresolved;
        drift.sources_ok = net.sources_ok;
        drift.sources_skipped = net.sources_skipped;
    }

    println!("local media verified: {}", drift.media_ok);
    println!("local media missing: {}", drift.media_missing.len());
    println!(
        "local media hash mismatch: {}",
        drift.media_hash_mismatch.len()
    );
    if !skip_network {
        println!("README snapshots unchanged: {}", drift.readme_unchanged);
        println!(
            "README snapshots changed upstream: {}",
            drift.readme_changed.len()
        );
        println!(
            "README snapshots unreachable: {}",
            drift.readme_unreachable.len()
        );
        println!("upstream URLs reachable: {}", drift.sources_ok);
        println!("upstream URLs gone: {}", drift.sources_gone.len());
        println!(
            "upstream URLs guarded (auth, rate limit, bot wall): {}",
            drift.sources_guarded.len()
        );
        println!(
            "upstream URLs unresolved (network): {}",
            drift.sources_unresolved.len()
        );
    }
    for item in &drift.media_missing {
        println!("  missing media: {item}");
    }
    for item in &drift.media_hash_mismatch {
        println!("  hash mismatch: {item}");
    }
    for item in &drift.readme_changed {
        println!(
            "  README changed: {} ({})",
            item["repository"].as_str().unwrap_or("?"),
            item["snapshot"].as_str().unwrap_or("null"),
        );
    }
    for item in &drift.sources_gone {
        println!(
            "  upstream gone: {} -> {} {}",
            item["where"].as_str().unwrap_or("?"),
            item["result"],
            item["url"].as_str().unwrap_or("?")
        );
    }
    for item in &drift.sources_unresolved {
        println!(
            "  upstream unresolved: {} -> {} {}",
            item["where"].as_str().unwrap_or("?"),
            item["result"],
            item["url"].as_str().unwrap_or("?")
        );
    }

    if write_report {
        let report = json!({
            "schema": SCHEMA,
            "checked_at": lib::now_iso_utc(),
            "network_checked": !skip_network,
            "local_media_verified": drift.media_ok,
            "local_media_missing": drift.media_missing,
            "local_media_hash_mismatch": drift.media_hash_mismatch,
            "readme_unchanged": drift.readme_unchanged,
            "readme_changed": drift.readme_changed,
            "readme_unreachable": drift.readme_unreachable,
            "upstream_urls_reachable": drift.sources_ok,
            "upstream_urls_gone": drift.sources_gone,
            "upstream_urls_guarded": drift.sources_guarded,
            "upstream_urls_unresolved": drift.sources_unresolved,
            "upstream_urls_skipped": drift.sources_skipped,
        });
        std::fs::create_dir_all(".build")
            .context("create private drift report directory .build")?;
        std::fs::write(REPORT, serde_json::to_string_pretty(&report)? + "\n")
            .with_context(|| format!("write private drift report {REPORT}"))?;
        println!("\nreport written to {REPORT}");
    }

    if strict && drift.any_drift() {
        // The Python original exited 1 silently; main.rs prints "error:" for
        // Err returns, so exit directly.
        std::process::exit(1);
    }
    Ok(())
}
