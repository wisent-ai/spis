use super::*;

pub(crate) fn catalogs(selected: Option<&str>) -> Result<Vec<PathBuf>> {
    let mut found: Vec<String> = std::fs::read_dir(".")?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| {
            name.ends_with("-examples")
                && Path::new(name).join("references").is_dir()
        })
        .collect();
    found.sort();
    let mut expected: Vec<String> = KNOWN_CATALOGS.iter().map(|value| value.to_string()).collect();
    expected.sort();
    if found != expected {
        anyhow::bail!(
            "catalog set differs from the exact 15-family contract; found {:?}, expected {:?}",
            found,
            expected
        );
    }
    if let Some(selected) = selected {
        if !KNOWN_CATALOGS.contains(&selected) {
            anyhow::bail!(
                "unknown catalog {selected}; exact corpus contains only {}",
                KNOWN_CATALOGS.join(", ")
            );
        }
        return Ok(vec![PathBuf::from(selected)]);
    }
    Ok(KNOWN_CATALOGS.iter().map(PathBuf::from).collect())
}

pub(crate) fn records_in(catalog: &Path) -> Result<Vec<PathBuf>> {
    let mut records: Vec<PathBuf> = std::fs::read_dir(catalog.join("references"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("reference.json"))
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
    if records.len() != RECORDS_PER_CATALOG || source_count != RECORDS_PER_CATALOG {
        anyhow::bail!(
            "{}: exact contract requires {RECORDS_PER_CATALOG} records and sources, found {} records and {source_count} sources",
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
    let mut jobs: usize = 8;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--catalog" => {
                i += 1;
                catalog = Some(rest.get(i).context("--catalog needs a value")?.clone());
            }
            "--apply" => apply = true,
            "--no-state-match" => no_state_match = true,
            "--jobs" => {
                i += 1;
                jobs = rest
                    .get(i)
                    .context("--jobs needs a value")?
                    .parse()
                    .context("--jobs expects an integer")?;
            }
            other => anyhow::bail!("unknown argument: {other}"),
        }
        i += 1;
    }
    let locate_states = !no_state_match;

    if !apply {
        println!("dry run: records are measured and reported, nothing is written");
    }

    let mut total = 0usize;
    let mut complete = 0usize;
    let mut gap_counter: Vec<(String, usize)> = Vec::new();

    for cat in catalogs(catalog.as_deref())? {
        let records = records_in(&cat)?;
        let results: parking_lot::Mutex<Vec<(PathBuf, Vec<String>)>> =
            parking_lot::Mutex::new(Vec::new());
        let errors: parking_lot::Mutex<Vec<(PathBuf, String)>> =
            parking_lot::Mutex::new(Vec::new());
        let next = AtomicUsize::new(0);
        let workers = jobs.max(1).min(records.len().max(1));
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
            let details = errors.into_iter()
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
                let key = gap_key(gap);
                if let Some(slot) = gap_counter.iter_mut().find(|(k, _)| *k == key) {
                    slot.1 += 1;
                } else {
                    gap_counter.push((key, 1));
                }
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
            let mut payload: Value = lib::read_json(index.to_str().context("non-UTF8 path")?)?;
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
                            json!(if gaps.is_empty() {
                                "complete"
                            } else {
                                "partial"
                            }),
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
    let mut ranked = gap_counter;
    ranked.sort_by(|a, b| b.1.cmp(&a.1)); // stable: ties keep first-seen order
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
