use super::*;

/// Every interface family in [`Family::ALL`] is required and keeps its
/// order; a catalog added with `spis catalog-type add` follows them in name
/// order.
pub(crate) fn catalogs(selected: Option<&str>) -> Result<Vec<PathBuf>> {
    // An entry that cannot be read refuses the listing: skipped, its catalog
    // read as missing from the corpus or simply went unverified.
    let mut found: Vec<String> = std::fs::read_dir(".")?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().to_string()))
        .collect::<std::io::Result<Vec<_>>>()
        .context("list the corpus catalogs")?
        .into_iter()
        .filter(|name| name.ends_with("-examples") && Path::new(name).join("references").is_dir())
        .collect();
    found.sort();
    let families: Vec<String> = Family::ALL.iter().map(|family| family.name()).collect();
    let missing: Vec<&String> = families
        .iter()
        .filter(|family| !found.contains(*family))
        .collect();
    if !missing.is_empty() {
        anyhow::bail!(
            "interface families missing from the corpus: {missing:?}; all {} are required",
            families.len()
        );
    }
    if let Some(selected) = selected {
        if !found.iter().any(|name| name == selected) {
            anyhow::bail!(
                "unknown catalog {selected}; the corpus holds {}",
                found.join(", ")
            );
        }
        return Ok(vec![PathBuf::from(selected)]);
    }
    let mut ordered: Vec<PathBuf> = families.iter().map(PathBuf::from).collect();
    ordered.extend(
        found
            .iter()
            .filter(|name| !families.contains(*name))
            .map(PathBuf::from),
    );
    Ok(ordered)
}

pub(crate) fn records_in(catalog: &Path) -> Result<Vec<PathBuf>> {
    let references = catalog.join("references");
    let mut records: Vec<PathBuf> = std::fs::read_dir(&references)?
        .map(|entry| entry.map(|entry| entry.path().join("reference.json")))
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("list {}", references.display()))?
        .into_iter()
        .filter(|path| path.is_file())
        .collect();
    records.sort();
    let sources: Value = lib::read_json(
        catalog
            .join("sources.json")
            .to_str()
            .context("non-UTF8 sources path")?,
    )?;
    let source_count = sources
        .get("examples")
        .and_then(Value::as_array)
        .map(Vec::len)
        .context("sources.json has no examples array")?;
    let family = catalog
        .to_str()
        .is_some_and(|name| reference_contract::named::<Family>(name).is_some());
    if family && (records.len() != RECORDS_PER_CATALOG || source_count != RECORDS_PER_CATALOG) {
        anyhow::bail!(
            "{}: the interface family contract requires {RECORDS_PER_CATALOG} records and sources, found {} records and {source_count} sources",
            catalog.display(),
            records.len()
        );
    }
    if records.len() != source_count {
        anyhow::bail!(
            "{}: {} records for {source_count} sources",
            catalog.display(),
            records.len()
        );
    }
    Ok(records)
}

pub(crate) fn gap_key(gap: &str) -> String {
    match gap.find([':', '(']) {
        Some(pos) => gap[..pos].trim().to_string(),
        None => gap.trim().to_string(),
    }
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut apply = false;
    let mut no_state_match = false;
    let mut jobs: Option<std::num::NonZeroUsize> = None;
    let mut arguments = rest.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--catalog" => {
                catalog = Some(
                    crate::commands::required(arguments.next(), "--catalog needs a value")?.clone(),
                );
            }
            "--apply" => apply = true,
            "--no-state-match" => no_state_match = true,
            "--jobs" => {
                jobs = Some(
                    arguments
                        .next()
                        .context("--jobs needs a value")?
                        .parse()
                        .context("--jobs expects a positive integer")?,
                );
            }
            other => return Err(crate::commands::usage(format!("unknown argument: {other}"))),
        }
    }
    // Without --jobs, as many records are measured at once as this machine
    // runs threads.
    let jobs = match jobs {
        Some(jobs) => jobs,
        None => std::thread::available_parallelism()
            .context("--jobs is not set and the machine's parallelism is unreadable")?,
    };
    let locate_states = !no_state_match;

    if !apply {
        println!("dry run: records are measured and reported, nothing is written");
    }

    let mut total = 0usize;
    let mut complete = 0usize;
    let mut gap_keys: Vec<String> = Vec::new();

    for cat in catalogs(catalog.as_deref())? {
        let records = records_in(&cat)?;
        let results: parking_lot::Mutex<Vec<(PathBuf, Vec<String>)>> =
            parking_lot::Mutex::new(Vec::new());
        let errors: parking_lot::Mutex<Vec<(PathBuf, String)>> =
            parking_lot::Mutex::new(Vec::new());
        let next = AtomicUsize::new(0);
        let workers = jobs.get().min(records.len());
        std::thread::scope(|s| {
            for _ in 0..workers {
                s.spawn(|| loop {
                    let idx = next.fetch_add(1, Ordering::SeqCst);
                    if idx >= records.len() {
                        break;
                    }
                    let path = &records[idx];
                    let outcome = if apply {
                        measure_apply(path, locate_states)
                    } else {
                        measure_dry(path, locate_states)
                    };
                    match outcome {
                        Ok(gaps) => results.lock().push((path.clone(), gaps)),
                        Err(error) => {
                            errors.lock().push((path.clone(), format!("{error:#}")));
                        }
                    }
                });
            }
        });
        let mut errors = errors.into_inner();
        errors.sort_by(|a, b| a.0.cmp(&b.0));
        if !errors.is_empty() {
            let error_count = errors.len();
            let details = errors
                .into_iter()
                .map(|(path, error)| format!("{}: {error}", path.display()))
                .collect::<Vec<_>>()
                .join("\n");
            anyhow::bail!("could not measure {error_count} reference record(s):\n{details}");
        }
        let mut results = results.into_inner();
        results.sort_by(|a, b| a.0.cmp(&b.0));

        let cat_complete = results.iter().filter(|(_, gaps)| gaps.is_empty()).count();
        total += results.len();
        complete += cat_complete;
        for (_, gaps) in &results {
            for gap in gaps {
                gap_keys.push(gap_key(gap));
            }
        }
        println!(
            "{}: {cat_complete}/{} complete",
            cat.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            results.len()
        );

        let index = cat.join("references.json");
        if apply && index.exists() {
            let mut payload: Value = lib::read_json(&index)?;
            let by_path: HashMap<String, &Vec<String>> = results
                .iter()
                .map(|(path, gaps)| {
                    (
                        format!(
                            "references/{}/reference.json",
                            path.parent()
                                .and_then(|p| p.file_name())
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default()
                        ),
                        gaps,
                    )
                })
                .collect();
            if let Some(refs) = payload.get_mut("references").and_then(|v| v.as_array_mut()) {
                for r in refs.iter_mut() {
                    let Some(key) = r.get("path").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(gaps) = by_path.get(key) else {
                        continue;
                    };
                    if let Some(obj) = r.as_object_mut() {
                        obj.insert(
                            "evidence_status".into(),
                            json!(spelling(&if gaps.is_empty() {
                                EvidenceStatus::Complete
                            } else {
                                EvidenceStatus::Partial
                            })),
                        );
                        obj.insert("evidence_gap_count".into(), json!(gaps.len()));
                    }
                }
            }
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("schema".into(), json!(INDEX_SCHEMA));
                obj.insert("measured_at".into(), json!(TODAY));
                obj.insert("complete_count".into(), json!(cat_complete));
                obj.insert("partial_count".into(), json!(results.len() - cat_complete));
            }
            std::fs::write(&index, serde_json::to_string_pretty(&payload)? + "\n")?;
        }
    }

    println!(
        "\nmeasured {total} records, {complete} complete, {} partial",
        total - complete
    );
    // Each gap kind once, in the order first seen, then the most frequent
    // first (a stable sort keeps first-seen order among ties).
    let mut ranked: Vec<(String, usize)> = Vec::new();
    for key in &gap_keys {
        if !ranked.iter().any(|(seen, _)| seen == key) {
            let count = gap_keys.iter().filter(|other| *other == key).count();
            ranked.push((key.clone(), count));
        }
    }
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    for (key, count) in ranked {
        println!("{count:5}  {key}");
    }
    Ok(())
}

/// Apply mode: measure in place and rewrite the record file (always rewritten,
/// mirroring the Python tool).
pub(crate) fn measure_apply(path: &Path, locate_states: bool) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut data: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let base = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let gaps = measure_value(&mut data, &base, locate_states)?;
    std::fs::write(path, serde_json::to_string_pretty(&data)? + "\n")?;
    Ok(gaps)
}

/// Dry run: measure an in-memory copy of the record; nothing is written.
pub(crate) fn measure_dry(path: &Path, locate_states: bool) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut data: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let base = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let gaps = measure_value(&mut data, &base, locate_states)?;
    Ok(gaps)
}
