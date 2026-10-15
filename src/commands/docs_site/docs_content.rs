use super::*;

/// The generated `DocPage` JSON consumed by the canonical documentation site.
pub(crate) const DOCS_CONTENT: &str = "docs-site/docs.json";

/// The validated plan the module is rendered from.
pub(crate) const DOCS_PLAN: &str = "docs-plan.json";

/// What the product declares about its own surfaces.
pub(crate) const DOCS_BRIEF: &str = "docs-brief.json";

// Editorial prose, authored once and carried through unchanged. Kept apart
// from everything else in this file because it is the only part that is not
// derived: a generator that invented intent would be exactly the defect this
// replaces. Each section is its own constant, so a page cannot ask for one
// that is not written and render it empty.

/// What Spis is, for the overview page.
pub(crate) const WHAT_SPIS_IS: &str = "Spis is an evidence-grade reference corpus: every record carries its source, hashes, provenance class, and measured state. A record exists only with its evidence, and a missing observation is recorded as an evidence gap rather than promoted into prose.";

/// That the command line and the desktop application reach the same commands.
pub(crate) const PARITY: &str = "Every operation below is available from both surfaces. The command line and the desktop application are two ways to reach the same commands, not a full product and a viewer: if a command can do something, the application can too.";

/// One documented command, straight off the shipped dispatch table.
pub(crate) fn command_bullets() -> Vec<String> {
    super::SUBCOMMANDS
        .iter()
        .map(|command| format!("`{}` — {}.", command.name, command.description))
        .collect()
}

/// One line per family: its engine, and what its worker requires of a host.
///
/// Derived from [`super::crawl::crawl_catalogs`] and
/// [`super::crawl::engine_preconditions`], so the documented host
/// requirements cannot drift from the ones the preflight actually enforces.
/// That drift is not hypothetical: the published documentation described host
/// requirements nowhere at all, while five of the six engines were submitting
/// workers that needed a program the preflight never checked.
pub(crate) fn family_rows() -> Vec<Value> {
    crate::commands::crawl::crawl_catalogs()
        .iter()
        .map(|(catalog, engine)| {
            let requires: Vec<String> =
                crate::commands::crawl::engine_preconditions(engine, catalog)
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

/// The same families as [`family_rows`], read from the declarations rather
/// than back out of their JSON, where a missing field had read as ''.
pub(crate) fn family_bullets() -> Vec<String> {
    crate::commands::crawl::crawl_catalogs()
        .iter()
        .map(|(catalog, engine)| {
            let requires = crate::commands::crawl::engine_preconditions(engine, catalog)
                .into_iter()
                .map(|command| format!("`{}`", command.join(" ")))
                .collect::<Vec<String>>()
                .join(", ");
            format!("`{catalog}` — {engine} engine; the host must answer {requires}.")
        })
        .collect()
}

/// The product's own surfaces, replacing a brief that probed a removed shim.
pub(crate) fn brief(version: &str) -> Value {
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
                    .map(|command| command.name)
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
        "problems": Value::Array(Vec::new()),
    })
}
