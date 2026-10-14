use super::*;

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut record: Option<String> = None;
    let mut host: Option<String> = None;
    let mut dry_run = false;

    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--record" => {
                i += 1;
                record =
                    Some(crate::commands::required(rest.get(i), "--record needs a value")?.clone());
            }
            "--host" => {
                i += 1;
                host =
                    Some(crate::commands::required(rest.get(i), "--host needs a value")?.clone());
            }
            "--dry-run" => dry_run = true,
            other => {
                if other.starts_with('-') || catalog.is_some() {
                    return Err(crate::commands::usage(format!("unknown argument: {other}")));
                }
                catalog = Some(other.to_string());
            }
        }
        i += 1;
    }
    let Some(catalog) = catalog else {
        return Err(crate::commands::usage("usage: spis capture-widths <catalog> [--record <NN|slug>] [--host <target>] [--dry-run]"));
    };

    let data = load_catalog(&catalog);
    let selected: Vec<(Map<String, Value>, Map<String, Value>)> = match &record {
        Some(selector) => vec![pick(&data.sources, &data.index, Some(selector))?],
        None => {
            let examples = catalog_list(&data.sources, "examples", "sources.json")?;
            let references = catalog_list(&data.index, "references", "references.json")?;
            examples
                .iter()
                .zip(references)
                .enumerate()
                .map(|(position, (e, r))| {
                    Ok((
                        catalog_object(e, "sources.json", "examples", position)?,
                        catalog_object(r, "references.json", "references", position)?,
                    ))
                })
                .collect::<Result<Vec<_>>>()?
        }
    };
    if selected.is_empty() {
        fail("nothing to capture; add records first");
    }

    let batch = format!("widths-{}", compact_stamp());
    let mut captures: Vec<Value> = Vec::new();
    for (example, entry) in &selected {
        let slug = slug_of(entry)?;
        let source_url = example
            .get("source_url")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
            .with_context(|| {
                format!("sources.json: example {slug} has no \"source_url\" to capture")
            })?;
        for (width, height) in WIDTHS {
            captures.push(json!({
                "batch": batch,
                "site_slug": format!("{slug}-{width}"),
                "source_url": source_url,
                "axis": "composition",
                "viewport": {"width": width, "height": height, "device_scale_factor": 1},
                "artifact_prefix": format!("{NAMESPACE}{batch}/{catalog}/{slug}/{width}/"),
                "full_page": true,
                "record_seconds": 0,
                "steps": [{"op": "settle"}],
            }));
        }
    }

    let host = crate::commands::crawl::weles_capture_host(host, "--host")?;
    let plan = json!({
        "schema": PLAN_SCHEMA,
        "batch": batch,
        "target": host,
        "captures": captures,
    });
    let plan_dir = work_root()?.join("landing-width-plans");
    std::fs::create_dir_all(&plan_dir)?;
    let plan_path = plan_dir.join(format!("{batch}.json"));
    std::fs::write(&plan_path, serde_json::to_string_pretty(&plan)? + "\n")?;

    if dry_run {
        println!(
            "dry run: planned {} captures across {} record(s); plan={}",
            captures.len(),
            selected.len(),
            plan_path.display()
        );
        for capture in &captures {
            println!(
                "  {} <- {} @ {}px",
                capture["site_slug"].as_str().unwrap_or_default(),
                capture["source_url"].as_str().unwrap_or_default(),
                capture["viewport"]["width"],
            );
        }
        return Ok(());
    }

    // One enqueue for every Weles capture plan Spis writes: the accessibility
    // audit's, which runs `stado workload run weles-capture` and checks every
    // action Stado returns against the plan. A refusal is Stado's own, with
    // its cause; Spis neither guesses from the words of an error nor submits
    // the batch a second way.
    let Some(plan_entries) = plan.as_object() else {
        bail!("the width plan for batch {batch} is not a JSON object");
    };
    crate::commands::audit_reference_accessibility::enqueue(&host, &plan_path, plan_entries)?;

    // Record the batch on each touched reference so retrieval can find it.
    for (_, entry) in &selected {
        let Some(relative) = entry["path"].as_str() else {
            bail!(
                "reference {} in {} has no \"path\", so its record cannot be updated with batch {batch}",
                Value::Object(entry.clone()),
                data.directory.display()
            );
        };
        let record_path = data.directory.join(relative);
        // A record that is not a JSON object is refused as a parse failure
        // naming the file; emptying it would write back a record holding
        // nothing but this batch.
        let mut record: Map<String, Value> =
            crate::read_json::<Map<String, Value>>(record_path.to_str().unwrap())?;
        append_unique(&mut record, "capture_batches", &record_path, batch.clone())?;
        append_unique(
            &mut record,
            "evidence_gaps",
            &record_path,
            "width captures enqueued; awaiting retrieval".to_string(),
        )?;
        crate::write_pretty_json(record_path.to_str().unwrap(), &Value::Object(record))?;
    }

    println!(
        "enqueued {} captures as {}; artifacts land under {NAMESPACE}{batch}/",
        captures.len(),
        batch
    );
    println!("retrieve with spis verify --apply after the host finishes the batch");
    Ok(())
}

/// Append `item` to the list under `key` of a record unless it is already
/// there. A field that exists but is not a list is refused naming the record
/// and what it holds, instead of being replaced.
fn append_unique(
    record: &mut Map<String, Value>,
    key: &str,
    record_path: &std::path::Path,
    item: String,
) -> Result<()> {
    let field = record
        .entry(key.to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    let Some(list) = field.as_array_mut() else {
        bail!(
            "{}: {key} holds {field} where a list of strings is expected; the record is left as it is",
            record_path.display()
        );
    };
    if !list.iter().any(|v| v.as_str() == Some(item.as_str())) {
        list.push(Value::String(item));
    }
    Ok(())
}
