use super::*;

/// Scaffold one numbered record: overview image, structure placeholder,
/// `references/<NN-slug>/reference.json`, and an index entry.
pub fn add(args: &AddArgs) -> Result<()> {
    let directory = catalog_dir(&args.catalog)?;
    let mut sources: Value = lib::read_json(&directory.join("sources.json").to_string_lossy())?;
    let mut index: Value = lib::read_json(&directory.join("references.json").to_string_lossy())?;

    let visual_source = PathBuf::from(&args.visual);
    if !visual_source.is_file() {
        bail!("reference: --visual {} is not a file", args.visual);
    }
    let count_before = sources["examples"].as_array().map(Vec::len).unwrap_or(0);
    let duplicate = sources["examples"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|example| {
            example["name"].as_str().map(|n| n.to_lowercase()) == Some(args.name.to_lowercase())
        });
    if duplicate {
        bail!("reference: a record named {:?} already exists", args.name);
    }

    let slug = format!("{:02}-{}", count_before + 1, kebab(&args.name)?);
    let extension = visual_source
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let suffix = if extension.is_empty() {
        "png".to_string()
    } else {
        extension
    };
    let images_dir = directory.join("images");
    if !images_dir.exists() {
        std::fs::create_dir(&images_dir)
            .with_context(|| format!("mkdir {}", images_dir.display()))?;
    }
    let visual_path = images_dir.join(format!("{slug}.{suffix}"));
    std::fs::copy(&visual_source, &visual_path)
        .with_context(|| format!("copy {} -> {}", args.visual, visual_path.display()))?;

    let (width, height) = image_dimensions(&visual_path)?;
    let digest = sha256_file(&visual_path)?;
    let today = today_iso();

    let orientation = if width >= height {
        "landscape"
    } else {
        "portrait"
    };
    let structure = json!({
        "analysis_kind": "deterministic-image-layout-v1",
        "image_sha256": digest,
        "orientation": orientation,
        "layout_model": "unanalyzed-scaffold",
        "panel_summary": "Scaffold region covering the full overview image; run analyze-structures to replace it.",
        "detected_separators": {"vertical": [], "horizontal": []},
        "visual_density": "unknown",
        "confidence": "low",
        "regions": [
            {
                "role": "full frame",
                "position": "center",
                "bounds": {"x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0},
                "evidence": "placeholder bounds over the whole scaffolded image",
            }
        ],
    });

    let example = json!({
        "name": args.name,
        "source_url": args.source_url,
        "category": args.category,
        "selection_note": args.selection_note,
        "visual": {
            "source_page_url": args.source_url,
            "source_image_url": args.owner.clone().unwrap_or_else(|| args.source_url.clone()),
            "local_path": format!("images/{}", visual_path.file_name().unwrap_or_default().to_string_lossy()),
            "capture_kind": "provided-file",
            "captured_at": today,
            "format": suffix,
            "width": width,
            "height": height,
            "original_width": width,
            "original_height": height,
            "bytes": visual_path.metadata().map(|m| m.len()).unwrap_or(0),
            "sha256": digest,
        },
        "interface_structure": structure,
    });
    sources["examples"]
        .as_array_mut()
        .context("sources.examples must be an array")?
        .push(example);

    let gaps = [
        "motion evidence absent",
        "first-success sequence not recorded",
        "state visuals below the three-state floor",
        "interaction map absent",
        "user journey not recorded",
        "motion analysis absent",
        "accessibility never measured against the product",
    ];
    let now = lib::now_iso_utc();
    let record_dir = directory.join("references").join(&slug);
    std::fs::create_dir_all(&record_dir)
        .with_context(|| format!("mkdir {}", record_dir.display()))?;
    let media_dir = record_dir.join("media");
    if !media_dir.exists() {
        std::fs::create_dir(&media_dir)
            .with_context(|| format!("mkdir {}", media_dir.display()))?;
    }
    let record = json!({
        "schema": super::reference_contract::RECORD_SCHEMA,
        "name": args.name,
        "product_url": args.source_url,
        "evidence_status": "partial",
        "upstream_owner": args.owner.clone().unwrap_or_else(|| args.source_url.clone()),
        "captured_at": today,
        "motion": [],
        "states": [],
        "interactions": [],
        "journey": {},
        "accessibility": {
            "measured": false,
            "observations": [],
            "unknowns": ["everything; no audit exists yet"],
        },
        "motion_provenance": [],
        "evidence_gaps": gaps,
        "measured_at": now,
    });
    lib::write_pretty_json(
        &record_dir.join("reference.json").to_string_lossy(),
        &record,
    )?;
    index["references"]
        .as_array_mut()
        .context("references must be an array")?
        .push(json!({
            "index": count_before + 1,
            "name": args.name,
            "path": format!("references/{slug}/reference.json"),
            "evidence_status": "partial",
            "evidence_gap_count": gaps.len(),
        }));

    save_all(&directory, &mut sources, &mut index)?;
    println!(
        "added {}/{slug}: {} ({} named gaps, status partial)",
        directory
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default()
            .into_owned(),
        args.name,
        gaps.len()
    );
    Ok(())
}

pub(crate) fn get(catalog: &str, identifier: &str) -> Result<()> {
    let directory = catalog_dir(catalog)?;
    let sources: Value = lib::read_json(&directory.join("sources.json").to_string_lossy())?;
    let index: Value = lib::read_json(&directory.join("references.json").to_string_lossy())?;
    let position = find_record(&directory, &index, identifier)?;
    let example = sources["examples"][position].clone();
    let entry = index["references"][position].clone();
    let record_path = directory.join(entry["path"].as_str().unwrap_or_default());
    let record: Value = lib::read_json(&record_path.to_string_lossy())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "example": example,
            "entry": entry,
            "record": record,
        }))?
    );
    Ok(())
}

/// Python truthiness for a JSON value (empty containers are falsy).
pub(crate) fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

pub(crate) fn remove(catalog: &str, identifier: &str, force: bool) -> Result<()> {
    let directory = catalog_dir(catalog)?;
    let mut sources: Value = lib::read_json(&directory.join("sources.json").to_string_lossy())?;
    let mut index: Value = lib::read_json(&directory.join("references.json").to_string_lossy())?;
    let position = find_record(&directory, &index, identifier)?;
    let record_path = directory.join(
        index["references"][position]["path"]
            .as_str()
            .unwrap_or_default(),
    );
    let record: Value = lib::read_json(&record_path.to_string_lossy())?;
    if (truthy(&record["motion"]) || truthy(&record["journey"])) && !force {
        bail!("reference: the record carries motion or journey evidence; pass --force to delete it permanently");
    }

    let record_dir = record_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| record_path.clone());
    std::fs::remove_dir_all(&record_dir)
        .with_context(|| format!("remove {}", record_dir.display()))?;
    let visual_local = sources["examples"][position]["visual"]["local_path"]
        .as_str()
        .map(str::to_string);
    if let Some(local) = visual_local {
        let visual_path = directory.join(local);
        if visual_path.is_file() {
            std::fs::remove_file(&visual_path)
                .with_context(|| format!("remove {}", visual_path.display()))?;
        }
    }
    sources["examples"]
        .as_array_mut()
        .context("sources.examples must be an array")?
        .remove(position);
    {
        let references = index["references"]
            .as_array_mut()
            .context("references must be an array")?;
        references.remove(position);
        for (new_index, entry_after) in references.iter_mut().enumerate() {
            entry_after["index"] = json!(new_index + 1);
        }
    }

    save_all(&directory, &mut sources, &mut index)?;
    println!(
        "removed {} record {identifier}",
        directory
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default()
            .into_owned()
    );
    Ok(())
}

/// One flag spec: long name plus whether it consumes a value.
pub(crate) struct FlagSpec {
    pub(crate) name: &'static str,
    pub(crate) takes_value: bool,
}
