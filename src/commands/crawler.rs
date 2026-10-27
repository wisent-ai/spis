//! One execution primitive over the crawl engine declared by the caller.
//! The crawl coordinator and host workers use the same grammar.

use super::crawl::Engine;
use anyhow::Result;

const USAGE: &str = "usage: spis crawler run ENGINE [engine arguments]\n\nENGINE is cli, docs, desktop, mobile, web or tui.\nRun `spis crawler run ENGINE --help` for its arguments.\nUse `spis crawl --help` for durable planning, submission and import.";

pub(super) fn run(arguments: &[String]) -> Result<()> {
    let Some((verb, arguments)) = arguments.split_first() else {
        println!("{USAGE}");
        return Ok(());
    };
    match verb.as_str() {
        "--help" | "-h" => {
            println!("{USAGE}");
            return Ok(());
        }
        "run" => {}
        other => {
            return Err(super::usage(format!(
                "unknown crawler operation: {other}; {USAGE}"
            )));
        }
    }
    let Some((name, arguments)) = arguments.split_first() else {
        return Err(super::usage(format!("crawler run requires ENGINE; {USAGE}")));
    };
    if matches!(name.as_str(), "--help" | "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    let engine = Engine::of(name).map_err(|error| {
        super::usage(format!("crawler run: {error:#}; {USAGE}"))
    })?;
    let arguments = match arguments.iter().find(|argument| {
        matches!(argument.as_str(), "--help" | "-h")
    }) {
        Some(help) => {
            if engine == Engine::Docs {
                println!(
                    "usage: spis crawler run docs --host TARGET --record SLUG --runtime-manifest-base64 DATA\nworker mode: spis crawler run docs --worker --site SLUG --artifact-uri URI --runtime-manifest-base64 DATA"
                );
                return Ok(());
            }
            std::slice::from_ref(help)
        }
        None => {
            super::remember_invocation_dir()?;
            super::corpus::activate_configured_root()?;
            arguments
        }
    };
    match engine {
        Engine::Cli => super::crawl_cli::run(arguments),
        Engine::Docs => super::crawl_docs::run(arguments),
        Engine::Desktop => super::crawl_desktop::run(arguments),
        Engine::Mobile => super::crawl_mobile::run(arguments),
        Engine::Web => super::crawl_web::run(arguments),
        Engine::Tui => super::crawl_tui::run(arguments),
    }
}

/// Traversal bounds come from the invocation or its declared environment.
/// Workers receive the resolved bounds explicitly from their coordinator.
pub(crate) fn traversal_limits(
    max_states: Option<usize>, max_depth: Option<usize>,
) -> Result<(usize, usize)> {
    let states = declared_limit(max_states, "--max-states", "SPIS_CRAWL_MAX_STATES")?;
    let depth = declared_limit(max_depth, "--max-depth", "SPIS_CRAWL_MAX_DEPTH")?;
    let states = std::num::NonZeroUsize::new(states)
        .ok_or_else(|| super::usage("--max-states must be positive"))?;
    Ok((states.get(), depth))
}

fn declared_limit(value: Option<usize>, flag: &str, variable: &str) -> Result<usize> {
    match value {
        Some(value) => Ok(value),
        None => {
            let declared = std::env::var(variable).map_err(|error| {
                super::usage(format!(
                    "{flag} was not supplied and {variable} cannot be read: {error}; pass {flag} or set {variable} in the coordinator environment"
                ))
            })?;
            declared.parse().map_err(|error| {
                super::usage(format!("{variable}: {declared:?} is not valid for {flag}: {error}"))
            })
        }
    }
}
