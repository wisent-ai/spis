use super::*;

pub(crate) fn collect_catalog(slug: &str, replace: bool) -> Result<(usize, Vec<Value>)> {
    let source_path = PathBuf::from(slug).join("sources.json");
    let mut catalog: Value = crate::read_json(source_path.to_str().context("path")?)?;
    let mut failures: Vec<Value> = Vec::new();
    let mut collected = 0usize;

    let total = catalog
        .get("examples")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{slug}/sources.json: examples must be an array"))?
        .len();

    for position in 0..total {
        let index = position + 1;
        let example_obj: &mut Map<String, Value> = catalog
            .get_mut("examples")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| anyhow!("{slug}/sources.json: examples must be an array"))?
            .get_mut(position)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| {
                anyhow!("{slug}/sources.json: examples[{position}] must be an object")
            })?;
        let name = example_obj["name"].as_str().unwrap_or_default().to_string();
        let source_url = example_obj["source_url"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        if example_obj
            .get("visual")
            .map(Value::is_object)
            .unwrap_or(false)
            && !replace
        {
            let visual = example_obj
                .get_mut("visual")
                .and_then(Value::as_object_mut)
                .unwrap();
            visual.insert(
                "source_page_url".to_string(),
                Value::String(source_url.clone()),
            );
            if let Some(image_url) = visual.get("source_image_url").and_then(Value::as_str) {
                let normalized = stable_provenance_url(image_url);
                visual.insert(
                    "source_image_url".to_string(),
                    Value::String(normalized.clone()),
                );
                write_catalog(&source_path, &catalog)?;
            }
            continue;
        }

        let primary_url = example_obj
            .get("visual_source_url")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| source_url.clone());
        let mut attempt_urls: Vec<(String, &'static str)> =
            vec![(primary_url.clone(), "official-source-image")];
        attempt_urls.push((
            format!("https://image.thum.io/get/width/1400/crop/1000/noanimate/{source_url}"),
            "remote-page-screenshot",
        ));

        let mut stored: Option<Value> = None;
        let mut last_error: Option<String> = None;
        for (attempt_index, (capture_url, kind)) in attempt_urls.iter().enumerate() {
            match store_image(Path::new(slug), index, &name, capture_url) {
                Ok(mut visual) => {
                    visual["capture_kind"] = json!(kind);
                    stored = Some(visual);
                    break;
                }
                Err(e) => {
                    last_error = Some(format!("{e:#}"));
                    // Only fall through to the screenshot service after the
                    // primary source failed.
                    let _ = attempt_index;
                }
            }
        }

        match stored {
            Some(mut visual) => {
                visual["source_page_url"] = Value::String(source_url.clone());
                example_obj.insert("visual".to_string(), visual);
                collected += 1;
                println!("{slug} {index:02}/50 image {name}");
            }
            None => {
                let error = last_error.unwrap_or_else(|| "unknown failure".to_string());
                failures.push(json!({
                    "index": index,
                    "name": name,
                    "url": primary_url,
                    "error": error,
                }));
                println!(
                    "{slug} {index:02}/50 FAILED {name}: {}",
                    failures
                        .last()
                        .and_then(|f| f["error"].as_str())
                        .unwrap_or("")
                );
            }
        }
        write_catalog(&source_path, &catalog)?;
    }

    let failure_path = Path::new(slug).join("image-collection-failures.json");
    if !failures.is_empty() {
        std::fs::write(
            &failure_path,
            serde_json::to_string_pretty(&failures)? + "\n",
        )?;
    } else if failure_path.exists() {
        std::fs::remove_file(&failure_path)?;
    }
    Ok((collected, failures))
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut selected: Vec<&str> = Vec::new();
    let mut replace = false;
    for arg in rest {
        match arg.as_str() {
            "--replace" => replace = true,
            other if !other.starts_with('-') => selected.push(other),
            other => bail!("unknown argument: {other}\nusage: spis collect-example-images [--replace] [catalog ...]"),
        }
    }
    let unknown: Vec<&&str> = selected.iter().filter(|s| !CATALOGS.contains(s)).collect();
    if !unknown.is_empty() {
        eprintln!(
            "usage: spis collect-example-images [--replace] [catalog ...]\n\
             collect-example-images: error: unknown catalog(s): {}",
            unknown
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        std::process::exit(2);
    }
    let chosen: Vec<&str> = if selected.is_empty() {
        CATALOGS.to_vec()
    } else {
        selected
    };

    let mut total_failures = 0usize;
    for slug in chosen {
        let (collected, failures) = collect_catalog(slug, replace)?;
        total_failures += failures.len();
        println!("{slug}: collected={collected} failures={}", failures.len());
    }
    if total_failures > 0 {
        eprintln!("image collection left {total_failures} unresolved entries");
        std::process::exit(1);
    }
    Ok(())
}
