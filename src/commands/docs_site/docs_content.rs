use super::*;

/// The generated `DocPage` JSON consumed by the canonical documentation site.
pub(crate) const DOCS_CONTENT: &str = "docs-site/docs.json";

/// The validated plan the module is rendered from.
pub(crate) const DOCS_PLAN: &str = "docs-plan.json";

/// What the product declares about its own surfaces.
pub(crate) const DOCS_BRIEF: &str = "docs-brief.json";

/// Editorial prose, authored once and carried through unchanged.
///
/// Kept apart from everything else in this file because it is the only part
/// that is not derived. A generator that invented intent would be exactly the
/// defect this replaces.
pub(crate) const AUTHORED_SECTIONS: &[(&str, &str)] = &[
    (
        "what-spis-is",
        "Spis is an evidence-grade reference corpus: every record carries its source, hashes, provenance class, and measured state. A record exists only with its evidence, and a missing observation is recorded as an evidence gap rather than promoted into prose.",
    ),
    (
        "parity",
        "Every operation below is available from both surfaces. The command line and the desktop application are two ways to reach the same commands, not a full product and a viewer: if a command can do something, the application can too.",
    ),
];

pub(crate) fn authored(slug: &str) -> &'static str {
    AUTHORED_SECTIONS
        .iter()
        .find(|(name, _)| *name == slug)
        .map(|(_, text)| *text)
        .unwrap_or_default()
}

/// One documented command, straight off the shipped dispatch table.
pub(crate) fn command_bullets() -> Vec<String> {
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
pub(crate) fn family_rows() -> Vec<Value> {
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

pub(crate) fn family_bullets() -> Vec<String> {
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
