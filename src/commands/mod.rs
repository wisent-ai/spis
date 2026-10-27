pub mod analyze_example_structures;
pub mod audit_reference_accessibility;
pub(crate) mod brama;
pub mod capture_cli_references;
pub mod capture_widths;
pub mod catalog_type;
pub mod check_upstream_drift;
pub mod collect_example_images;
pub(crate) mod control_safety;
pub mod corpus;
pub mod crawl;
pub mod crawl_cli;
pub mod crawl_desktop;
pub mod crawl_docs;
pub mod crawl_mobile;
pub mod crawl_tui;
pub mod crawl_web;
mod crawler;
pub mod curate_catalog;
pub mod discover;
mod docs;
pub mod docs_corpus;
pub mod docs_site;
pub mod generate_example_catalogs;
pub mod reference_contract;
pub mod reference_record;
pub mod serve;
mod subcommands;
pub mod verify_reference_evidence;

use anyhow::{Context, Result};
use std::io::Write;
use subcommands::{Activation, SUBCOMMANDS};

/// The invocation itself is wrong: a missing or extra argument, an unknown
/// verb. A type rather than a phrase, so `main` tells it from a failure of a
/// well-formed command without reading the message: it exits 2, a failure 1.
#[derive(Debug)]
pub struct Usage(pub String);

impl std::fmt::Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Usage {}

/// A usage error ready for `?`: `return Err(usage("usage: spis …"))`.
pub fn usage(text: impl Into<String>) -> anyhow::Error {
    Usage(text.into()).into()
}

/// Whether a failure is a usage error anywhere in its chain.
pub fn is_usage(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| cause.is::<Usage>())
}

/// A flag's value that the invocation must carry: absent is a usage error.
/// `required(rest.get(i), "--record needs a value")?`
pub fn required<T>(value: Option<T>, text: &str) -> Result<T> {
    value.ok_or_else(|| usage(text))
}

/// A flag's value parsed into its type: a value that does not parse is a
/// usage error naming the flag, the value given and the parse failure.
pub fn parsed<T>(value: Option<&String>, flag: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let raw = required(value, &format!("{flag} needs a value"))?;
    raw.parse()
        .map_err(|error| usage(format!("{flag}: {raw:?} is not valid: {error}")))
}

fn asks_help(rest: &[String]) -> bool {
    rest.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h"))
}

fn dispatch(name: &str, rest: &[String]) -> Result<bool> {
    let Some(command) = SUBCOMMANDS.iter().find(|candidate| candidate.name == name) else {
        eprintln!("unknown subcommand: {name}");
        write_usage(&mut std::io::stderr());
        return Ok(false);
    };
    // Help is answered before the corpus is activated and before the module
    // runs: it needs no corpus, and a module that does not read `--help`
    // would otherwise do its work.
    if asks_help(rest) && !command.own_help {
        println!(
            "usage: spis {} [flags]\n\n{}\n\nThis command was not run. Its flags and refusals are in the CLI reference, https://spis.wisent.com/docs/cli.",
            command.name, command.description
        );
        return Ok(true);
    }
    if !asks_help(rest) && matches!(command.activation, Activation::Corpus) {
        remember_invocation_dir()?;
        corpus::activate_configured_root()?;
    }
    match name {
        "onboarding" => crate::onboarding::run(rest)?,
        "corpus" => corpus::run(rest)?,
        "crawl" => crawl::run(rest)?,
        "crawler" => crawler::run(rest)?,
        "curate-catalog" => curate_catalog::run(rest)?,
        "docs" => docs::run(rest)?,
        "discover" => discover::run(rest)?,
        "reference-record" => reference_record::run(rest)?,
        "verify-reference-evidence" => verify_reference_evidence::run(rest)?,
        "check-upstream-drift" => check_upstream_drift::run(rest)?,
        "catalog-type" => catalog_type::run(rest)?,
        "generate-example-catalogs" => generate_example_catalogs::run(rest)?,
        "analyze-example-structures" => analyze_example_structures::run(rest)?,
        "collect-example-images" => collect_example_images::run(rest)?,
        "capture-widths" => capture_widths::run(rest)?,
        "audit-reference-accessibility" => audit_reference_accessibility::run(rest)?,
        "serve" => serve::run(rest)?,
        "capture-cli-references" => capture_cli_references::run(rest)?,
        _ => unreachable!("every SUBCOMMANDS entry is dispatched"),
    }
    Ok(true)
}

static INVOCATION_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

fn remember_invocation_dir() -> Result<()> {
    let here =
        std::env::current_dir().context("read the working directory the command was typed in")?;
    let _ = INVOCATION_DIR.set(here);
    Ok(())
}

/// A path the operator typed, resolved against the directory the command was
/// typed in. Activating the adopted corpus changes the process directory, so
/// a relative path read afterwards would otherwise point into the corpus.
pub(crate) fn operator_path(path: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(path);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match INVOCATION_DIR.get() {
        Some(dir) => dir.join(path),
        None => path.to_path_buf(),
    }
}

pub fn run(args: &[String]) -> Result<bool> {
    match args.split_first() {
        None => {
            write_usage(&mut std::io::stdout());
            Ok(true)
        }
        Some((name, _)) if matches!(name.as_str(), "help" | "--help" | "-h") => {
            write_usage(&mut std::io::stdout());
            Ok(true)
        }
        Some((name, rest)) => dispatch(name, rest),
    }
}

/// The subcommand list: to stdout when it was asked for, to stderr beside an
/// unknown subcommand's refusal.
fn write_usage(out: &mut dyn std::io::Write) {
    let _ = writeln!(out, "spis — evidence-grade reference corpus tooling\n\nUSAGE:\n  spis <subcommand> [flags]\n\nSUBCOMMANDS:");
    for command in SUBCOMMANDS {
        let _ = writeln!(out, "  {} — {}", command.name, command.description);
    }
    let _ = writeln!(out, "\nRun `spis <subcommand> --help` for details.");
}
