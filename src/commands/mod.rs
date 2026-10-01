pub mod analyze_example_structures;
pub mod audit_reference_accessibility;
pub mod capture_widths;
pub mod capture_wisent_references;
pub mod catalog_type;
pub mod check_upstream_drift;
pub mod corpus;
pub mod collect_example_images;
pub mod crawl;
pub mod crawl_cli;
pub mod crawl_desktop;
pub mod crawl_docs;
pub mod crawl_mobile;
pub mod crawl_tui;
pub mod crawl_web;
pub mod curate_marketing_catalogs;
pub mod discover;
pub mod docs_corpus;
pub mod docs_site;
pub mod generate_example_catalogs;
pub mod reference_contract;
pub mod reference_record;
pub mod verify_reference_evidence;

use anyhow::Result;
use std::io::Write;

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

/// The secret a Skarbiec `ITEM#FIELD` reference in `variable` names, `None`
/// when the variable is unset. Only the reference travels in the
/// environment; the value is read from `skarbiec get` (cli.md rule 15).
/// `SKARBIEC_BIN` names another executable.
pub fn skarbiec_secret(variable: &str) -> Result<Option<String>> {
    let Some(reference) = std::env::var(variable).ok().filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    let reference = reference.trim();
    let Some((item, field)) = reference.rsplit_once('#').filter(|(item, field)| !item.is_empty() && !field.is_empty()) else {
        return Err(usage(format!("{variable} must be a Skarbiec reference ITEM#FIELD, not {reference:?}")));
    };
    let binary = std::env::var("SKARBIEC_BIN").unwrap_or_else(|_| "skarbiec".into());
    let output = std::process::Command::new(&binary)
        .args(["get", item, "--field", field])
        .output()
        .map_err(|error| anyhow::anyhow!("{variable}: {binary} could not be run: {error}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "{variable}: skarbiec get {item} --field {field} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let secret = String::from_utf8(output.stdout)
        .map_err(|_| anyhow::anyhow!("{variable}: skarbiec returned a value that is not UTF-8"))?
        .trim_end_matches('\n')
        .to_string();
    if secret.is_empty() {
        anyhow::bail!("{variable}: Skarbiec item {item} field {field} is empty");
    }
    Ok(Some(secret))
}

/// One subcommand: its name, what it does, and whether its own module
/// answers `--help` with its flags. A module that does not is answered here,
/// so `--help` never reaches code that would do the command's work.
struct Subcommand {
    name: &'static str,
    description: &'static str,
    own_help: bool,
}

const fn sub(name: &'static str, description: &'static str, own_help: bool) -> Subcommand {
    Subcommand { name, description, own_help }
}

const SUBCOMMANDS: &[Subcommand] = &[
    sub("onboarding", "show or reset the first-use walkthrough", true),
    sub("corpus", "adopt or inspect an existing canonical reference corpus", true),
    sub("docs-site", "generate this product's documentation from its own tables", true),
    sub("crawl", "plan, submit, track, resume and import every crawler", true),
    sub("crawl-cli", "crawl real CLI products through a PTY on Stado", true),
    sub("crawl-docs", "full-text crawl of the 50-reference documentation set", false),
    sub("crawl-mobile", "crawl real iOS or Android apps through Appium", true),
    sub("crawl-desktop", "crawl real macOS or desktop apps through Cua Driver", true),
    sub("crawl-web", "crawl real browser products through Weles on Stado", true),
    sub("crawl-tui", "crawl real terminal applications through a PTY on Stado", true),
    sub("docs-corpus", "read and import immutable documentation retrieval corpora", false),
    sub("discover", "discover important pages behind a start URL", false),
    sub("reference-record", "manage numbered reference records in a catalog", false),
    sub("verify-reference-evidence", "measure and verify evidence fields of records", false),
    sub("check-upstream-drift", "detect drift between corpus and upstream sources", false),
    sub("catalog-type", "manage typed catalogs (add/edit/rename/remove)", true),
    sub("generate-example-catalogs", "validate catalogs and write the JSON index", true),
    sub("analyze-example-structures", "structural analysis of example screenshots", false),
    sub("collect-example-images", "collect cover images for examples", false),
    sub("capture-widths", "enqueue multi-width Weles capture batches", false),
    sub("audit-reference-accessibility", "run axe audits over captured references", true),
    sub("capture-wisent-references", "pty-capture product CLIs into records", false),
    sub("curate-marketing-catalogs", "write validated pricing and landing candidates for Weles capture", false),
];

fn asks_help(rest: &[String]) -> bool {
    rest.iter().any(|arg| arg == "--help" || arg == "-h")
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
            "usage: spis {} [flags]\n\n{}\n\nThis command was not run. Its flags and refusals are in the CLI reference, https://spis.wisent.com/docs/cli-reference.",
            command.name, command.description
        );
        return Ok(true);
    }
    if !asks_help(rest) && !matches!(name, "onboarding" | "corpus" | "docs-site") {
        corpus::activate_configured_root()?;
    }
    match name {
        "onboarding" => crate::onboarding::run(rest)?,
        "corpus" => corpus::run(rest)?,
        "crawl" => crawl::run(rest)?,
        "crawl-docs" => crawl_docs::run(rest)?,
        "crawl-cli" => crawl_cli::run(rest)?,
        "crawl-desktop" => crawl_desktop::run(rest)?,
        "crawl-mobile" => crawl_mobile::run(rest)?,
        "crawl-web" => crawl_web::run(rest)?,
        "crawl-tui" => crawl_tui::run(rest)?,
        "curate-marketing-catalogs" => curate_marketing_catalogs::run(rest)?,
        "docs-corpus" => docs_corpus::run(rest)?,
        "docs-site" => docs_site::run(rest)?,
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
        "capture-wisent-references" => capture_wisent_references::run(rest)?,
        _ => unreachable!("every SUBCOMMANDS entry is dispatched"),
    }
    Ok(true)
}

pub fn run(args: &[String]) -> Result<bool> {
    match args.first().map(|s| s.as_str()) {
        None | Some("help") | Some("--help") | Some("-h") => {
            write_usage(&mut std::io::stdout());
            Ok(true)
        }
        Some(name) => {
            let rest: Vec<String> = args.iter().skip(1).cloned().collect();
            dispatch(name, &rest)
        }
    }
}

/// The subcommand list: to stdout when it was asked for, to stderr beside an
/// unknown subcommand's refusal.
fn write_usage(out: &mut dyn std::io::Write) {
    let _ = writeln!(out, "spis — evidence-grade reference corpus tooling\n\nUSAGE:\n  spis <subcommand> [flags]\n\nSUBCOMMANDS:");
    for command in SUBCOMMANDS {
        let _ = writeln!(out, "  {:<32} {}", command.name, command.description);
    }
    let _ = writeln!(out, "\nRun `spis <subcommand> --help` for details.");
}
