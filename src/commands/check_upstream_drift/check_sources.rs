use super::*;

pub(crate) fn check_sources(drift: &Mutex<Drift>) -> Result<()> {
    let pairs = collect_urls()?;
    pool(&pairs, |pair| {
        let (url, where_, expected) = pair;
        let client = agent();
        let (mut state, detail) = url_state(&client, url);
        match state {
            "reachable" => {
                drift.lock().sources_ok.push(url.clone());
                return;
            }
            // A recorded source the HTTP client has no scheme for (a
            // repository path, a mailto) cannot be asked; it is listed.
            "skipped" => {
                drift
                    .lock()
                    .sources_skipped
                    .push(format!("{url}: {detail}"));
                return;
            }
            _ => {}
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
            "expected": expected,
        });
        let mut guard = drift.lock();
        match state {
            "gone" => guard.sources_gone.push(item),
            "guarded" => guard.sources_guarded.push(item),
            _ => guard.sources_unresolved.push(item),
        }
    })
}

/// Whether `path` exists; one that cannot be examined is refused naming it.
fn present(path: &std::path::Path) -> Result<bool> {
    path.try_exists()
        .with_context(|| format!("examine {}", path.display()))
}

pub(crate) fn check_local_media(drift: &mut Drift) -> Result<()> {
    for record in reference_records()? {
        let data: Value = lib::read_json(&record)?;
        let base = record.parent().context("record without parent")?;
        // Every media entry the record carries, whichever list holds it (the
        // recording under motion, the still states, or any list a later
        // record schema adds), is one that names a local_path beside the
        // record; an entry without one carries no local media. Each such
        // file must exist and, where the record states its sha256, still
        // hash to it.
        let lists = data
            .as_object()
            .into_iter()
            .flat_map(|object| object.values());
        for entries in lists.filter_map(Value::as_array) {
            for entry in entries {
                let Some(path) = entry.get("local_path").and_then(Value::as_str) else {
                    continue;
                };
                let local = base.join(path);
                let rel = local.to_string_lossy().to_string();
                if !present(&local)? {
                    drift.media_missing.push(rel);
                    continue;
                }
                let Some(recorded) = entry.get("sha256").and_then(|v| v.as_str()) else {
                    continue;
                };
                if crate::sha256_file(&local)? != recorded {
                    drift.media_hash_mismatch.push(rel);
                } else {
                    drift.media_ok.push(rel);
                }
            }
        }
    }
    Ok(())
}

/// Sorted `*-examples/references/*/reference.json` under the working
/// directory. A directory or entry that cannot be read is refused naming it:
/// skipped, its records were reported as having no drift.
pub(crate) fn reference_records() -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for catalog in sorted_example_dirs()? {
        let refs_dir = catalog.join("references");
        let entries = match std::fs::read_dir(&refs_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("list {}", refs_dir.display()))
            }
        };
        for entry in entries {
            let entry =
                entry.with_context(|| format!("read an entry of {}", refs_dir.display()))?;
            let candidate = entry.path().join("reference.json");
            if present(&candidate)? {
                out.push(candidate);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Sorted `*-examples` directories under the working directory; one that
/// cannot be listed, or an entry that cannot be read, is refused.
pub(crate) fn sorted_example_dirs() -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(".").context("list the working directory")? {
        let entry = entry.context("read an entry of the working directory")?;
        if !entry.file_name().to_string_lossy().ends_with("-examples") {
            continue;
        }
        let kind = entry
            .file_type()
            .with_context(|| format!("examine {}", entry.path().display()))?;
        if kind.is_dir() {
            dirs.push(PathBuf::from(entry.file_name()));
        }
    }
    dirs.sort();
    Ok(dirs)
}

/// `name` in every catalog that holds it; one that cannot be examined is
/// refused naming it, not skipped as a catalog without the file.
pub(crate) fn sorted_glob_examples_files(name: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for catalog in sorted_example_dirs()? {
        let file = catalog.join(name);
        if present(&file)? {
            files.push(file);
        }
    }
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

    println!("local media verified: {}", drift.media_ok.len());
    println!("local media missing: {}", drift.media_missing.len());
    println!(
        "local media hash mismatch: {}",
        drift.media_hash_mismatch.len()
    );
    if !skip_network {
        println!(
            "README snapshots unchanged: {}",
            drift.readme_unchanged.len()
        );
        println!(
            "README snapshots changed upstream: {}",
            drift.readme_changed.len()
        );
        println!(
            "README snapshots unreachable: {}",
            drift.readme_unreachable.len()
        );
        println!("upstream URLs reachable: {}", drift.sources_ok.len());
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
            item["repository"], item["snapshot"]
        );
    }
    for item in &drift.sources_gone {
        println!(
            "  upstream gone: {} -> {} {}",
            item["where"], item["result"], item["url"]
        );
    }
    for item in &drift.sources_unresolved {
        println!(
            "  upstream unresolved: {} -> {} {}",
            item["where"], item["result"], item["url"]
        );
    }

    if write_report {
        let report = json!({
            "schema": SCHEMA,
            "checked_at": lib::now_iso_utc(),
            "network_checked": !skip_network,
            "local_media_verified": drift.media_ok.len(),
            "local_media_missing": drift.media_missing,
            "local_media_hash_mismatch": drift.media_hash_mismatch,
            "readme_unchanged": drift.readme_unchanged.len(),
            "readme_changed": drift.readme_changed,
            "readme_unreachable": drift.readme_unreachable,
            "upstream_urls_reachable": drift.sources_ok.len(),
            "upstream_urls_gone": drift.sources_gone,
            "upstream_urls_guarded": drift.sources_guarded,
            "upstream_urls_unresolved": drift.sources_unresolved,
            "upstream_urls_skipped": drift.sources_skipped.len(),
        });
        std::fs::create_dir_all(".build")
            .context("create private drift report directory .build")?;
        std::fs::write(REPORT, serde_json::to_string_pretty(&report)? + "\n")
            .with_context(|| format!("write private drift report {REPORT}"))?;
        println!("\nreport written to {REPORT}");
    }

    if strict && drift.any_drift() {
        anyhow::bail!("--strict: upstream drift was found; the lines above name each item");
    }
    Ok(())
}
