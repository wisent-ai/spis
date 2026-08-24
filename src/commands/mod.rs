pub mod analyze_example_structures;
pub mod analyze_readme_examples;
pub mod audit_reference_accessibility;
pub mod capture_wisent_references;
pub mod capture_widths;
pub mod catalog_type;
pub mod check_upstream_drift;
pub mod collect_example_images;
pub mod crawl_docs;
pub mod docs_corpus;
pub mod discover;
pub mod generate_example_catalogs;
pub mod guidelines_rs;
pub mod reference_contract;
pub mod reference_record;
pub mod scrape_products;
pub mod sync_readme_examples;
pub mod verify_reference_evidence;

use anyhow::Result;

const SUBCOMMANDS: &[(&str, &str)] = &[
    ("crawl-docs", "full-text crawl of the 50-reference documentation set"),
    ("docs-corpus", "read-only JSON views over the crawled corpus"),
    ("discover", "discover important pages behind a start URL"),
    ("reference-record", "manage numbered reference records in a catalog"),
    ("verify-reference-evidence", "measure and verify evidence fields of records"),
    ("check-upstream-drift", "detect drift between corpus and upstream sources"),
    ("catalog-type", "manage typed catalogs (add/edit/rename/remove)"),
    ("generate-example-catalogs", "render catalog READMEs and indexes"),
    ("analyze-example-structures", "structural analysis of example screenshots"),
    ("analyze-readme-examples", "statistical analysis of readme snapshots"),
    ("guidelines", "draft writing guidelines for a catalog"),
    ("sync-readme-examples", "refresh readme snapshots from GitHub"),
    ("collect-example-images", "collect cover images for examples"),
    ("capture-widths", "enqueue multi-width Weles capture batches"),
    ("audit-reference-accessibility", "run axe audits over captured references"),
    ("capture-wisent-references", "pty-capture product CLIs into records"),
];

fn dispatch(name: &str, rest: &[String]) -> Result<bool> {
    match name {
        "crawl-docs" => crawl_docs::run(rest)?,
        "docs-corpus" => docs_corpus::run(rest)?,
        "discover" => discover::run(rest)?,
        "reference-record" => reference_record::run(rest)?,
        "verify-reference-evidence" => verify_reference_evidence::run(rest)?,
        "check-upstream-drift" => check_upstream_drift::run(rest)?,
        "catalog-type" => catalog_type::run(rest)?,
        "generate-example-catalogs" => generate_example_catalogs::run(rest)?,
        "analyze-example-structures" => analyze_example_structures::run(rest)?,
        "analyze-readme-examples" => analyze_readme_examples::run(rest)?,
        "guidelines" => guidelines_rs::run(rest)?,
        "scrape-products" => scrape_products::run(rest)?,
        "sync-readme-examples" => sync_readme_examples::run(rest)?,
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
