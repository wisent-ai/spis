//! The three catalog-type subcommands. The schema names are the reference
//! contract's, and every count written is counted from what is written.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;

use super::{load_sources, parse_args, present, regenerate, save_sources};
use crate::commands::reference_contract::{CATALOG_SCHEMA, INDEX_SCHEMA};

pub(super) fn add(rest: &[String]) -> Result<()> {
    let args = parse_args(rest, &["title", "description"])?;
    let slug = args.slug()?;
    let directory = PathBuf::from(&slug);
    if present(&directory)? {
        bail!("{} already exists", directory.display());
    }
    let title = match &args.title {
        Some(title) if !title.is_empty() => title.clone(),
        _ => bail!("--title is required for add"),
    };
    // The corpus refuses a catalog without a description, so add does too,
    // before anything is created.
    let description = match &args.description {
        Some(description) if !description.trim().is_empty() => description.clone(),
        _ => bail!("--description is required for add"),
    };
    std::fs::create_dir(&directory).with_context(|| format!("creating {}", directory.display()))?;
    std::fs::create_dir(directory.join("references"))
        .with_context(|| format!("creating {}/references", directory.display()))?;
    // A new catalog holds no examples and no references yet; its counts are
    // those of the empty lists it is written with.
    let examples: Vec<Value> = Vec::new();
    let references: Vec<Value> = Vec::new();
    let sources = json!({
        "schema": CATALOG_SCHEMA,
        "catalog": slug,
        "slug": slug,
        "title": title,
        "description": description,
        "status": "scaffolded",
        "curated_at": crate::today_utc(),
        "count": examples.len(),
        "examples": examples,
        "visual_count": examples.len(),
        "structure_count": examples.len(),
    });
    save_sources(&directory, &sources)?;
    let index = directory.join("references.json");
    std::fs::write(
        &index,
        serde_json::to_string_pretty(&json!({
            "schema": INDEX_SCHEMA,
            "catalog": slug,
            "generated_at": crate::now_iso_utc(),
            "reference_count": references.len(),
            "complete_count": references.len(),
            "partial_count": references.len(),
            "references": references,
        }))? + "\n",
    )
    .with_context(|| format!("writing {}", index.display()))?;
    // A refused catalog is not left behind: it would make every later
    // regeneration refuse the corpus for the same reason.
    if let Err(refused) = regenerate() {
        match std::fs::remove_dir_all(&directory) {
            Ok(()) => bail!("{refused:#}\nso {} was removed again", directory.display()),
            Err(error) => bail!(
                "{refused:#}\nand {} could not be removed ({error})",
                directory.display()
            ),
        }
    }
    println!(
        "added {} ({title}); scaffolded with no records",
        directory.display()
    );
    Ok(())
}

pub(super) fn edit(rest: &[String]) -> Result<()> {
    let args = parse_args(rest, &["title", "description", "status", "rename"])?;
    let slug = args.slug()?;
    let directory = PathBuf::from(&slug);
    if !present(&directory)? {
        bail!("{} does not exist", directory.display());
    }
    let mut sources = load_sources(&directory)?;
    let object = sources
        .as_object_mut()
        .with_context(|| format!("{}/sources.json is not a JSON object", directory.display()))?;
    let mut changed: Vec<String> = Vec::new();
    let mut new_slug: Option<String> = None;
    for (field, value) in [
        ("title", &args.title),
        ("description", &args.description),
        ("status", &args.status),
    ] {
        if let Some(value) = value {
            object.insert(field.into(), json!(value));
            changed.push(field.into());
        }
    }
    if let Some(rename) = &args.rename {
        let target = super::normalize(rename)?;
        let new_directory = PathBuf::from(&target);
        if present(&new_directory)? {
            bail!("{} already exists", new_directory.display());
        }
        object.insert("slug".into(), json!(target));
        object.insert("catalog".into(), json!(target));
        changed.push(format!("slug -> {target}"));
        new_slug = Some(target);
    }
    if changed.is_empty() {
        bail!("nothing to edit: pass --title, --description, --status, or --rename");
    }
    save_sources(&directory, &sources)?;
    if let Some(target) = &new_slug {
        std::fs::rename(&directory, target)
            .with_context(|| format!("renaming {} to {target}", directory.display()))?;
    }
    regenerate()?;
    println!("edited {}: {}", slug, changed.join(", "));
    Ok(())
}

pub(super) fn remove(rest: &[String]) -> Result<()> {
    let args = parse_args(rest, &["force"])?;
    let slug = args.slug()?;
    let directory = PathBuf::from(&slug);
    if !present(&directory)? {
        bail!("{} does not exist", directory.display());
    }
    let sources = load_sources(&directory)?;
    // A catalog holds its evidence in `examples`, the list `add` scaffolds and
    // every reader of sources.json reads.
    let examples = sources
        .get("examples")
        .and_then(Value::as_array)
        .with_context(|| {
            format!(
                "{}/sources.json states no examples list",
                directory.display()
            )
        })?;
    let references_dir = directory.join("references");
    let stored = if present(&references_dir)? {
        std::fs::read_dir(&references_dir)
            .with_context(|| format!("listing {}", references_dir.display()))?
            .collect::<std::io::Result<Vec<_>>>()
            .with_context(|| format!("listing {}", references_dir.display()))?
    } else {
        Vec::new()
    };
    if !(examples.is_empty() && stored.is_empty()) && !args.force {
        bail!(
            "{} holds {} example(s) and {} reference director(ies); passing --force deletes that evidence permanently",
            directory.display(),
            examples.len(),
            stored.len()
        );
    }
    std::fs::remove_dir_all(&directory)
        .with_context(|| format!("removing {}", directory.display()))?;
    regenerate()?;
    println!("removed {}", directory.display());
    Ok(())
}
