pub mod analyze_example_structures;
pub mod audit_reference_accessibility;
pub mod capture_widths;
pub mod capture_wisent_references;
pub mod catalog_type;
pub mod check_upstream_drift;
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
pub mod generate_example_catalogs;
pub mod reference_contract;
pub mod reference_record;
pub mod verify_reference_evidence;

use anyhow::Result;

const SUBCOMMANDS: &[(&str, &str)] = &[
    (
        "crawl",
        "plan, submit, track, resume and import every crawler",
    ),
    (
        "crawl-cli",
        "crawl real CLI products through a PTY on Stado",
    ),
    (
        "crawl-docs",
        "full-text crawl of the 50-reference documentation set",
    ),
    (
        "crawl-mobile",
        "crawl real iOS or Android apps through Appium",
    ),
    (
        "crawl-desktop",
        "crawl real macOS or desktop apps through Cua Driver",
    ),
    (
        "crawl-web",
        "crawl real browser products through Weles on Stado",
    ),
    (
        "crawl-tui",
        "crawl real terminal applications through a PTY on Stado",
    ),
    (
        "docs-corpus",
        "read and import immutable documentation retrieval corpora",
    ),
    ("discover", "discover important pages behind a start URL"),
    (
        "reference-record",
        "manage numbered reference records in a catalog",
    ),
    (
        "verify-reference-evidence",
        "measure and verify evidence fields of records",
    ),
    (
        "check-upstream-drift",
        "detect drift between corpus and upstream sources",
    ),
    (
        "catalog-type",
        "manage typed catalogs (add/edit/rename/remove)",
    ),
    (
        "generate-example-catalogs",
        "validate catalogs and write the JSON index",
    ),
    (
        "analyze-example-structures",
        "structural analysis of example screenshots",
    ),
    (
        "collect-example-images",
        "collect cover images for examples",
    ),
    (
        "capture-widths",
        "enqueue multi-width Weles capture batches",
    ),
    (
        "audit-reference-accessibility",
        "run axe audits over captured references",
    ),
    (
        "capture-wisent-references",
        "pty-capture product CLIs into records",
    ),
    (
        "curate-marketing-catalogs",
        "write validated pricing and landing candidates for Weles capture",
    ),
];

fn dispatch(name: &str, rest: &[String]) -> Result<bool> {
    match name {
        "crawl" => crawl::run(rest)?,
        "crawl-docs" => crawl_docs::run(rest)?,
        "crawl-cli" => crawl_cli::run(rest)?,
        "crawl-desktop" => crawl_desktop::run(rest)?,
        "crawl-mobile" => crawl_mobile::run(rest)?,
        "crawl-web" => crawl_web::run(rest)?,
        "crawl-tui" => crawl_tui::run(rest)?,
        "curate-marketing-catalogs" => curate_marketing_catalogs::run(rest)?,
        "docs-corpus" => docs_corpus::run(rest)?,
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
        _ => {
            eprintln!("unknown subcommand: {name}");
            print_usage();
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn run(args: &[String]) -> Result<bool> {
    match args.first().map(|s| s.as_str()) {
        None | Some("help") | Some("--help") | Some("-h") => {
            print_usage();
            Ok(true)
        }
        Some(name) => {
            let rest: Vec<String> = args.iter().skip(1).cloned().collect();
            dispatch(name, &rest)
        }
    }
}

fn print_usage() {
    eprintln!("spis — evidence-grade reference corpus tooling\n\nUSAGE:\n  spis <subcommand> [flags]\n\nSUBCOMMANDS:");
    for (name, desc) in SUBCOMMANDS {
        eprintln!("  {name:<32} {desc}");
    }
    eprintln!("\nRun `spis <subcommand> --help` for details.");
}
