use super::*;

pub fn run(rest: &[String]) -> Result<()> {
    let mut check = false;
    for arg in rest {
        match arg.as_str() {
            "--check" => check = true,
            "--help" | "-h" => {
                println!("usage: spis generate-example-catalogs [--check]");
                println!("  --check  validate only, write nothing");
                return Ok(());
            }
            other => return Err(crate::commands::usage(format!("unknown argument: {other}"))),
        }
    }

    let slugs = discovered_catalogs()?;
    let mut catalogs: Vec<Value> = Vec::new();
    for slug in &slugs {
        catalogs.push(load_catalog(slug)?);
    }

    if check {
        for (slug, catalog) in slugs.iter().zip(&catalogs) {
            let index = full_reference_index(catalog);
            println!(
                "{slug}: {} complete, {} partial, {}",
                index["complete_count"],
                index["partial_count"],
                provenance_sentence(slug, &index["measured_provenance"])?
            );
        }
        return Ok(());
    }

    let generated_at = catalogs
        .iter()
        .filter_map(|c| c["curated_at"].as_str())
        .max()
        .expect("curated_at present on every loaded catalog");

    let catalog_entries: Vec<Value> = catalogs
        .iter()
        .map(|catalog| {
            let index = full_reference_index(catalog);
            let slug = catalog["catalog"].as_str().unwrap_or("");
            json!({
                "slug": slug,
                "title": catalog["title"],
                "description": catalog["description"],
                "count": catalog["count"],
                "image_count": catalog["visual_count"],
                "structure_count": catalog["structure_count"],
                "complete_record_count": index["complete_count"],
                "partial_record_count": index["partial_count"],
                "measured_provenance": index["measured_provenance"],
                "source": format!("{slug}/sources.json"),
                "full_reference_source": format!("{slug}/references.json"),
            })
        })
        .collect();

    // Every count is summed from what each catalog states; a catalog that
    // states none is refused naming it, never added as zero.
    let sum = |name: &str, read: &dyn Fn(&Value) -> Value| -> Result<u64> {
        catalogs
            .iter()
            .map(|catalog| {
                read(catalog).as_u64().with_context(|| {
                    format!(
                        "catalog {} states no {name}",
                        catalog["slug"]
                            .as_str()
                            .map_or_else(|| catalog.to_string(), str::to_string)
                    )
                })
            })
            .sum::<Result<u64>>()
    };
    let record_count = sum("reference_count", &|c| {
        full_reference_index(c)["reference_count"].clone()
    })?;
    let complete_record_count = sum("complete_count", &|c| {
        full_reference_index(c)["complete_count"].clone()
    })?;
    let partial_record_count = sum("partial_count", &|c| {
        full_reference_index(c)["partial_count"].clone()
    })?;
    if complete_record_count + partial_record_count != record_count {
        bail!(
            "generated status counts cover {} of {record_count} records",
            complete_record_count + partial_record_count
        );
    }

    let index = json!({
        "schema": CATALOG_SCHEMA,
        "generated_at": generated_at,
        "catalog_count": catalogs.len(),
        "example_count": sum("count", &|c| c["count"].clone())?,
        "image_count": sum("visual_count", &|c| c["visual_count"].clone())?,
        "structure_count": sum("structure_count", &|c| c["structure_count"].clone())?,
        "record_count": record_count,
        "complete_record_count": complete_record_count,
        "partial_record_count": partial_record_count,
        "locally_driven_motion_count": sum("locally_driven_count", &|c| full_reference_index(c)["locally_driven_count"].clone())?,
        "catalogs": catalog_entries,
    });
    lib::write_pretty_json("example-catalogs.json", &index)?;
    let stats = json!({
        "schema": "wisent.catalog-stats.v1",
        "generated_at": index["generated_at"],
        "catalog_count": index["catalog_count"],
        "record_count": index["record_count"],
        "complete_record_count": index["complete_record_count"],
        "partial_record_count": index["partial_record_count"],
    });
    lib::write_pretty_json("catalog-stats.json", &stats)?;
    Ok(())
}
