//! Which spis subcommand each loopback endpoint runs. Every endpoint is the
//! CLI command it names, with the same arguments; there is no second
//! implementation behind the HTTP surface.

use http::StatusCode;
use serde_json::{Map, Value};
use std::num::NonZeroU64;

/// A refusal that becomes a non-2xx `{"error": sentence}` answer.
pub struct Refusal {
    pub status: StatusCode,
    pub sentence: String,
}

pub fn bad_request(sentence: &str) -> Refusal {
    Refusal {
        status: StatusCode::BAD_REQUEST,
        sentence: sentence.to_string(),
    }
}

/// Endpoints that answer with the subcommand's JSON document instead of a
/// stream of its output.
pub fn answers_document(name: &str) -> bool {
    matches!(name, "docs-status" | "docs-search" | "docs-show")
}

fn field_refusal(
    body: &Map<String, Value>, key: &str, expected: &str, operation: &str,
) -> Refusal {
    let sentence = match body.get(key) {
        Some(value) => format!("{operation}: field '{key}' requires {expected}; received {value}"),
        None => format!("{operation}: missing field '{key}' ({expected})"),
    };
    Refusal { status: StatusCode::BAD_REQUEST, sentence }
}

fn text(body: &Map<String, Value>, key: &str, sentence: &str) -> Result<String, Refusal> {
    body.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| field_refusal(body, key, "a nonempty string", sentence))
}

fn positive(
    body: &Map<String, Value>, key: &str, sentence: &str,
) -> Result<NonZeroU64, Refusal> {
    body.get(key)
        .and_then(Value::as_u64)
        .and_then(NonZeroU64::new)
        .ok_or_else(|| field_refusal(body, key, "a positive whole number", sentence))
}

fn flag(body: &Map<String, Value>, key: &str, operation: &str) -> Result<bool, Refusal> {
    match body.get(key) {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(field_refusal(body, key, "a Boolean", operation)),
    }
}

/// The exact spis invocation an endpoint stands for.
pub fn argv(name: &str, body: &Map<String, Value>) -> Result<Vec<String>, Refusal> {
    let owned = |words: &[&str]| {
        words
            .iter()
            .map(|word| word.to_string())
            .collect::<Vec<_>>()
    };
    match name {
        "corpus-adopt" => {
            let path = text(
                body,
                "path",
                "adopting a corpus requires its directory path",
            )?;
            if !std::path::Path::new(&path).is_absolute() {
                return Err(bad_request(
                    "adopting a corpus requires an absolute directory path",
                ));
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
        "docs-status" => Ok(owned(&["docs", "corpus", "status"])),
        "docs-search" => {
            let query = text(body, "query", "searching the docs corpus requires a query")?;
            let mut argv = vec![
                "docs".into(),
                "corpus".into(),
                "search".into(),
                "--query".into(),
                query,
            ];
            if body.contains_key("site") {
                let site = text(body, "site", "filtering a docs search by site")?;
                argv.extend(["--site".to_string(), site]);
            }
            if body.contains_key("limit") {
                let limit = positive(body, "limit", "limiting docs search results")?;
                argv.extend(["--limit".to_string(), limit.to_string()]);
            }
            Ok(argv)
        }
        "docs-show" => {
            let site = text(body, "site", "reading a docs page requires a site slug")?;
            let url = text(body, "url", "reading a docs page requires its URL")?;
            Ok(vec![
                "docs".into(),
                "corpus".into(),
                "show".into(),
                "--site".into(),
                site,
                "--url".into(),
                url,
            ])
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
            text(
                body,
                "selectionNote",
                "adding a record requires a selection note",
            )?,
            "--visual".into(),
            text(body, "visual", "adding a record requires an image path")?,
        ]),
        "reference-edit" => {
            let slug = text(body, "slug", "editing a record requires a catalog slug")?;
            let number = positive(body, "number", "editing a reference record")?;
            let mut argv = vec![
                "reference-record".into(),
                "edit".into(),
                slug,
                number.to_string(),
            ];
            let mut changed = false;
            for (key, flag) in [
                ("sourceUrl", "--source-url"),
                ("category", "--category"),
                ("selectionNote", "--selection-note"),
                ("owner", "--owner"),
            ] {
                if body.contains_key(key) {
                    let value = text(
                        body,
                        key,
                        &format!("editing a record requires nonempty {key}"),
                    )?;
                    argv.extend([flag.into(), value]);
                    changed = true;
                }
            }
            if !changed {
                return Err(bad_request(
                    "editing a record requires sourceUrl, category, selectionNote or owner",
                ));
            }
            Ok(argv)
        }
        "reference-remove" => {
            let slug = text(body, "slug", "removing a record requires a catalog slug")?;
            let number = positive(body, "number", "removing a reference record")?;
            let force = flag(body, "force", "removing a reference record")?;
            let mut argv = vec![
                "reference-record".into(),
                "remove".into(),
                slug,
                number.to_string(),
            ];
            if force {
                argv.push("--force".into());
            }
            Ok(argv)
        }
        "catalog-add" => Ok(vec![
            "catalog-type".into(),
            "add".into(),
            text(body, "slug", "adding a product type requires a slug")?,
            "--title".into(),
            text(body, "title", "adding a product type requires a title")?,
            "--description".into(),
            text(
                body,
                "description",
                "adding a product type requires a description",
            )?,
        ]),
        "catalog-edit" => {
            let mut argv = vec![
                "catalog-type".into(),
                "edit".into(),
                text(body, "slug", "editing a product type requires its slug")?,
            ];
            let mut changes = Vec::new();
            for (key, flag) in [
                ("title", "--title"),
                ("description", "--description"),
                ("status", "--status"),
                ("rename", "--rename"),
            ] {
                if body.contains_key(key) {
                    let value = text(body, key, "editing a product type")?;
                    changes.extend([flag.into(), value]);
                }
            }
            if changes.is_empty() {
                return Err(bad_request(
                    "editing a product type requires a title, description, status or new slug",
                ));
            }
            argv.extend(changes);
            Ok(argv)
        }
        "catalog-remove" => {
            let force = flag(body, "force", "removing a product type")?;
            let mut argv = vec![
                "catalog-type".into(),
                "remove".into(),
                text(body, "slug", "removing a product type requires its slug")?,
            ];
            if force {
                argv.push("--force".into());
            }
            Ok(argv)
        }
        _ => Err(Refusal {
            status: StatusCode::NOT_FOUND,
            sentence: format!("unknown endpoint: POST /v1/{name}"),
        }),
    }
}

/// The tool's own refusal: its last stderr line, without the `error: ` prefix.
pub fn refusal_sentence(stderr: &str) -> String {
    match stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .last()
    {
        Some(line) => match line.strip_prefix("error: ") {
            Some(said) => said.to_string(),
            None => line.to_string(),
        },
        None => "The operation failed without reporting a reason.".to_string(),
    }
}
