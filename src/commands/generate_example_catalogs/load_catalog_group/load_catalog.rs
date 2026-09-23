use super::*;

pub(crate) fn load_catalog(slug: &str) -> Result<Value> {
    let source_path = Path::new(slug).join("sources.json");
    let source_path_str = source_path.to_string_lossy().to_string();
    let mut catalog: Value = lib::read_json(source_path.to_str().unwrap())?;

    if catalog.get("schema").and_then(Value::as_str) != Some(CATALOG_SCHEMA) {
        bail!("{source_path_str}: expected schema '{CATALOG_SCHEMA}'");
    }
    if catalog.get("catalog").and_then(Value::as_str) != Some(slug) {
        bail!("{source_path_str}: catalog must equal directory name");
    }

    let examples: Vec<Value> = catalog
        .get("examples")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| anyhow!("{source_path_str}: examples must be a list"))?;

    if examples.len() != RECORDS_PER_CATALOG {
        bail!(
            "{source_path_str}: exact corpus contract requires {RECORDS_PER_CATALOG} sources, found {}",
            examples.len()
        );
    }

    if catalog.get("count").and_then(Value::as_u64) != Some(examples.len() as u64) {
        bail!("{source_path_str}: count does not match the {} examples", examples.len());
    }
    let measured_visuals = examples.iter().filter(|example| {
        example.pointer("/visual/capture_status").and_then(Value::as_str) != Some("pending-weles")
    }).count() as u64;
    let measured_structures = examples.iter().filter(|example| {
        example.pointer("/interface_structure/analysis_status").and_then(Value::as_str) != Some("pending-weles")
    }).count() as u64;
    if catalog.get("visual_count").and_then(Value::as_u64) != Some(measured_visuals) {
        bail!("{source_path_str}: visual_count does not match the {measured_visuals} retained captures");
    }
    if catalog.get("structure_count").and_then(Value::as_u64) != Some(measured_structures) {
        bail!("{source_path_str}: structure_count does not match the {measured_structures} retained analyses");
    }

    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut urls: BTreeSet<String> = BTreeSet::new();
    let catalog_dir = Path::new(slug);
    for (offset, example) in examples.iter().enumerate() {
        let index = offset + 1;
        let required = [
            "name",
            "source_url",
            "category",
            "selection_note",
            "visual",
            "interface_structure",
        ];
        let missing: Vec<&str> = required
            .iter()
            .filter(|key| !py_truthy(example.get(**key)))
            .copied()
            .collect();
        if !missing.is_empty() {
            bail!(
                "{source_path_str}: example {index} is missing {}",
                missing_list(&missing)
            );
        }
        let source_url = example
            .get("source_url")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !url_ok(source_url) {
            bail!("{source_path_str}: example {index} has an invalid URL");
        }
        let name = example.get("name").and_then(Value::as_str).unwrap_or("");
        let folded_name = name.to_lowercase();
        if !names.insert(folded_name) {
            bail!("{source_path_str}: duplicate example name '{name}'");
        }
        if !urls.insert(source_url.to_string()) {
            bail!("{source_path_str}: duplicate source URL '{source_url}'");
        }

        let pending_visual = example.pointer("/visual/capture_status").and_then(Value::as_str) == Some("pending-weles");
        let pending_structure = example.pointer("/interface_structure/analysis_status").and_then(Value::as_str) == Some("pending-weles");
        if pending_visual || pending_structure {
            if pending_visual && pending_structure {
                continue;
            }
            bail!("{source_path_str}: example {index} has mismatched pending visual and structure evidence");
        }

        let visual = example.get("visual").expect("checked above");
        let mut visual_required: Vec<&str> = vec![
            "source_page_url",
            "local_path",
            "capture_kind",
            "captured_at",
            "format",
            "width",
            "height",
            "bytes",
            "sha256",
        ];
        let capture_kind = visual
            .get("capture_kind")
            .and_then(Value::as_str)
            .unwrap_or("");
        if capture_kind == "local-terminal-render" {
            visual_required.push("source_recording_path");
        } else if capture_kind != "local-browser-screenshot" {
            visual_required.push("source_image_url");
        }
        let visual_missing: Vec<&str> = visual_required
            .iter()
            .filter(|key| !py_truthy(visual.get(**key)))
            .copied()
            .collect();
        if !visual_missing.is_empty() {
            bail!(
                "{source_path_str}: example {index} visual is missing {}",
                missing_list(&visual_missing)
            );
        }
        let visual_local = visual
            .get("local_path")
            .and_then(Value::as_str)
            .unwrap_or("");
        let image_path = catalog_dir.join(visual_local);
        let contained = resolve_evidence_path(catalog_dir, visual_local, &source_path_str).is_ok();
        if !contained || !image_path.is_file() {
            bail!("{source_path_str}: example {index} visual path is unavailable");
        }
        let payload = std::fs::read(&image_path)?;
        if Some(payload.len() as u64) != visual.get("bytes").and_then(Value::as_u64) {
            bail!("{source_path_str}: example {index} visual byte count differs");
        }
        if Some(lib::sha256_hex(&payload).as_str()) != visual.get("sha256").and_then(Value::as_str)
        {
            bail!("{source_path_str}: example {index} visual digest differs");
        }

        let structure = example.get("interface_structure").expect("checked above");
        let structure_required = [
            "analysis_kind",
            "image_sha256",
            "orientation",
            "layout_model",
            "panel_summary",
            "regions",
            "detected_separators",
            "visual_density",
            "confidence",
        ];
        // Python tests structure.get(key) in (None, "") here: zero and false pass.
        let structure_missing: Vec<&str> = structure_required
            .iter()
            .filter(|key| {
                matches!(structure.get(**key), None | Some(Value::Null))
                    || structure.get(**key).and_then(Value::as_str) == Some("")
            })
            .copied()
            .collect();
        if !structure_missing.is_empty() {
            bail!(
                "{source_path_str}: example {index} structure is missing {}",
                missing_list(&structure_missing)
            );
        }
        if structure.get("image_sha256") != visual.get("sha256") {
            bail!("{source_path_str}: example {index} structure describes another image");
        }
        let regions_ok = structure
            .get("regions")
            .and_then(Value::as_array)
            .map(|r| !r.is_empty())
            .unwrap_or(false);
        if !regions_ok {
            bail!("{source_path_str}: example {index} has no structural regions");
        }
    }

    catalog
        .as_object_mut()
        .expect("catalog is an object")
        .insert(
            "full_reference_catalog".into(),
            load_full_references(slug, &examples)?,
        );
    Ok(catalog)
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Comma-separated "N label" sentence, strongest-provenance first.
pub(crate) fn provenance_sentence(measured_provenance: &Value) -> String {
    let mut pairs: Vec<(String, usize)> = measured_provenance
        .as_object()
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as usize))
                .collect()
        })
        .unwrap_or_default();
    pairs.sort_by(|a, b| b.1.cmp(&a.1)); // stable: ties stay in stored order
    if pairs.is_empty() {
        return "no measured motion".into();
    }
    pairs
        .iter()
        .map(|(name, count)| format!("{count} {}", provenance_label(name)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn full_reference_index<'a>(catalog: &'a Value) -> &'a Value {
    catalog
        .get("full_reference_catalog")
        .expect("attached by loader")
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub(crate) fn discovered_catalogs() -> Result<Vec<String>> {
    let mut found: Vec<String> = std::fs::read_dir(".")?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| {
            name.ends_with("-examples")
                && Path::new(name).join("references").is_dir()
        })
        .collect();
    found.sort();
    let mut expected: Vec<String> = CATALOGS.iter().map(|value| value.to_string()).collect();
    expected.sort();
    if found != expected {
        bail!(
            "catalog set differs from the exact 15-family contract; found {:?}, expected {:?}",
            found,
            expected
        );
    }
    Ok(CATALOGS.iter().map(|value| value.to_string()).collect())
}
