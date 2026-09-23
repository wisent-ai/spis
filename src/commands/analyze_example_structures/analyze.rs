use super::*;

pub(crate) fn analyze(example: &Value, catalog: &str, image_path: &Path) -> Result<Value> {
    use image::GenericImageView;
    let payload = std::fs::read(image_path)?;
    let image = image::open(image_path)?;
    let (orig_w, orig_h) = image.dimensions();

    // PIL thumbnail((900, 900)): shrink-only, aspect preserved, LANCZOS.
    let factor = (900.0 / orig_w as f64).min(900.0 / orig_h as f64).min(1.0);
    let working = if factor < 1.0 {
        let nw = std::cmp::max(1, (orig_w as f64 * factor).round() as u32);
        let nh = std::cmp::max(1, (orig_h as f64 * factor).round() as u32);
        image.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3)
    } else {
        image.clone()
    };
    let rgb = working.to_rgb8();
    let (w, h) = (rgb.width() as usize, rgb.height() as usize);
    let mut gray = Gray {
        data: Vec::with_capacity(w * h),
        width: w,
        height: h,
    };
    for px in rgb.pixels() {
        gray.data
            .push((px[0] as f32 + px[1] as f32 + px[2] as f32) / 3.0);
    }

    let vertical = separator_positions(&gray, true);
    let horizontal = separator_positions(&gray, false);
    let (layout, summary, regions, confidence) =
        classify_layout(example, catalog, &vertical, &horizontal);

    let mut gradients: Vec<f32> = Vec::with_capacity(h * w.saturating_sub(1));
    for y in 0..h {
        for x in 0..w.saturating_sub(1) {
            gradients.push((gray.at(y, x + 1) - gray.at(y, x)).abs());
        }
    }
    let edge_density = if gradients.is_empty() {
        0.0
    } else {
        gradients.iter().filter(|&&g| g as f64 > 24.0).count() as f64 / gradients.len() as f64
    };
    let density = if edge_density >= 0.14 {
        "high"
    } else if edge_density >= 0.075 {
        "medium"
    } else {
        "low"
    };
    let orientation = if orig_w as f64 > orig_h as f64 * 1.1 {
        "landscape"
    } else if orig_h as f64 > orig_w as f64 * 1.1 {
        "portrait"
    } else {
        "square"
    };

    Ok(json!({
        "analysis_kind": "deterministic-image-layout-v1",
        "image_sha256": lib::sha256_hex(&payload),
        "orientation": orientation,
        "layout_model": layout,
        "panel_summary": summary,
        "regions": regions,
        "detected_separators": { "vertical": vertical, "horizontal": horizontal },
        "visual_density": density,
        "confidence": confidence,
    }))
}

pub(crate) fn analyze_catalog(slug: &str) -> Result<(usize, Vec<Value>)> {
    let source_path = Path::new(slug).join("sources.json");
    let mut catalog: Value = lib::read_json(source_path.to_str().unwrap())?;
    let mut failures: Vec<Value> = Vec::new();
    let mut count = 0usize;
    let example_count = catalog
        .get("examples")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    for position in 0..example_count {
        let index = position + 1;
        let (name, local_path) = {
            let example = &catalog["examples"][position];
            (
                example
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                example
                    .get("visual")
                    .and_then(|v| v.get("local_path"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
            )
        };
        match local_path {
            None => failures.push(json!({"index": index, "name": name, "error": "visual missing"})),
            Some(local_path) => {
                let image_path = Path::new(slug).join(&local_path);
                let example = catalog["examples"][position].clone();
                match analyze(&example, slug, &image_path) {
                    Ok(structure) => {
                        if let Some(obj) = catalog["examples"][position].as_object_mut() {
                            obj.insert("interface_structure".into(), structure);
                        }
                        count += 1;
                    }
                    Err(error) => failures
                        .push(json!({"index": index, "name": name, "error": format!("{error:#}")})),
                }
            }
        }
    }

    if failures.is_empty() && count == example_count {
        if let Some(obj) = catalog.as_object_mut() {
            obj.insert("schema".into(), json!("wisent.example-catalog.v2"));
            obj.insert("visual_count".into(), json!(count));
            obj.insert("structure_count".into(), json!(count));
        }
    }
    std::fs::write(&source_path, serde_json::to_string_pretty(&catalog)? + "\n")
        .with_context(|| format!("write {}", source_path.display()))?;
    let failure_path = Path::new(slug).join("structure-analysis-failures.json");
    if !failures.is_empty() {
        std::fs::write(
            &failure_path,
            serde_json::to_string_pretty(&failures)? + "\n",
        )?;
    } else {
        let _ = std::fs::remove_file(&failure_path);
    }
    println!("{slug}: analyzed={count} failures={}", failures.len());
    Ok((count, failures))
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut requested: Vec<String> = Vec::new();
    for arg in rest {
        if arg.starts_with('-') {
            bail!("unknown argument: {arg}");
        }
        requested.push(arg.clone());
    }
    let mut unknown: Vec<&String> = requested
        .iter()
        .filter(|c| !CATALOGS.contains(&c.as_str()))
        .collect();
    unknown.sort();
    if !unknown.is_empty() {
        let names: Vec<&str> = unknown.iter().map(|s| s.as_str()).collect();
        bail!("unknown catalog(s): {}", names.join(", "));
    }
    let selected: Vec<&str> = if requested.is_empty() {
        CATALOGS.to_vec()
    } else {
        requested.iter().map(String::as_str).collect()
    };

    let mut failures = 0usize;
    for slug in selected {
        let (_, catalog_failures) = analyze_catalog(slug)?;
        failures += catalog_failures.len();
    }
    if failures > 0 {
        bail!("structure analysis left {failures} unresolved entries");
    }
    Ok(())
}
