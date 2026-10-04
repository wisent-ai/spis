use super::*;

/// The plan every page is rendered from.
pub(crate) fn plan(version: &str) -> Value {
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
            "corpus-room": {
                "kind": "code",
                "location": "src/commands/crawl/cancel_group/extract_attempt_archive.rs volume_room",
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
                "description": "Every subcommand the shipped binary dispatches. `spis --help` prints this list to standard output and exits 0; `spis <subcommand> --help` never runs the subcommand and needs no adopted corpus: a subcommand with its own usage prints its flags, every other one its usage line and description.",
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
                "description": "A corpus holds every page its site declares; disk space is its only bound.",
                "sections": [
                    {
                        "title": "What bounds a corpus",
                        "paragraphs": [
                            "A documentation corpus holds every in-scope page its site's sitemaps, llms.txt or landing navigation name. No page count, source count or download total is chosen in Spis.",
                            "The bound is the free space of the volume a corpus is written to, measured when the run begins: a page that would not fit is recorded with the status `corpus_limit` and that number. Extracting an attempt archive, auditing a worker's tree and importing a corpus each measure their volume the same way and refuse an archive that needs more bytes or files than it reports free, naming both.",
                        ],
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
