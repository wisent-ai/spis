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
        let name = record["name"]
            .as_str()
            .with_context(|| format!("record {} has no product name", path.display()))?;
        let prod = product_by_name(name).with_context(|| {
            format!(
                "record {} names product {name:?}, which the capture plan does not declare",
                path.display()
            )
        })?;
        let columns = motion["width"]
            .as_u64()
            .and_then(std::num::NonZeroU64::new)
            .with_context(|| {
                format!(
                    "record {} has no positive captured terminal width",
                    path.display()
                )
            })?;
        let width = overview["width"]
            .as_u64()
            .and_then(std::num::NonZeroU64::new)
            .with_context(|| format!("record {} has no positive overview width", path.display()))?;
        let height = overview["height"]
            .as_u64()
            .and_then(std::num::NonZeroU64::new)
            .with_context(|| {
                format!("record {} has no positive overview height", path.display())
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
                "orientation": if width >= height { "landscape" } else { "portrait" },
                "layout_model": "single-terminal-surface",
                "panel_summary": format!("One {columns}-column pseudo-terminal surface retaining the product help as selectable text."),
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
                "state_count": record["states"].as_array().map(Vec::len),
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
        "capture_host": host_facts()?.host.clone(),
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
