use super::*;

/// The plan every page is rendered from.
pub(crate) fn plan(version: &str) -> Value {
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
pub(crate) fn docs_payload(plan: &Value) -> Result<Value> {
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

pub(crate) fn docs_content(plan: &Value) -> Result<String> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&docs_payload(plan)?)?
    ))
}

pub(crate) fn write_if_changed(path: &Path, contents: &str, check: bool, stale: &mut Vec<String>) -> Result<()> {
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
