use super::*;

pub(crate) fn write_index() -> Result<Value> {
    let records = load_records()?;
    let mut references = Vec::new();
    for (i, (path, record)) in records.iter().enumerate() {
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .with_context(|| {
                format!(
                    "{} is not inside a named reference directory",
                    path.display()
                )
            })?;
        // Every record the capture writes states its status and its gap list;
        // one that states neither is refused, never indexed as pending with
        // no gaps.
        let evidence_status = record
            .get("evidence_status")
            .filter(|status| status.is_string())
            .cloned()
            .with_context(|| format!("{} states no evidence_status", path.display()))?;
        let evidence_gap_count = record
            .get("evidence_gaps")
            .and_then(|g| g.as_array())
            .map(Vec::len)
            .with_context(|| format!("{} states no evidence_gaps list", path.display()))?;
        references.push(json!({
            "index": i + 1,
            "name": record["name"].clone(),
            "path": format!("references/{parent}/reference.json"),
            "evidence_status": evidence_status,
            "evidence_gap_count": evidence_gap_count,
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
    println!("Products of the plan on this host ({}):", host_sentence()?);
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
            QuickOutcome::Ran(status, first) if status.success() => first,
            // `status` prints the exit code or the signal that ended it.
            QuickOutcome::Ran(status, first) => {
                format!(
                    "(no version flag; `{}` ended with {status}) {first}",
                    product.version_cmd
                )
            }
            QuickOutcome::Failed(cause) => format!("(version command did not run) {cause}"),
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
                return Err(crate::commands::usage(format!(
                    "unknown flag: {other}\n{USAGE}"
                )))
            }
        }
    }
    let Some(plan_path) = plan_path else {
        return Err(crate::commands::usage(format!(
            "--plan is required\n{USAGE}"
        )));
    };
    let loaded = load_plan(&plan_path)?;
    for slug in &wanted {
        if !loaded.products.iter().any(|product| &product.slug == slug) {
            return Err(crate::commands::usage(format!(
                "--product {slug} is not in {}; it declares: {}",
                plan_path.display(),
                loaded
                    .products
                    .iter()
                    .map(|product| product.slug.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    }

    if list {
        return cmd_list();
    }

    let catalog = catalog_dir();
    std::fs::create_dir_all(&catalog)?;
    std::fs::create_dir_all(catalog.join("references"))?;
    std::fs::create_dir_all(scratch_root()?)?;

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
                motion["duration_seconds"],
                motion["frame_count"],
                json!(record["states"].as_array().map(Vec::len)),
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
    println!(
        "next: spis verify-reference-evidence --catalog {} --apply",
        plan().catalog
    );
    Ok(())
}
