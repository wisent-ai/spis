pub mod analyze_example_structures;
pub mod analyze_readme_examples;
pub mod audit_reference_accessibility;
pub mod capture_wisent_references;
pub mod capture_widths;
pub mod catalog_type;
pub mod check_upstream_drift;
pub mod collect_example_images;
pub mod crawl_docs;
pub mod discover;
pub mod generate_example_catalogs;
pub mod guidelines_rs;
pub mod reference_contract;
pub mod reference_record;
pub mod sync_readme_examples;
pub mod verify_reference_evidence;

use anyhow::Result;

pub fn run(args: &[String]) -> Result<bool> {
    let rest: &[String] = if args.len() >= 2 { &args[1..] } else { &[] };
    match args.first().map(|s| s.as_str()) {
        None => {
            print_usage();
            Ok(true)
        }
        Some("crawl-docs") => {
            crawl_docs::run(rest)?;
            Ok(true)
        }
        Some("discover") => {
            discover::run(rest)?;
            Ok(true)
        }
        Some("reference-record") => {
            reference_record::run(rest)?;
            Ok(true)
        }
        Some("verify-reference-evidence") => {
            verify_reference_evidence::run(rest)?;
            Ok(true)
        }
        Some("check-upstream-drift") => {
            check_upstream_drift::run(rest)?;
            Ok(true)
        }
        Some("catalog-type") => {
            catalog_type::run(rest)?;
            Ok(true)
        }
        Some("generate-example-catalogs") => {
            generate_example_catalogs::run(rest)?;
            Ok(true)
        }
        Some("analyze-example-structures") => {
            analyze_example_structures::run(rest)?;
            Ok(true)
        }
        Some("analyze-readme-examples") => {
            analyze_readme_examples::run(rest)?;
            Ok(true)
        }
        Some("guidelines") => {
            guidelines_rs::run(rest)?;
            Ok(true)
        }
        Some("sync-readme-examples") => {
            sync_readme_examples::run(rest)?;
            Ok(true)
        }
        Some("collect-example-images") => {
            collect_example_images::run(rest)?;
            Ok(true)
        }
        Some("capture-widths") => {
            capture_widths::run(rest)?;
            Ok(true)
        }
        Some("audit-reference-accessibility") => {
            audit_reference_accessibility::run(rest)?;
            Ok(true)
        }
        Some("capture-wisent-references") => {
            capture_wisent_references::run(rest)?;
            Ok(true)
        }
        Some("help") | Some("--help") | Some("-h") => {
            print_usage();
            Ok(true)
        }
        Some(other) => {
            eprintln!("unknown subcommand: {other}");
            print_usage();
            Ok(false)
        }
    }
}

fn print_usage() {
    eprintln!(
        "spis — evidence-grade reference corpus tooling\n\
         \nUSAGE:\n  \
         spis <subcommand> [flags]\n\
         \nSUBCOMMANDS:\n  \
         crawl-docs                    full-text crawl of the 50-reference documentation set\n  \
         discover                      discover important pages behind a start URL\n  \
         reference-record              manage numbered reference records in a catalog\n  \
         verify-reference-evidence     measure and verify evidence fields of records\n  \
         check-upstream-drift          detect drift between corpus and upstream sources\n  \
         catalog-type                  manage typed catalogs (add/edit/rename/remove)\n  \
         generate-example-catalogs     render catalog READMEs and indexes\n  \
         analyze-example-structures    structural analysis of example screenshots\n  \
         analyze-readme-examples       statistical analysis of readme snapshots\n  \
         guidelines                    draft writing guidelines for a catalog\n  \
         sync-readme-examples          refresh readme snapshots from GitHub\n  \
         collect-example-images        collect cover images for examples\n  \
         capture-widths                enqueue multi-width Weles capture batches\n  \
         audit-reference-accessibility run axe audits over captured references\n  \
         capture-wisent-references     pty-capture product CLIs into records\n\
         \nRun `spis <subcommand> --help` for details."
    );
}
