//! `spis docs-site` — generate the product documentation from the product.
//!
//! ## Why this exists
//!
//! `docs-site/docs.json` is the generated product-document payload consumed by
//! the canonical `wisent-ai/spis-landing` site's existing `DocPage` loader.
//! Older releases wrote `docs-site/docs.ts`, but no deployed site imported
//! that module. The JSON output can also be written directly into the landing
//! checkout with `--site-content PATH`, so publishing does not require a
//! hand-maintained copy or a second documentation renderer.
//! The earlier generated inputs also drifted from the shipped product:
//!
//! * `docs-brief.json` declares the CLI surface by probing `bin/spis`, the
//!   Python-era shim, and lists nine commands — `analyze-readmes`, `capture`,
//!   `catalogs`, `drift`, `sync-readmes`, `verify` and three more — NOT ONE of
//!   which exists in the shipped binary. The real surface is `crawl`,
//!   `crawl-docs`, `crawl-cli`, `crawl-desktop`, `crawl-mobile`, `crawl-web`,
//!   `docs-corpus`, and the rest of [`super::SUBCOMMANDS`].
//! * it declares `13` interface families; there are fifteen in
//!   [`super::crawl::CATALOGS`].
//! * it declares no graphical surface at all, and the published prose calls
//!   Spis "the command-line machinery that maintains it" — while
//!   `wisent-ai/spis-desktop` ships a macOS application that starts crawls,
//!   tracks runs and searches the corpus. A product whose documentation does
//!   not mention its own graphical surface cannot be checked for the parity
//!   this fleet requires.
//!
//! Patching those sentences would have left the process that produced them
//! exactly as broken. So this module IS the process: every page is derived
//! from the running product — the subcommand table, the catalog table, each
//! engine's declared preconditions, the corpus bound — and `--check` fails
//! when the checked-in output no longer matches what the product would
//! generate. A documentation claim that drifts from the code is then a failing
//! command rather than a sentence nobody re-read.
//!
//! Content that is genuinely editorial (what Spis is for, the words it uses)
//! stays authored, in [`AUTHORED_SECTIONS`], and is carried through verbatim.
//! The distinction is deliberate: prose about intent is not derivable, and
//! pretending to derive it would be the same lie one level up.

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// The generated `DocPage` JSON consumed by the canonical documentation site.
const DOCS_CONTENT: &str = "docs-site/docs.json";
/// The validated plan the module is rendered from.
const DOCS_PLAN: &str = "docs-plan.json";
/// What the product declares about its own surfaces.
const DOCS_BRIEF: &str = "docs-brief.json";

/// Editorial prose, authored once and carried through unchanged.
///
/// Kept apart from everything else in this file because it is the only part
/// that is not derived. A generator that invented intent would be exactly the
/// defect this replaces.
const AUTHORED_SECTIONS: &[(&str, &str)] = &[
    (
        "what-spis-is",
        "Spis is an evidence-grade reference corpus: every record carries its source, hashes, provenance class, and measured state. A record exists only with its evidence, and a missing observation is recorded as an evidence gap rather than promoted into prose.",
    ),
    (
        "parity",
        "Every operation below is available from both surfaces. The command line and the desktop application are two ways to reach the same commands, not a full product and a viewer: if a command can do something, the application can too.",
    ),
];

fn authored(slug: &str) -> &'static str {
    AUTHORED_SECTIONS
        .iter()
        .find(|(name, _)| *name == slug)
        .map(|(_, text)| *text)
        .unwrap_or_default()
}

/// One documented command, straight off the shipped dispatch table.
fn command_bullets() -> Vec<String> {
    super::SUBCOMMANDS
        .iter()
        .map(|(name, summary)| format!("`{name}` — {summary}."))
        .collect()
}

/// One line per family: its engine, and what its worker requires of a host.
///
/// Derived from [`super::crawl::CATALOGS`] and
/// [`super::crawl::engine_preconditions`], so the documented host
/// requirements cannot drift from the ones the preflight actually enforces.
/// That drift is not hypothetical: the published documentation described host
/// requirements nowhere at all, while five of the six engines were submitting
/// workers that needed a program the preflight never checked.
fn family_rows() -> Vec<Value> {
    super::crawl::CATALOGS
        .iter()
        .map(|(catalog, engine)| {
            let requires: Vec<String> = super::crawl::engine_preconditions(engine, catalog)
                .into_iter()
                .map(|command| command.join(" "))
                .collect();
            json!({
                "family": catalog,
                "engine": engine,
                "requires": requires,
            })
        })
        .collect()
}

fn family_bullets() -> Vec<String> {
    family_rows()
        .iter()
        .map(|row| {
            let family = row["family"].as_str().unwrap_or_default();
            let engine = row["engine"].as_str().unwrap_or_default();
            let requires = row["requires"]
                .as_array()
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(|value| format!("`{value}`"))
                        .collect::<Vec<String>>()
                        .join(", ")
                })
                .unwrap_or_default();
            format!("`{family}` — {engine} engine; the host must answer {requires}.")
        })
        .collect()
}

/// The product's own surfaces, replacing a brief that probed a removed shim.
fn brief(version: &str) -> Value {
    json!({
        "ok": true,
        "product": "spis",
        "version": version,
        "generated_by": "spis docs-site",
        "surfaces": {
            "cli": {
                "declared": true,
                "binary": "target/release/spis",
                "commands": super::SUBCOMMANDS
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<&str>>(),
            },
            // Declared, because it exists and because the fleet requires
            // parity with the command line. The previous brief declared no
            // graphical surface, so nothing could be held to that rule.
            "desktop": {
                "declared": true,
                "repository": "https://github.com/wisent-ai/spis-desktop",
                "parity_with": "cli",
                "operations": [
                    "start a crawl for one product family or for all of them",
                    "track a run and its per-record jobs",
                    "ask whether one host can run one family, before anything is claimed",
                    "read the documentation corpus, including the corpus bound and what falls outside it",
                    "adopt an existing canonical corpus through the same product operation as the CLI",
                ],
            },
            "api": { "declared": false },
            "config": {
                "declared": true,
                "path": "~/.config/spis/corpus.json",
                "owner": "spis corpus adopt",
            },
        },
        "families": family_rows(),
        "corpus_bound": super::docs_corpus::MAX_PAGE_RECORDS,
        "problems": Value::Array(Vec::new()),
    })
}

/// The plan every page is rendered from.
fn plan(version: &str) -> Value {
    let over_bound: Vec<String> = super::docs_corpus::sites_over_corpus_bound()
        .into_iter()
        .map(|(slug, declared)| {
            format!(
                "`{slug}` declares {declared} in-scope pages against the {}-page bound.",
                super::docs_corpus::MAX_PAGE_RECORDS
            )
        })
        .collect();
    json!({
        "product": "spis",
        "version": version,
        "generated_by": "spis docs-site",
        "sources": {
            "subcommands": { "kind": "code", "location": "src/commands/mod.rs SUBCOMMANDS" },
            "families": { "kind": "code", "location": "src/commands/crawl.rs CATALOGS" },
            "preconditions": {
                "kind": "code",
                "location": "src/commands/crawl.rs engine_preconditions",
            },
            "corpus-bound": {
                "kind": "code",
                "location": "src/commands/docs_corpus.rs MAX_PAGE_RECORDS",
            },
            "corpus-adoption": {
                "kind": "code",
                "location": "src/commands/corpus.rs",
            },
        },
        "pages": [
            {
                "slug": "overview",
                "nav": "Overview",
                "eyebrow": "Getting started",
                "title": "Spis overview",
                "description": "What Spis is, and the two surfaces that drive it.",
                "sections": [
                    { "title": "What Spis is", "paragraphs": [authored("what-spis-is")] },
                    {
                        "title": "Two surfaces, one set of operations",
                        "paragraphs": [authored("parity")],
                    },
                ],
            },
            {
                "slug": "corpus-adoption",
                "nav": "Adopt a corpus",
                "eyebrow": "Getting started",
                "title": "Start with an existing corpus",
                "description": "Adopt one complete canonical corpus in place, from the CLI or the desktop application.",
                "sections": [
                    {
                        "title": "Accepted input",
                        "code": [
                            "spis corpus adopt /absolute/path/to/unpacked-corpus",
                            "spis corpus status",
                        ],
                        "paragraphs": [
                            "The selected directory must contain canonical `example-catalogs.json`; each indexed catalog must carry matching `sources.json` and `references.json`; and every indexed `reference.json` file must retain the full-product-reference schema, evidence status, and evidence gaps. Spis validates that complete graph before atomically writing `~/.config/spis/corpus.json`.",
                            "The corpus stays in place. Subsequent corpus commands and Spis Desktop resolve the saved root, so original provenance, screenshots, recordings, receipts, and other referenced files are not copied into a reduced store. Re-adopting the same canonical path reports every reference unchanged and creates no duplicate.",
                        ],
                    },
                    {
                        "title": "Desktop first use and replay",
                        "paragraphs": [
                            "On first launch, Choose Corpus opens a native directory picker and sends only the selected path to Spis's loopback API. The API invokes the same `corpus adopt` operation as the CLI; Swift does not parse or copy a corpus. The same control remains under Manage, beside Show it again for replaying onboarding.",
                        ],
                    },
                    {
                        "title": "Refusals",
                        "bullets": [
                            "ZIP, tar, gzip, bzip2, xz, zstd, 7z, and rar archives are refused with an instruction to unpack them first.",
                            "A missing or noncanonical index, duplicate or unsafe catalog slug, escaping path or symlink, mismatched schema or catalog identity, missing record file, count mismatch, or record without evidence status and evidence gaps refuses the whole adoption. The previous saved corpus remains active.",
                            "CLI JSON reports state, root, catalog, reference and file counts, index SHA-256, imported, unchanged, conflicting, rejected, and a result message. Desktop keeps that exact output or refusal selectable.",
                        ],
                    },
                ],
            },
            {
                "slug": "cli-reference",
                "nav": "CLI reference",
                "eyebrow": "Reference",
                "title": "CLI reference",
                "description": "Every subcommand the shipped binary dispatches.",
                "sections": [
                    { "title": "Commands", "bullets": command_bullets() },
                ],
            },
            {
                "slug": "running-crawls",
                "nav": "Running a crawl",
                "eyebrow": "Operating",
                "title": "Running a crawl",
                "description": "Start a run for one family or all of them, from either surface, and ask a host whether it can carry the work first.",
                "sections": [
                    {
                        "title": "Ask before you claim",
                        "paragraphs": [
                            "`spis crawl preflight --catalog FAMILY --host HOST` runs that family's declared preconditions through Stado's approved read-only probes. It starts nothing, claims no slot, and exits non-zero when the host cannot run the family. The desktop application asks the same question through Check host readiness and shows each refusal in the host's own words.",
                        ],
                    },
                    {
                        "title": "Start and track a run",
                        "code": [
                            "spis crawl start --catalog documentation-site-examples",
                            "spis crawl status --run RUN_ID",
                            "spis crawl resume --run RUN_ID",
                            "spis crawl import --run RUN_ID",
                        ],
                        "paragraphs": [
                            "The desktop application starts a run for one family or for all fifteen, tracks the run and its per-record jobs, and resumes or imports the same run.",
                        ],
                    },
                    {
                        "title": "What each family requires of a host",
                        "bullets": family_bullets(),
                    },
                ],
            },
            {
                "slug": "corpus-limits",
                "nav": "Corpus limits",
                "eyebrow": "Operating",
                "title": "Corpus limits",
                "description": "One site is one corpus, and a corpus holds a bounded number of pages.",
                "sections": [
                    {
                        "title": "The bound, and what falls outside it",
                        "paragraphs": [
                            format!(
                                "A documentation corpus holds at most {} page records. One site is one corpus: the record identity, the corpus selection and the catalog's refusal of a duplicate source URL all assume it, so a site declaring more in-scope pages than the bound cannot deliver the remainder under another record.",
                                super::docs_corpus::MAX_PAGE_RECORDS
                            ),
                            "A run in that position ends in the named state `retrieval_over_capacity`, and its report carries `corpus_bound`, `pages_outside_corpus` and `pages_outside_corpus_exact`. That is a capacity decision for an operator, not a failed retrieval: no number of retries makes the material fit. `spis docs-corpus status` and the desktop application both show those three numbers per site.",
                        ],
                    },
                    {
                        "title": "Sites that exceed the bound today",
                        "bullets": over_bound,
                    },
                ],
            },
        ],
    })
}

/// Render the plan into the canonical site's existing `DocPage` JSON model.
fn docs_payload(plan: &Value) -> Result<Value> {
    let pages = plan
        .get("pages")
        .and_then(Value::as_array)
        .context("generated plan carries no pages")?;
    let published_pages = pages
        .iter()
        .filter(|page| page["slug"] == "corpus-adoption");
    let mut rendered_pages = Vec::with_capacity(1);
    for page in published_pages {
        let sections = page
            .get("sections")
            .and_then(Value::as_array)
            .context("generated page carries no sections")?
            .iter()
            .map(|section| {
                let mut blocks = Vec::new();
                if let Some(lines) = section.get("code").and_then(Value::as_array) {
                    let content = lines
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !content.is_empty() {
                        blocks.push(json!({
                            "type": "code",
                            "language": "bash",
                            "content": content,
                        }));
                    }
                }
                if let Some(paragraphs) = section.get("paragraphs").and_then(Value::as_array) {
                    blocks.extend(paragraphs.iter().filter_map(Value::as_str).map(|text| {
                        json!({
                            "type": "paragraph",
                            "text": text,
                        })
                    }));
                }
                if let Some(items) = section.get("bullets").and_then(Value::as_array) {
                    blocks.push(json!({
                        "type": "list",
                        "ordered": false,
                        "items": items,
                    }));
                }
                json!({
                    "heading": section.get("title").and_then(Value::as_str).unwrap_or_default(),
                    "blocks": blocks,
                })
            })
            .collect::<Vec<_>>();
        rendered_pages.push(json!({
            "slug": page.get("slug").and_then(Value::as_str).unwrap_or_default(),
            "nav": page.get("nav").and_then(Value::as_str).unwrap_or_default(),
            "title": page.get("title").and_then(Value::as_str).unwrap_or_default(),
            "description": page.get("description").and_then(Value::as_str).unwrap_or_default(),
            "category": page.get("eyebrow").and_then(Value::as_str).unwrap_or("Guides"),
            "sections": sections,
        }));
    }
    Ok(json!({
        "schema": "wisent.spis-doc-pages.v1",
        "generatedBy": "spis docs-site",
        "pages": rendered_pages,
    }))
}

fn docs_content(plan: &Value) -> Result<String> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&docs_payload(plan)?)?
    ))
}

fn write_if_changed(path: &Path, contents: &str, check: bool, stale: &mut Vec<String>) -> Result<()> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    if existing == contents {
        return Ok(());
    }
    if check {
        stale.push(path.display().to_string());
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut check = false;
    let site_content = PathBuf::from(DOCS_CONTENT);
    let mut landing_content = None;
    let mut arguments = rest.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--check" => check = true,
            "--site-content" => {
                let path = arguments
                    .next()
                    .context("--site-content requires a path")?;
                landing_content = Some(PathBuf::from(path));
            }
            "--help" | "-h" => {
                println!(
                    "usage: spis docs-site [--check] [--site-content PATH]\n\
                     Generates {DOCS_BRIEF}, {DOCS_PLAN} and canonical DocPage JSON from this\n\
                     repository's own tables. The JSON is written to {DOCS_CONTENT}; pass the\n\
                     landing checkout's src/content/product-docs.json with --site-content to\n\
                     update that deployed consumer in the same operation. --check writes nothing\n\
                     and exits 1 when any selected output no longer matches the product."
                );
                return Ok(());
            }
            other => bail!("unknown argument: {other}"),
        }
    }
    let version = env!("CARGO_PKG_VERSION");
    let plan = plan(version);
    let brief = brief(version);
    let content = docs_content(&plan)?;
    let root = PathBuf::from(".");
    let mut stale = Vec::new();
    write_if_changed(
        &root.join(DOCS_BRIEF),
        &format!("{}\n", serde_json::to_string_pretty(&brief)?),
        check,
        &mut stale,
    )?;
    write_if_changed(
        &root.join(DOCS_PLAN),
        &format!("{}\n", serde_json::to_string_pretty(&plan)?),
        check,
        &mut stale,
    )?;
    write_if_changed(&site_content, &content, check, &mut stale)?;
    if let Some(path) = landing_content.as_ref().filter(|path| *path != &site_content) {
        write_if_changed(path, &content, check, &mut stale)?;
    }
    if check {
        if stale.is_empty() {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schema": "wisent.spis-docs-site.v1",
                    "state": "current",
                    "site_content": site_content,
                    "landing_content": landing_content,
                    "families": super::crawl::CATALOGS.len(),
                    "commands": super::SUBCOMMANDS.len(),
                }))?
            );
            return Ok(());
        }
        bail!(
            "committed documentation no longer matches the product: {}. Run `spis docs-site` \
             and commit the result; do not edit the generated files by hand",
            stale.join(", ")
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "wisent.spis-docs-site.v1",
            "state": "generated",
            "wrote": [DOCS_BRIEF, DOCS_PLAN, DOCS_CONTENT],
            "landing_content": landing_content,
            "families": super::crawl::CATALOGS.len(),
            "commands": super::SUBCOMMANDS.len(),
            "corpus_bound": super::docs_corpus::MAX_PAGE_RECORDS,
        }))?
    );
    let _ = lib::now_iso_utc();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cli_reference_names_the_commands_the_binary_dispatches() {
        let bullets = command_bullets();
        assert_eq!(bullets.len(), super::super::SUBCOMMANDS.len());
        assert!(bullets.iter().any(|line| line.starts_with("`crawl`")));
        assert!(bullets.iter().any(|line| line.starts_with("`docs-corpus`")));
        // The removed Python-era shim's commands must not reappear: the
        // committed brief listed `capture`, `catalogs`, `drift`, `verify` and
        // `sync-readmes`, none of which the binary has.
        for ghost in ["`capture`", "`catalogs`", "`drift`", "`verify`", "`sync-readmes`"] {
            assert!(
                !bullets.iter().any(|line| line.starts_with(ghost)),
                "{ghost} is not a command this binary dispatches"
            );
        }
    }

    #[test]
    fn every_family_is_documented_with_the_preconditions_the_preflight_enforces() {
        let rows = family_rows();
        assert_eq!(rows.len(), 15, "fifteen families, not the documented 13");
        for row in &rows {
            let requires: Vec<&str> = row["requires"]
                .as_array()
                .expect("a requires list")
                .iter()
                .filter_map(Value::as_str)
                .collect();
            assert!(
                requires.contains(&"cargo --version"),
                "{} does not document the worker program",
                row["family"]
            );
        }
    }

    #[test]
    fn the_documented_bound_is_the_bound_the_code_enforces() {
        let generated = plan("test");
        let text = serde_json::to_string(&generated).expect("a plan");
        assert!(text.contains(&super::super::docs_corpus::MAX_PAGE_RECORDS.to_string()));
        // And the four real sites are named rather than described.
        assert!(text.contains("21-google-cloud-documentation"));
        assert!(text.contains("01-mdn-web-docs"));
    }

    #[test]
    fn the_desktop_surface_is_declared_so_parity_can_be_checked() {
        let declared = brief("test");
        assert_eq!(declared["surfaces"]["desktop"]["declared"], json!(true));
        assert_eq!(declared["surfaces"]["desktop"]["parity_with"], json!("cli"));
    }

    #[test]
    fn the_site_payload_uses_the_canonical_doc_page_model() {
        let payload = docs_payload(&plan("test")).expect("a payload");
        assert_eq!(payload["schema"], json!("wisent.spis-doc-pages.v1"));
        assert_eq!(payload["generatedBy"], json!("spis docs-site"));
        let pages = payload["pages"].as_array().expect("pages");
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0]["slug"], json!("corpus-adoption"));
        assert_eq!(pages[0]["category"], json!("Getting started"));
        assert_eq!(
            pages[0]["sections"][0]["blocks"][0]["type"],
            json!("code")
        );
    }
}
