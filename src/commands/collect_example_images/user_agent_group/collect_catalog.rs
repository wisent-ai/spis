use super::*;

/// Collect each example's official visual into `slug`: the
/// `visual_source_url` the catalog declares, or the example's own page when
/// it declares none. An example whose visual cannot be fetched is a failure
/// carrying the fetch's own error; no other source stands in for it.
pub(crate) fn collect_catalog(slug: &str, replace: bool) -> Result<(usize, Vec<Value>)> {
    let source_path = PathBuf::from(slug).join("sources.json");
    let mut catalog: Value = crate::read_json(&source_path)?;
    let mut failures: Vec<Value> = Vec::new();
    let mut collected: Vec<String> = Vec::new();

    let positions: Vec<usize> = catalog
        .get("examples")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{slug}/sources.json: examples must be an array"))?
        .iter()
        .enumerate()
        .map(|(position, _)| position)
        .collect();
    let total = positions.len();

    for position in positions {
        let index = position + std::num::NonZeroUsize::MIN.get();
        let example_obj: &mut Map<String, Value> = catalog
            .get_mut("examples")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| anyhow!("{slug}/sources.json: examples must be an array"))?
            .get_mut(position)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| {
                anyhow!("{slug}/sources.json: examples[{position}] must be an object")
            })?;
        let field = |key: &str| -> Result<String> {
            let text = example_obj
                .get(key)
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty());
            Ok(text
                .with_context(|| {
                    format!("{slug}/sources.json: examples[{position}] has no \"{key}\"")
                })?
                .to_string())
        };
        let (name, source_url) = (field("name")?, field("source_url")?);

        if !replace {
            if let Some(visual) = example_obj.get_mut("visual").and_then(Value::as_object_mut) {
                visual.insert(
                    "source_page_url".to_string(),
                    Value::String(source_url.clone()),
                );
                if let Some(image_url) = visual.get("source_image_url").and_then(Value::as_str) {
                    let normalized = stable_provenance_url(image_url);
                    visual.insert("source_image_url".to_string(), Value::String(normalized));
                    write_catalog(&source_path, &catalog)?;
                }
                continue;
            }
        }

        // visual_source_url is the catalog's optional pointer at a better
        // image than the page itself; without it the page is the source.
        let visual_url = match example_obj.get("visual_source_url").and_then(Value::as_str) {
            Some(declared) if !declared.is_empty() => declared.to_string(),
            _ => source_url.clone(),
        };
        match store_image(Path::new(slug), index, &name, &visual_url) {
            Ok(mut visual) => {
                visual["capture_kind"] = json!("official-source-image");
                visual["source_page_url"] = Value::String(source_url.clone());
                example_obj.insert("visual".to_string(), visual);
                println!("{slug} {index}/{total} image {name}");
                collected.push(name);
            }
            Err(error) => {
                let error = format!("{error:#}");
                println!("{slug} {index}/{total} FAILED {name}: {error}");
                failures.push(json!({
                    "index": index,
                    "name": name,
                    "url": visual_url,
                    "error": error,
                }));
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
    } else {
        // A previous run's failure report that stays would read as this
        // clean run's failures, so one that cannot be removed refuses.
        crate::discard(&failure_path)?;
    }
    Ok((collected.len(), failures))
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut selected: Vec<&str> = Vec::new();
    let mut replace = false;
    for arg in rest {
        match arg.as_str() {
            "--replace" => replace = true,
            other if !other.starts_with('-') => selected.push(other),
            other => return Err(crate::commands::usage(format!("unknown argument: {other}\nusage: spis collect-example-images [--replace] [catalog ...]"))),
        }
    }
    let families: Vec<String> = crate::commands::reference_contract::Family::ALL
        .iter()
        .map(|family| family.name())
        .collect();
    let unknown: Vec<&str> = selected
        .iter()
        .copied()
        .filter(|name| !families.iter().any(|family| family == name))
        .collect();
    if !unknown.is_empty() {
        return Err(crate::commands::usage(format!(
            "unknown catalog(s): {}\nusage: spis collect-example-images [--replace] [catalog ...]",
            unknown.join(", ")
        )));
    }
    let chosen: Vec<&str> = if selected.is_empty() {
        families.iter().map(String::as_str).collect()
    } else {
        selected
    };

    let mut unresolved: Vec<Value> = Vec::new();
    for slug in chosen {
        let (collected, failures) = collect_catalog(slug, replace)?;
        println!("{slug}: collected={collected} failures={}", failures.len());
        unresolved.extend(failures);
    }
    if !unresolved.is_empty() {
        bail!(
            "image collection left {} unresolved entries; each catalog's image-collection-failures.json holds their errors",
            unresolved.len()
        );
    }
    Ok(())
}
