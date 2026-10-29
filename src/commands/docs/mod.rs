//! Document corpus operations and product documentation generation.

use anyhow::Result;

fn help() {
    println!("{}\n{}", super::docs_corpus::USAGE, super::docs_site::USAGE);
}

pub(super) fn run(arguments: &[String]) -> Result<()> {
    if super::asks_help(arguments) {
        match arguments.first().map(String::as_str) {
            Some("corpus") => println!("{}", super::docs_corpus::USAGE),
            Some("site") => super::docs_site::print_help(),
            _ => help(),
        }
        return Ok(());
    }
    let Some((area, arguments)) = arguments.split_first() else {
        help();
        return Ok(());
    };
    match area.as_str() {
        "help" => {
            help();
            Ok(())
        }
        "corpus" => {
            if arguments.is_empty() {
                println!("{}", super::docs_corpus::USAGE);
                Ok(())
            } else {
                super::docs_corpus::run(arguments)
            }
        }
        "site" => {
            let Some((verb, arguments)) = arguments.split_first() else {
                super::docs_site::print_help();
                return Ok(());
            };
            if verb != "generate" {
                return Err(super::usage(format!(
                    "unknown docs site operation {verb}; {}",
                    super::docs_site::USAGE
                )));
            }
            super::docs_site::run(arguments)
        }
        other => Err(super::usage(format!(
            "unknown docs operation {other}; {}\n{}",
            super::docs_corpus::USAGE,
            super::docs_site::USAGE
        ))),
    }
}
