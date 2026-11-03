//! The GitHub star request: after a person's fifth reference record added
//! (`reference-record add`) or corpus query answered (`docs corpus search`,
//! `docs corpus show`) on a terminal, Spis asks once for a star on its public
//! repository. The request is the `star-request` journey every Wisent product
//! walks, run by the shared onboarding runtime and kept per user in
//! `$XDG_STATE_HOME/spis/onboarding.json` (default
//! `~/.local/state/spis/onboarding.json`); `spis star` reads, dismisses and
//! resets it. The threshold is the runtime's, the operator's number; the
//! repository is this package's `repository`, the catalog's
//! `star_repository`.

use anyhow::Result;
use wisent_onboarding_client::{JourneyError, ProductStarRequest, StarProduct};

const USAGE: &str = "usage: spis star [status|dismiss|reset] [--json]";

fn request() -> Result<ProductStarRequest, JourneyError> {
    let repository = env!("CARGO_PKG_REPOSITORY");
    let product = StarProduct::from_package("spis", "Spis", repository)?;
    ProductStarRequest::new(product)
}

/// Whether the command `name rest` that just succeeded is one a person uses
/// Spis for: adding a reference record, or a corpus query answered.
pub(super) fn counts(name: &str, rest: &[String]) -> bool {
    let words: Vec<&str> = rest.iter().map(String::as_str).collect();
    match (name, words.as_slice()) {
        ("reference-record", ["add", ..]) => true,
        ("docs", ["corpus", "search" | "show", ..]) => true,
        _ => false,
    }
}

/// Count one such command, when a person reads this terminal: the request,
/// on the one that makes it, goes to standard error, so standard output
/// keeps only the answer; a request that cannot be kept is said there too,
/// and the command's own answer stands.
pub(super) fn observe() {
    match request() {
        Ok(request) => {
            request.observe_reporting();
        }
        Err(error) => {
            eprintln!("spis: this run is not counted toward the GitHub star request: {error}")
        }
    }
}

/// Count one job Spis Desktop asked `spis serve` for, when it is one a
/// person uses Spis for (`argv` is the subcommand the job ran): the request,
/// on the job that makes it, as the `star_request` object the app shows. A
/// request that cannot be kept is said on the server's standard error and
/// the job's own answer stands.
pub(super) fn desktop_event(argv: &[String]) -> Option<serde_json::Value> {
    let (name, rest) = argv.split_first()?;
    if !counts(name, rest) {
        return None;
    }
    match request().and_then(|request| request.observe()) {
        Ok(Some(made)) => Some(serde_json::json!({
            "type": "star_request",
            "title": made.title,
            "body": made.body,
            "repository": made.repository,
            "actions": made.actions,
        })),
        Ok(None) => None,
        Err(error) => {
            eprintln!(
                "spis serve: this job is not counted toward the GitHub star request: {error}"
            );
            None
        }
    }
}

/// `spis star [status|dismiss|reset] [--json]`.
pub(super) fn run(rest: &[String]) -> Result<()> {
    if rest
        .iter()
        .any(|word| matches!(word.as_str(), "--help" | "-h"))
    {
        println!("{USAGE}\n\nThe GitHub star request Spis makes once after five reference-record additions or corpus searches on a terminal. https://spis.wisent.com/docs/cli/star");
        return Ok(());
    }
    let json = rest.iter().any(|word| word == "--json");
    let words: Vec<String> = rest
        .iter()
        .filter(|word| *word != "--json")
        .cloned()
        .collect();
    let answered = request()?.command(&words, json).map_err(|refusal| {
        match refusal.starts_with("star: ") {
            true => super::usage(refusal),
            false => anyhow::Error::msg(refusal),
        }
    })?;
    print!("{answered}");
    Ok(())
}
