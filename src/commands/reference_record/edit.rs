use super::*;

/// The metadata `add` takes from its flags that `edit` may change afterwards.
/// The name is not among them: it is the record's directory slug and image
/// file name, so a different name is a different record (`remove` then `add`).
#[derive(Debug, Clone, Default)]
pub struct EditArgs {
    pub catalog: String,
    pub identifier: String,
    pub source_url: Option<String>,
    pub category: Option<String>,
    pub selection_note: Option<String>,
    pub owner: Option<String>,
}

/// Change the metadata of one existing record in `sources.json` and its
/// `reference.json`, then refresh the generated indexes. Evidence fields
/// (motion, states, journey, accessibility) are measured by the pipeline and
/// are not editable here.
pub(crate) fn edit(args: &EditArgs) -> Result<()> {
    if args.source_url.is_none()
        && args.category.is_none()
        && args.selection_note.is_none()
        && args.owner.is_none()
    {
        return Err(crate::commands::usage(
            "reference: edit changes nothing without at least one of --source-url, --category, --selection-note or --owner",
        ));
    }
    let directory = catalog_dir(&args.catalog)?;
    let mut sources: Value = lib::read_json(&directory.join("sources.json").to_string_lossy())?;
    let mut index: Value = lib::read_json(&directory.join("references.json").to_string_lossy())?;
    let position = find_record(&directory, &index, &args.identifier)?;
    let record_path = directory.join(
        index["references"][position]["path"]
            .as_str()
            .with_context(|| {
                format!(
                    "reference: references[{position}].path is missing in {}",
                    directory.display()
                )
            })?,
    );
    let mut record: Value = lib::read_json(&record_path.to_string_lossy())?;
    let example = sources["examples"]
        .get_mut(position)
        .with_context(|| format!("reference: sources.json has no example at position {position} though references.json does"))?;

    let mut changed = Vec::new();
    if let Some(url) = &args.source_url {
        example["source_url"] = json!(url);
        example["visual"]["source_page_url"] = json!(url);
        record["product_url"] = json!(url);
        changed.push("source-url");
    }
    if let Some(category) = &args.category {
        example["category"] = json!(category);
        changed.push("category");
    }
    if let Some(note) = &args.selection_note {
        example["selection_note"] = json!(note);
        changed.push("selection-note");
    }
    if let Some(owner) = &args.owner {
        example["visual"]["source_image_url"] = json!(owner);
        record["upstream_owner"] = json!(owner);
        changed.push("owner");
    }
    let name = example["name"].as_str().unwrap_or_default().to_string();

    lib::write_pretty_json(&record_path.to_string_lossy(), &record)?;
    save_all(&directory, &mut sources, &mut index)?;
    println!(
        "edited {}/{}: {name} ({})",
        directory
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        args.identifier,
        changed.join(", ")
    );
    Ok(())
}
