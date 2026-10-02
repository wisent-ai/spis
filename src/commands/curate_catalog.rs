//! `spis curate-catalog --selector FILE [--apply] [--replace]`: write the
//! capture-pending records of one catalog from a declared selector file.
//!
//! The selector is data the operator edits, not a list compiled in: a header
//! of `key: value` lines (`catalog`, `title`, `description`, `category`,
//! `count`), a blank line, then one `Name<TAB>URL` source per line. The
//! command creates attributed candidates only; visual, structure and semantic
//! evidence stay pending until Weles artifacts are imported and measured.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

struct Selector {
    catalog: String,
    title: String,
    description: String,
    category: String,
    count: usize,
    sources: Vec<(String, String)>,
}

fn slug(value: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            out.push(character);
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Read a selector file; every header key is required and every source line
/// is `Name<TAB>URL`, so a malformed line is refused by its number.
fn read_selector(path: &Path) -> Result<Selector> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading selector {}", path.display()))?;
    let (header, body) = text
        .split_once("\n\n")
        .with_context(|| format!("{}: a selector has a header, a blank line, then one source per line", path.display()))?;
    let mut field = |key: &str| -> Result<String> {
        header
            .lines()
            .find_map(|line| line.strip_prefix(key).and_then(|rest| rest.strip_prefix(':')))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .with_context(|| format!("{}: the header has no `{key}:` line", path.display()))
    };
    let catalog = field("catalog")?;
    let title = field("title")?;
    let description = field("description")?;
    let category = field("category")?;
    let count: usize = field("count")?.parse().with_context(|| format!("{}: `count:` is not a number", path.display()))?;
    let mut sources = Vec::new();
    for (offset, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((name, url)) = line.split_once('\t') else {
            bail!("{}: source line {} is not `Name<TAB>URL`: {line:?}", path.display(), offset + 1);
        };
        sources.push((name.trim().to_string(), url.trim().to_string()));
    }
    Ok(Selector { catalog, title, description, category, count, sources })
}

fn validate(selector: &Selector) -> Result<()> {
    let category = &selector.category;
    if selector.sources.len() != selector.count {
        bail!("{category}: the selector declares {} sources and lists {}", selector.count, selector.sources.len());
    }
    let mut names = BTreeSet::new();
    let mut urls = BTreeSet::new();
    for (name, url) in &selector.sources {
        if !names.insert(name.to_ascii_lowercase()) || !urls.insert(url.as_str()) {
            bail!("{category}: duplicate {name} / {url}");
        }
        if !url.starts_with("https://") {
            bail!("{category}: source is not HTTPS: {url}");
        }
        if category == "pricing" {
            let lower = url.to_ascii_lowercase();
            if !lower.contains("pricing") && !lower.contains("plan") {
                bail!("pricing source does not identify pricing/plans: {url}");
            }
        }
    }
    Ok(())
}

fn write_catalog(selector: &Selector, replace: bool) -> Result<()> {
    let catalog = selector.catalog.as_str();
    let root = Path::new(catalog);
    let references = root.join("references");
    if references.exists() {
        if references.read_dir()?.next().is_some() && !replace {
            bail!("{catalog}: references are non-empty; pass --replace");
        }
        std::fs::remove_dir_all(&references)?;
    }
    let today = crate::now_iso_utc()[..10].to_string();
    let mut examples = Vec::new();
    let mut index = Vec::new();
    for (offset, (name, url)) in selector.sources.iter().enumerate() {
        let number = offset + 1;
        let directory_name = format!("{number:02}-{}", slug(name));
        let directory = references.join(&directory_name);
        std::fs::create_dir_all(&directory)?;
        let gaps: Vec<&str> = "Weles capture not imported;motion evidence absent;state visuals below the three-state floor;interaction variants absent;journey variants absent;motion analysis absent;accessibility variants absent"
            .split(';')
            .collect();
        let record = json!({
            "schema": "wisent.full-product-reference.v2",
            "name": name,
            "product_url": url,
            "evidence_status": "partial",
            "upstream_owner": name,
            "captured_at": today,
            "motion": [],
            "states": [],
            "interactions": [],
            "journey": Value::Null,
            "motion_analysis": Value::Null,
            "accessibility": {"measured": false, "observations": [], "unknowns": ["Weles accessibility variants not executed yet"]},
            "motion_provenance": [],
            "evidence_gaps": gaps,
        });
        std::fs::write(directory.join("reference.json"), serde_json::to_string_pretty(&record)? + "\n")?;
        examples.push(json!({
            "name": name,
            "source_url": url,
            "category": selector.category,
            "selection_note": "Official destination declared by the catalog's selector file; Weles must prove the family surface before completeness.",
            "visual": {"capture_status": "pending-weles"},
            "interface_structure": {"analysis_status": "pending-weles"}
        }));
        index.push(json!({
            "index": number,
            "name": name,
            "path": format!("references/{directory_name}/reference.json"),
            "evidence_status": "partial",
            "evidence_gap_count": gaps.len()
        }));
    }
    crate::write_pretty_json(root.join("sources.json").to_str().context("sources path is not UTF-8")?, &json!({
        "schema": "wisent.example-catalog.v2",
        "catalog": catalog,
        "slug": catalog,
        "title": selector.title,
        "description": selector.description,
        "status": "capture-pending",
        "curated_at": today,
        "count": selector.sources.len(),
        "examples": examples,
        "visual_count": 0,
        "structure_count": 0
    }))?;
    crate::write_pretty_json(root.join("references.json").to_str().context("index path is not UTF-8")?, &json!({
        "schema": "wisent.full-reference-catalog.v2",
        "catalog": catalog,
        "reference_count": selector.sources.len(),
        "references": index
    }))?;
    Ok(())
}

const USAGE: &str = "usage: spis curate-catalog --selector FILE [--apply] [--replace]\n\n\
The selector is a text file: `catalog:`, `title:`, `description:`, `category:` and `count:` \
header lines, a blank line, then one `Name<TAB>URL` source per line. Without --apply the \
selector is only validated; --replace overwrites a catalog whose references are not empty.";

pub fn run(rest: &[String]) -> Result<()> {
    if rest.iter().any(|value| value == "--help" || value == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    let mut selector_path = None;
    let mut apply = false;
    let mut replace = false;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--selector" => {
                index += 1;
                selector_path = Some(super::required(rest.get(index), "--selector needs a file")?.clone());
            }
            "--apply" => apply = true,
            "--replace" => replace = true,
            other => return Err(super::usage(format!("unknown argument {other}\n{USAGE}"))),
        }
        index += 1;
    }
    let selector_path = super::required(selector_path, USAGE)?;
    let selector = read_selector(Path::new(&selector_path))?;
    validate(&selector)?;
    if !apply {
        println!(
            "{}: {} attributable candidates in {}\npass --apply to write capture-pending records",
            selector.catalog, selector.sources.len(), selector.category
        );
        return Ok(());
    }
    write_catalog(&selector, replace)?;
    println!("wrote {} capture-pending records into {}", selector.sources.len(), selector.catalog);
    Ok(())
}
