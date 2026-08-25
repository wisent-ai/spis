//! `spis catalog-type` — add, edit, or remove a product-type catalog.
//!
//! A product type is one `*-examples/` directory: a family of reference records
//! with its own sources, index, and evidence floor. This tool only scaffolds and
//! maintains the structure; it never fabricates records. After every mutation the
//! index is refreshed by re-execing this binary with `generate-example-catalogs`
//! (Main confirmed R3 ports that subcommand under this exact name).
//!
//! Ported 1:1 from the former catalog-type.py.

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

const SUFFIX: &str = "-examples";

fn fail(message: &str) -> ! {
    eprintln!("catalog-type: {message}");
    std::process::exit(1);
}

/// UTC date (YYYY-MM-DD). The Python original used the local date; the Rust
/// tooling elsewhere is UTC-only, so the scaffold date is UTC.
fn today_utc() -> String {
    lib::now_iso_utc()[..10].to_string()
}

fn normalize(slug: &str) -> String {
    let base = slug.strip_suffix(SUFFIX).unwrap_or(slug);
    let valid = !base.is_empty()
        && base.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && base == base.to_lowercase();
    if !valid {
        fail(&format!("slug must be lowercase kebab-case, got {slug:?}"));
    }
    format!("{base}{SUFFIX}")
}

fn load_sources(directory: &std::path::Path) -> Value {
    let sources = directory.join("sources.json");
    if !sources.is_file() {
        fail(&format!(
            "{}/sources.json is missing; not a managed catalog",
            directory.file_name().unwrap_or_default().to_string_lossy()
        ));
    }
    match std::fs::read_to_string(&sources)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    {
        Some(v) => v,
        None => fail("sources.json is not valid JSON"),
    }
}

fn save_sources(directory: &std::path::Path, sources: &Value) -> Result<()> {
    std::fs::write(
        directory.join("sources.json"),
        serde_json::to_string_pretty(sources)? + "\n",
    )?;
    Ok(())
}

fn regenerate() {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("spis"));
    match Command::new(exe).arg("generate-example-catalogs").output() {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            fail(&format!(
                "index regeneration refused the change:\n{stdout}{stderr}"
            ));
        }
        Err(e) => fail(&format!("index regeneration refused the change:\n{e}")),
    }
}

struct Parsed<'a> {
    positional: Option<&'a String>,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    rename: Option<String>,
    force: bool,
}

fn parse_args<'a>(rest: &'a [String], what: &[&str]) -> Result<Parsed<'a>> {
    let mut parsed = Parsed {
        positional: None,
        title: None,
        description: None,
        status: None,
        rename: None,
        force: false,
    };
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        macro_rules! take {
            ($name:expr) => {{
                i += 1;
                rest.get(i)
                    .cloned()
                    .with_context(|| format!("{} needs a value", $name))?
            }};
        }
        match arg {
            "--title" if what.contains(&"title") => parsed.title = Some(take!("--title")),
            "--description" if what.contains(&"description") => {
                parsed.description = Some(take!("--description"))
            }
            "--status" if what.contains(&"status") => parsed.status = Some(take!("--status")),
            "--rename" if what.contains(&"rename") => parsed.rename = Some(take!("--rename")),
            "--force" if what.contains(&"force") => parsed.force = true,
            other => {
                if !other.starts_with("--") && parsed.positional.is_none() {
                    parsed.positional = Some(&rest[i]);
                } else {
                    bail!("unrecognized argument: {other}");
                }
            }
        }
        i += 1;
    }
    Ok(parsed)
}

fn cmd_add(rest: &[String]) -> Result<()> {
    let args = parse_args(rest, &["title", "description"])?;
    let slug_arg = args
        .positional
        .context("the following arguments are required: slug")?;
    let slug = normalize(slug_arg);
    let directory = PathBuf::from(&slug);
    if directory.exists() {
        fail(&format!("{} already exists", directory.display()));
    }
    let title = match &args.title {
        Some(t) if !t.is_empty() => t.clone(),
        _ => fail("--title is required for add"),
    };
    std::fs::create_dir(&directory)?;
    std::fs::create_dir(directory.join("references"))?;
    let sources = json!({
        "schema": "wisent.example-catalog.v2",
        "catalog": slug,
        "slug": slug,
        "title": title,
        "description": args.description.unwrap_or_default(),
        "status": "scaffolded",
        "curated_at": today_utc(),
        "count": 0,
        "examples": [],
        "visual_count": 0,
        "structure_count": 0,
    });
    save_sources(&directory, &sources)?;
    std::fs::write(
        directory.join("references.json"),
        serde_json::to_string_pretty(&json!({
            "schema": "wisent.full-reference-catalog.v2",
            "catalog": slug,
            "generated_at": lib::now_iso_utc(),
            "reference_count": 0,
            "complete_count": 0,
            "partial_count": 0,
            "references": [],
        }))? + "\n",
    )?;
    regenerate();
    println!(
        "added {} ({title}); scaffolded with zero records",
        directory.display()
    );
    Ok(())
}

fn cmd_edit(rest: &[String]) -> Result<()> {
    let args = parse_args(rest, &["title", "description", "status", "rename"])?;
    let slug_arg = args
        .positional
        .context("the following arguments are required: slug")?;
    let slug = normalize(slug_arg);
    let directory = PathBuf::from(&slug);
    if !directory.is_dir() {
        fail(&format!("{} does not exist", directory.display()));
    }
    let mut sources = load_sources(&directory);
    let obj = match sources.as_object_mut() {
        Some(o) => o,
        None => fail("sources.json is not a JSON object"),
    };
    let mut changed: Vec<String> = Vec::new();
    let mut new_slug: Option<String> = None;

    if let Some(title) = &args.title {
        obj.insert("title".into(), json!(title));
        changed.push("title".into());
    }
    if let Some(description) = &args.description {
        obj.insert("description".into(), json!(description));
        changed.push("description".into());
    }
    if let Some(status) = &args.status {
        obj.insert("status".into(), json!(status));
        changed.push("status".into());
    }
    if let Some(rename) = &args.rename {
        let target = normalize(rename);
        let new_directory = PathBuf::from(&target);
        if new_directory.exists() {
            fail(&format!("{} already exists", new_directory.display()));
        }
        obj.insert("slug".into(), json!(target));
        obj.insert("catalog".into(), json!(target));
        changed.push(format!("slug -> {target}"));
        new_slug = Some(target);
    }
    if changed.is_empty() {
        fail("nothing to edit: pass --title, --description, --status, or --rename");
    }
    save_sources(&directory, &sources)?;
    if let Some(target) = &new_slug {
        std::fs::rename(&directory, target)?;
    }
    regenerate();
    println!("edited {}: {}", slug, changed.join(", "));
    Ok(())
}

fn cmd_remove(rest: &[String]) -> Result<()> {
    let args = parse_args(rest, &["force"])?;
    let slug_arg = args
        .positional
        .context("the following arguments are required: slug")?;
    let slug = normalize(slug_arg);
    let directory = PathBuf::from(&slug);
    if !directory.is_dir() {
        fail(&format!("{} does not exist", directory.display()));
    }
    let sources = load_sources(&directory);
    // The Python original counted `records` here even though add scaffolds an
    // `examples` array; keep the same key for parity.
    let record_count = sources
        .get("records")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let references_dir = directory.join("references");
    let stored = if references_dir.is_dir() {
        std::fs::read_dir(&references_dir)?.count()
    } else {
        0
    };
    if (record_count > 0 || stored > 0) && !args.force {
        fail(&format!(
            "{} holds {record_count} indexed record(s) and {stored} \
             reference director(ies); passing --force deletes that evidence permanently",
            directory.display()
        ));
    }
    std::fs::remove_dir_all(&directory)?;
    regenerate();
    println!("removed {}", directory.display());
    Ok(())
}

pub fn run(rest: &[String]) -> Result<()> {
    let (command, rest) = rest
        .split_first()
        .context("usage: spis catalog-type <add|edit|remove> <slug> [flags]")?;
    match command.as_str() {
        "add" => cmd_add(rest),
        "edit" => cmd_edit(rest),
        "remove" => cmd_remove(rest),
        other => bail!("unknown catalog-type command: {other}"),
    }
}
