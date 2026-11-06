//! `spis refusals <list|show|check>`: the ways a Spis command refuses that
//! are declared, each once as a cause of a `wisent_errors::Operation`
//! naming what failed, why, the state the refusal shows, the fix and the
//! real test that provokes it; `check` refuses a cause whose test the
//! checkout does not hold.

use std::path::Path;

use anyhow::{bail, Result};
use wisent_errors::{Cause, Code, Operation};

use super::{required, usage};

/// The exit status of a refusal that is neither a usage error nor
/// retryable, as `main` leaves it.
const FAILURE_EXIT: i32 = 1;

/// The documentation page every cause is a line on.
const DOCS: &str = "https://spis.wisent.com/docs/cli/refusals/list";

const PLAIN: Cause = Cause {
    id: "",
    code: Code::Refused,
    usage: false,
    exit: None,
    failed: "",
    cause: "",
    observed: "",
    remedy: "",
    test: "",
};

pub static UNKNOWN_SUBCOMMAND: Cause = Cause {
    id: "unknown-subcommand",
    usage: true,
    failed: "choosing the subcommand",
    cause: "the first word names no Spis subcommand",
    observed: "the refusal names the word and lists every subcommand",
    remedy: "run `spis` to list the subcommands and pick one",
    test: "tests/cli/main.rs::exercise",
    ..PLAIN
};

pub static CATALOG_EXISTS: Cause = Cause {
    id: "catalog-exists",
    failed: "adding a catalog type",
    cause: "a catalog of that slug is already in the corpus",
    observed: "the refusal names the catalog directory",
    remedy: "edit the catalog that exists with `spis catalog-type edit`, or choose another slug",
    test: "tests/catalog_type/main.rs::lifecycle",
    ..PLAIN
};

pub static CATALOG_MISSING: Cause = Cause {
    id: "catalog-missing",
    code: Code::NotFound,
    failed: "finding the catalog a command names",
    cause: "no directory `<slug>-examples` is in the corpus",
    observed: "the refusal names the catalog directory it looked for",
    remedy: "`spis catalog-type list` names the catalogs that exist",
    test: "tests/catalog_type/main.rs::lifecycle",
    ..PLAIN
};

pub static CATALOG_DAMAGED: Cause = Cause {
    id: "catalog-damaged",
    code: Code::Config,
    failed: "reading a catalog's sources.json or references.json",
    cause: "the file lacks the list it must hold, holds something else there, or an entry is not a record object",
    observed: "the refusal names the file, the key, the position and what it holds",
    remedy: "restore the file from the corpus history, or correct the entry the refusal names",
    test: "tests/capture_widths/main.rs::refusals",
    ..PLAIN
};

pub static CURATION_INVALID: Cause = Cause {
    id: "curation-invalid",
    failed: "curating a catalog from a selector",
    cause: "a selector row leaves its category empty",
    observed: "the refusal names the empty category column; no catalog is written",
    remedy: "give every row its category, or the header's",
    test: "tests/catalog_type/main.rs::curated",
    ..PLAIN
};

pub static REFUSALS_UNMET: Cause = Cause {
    id: "refusals-unmet",
    failed: "holding the declared refusals to the error standard",
    cause: "a cause is declared incompletely, or names a test its file does not declare",
    observed: "the refusal names each cause and what it lacks",
    remedy: "complete the declaration in src/commands/refusals.rs, or add the test it names",
    test: "tests/refusals/main.rs::check_names_each_cause_a_checkout_lacks",
    ..PLAIN
};

pub static SPIS: Operation = Operation {
    id: "spis",
    command: "spis",
    summary: "Builds and serves the evidence-grade reference corpus; a refusal names the step, the state and the fix.",
    docs: DOCS,
    causes: &[
        &UNKNOWN_SUBCOMMAND,
        &CATALOG_EXISTS,
        &CATALOG_MISSING,
        &CATALOG_DAMAGED,
        &CURATION_INVALID,
        &REFUSALS_UNMET,
    ],
};

pub static OPERATIONS: &[&Operation] = &[&SPIS];

/// Everything that keeps the declarations from meeting the standard: each
/// operation's own problems, and each cause whose test file under `root`
/// does not declare the function it names.
pub fn problems(root: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    for operation in OPERATIONS {
        problems.extend(operation.problems());
        for cause in operation.causes {
            let Some((file, name)) = cause.test.split_once("::") else {
                continue;
            };
            let path = root.join(file);
            match std::fs::read_to_string(&path) {
                Ok(text) if text.contains(&format!("fn {name}(")) => {}
                Ok(_) => problems.push(format!(
                    "cause {}: {} declares no test function `{name}`",
                    cause.id,
                    path.display()
                )),
                Err(error) => problems.push(format!(
                    "cause {}: its test file {} cannot be read: {error}",
                    cause.id,
                    path.display()
                )),
            }
        }
    }
    problems
}

pub fn run(args: &[String]) -> Result<()> {
    let markdown = args.iter().any(|arg| arg == "--markdown");
    match args.first().map(String::as_str) {
        Some("list") => {
            if markdown {
                let sections: Vec<String> =
                    OPERATIONS.iter().map(|o| o.markdown(FAILURE_EXIT)).collect();
                println!("{}", sections.join("\n"));
            } else {
                let catalogues: Vec<String> =
                    OPERATIONS.iter().map(|o| o.to_json(FAILURE_EXIT)).collect();
                println!("{{\"operations\":[{}]}}", catalogues.join(","));
            }
            Ok(())
        }
        Some("show") => {
            let id = required(args.get(1).filter(|id| !id.starts_with("--")), "show needs <id>")?;
            let Some(operation) = OPERATIONS.iter().find(|o| o.id == id.as_str()) else {
                let declared: Vec<&str> = OPERATIONS.iter().map(|o| o.id).collect();
                bail!(
                    "no operation {id} declares its refusals; the operations that do: {}",
                    declared.join(", ")
                );
            };
            if markdown {
                println!("{}", operation.markdown(FAILURE_EXIT));
            } else {
                println!("{}", operation.to_json(FAILURE_EXIT));
            }
            Ok(())
        }
        Some("check") => {
            let root = match args.iter().position(|arg| arg == "--root") {
                Some(at) => std::path::PathBuf::from(required(args.get(at + 1), "--root needs a value")?),
                None => std::env::current_dir()?,
            };
            let found = problems(&root);
            if !found.is_empty() {
                bail!(
                    "the declared refusals do not meet the error standard under {}: {}",
                    root.display(),
                    found.join("; ")
                );
            }
            let causes: usize = OPERATIONS.iter().map(|o| o.causes.len()).sum();
            println!(
                "{{\"operations\":{},\"causes\":{causes},\"problems\":[]}}",
                OPERATIONS.len()
            );
            Ok(())
        }
        _ => Err(usage("usage: spis refusals <list|show <id>|check [--root <dir>]> [--markdown]")),
    }
}
