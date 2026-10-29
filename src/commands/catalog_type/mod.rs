//! `spis catalog-type` — add, edit, or remove a product-type catalog.
//!
//! A product type is one `*-examples/` directory: a family of reference records
//! with its own sources, index, and evidence floor. This tool only scaffolds and
//! maintains the structure; it never fabricates records. After every mutation the
//! index is refreshed by re-execing this binary with `generate-example-catalogs`.
//!
//! Every refusal is an error that names its cause and reaches `spis`'s own
//! exit, rather than a process exit of its own.

mod commands;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

const SUFFIX: &str = "-examples";

/// `<slug>-examples` for a lowercase kebab-case slug given with or without
/// the suffix; any other slug is refused.
fn normalize(slug: &str) -> Result<String> {
    let base = match slug.strip_suffix(SUFFIX) {
        Some(base) => base,
        None => slug,
    };
    let valid = !base.is_empty()
        && base.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && base == base.to_lowercase();
    if !valid {
        bail!("slug must be lowercase kebab-case, got {slug:?}");
    }
    Ok(format!("{base}{SUFFIX}"))
}

/// Whether `path` exists; one that cannot be examined is refused naming it.
fn present(path: &Path) -> Result<bool> {
    path.try_exists()
        .with_context(|| format!("examining {}", path.display()))
}

fn load_sources(directory: &Path) -> Result<Value> {
    let sources = directory.join("sources.json");
    if !present(&sources)? {
        bail!(
            "{} is missing; {} is not a managed catalog",
            sources.display(),
            directory.display()
        );
    }
    let text = std::fs::read_to_string(&sources)
        .with_context(|| format!("{} cannot be read", sources.display()))?;
    serde_json::from_str::<Value>(&text)
        .with_context(|| format!("{} is not valid JSON", sources.display()))
}

fn save_sources(directory: &Path, sources: &Value) -> Result<()> {
    let path = directory.join("sources.json");
    std::fs::write(&path, serde_json::to_string_pretty(sources)? + "\n")
        .with_context(|| format!("writing {}", path.display()))
}

/// Rebuild the index; a refusal carries the generator's own output.
fn regenerate() -> Result<()> {
    let exe = std::env::current_exe()
        .context("cannot locate the running spis binary to regenerate the index with")?;
    let mut command = Command::new(exe);
    command.arg("generate-example-catalogs");
    let out = stado_wait::output(&mut command).context("running generate-example-catalogs")?;
    if out.status.success() {
        return Ok(());
    }
    Err(anyhow!(
        "index regeneration refused the change:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    ))
}

struct Parsed<'a> {
    positional: Option<&'a String>,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    rename: Option<String>,
    force: bool,
}

impl Parsed<'_> {
    /// The catalog slug every subcommand takes first, normalized.
    fn slug(&self) -> Result<String> {
        normalize(
            self.positional
                .context("the following arguments are required: slug")?,
        )
    }
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
    let mut arguments = rest.iter();
    while let Some(argument) = arguments.next() {
        let mut value = |name: &str| -> Result<String> {
            arguments
                .next()
                .cloned()
                .with_context(|| format!("{name} needs a value"))
        };
        match argument.as_str() {
            "--title" if what.contains(&"title") => parsed.title = Some(value("--title")?),
            "--description" if what.contains(&"description") => {
                parsed.description = Some(value("--description")?)
            }
            "--status" if what.contains(&"status") => parsed.status = Some(value("--status")?),
            "--rename" if what.contains(&"rename") => parsed.rename = Some(value("--rename")?),
            "--force" if what.contains(&"force") => parsed.force = true,
            other if !other.starts_with("--") && parsed.positional.is_none() => {
                parsed.positional = Some(argument)
            }
            other => {
                return Err(crate::commands::usage(format!(
                    "unrecognized argument: {other}\n{USAGE}"
                )))
            }
        }
    }
    Ok(parsed)
}

const USAGE: &str = "usage: spis catalog-type <add|edit|remove> <slug> [flags]
  spis catalog-type add <slug> --title TEXT --description TEXT
      scaffold a new <slug>-examples catalog and refresh the index
  spis catalog-type edit <slug> [--title TEXT] [--description TEXT] [--status TEXT] [--rename NEW-SLUG]
      change a catalog's metadata or rename its directory
  spis catalog-type remove <slug> [--force]
      delete a catalog; --force is required while it holds records or stored references";

pub fn run(rest: &[String]) -> Result<()> {
    // `--help` or `-h` anywhere answers with the usage and never mutates a
    // catalog.
    if rest.is_empty() || rest.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    let (command, rest) = rest.split_first().context(USAGE)?;
    match command.as_str() {
        "add" => commands::add(rest),
        "edit" => commands::edit(rest),
        "remove" => commands::remove(rest),
        other => Err(crate::commands::usage(format!(
            "unknown catalog-type command: {other}; it takes add, edit or remove\n{USAGE}"
        ))),
    }
}
