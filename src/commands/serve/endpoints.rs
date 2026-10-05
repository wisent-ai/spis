//! Which spis subcommand each loopback endpoint runs. Every endpoint is the
//! CLI command it names, with the same arguments; there is no second
//! implementation behind the HTTP surface.

use serde_json::{Map, Value};

/// A refusal that becomes a non-2xx `{"error": sentence}` answer.
pub struct Refusal {
    pub status: u16,
    pub sentence: String,
}

pub fn bad_request(sentence: &str) -> Refusal {
    Refusal { status: 400, sentence: sentence.to_string() }
}

/// Endpoints that answer with the subcommand's JSON document instead of a
/// stream of its output.
pub fn answers_document(name: &str) -> bool {
    matches!(name, "docs-status" | "docs-search" | "docs-show")
}

fn text(body: &Map<String, Value>, key: &str, sentence: &str) -> Result<String, Refusal> {
    body.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| bad_request(sentence))
}

fn positive(value: Option<&Value>) -> Option<u64> {
    value.and_then(Value::as_u64).filter(|number| *number >= 1)
}

/// The exact spis invocation an endpoint stands for.
pub fn argv(name: &str, body: &Map<String, Value>) -> Result<Vec<String>, Refusal> {
    let owned = |words: &[&str]| words.iter().map(|word| word.to_string()).collect::<Vec<_>>();
    match name {
        "corpus-adopt" => {
            let path = text(body, "path", "adopting a corpus requires its directory path")?;
            if !std::path::Path::new(&path).is_absolute() {
                return Err(bad_request("adopting a corpus requires an absolute directory path"));
            }
            Ok(vec!["corpus".into(), "adopt".into(), path])
        }
        "catalogs-check" => Ok(owned(&["generate-example-catalogs", "--check"])),
        "drift" => Ok(owned(&["check-upstream-drift"])),
        "verify" => Ok(owned(&["verify-reference-evidence"])),
        "capture-dry-run" => {
            let catalog = text(body, "catalog", "a capture plan requires a catalog slug")?;
            Ok(vec!["capture-widths".into(), catalog, "--dry-run".into()])
        }
        "docs-status" => Ok(owned(&["docs-corpus", "status"])),
        "docs-search" => {
            let query = text(body, "query", "searching the docs corpus requires a query")?;
            let mut argv = vec!["docs-corpus".into(), "search".into(), "--query".into(), query];
            if let Ok(site) = text(body, "site", "") {
                argv.extend(["--site".to_string(), site]);
            }
            if let Some(limit) = body.get("limit") {
                let limit = positive(Some(limit))
                    .ok_or_else(|| bad_request("a docs search limit must be a positive number"))?;
                argv.extend(["--limit".to_string(), limit.to_string()]);
            }
            Ok(argv)
        }
        "docs-show" => {
            let site = text(body, "site", "reading a docs page requires a site slug")?;
            let url = text(body, "url", "reading a docs page requires its URL")?;
            Ok(vec!["docs-corpus".into(), "show".into(), "--site".into(), site, "--url".into(), url])
        }
        "reference-add" => Ok(vec![
            "reference-record".into(),
            "add".into(),
            text(body, "slug", "adding a record requires a catalog slug")?,
            "--name".into(),
            text(body, "name", "adding a record requires a name")?,
            "--source-url".into(),
            text(body, "sourceUrl", "adding a record requires a source URL")?,
            "--category".into(),
            text(body, "category", "adding a record requires a category")?,
            "--selection-note".into(),
            text(body, "selectionNote", "adding a record requires a selection note")?,
            "--visual".into(),
            text(body, "visual", "adding a record requires an image path")?,
        ]),
        "reference-remove" => {
            let slug = text(body, "slug", "removing a record requires a catalog slug")?;
            let number = positive(body.get("number"))
                .ok_or_else(|| bad_request("removing a record requires its number"))?;
            let mut argv = vec!["reference-record".into(), "remove".into(), slug, number.to_string()];
            if body.get("force").and_then(Value::as_bool) == Some(true) {
                argv.push("--force".into());
            }
            Ok(argv)
        }
        _ => Err(Refusal { status: 404, sentence: format!("unknown endpoint: POST /v1/{name}") }),
    }
}

/// The tool's own refusal: its last stderr line, without the `error: ` prefix.
pub fn refusal_sentence(stderr: &str) -> String {
    match stderr.lines().map(str::trim).filter(|line| !line.is_empty()).last() {
        Some(line) => line.strip_prefix("error: ").unwrap_or(line).to_string(),
        None => "The operation failed without reporting a reason.".to_string(),
    }
}
