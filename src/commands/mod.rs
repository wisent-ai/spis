pub mod crawl_docs;

use anyhow::Result;

pub fn run(args: &[String]) -> Result<bool> {
    match args.first().map(|s| s.as_str()) {
        None => {
            print_usage();
            Ok(true)
        }
        Some("crawl-docs") => {
            crawl_docs::run(&args[1..])?;
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
         crawl-docs  full-text crawl of the 50-reference documentation set\n\
         \nRun `spis crawl-docs --help` for flags."
    );
}
