use super::*;

pub(crate) fn write_sources() -> Result<Value> {
    let records = load_records()?;
    let mut examples = Vec::new();
    for (path, record) in &records {
        let motion = &record["motion"][0];
        let overview = &record["states"][1];
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let slug = parent
            .split_once('-')
            .map(|(_, rest)| rest)
            .unwrap_or(parent);
        let name = record["name"].as_str().with_context(|| {
            format!("record {} has no product name", path.display())
        })?;
        let prod = product_by_name(name).with_context(|| {
            format!(
                "record {} names product {name:?}, which the capture plan does not declare",
                path.display()
            )
        })?;
        examples.push(json!({
            "name": record["name"].clone(),
            "slug": slug,
            "source_url": record["product_url"].clone(),
            "repository": record["repository"].clone(),
            "category": prod.category,
            "selection_note": prod.selection_note,
            "installed": record["installed"].clone(),
            "reference_path": format!("references/{parent}/reference.json"),
            "visual": {
                "source_page_url": record["product_url"].clone(),
                "source_recording_path": format!("references/{parent}/media/session.cast"),
                "local_path": format!("references/{parent}/{}", overview["local_path"].as_str().unwrap_or_default()),
                "capture_kind": "local-terminal-render",
                "captured_at": record["captured_at"].clone(),
                "format": "png",
                "width": overview["width"].clone(),
                "height": overview["height"].clone(),
                "bytes": overview["bytes"].clone(),
                "sha256": overview["sha256"].clone(),
            },
            "interface_structure": {
                "analysis_kind": "deterministic-terminal-layout-v1",
                "image_sha256": overview["sha256"].clone(),
                "orientation": if overview["width"].as_i64().unwrap_or(0) >= overview["height"].as_i64().unwrap_or(0) { "landscape" } else { "portrait" },
                "layout_model": "single-terminal-surface",
                "panel_summary": "One 100-column pseudo-terminal surface retaining the product help as selectable text.",
                "regions": [{
                    "role": "terminal transcript",
                    "position": "full canvas",
                    "bounds": {"x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0},
                }],
                "detected_separators": [],
                "visual_density": "medium",
                "confidence": 1.0,
            },
            "evidence": {
                "kind": "asciinema-v2-terminal-cast plus deterministic renders of it",
                "local_path": format!("references/{parent}/media/session.cast"),
                "duration_seconds": motion["duration_seconds"].clone(),
                "frame_count": motion["frame_count"].clone(),
                "bytes": motion["bytes"].clone(),
                "sha256": motion["sha256"].clone(),
                "state_count": record["states"].as_array().map(|v| v.len()).unwrap_or(0),
                "captured_at": record["captured_at"].clone(),
            },
        }));
    }

    let payload = json!({
        "schema": SOURCES_SCHEMA,
        "catalog": catalog_dir().file_name().and_then(|n| n.to_str()).unwrap_or_default(),
        "title": plan().title,
        "description": "The products of the capture plan with a runnable CLI on the capture host, each measured by running it: \
                        version form, top-level help, one subcommand help surface, one invalid flag, a Ctrl-C on an \
                        unsubmitted line, the recovering help, and the same help with NO_COLOR=1.",
        "catalog_scope": format!(
            "This catalog is bounded by the capture plan and the capture host: it contains one record for each \
             of the {} planned products with a runnable CLI on this workstation, and nothing else. It is not a \
             curated fifty and not a sample of a company's product surface; a planned product whose CLI is not \
             installed here is absent by construction, and products that ship only as an app, a web application \
             or a service are not planned.",
            examples.len()
        ),
        "capture_host": host_facts().host.clone(),
        "excluded_from_scope": plan().exclusions.iter().map(|exclusion| json!({
            "binary": exclusion.binary,
            "resolved": exclusion.resolved,
            "reason": exclusion.reason,
        })).collect::<Vec<_>>(),
        "curated_at": today_utc(),
        "count": examples.len(),
        "visual_count": examples.len(),
        "structure_count": examples.len(),
        "examples": examples,
    });
    std::fs::write(
        catalog_dir().join("sources.json"),
        serde_json::to_string_pretty(&payload)? + "\n",
    )?;
    Ok(payload)
}

pub(crate) fn write_index() -> Result<Value> {
    let records = load_records()?;
    let mut references = Vec::new();
    for (i, (path, record)) in records.iter().enumerate() {
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        references.push(json!({
            "index": i + 1,
            "name": record["name"].clone(),
            "path": format!("references/{parent}/reference.json"),
            "evidence_status": record.get("evidence_status").cloned().unwrap_or(json!("pending-verification")),
            "evidence_gap_count": record.get("evidence_gaps").and_then(|g| g.as_array()).map(|a| a.len()).unwrap_or(0),
        }));
    }
    let complete_count = references
        .iter()
        .filter(|r| r["evidence_status"].as_str() == Some("complete"))
        .count();
    let payload = json!({
        "schema": INDEX_SCHEMA,
        "catalog": catalog_dir().file_name().and_then(|n| n.to_str()).unwrap_or_default(),
        "generated_at": today_utc(),
        "reference_count": references.len(),
        "complete_count": complete_count,
        "partial_count": references.len() - complete_count,
        "references": references,
    });
    std::fs::write(
        catalog_dir().join("references.json"),
        serde_json::to_string_pretty(&payload)? + "\n",
    )?;
    Ok(payload)
}

// ----------------------------------------------------------------------- cli

pub(crate) fn cmd_list() -> Result<()> {
    println!("Products of the plan on this host ({}):", host_sentence());
    println!();
    let mut found = 0usize;
    for (index, product) in products().iter().enumerate() {
        let (path, outcome) = quick_version(product);
        let path = match path {
            Some(p) => p,
            None => {
                println!(
                    "{:2}. {:<26} MISSING (`{}` not on PATH)",
                    index + 1,
                    product.name,
                    product.binary
                );
                continue;
            }
        };
        found += 1;
        let identity = match outcome {
            QuickOutcome::Ran(0, first) => first,
            QuickOutcome::Ran(rc, first) => {
                format!(
                    "(no version flag; `{}` exits {rc}) {first}",
                    product.version_cmd
                )
            }
            QuickOutcome::Failed(tag) => {
                format!(
                    "(no version flag; `{}` exits None) {tag}",
                    product.version_cmd
                )
            }
            QuickOutcome::Missing => unreachable!(),
        };
        println!(
            "{:2}. {:<26} {:<38} {}",
            index + 1,
            product.name,
            product.repository,
            path
        );
        println!("{:<4}{:<40} -> {}", "", product.version_cmd, identity);
    }
    println!();
    println!(
        "{} of {} planned products are installed and runnable here.",
        found,
        products().len()
    );
    println!();
    println!("Excluded from scope:");
    for exclusion in &plan().exclusions {
        println!("  {:<14} {}", exclusion.binary, exclusion.reason);
    }
    Ok(())
}

const USAGE: &str = "usage: spis capture-cli-references --plan <file.json> [--list] [--product <slug>]... [--catalog-only]";

pub fn run(rest: &[String]) -> Result<()> {
    let mut list = false;
    let mut catalog_only = false;
    let mut wanted: Vec<String> = Vec::new();
    let mut plan_path: Option<PathBuf> = None;
    let mut it = rest.iter();
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--help" | "-h" => {
                println!("{USAGE}\n\n  --plan <file.json>  the capture plan ({PLAN_SCHEMA}): catalog, products, exclusions\n  --list              run each planned product's version probe and report what is installed\n  --product <slug>    capture only this planned product; repeatable\n  --catalog-only      rebuild sources.json and references.json from the records on disk");
                return Ok(());
            }
            "--list" => list = true,
            "--catalog-only" => catalog_only = true,
            "--plan" => {
                let path = it
                    .next()
                    .ok_or_else(|| crate::commands::usage("--plan requires a path"))?;
                plan_path = Some(crate::commands::operator_path(path));
            }
            "--product" => {
                let slug = it
                    .next()
                    .ok_or_else(|| crate::commands::usage("--product requires a slug"))?;
                wanted.push(slug.clone());
            }
            other => {
                return Err(crate::commands::usage(format!("unknown flag: {other}\n{USAGE}")))
            }
        }
    }
    let Some(plan_path) = plan_path else {
        return Err(crate::commands::usage(format!("--plan is required\n{USAGE}")));
    };
    let loaded = load_plan(&plan_path)?;
    for slug in &wanted {
        if !loaded.products.iter().any(|product| &product.slug == slug) {
            return Err(crate::commands::usage(format!(
                "--product {slug} is not in {}; it declares: {}",
                plan_path.display(),
                loaded.products.iter().map(|product| product.slug.as_str()).collect::<Vec<_>>().join(", ")
            )));
        }
    }

    if list {
        return cmd_list();
    }

    let catalog = catalog_dir();
    std::fs::create_dir_all(&catalog)?;
    std::fs::create_dir_all(catalog.join("references"))?;
    std::fs::create_dir_all(scratch_root())?;

    if !catalog_only {
        font(terminal().font_px as f32)?;
        let selected: Vec<&Product> = products()
            .iter()
            .filter(|p| wanted.is_empty() || wanted.iter().any(|w| w == &p.slug))
            .collect();
        let missing: Vec<&str> = selected
            .iter()
            .filter(|p| resolve(p).is_none())
            .map(|p| p.binary.as_str())
            .collect();
        if !missing.is_empty() {
            bail!("not on PATH: {}", missing.join(", "));
        }
        for (i, product) in products().iter().enumerate() {
            if !selected.iter().any(|s| std::ptr::eq(*s, product)) {
                continue;
            }
            println!("[{:02}/{:02}] {}", i + 1, products().len(), product.name);
            let run = capture(i + 1, product)?;
            let measured = measure(&run);
            let (ref_dir, record) = write_reference(&run, &measured)?;
            let motion = &record["motion"][0];
            let rel = ref_dir
                .strip_prefix(root())
                .unwrap_or(&ref_dir)
                .to_string_lossy()
                .into_owned();
            println!(
                "    -> {rel}: {} s, {} events, {} states",
                g(motion["duration_seconds"].as_f64().unwrap_or(0.0)),
                motion["frame_count"],
                record["states"].as_array().map(|v| v.len()).unwrap_or(0),
            );
        }
    }

    let sources = write_sources()?;
    let index_payload = write_index()?;
    println!();
    println!(
        "catalog {}: {} products, {} records written",
        sources["catalog"].as_str().unwrap_or_default(),
        sources["count"],
        index_payload["reference_count"]
    );
    println!("next: spis verify-reference-evidence --catalog {} --apply", plan().catalog);
    Ok(())
}
