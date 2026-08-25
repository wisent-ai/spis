//! `spis reference-record` — Rust port of `reference-record.py`.
//!
//! Add, get, or remove a single reference record inside a product-type catalog.
//!
//! A record is one numbered product reference: an overview image plus
//! `references/<NN-slug>/reference.json`. Adding scaffolds the record honestly —
//! motion, states, journey, and accessibility start empty and are named in
//! `evidence_gaps`, so the record is `partial` until the pipeline measures it.
//! The generated index is refreshed after every mutation.

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Catalog slug -> directory, appending `-examples` when missing.
fn catalog_dir(slug: &str) -> Result<PathBuf> {
    let named = if slug.ends_with("-examples") {
        slug.to_string()
    } else {
        format!("{slug}-examples")
    };
    let directory = PathBuf::from(&named);
    if !directory.is_dir() {
        bail!("reference: {named} does not exist");
    }
    Ok(directory)
}

fn kebab(name: &str) -> Result<String> {
    let mut slug = String::new();
    let mut prev_sep = true;
    for c in name.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            slug.push(c);
            prev_sep = false;
        } else if !prev_sep && !slug.is_empty() {
            slug.push('-');
            prev_sep = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        bail!("reference: --name {name:?} produces an empty slug");
    }
    Ok(slug)
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(lib::sha256_hex(&bytes))
}

/// Image dimensions via macOS `sips`, exactly like the original script.
fn image_dimensions(path: &Path) -> Result<(i64, i64)> {
    let output = std::process::Command::new("sips")
        .args(["-g", "pixelWidth", "-g", "pixelHeight"])
        .arg(path)
        .output()
        .context("run sips")?;
    if !output.status.success() {
        bail!(
            "reference: cannot read image dimensions: {}",
            path.display()
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut width = None;
    let mut height = None;
    for line in stdout.lines() {
        if line.contains("pixelWidth") {
            width = line.rsplit(':').next().and_then(|v| v.trim().parse().ok());
        }
        if line.contains("pixelHeight") {
            height = line.rsplit(':').next().and_then(|v| v.trim().parse().ok());
        }
    }
    match (width, height) {
        (Some(w), Some(h)) => Ok((w, h)),
        _ => bail!(
            "reference: cannot read image dimensions: {}",
            path.display()
        ),
    }
}

fn today_iso() -> String {
    lib::now_iso_utc()[..10].to_string()
}

/// Refresh the counters on both catalog files and regenerate the rendered indexes.
fn save_all(directory: &Path, sources: &mut Value, index: &mut Value) -> Result<()> {
    let examples = sources["examples"].as_array().map(Vec::len).unwrap_or(0);
    sources["count"] = json!(examples);
    sources["visual_count"] = json!(examples);
    sources["structure_count"] = json!(examples);
    lib::write_pretty_json(&directory.join("sources.json").to_string_lossy(), sources)?;

    let references = index["references"].as_array().cloned().unwrap_or_default();
    index["reference_count"] = json!(references.len());
    index["complete_count"] = json!(references
        .iter()
        .filter(|r| r["evidence_status"] == "complete")
        .count());
    index["partial_count"] = json!(references
        .iter()
        .filter(|r| r["evidence_status"] == "partial")
        .count());
    let now = lib::now_iso_utc();
    index["generated_at"] = json!(now.clone());
    index["measured_at"] = json!(now);
    lib::write_pretty_json(&directory.join("references.json").to_string_lossy(), index)?;

    super::reference_contract::regenerate_index()
}

/// Locate a record by 1-based number or normalized name; returns its position.
fn find_record(directory: &Path, index: &Value, identifier: &str) -> Result<usize> {
    let number: Option<usize> =
        if !identifier.is_empty() && identifier.chars().all(|c| c.is_ascii_digit()) {
            identifier.parse().ok()
        } else {
            None
        };
    let wanted = identifier.to_lowercase();
    let references = index["references"].as_array().cloned().unwrap_or_default();
    for (position, entry) in references.iter().enumerate() {
        let name_key = entry["name"]
            .as_str()
            .unwrap_or_default()
            .to_lowercase()
            .replace('_', "-")
            .replace(' ', "-");
        if number == Some(position + 1) || name_key == wanted {
            return Ok(position);
        }
    }
    bail!(
        "reference: record {:?} not found in {}",
        identifier,
        directory.display()
    )
}

#[derive(Debug, Clone)]
pub struct AddArgs {
    pub catalog: String,
    pub name: String,
    pub source_url: String,
    pub category: String,
    pub selection_note: String,
    pub visual: String,
    pub owner: Option<String>,
}

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

fn get(catalog: &str, identifier: &str) -> Result<()> {
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
fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn remove(catalog: &str, identifier: &str, force: bool) -> Result<()> {
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
struct FlagSpec {
    name: &'static str,
    takes_value: bool,
}

/// Minimal argparse stand-in: `--flag value`, `--flag=value`, `store_true` flags,
/// and interleaved positionals collected in order.
fn parse_flags(
    rest: &[String],
    value_specs: &[FlagSpec],
    boolean_specs: &[FlagSpec],
) -> Result<(Vec<String>, Vec<(String, Option<String>)>)> {
    let all: Vec<&FlagSpec> = value_specs.iter().chain(boolean_specs.iter()).collect();
    let mut positionals = Vec::new();
    let mut flags = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        if !arg.starts_with("--") {
            positionals.push(arg);
            i += 1;
            continue;
        }
        let (name, inline_value) = match arg.split_once('=') {
            Some((n, v)) => (n.to_string(), Some(v.to_string())),
            None => (arg.clone(), None),
        };
        let spec = all
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| anyhow::anyhow!("reference: unrecognized argument {name}"))?;
        if !spec.takes_value {
            flags.push((spec.name.to_string(), None));
            i += 1;
            continue;
        }
        let value = match inline_value {
            Some(v) => v,
            None => {
                i += 1;
                rest.get(i).cloned().ok_or_else(|| {
                    anyhow::anyhow!("reference: argument {name}: expected one argument")
                })?
            }
        };
        flags.push((spec.name.to_string(), Some(value)));
        i += 1;
    }
    Ok((positionals, flags))
}

fn require_flag(flags: &[(String, Option<String>)], name: &str) -> Result<String> {
    flags
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| v.clone())
        .ok_or_else(|| anyhow::anyhow!("reference: the following arguments are required: {name}"))
}

fn optional_flag(flags: &[(String, Option<String>)], name: &str) -> Option<String> {
    flags
        .iter()
        .find(|(n, _)| n == name)
        .and_then(|(_, v)| v.clone())
}

fn require_positionals(positionals: &[String], names: &[&str]) -> Result<Vec<String>> {
    if positionals.len() < names.len() {
        bail!(
            "reference: the following arguments are required: {}",
            names[positionals.len()..].join(" ")
        );
    }
    Ok(positionals[..names.len()].to_vec())
}

const ADD_SPECS: &[FlagSpec] = &[
    FlagSpec {
        name: "--name",
        takes_value: true,
    },
    FlagSpec {
        name: "--source-url",
        takes_value: true,
    },
    FlagSpec {
        name: "--category",
        takes_value: true,
    },
    FlagSpec {
        name: "--selection-note",
        takes_value: true,
    },
    FlagSpec {
        name: "--visual",
        takes_value: true,
    },
    FlagSpec {
        name: "--owner",
        takes_value: true,
    },
];

/// `spis reference-record <add|get|remove> ...`
pub fn run(rest: &[String]) -> Result<()> {
    let Some(command) = rest.first() else {
        bail!(
            "reference: usage: spis reference-record <add|get|remove> [flags] \
             (add <catalog> --name N --source-url U --category C --selection-note S \
             --visual F [--owner O]; get <catalog> <NN|slug>; \
             remove <catalog> <NN|slug> [--force])"
        );
    };
    match command.as_str() {
        "add" => {
            let (mut positionals, flags) = parse_flags(&rest[1..], ADD_SPECS, &[])?;
            if positionals.is_empty() {
                bail!("reference: the following arguments are required: catalog");
            }
            let catalog = positionals.remove(0);
            add(&AddArgs {
                catalog,
                name: require_flag(&flags, "--name")?,
                source_url: require_flag(&flags, "--source-url")?,
                category: require_flag(&flags, "--category")?,
                selection_note: require_flag(&flags, "--selection-note")?,
                visual: require_flag(&flags, "--visual")?,
                owner: optional_flag(&flags, "--owner"),
            })
        }
        "get" | "remove" => {
            let extra_specs: &[FlagSpec] = if command == "get" {
                &[]
            } else {
                &[FlagSpec {
                    name: "--force",
                    takes_value: false,
                }]
            };
            let (positionals, flags) = parse_flags(&rest[1..], &[], extra_specs)?;
            let pos = require_positionals(&positionals, &["catalog", "identifier"])?;
            if command == "get" {
                get(&pos[0], &pos[1])
            } else {
                let force = flags.iter().any(|(n, _)| n == "--force");
                remove(&pos[0], &pos[1], force)
            }
        }
        other => bail!("reference: unknown command {other:?}"),
    }
}
