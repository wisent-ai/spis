//! The subcommand table `spis` dispatches by and lists in its usage.

/// One subcommand: its name, what it does, and whether its own module
/// answers `--help` with its flags. A module that does not is answered by
/// the dispatcher, so `--help` never reaches code that would do the
/// command's work.
pub(super) struct Subcommand {
    pub(super) name: &'static str,
    pub(super) description: &'static str,
    pub(super) own_help: bool,
}

const fn sub(name: &'static str, description: &'static str, own_help: bool) -> Subcommand {
    Subcommand {
        name,
        description,
        own_help,
    }
}

pub(super) const SUBCOMMANDS: &[Subcommand] = &[
    sub(
        "onboarding",
        "show or reset the first-use walkthrough",
        true,
    ),
    sub(
        "corpus",
        "adopt or inspect an existing canonical reference corpus",
        true,
    ),
    sub(
        "docs-site",
        "generate this product's documentation from its own tables",
        true,
    ),
    sub(
        "crawl",
        "plan, submit, track, resume and import every crawler",
        true,
    ),
    sub(
        "crawl-cli",
        "crawl real CLI products through a PTY on Stado",
        true,
    ),
    sub(
        "crawl-docs",
        "full-text crawl of the 50-reference documentation set",
        false,
    ),
    sub(
        "crawl-mobile",
        "crawl real iOS or Android apps through Appium",
        true,
    ),
    sub(
        "crawl-desktop",
        "crawl real macOS or desktop apps through Cua Driver",
        true,
    ),
    sub(
        "crawl-web",
        "crawl real browser products through Weles on Stado",
        true,
    ),
    sub(
        "crawl-tui",
        "crawl real terminal applications through a PTY on Stado",
        true,
    ),
    sub(
        "docs-corpus",
        "read and import immutable documentation retrieval corpora",
        false,
    ),
    sub(
        "discover",
        "discover important pages behind a start URL",
        false,
    ),
    sub(
        "reference-record",
        "manage numbered reference records in a catalog",
        false,
    ),
    sub(
        "verify-reference-evidence",
        "measure and verify evidence fields of records",
        false,
    ),
    sub(
        "check-upstream-drift",
        "detect drift between corpus and upstream sources",
        false,
    ),
    sub(
        "catalog-type",
        "manage typed catalogs (add/edit/rename/remove)",
        true,
    ),
    sub(
        "generate-example-catalogs",
        "validate catalogs and write the JSON index",
        true,
    ),
    sub(
        "analyze-example-structures",
        "structural analysis of example screenshots",
        false,
    ),
    sub(
        "collect-example-images",
        "collect cover images for examples",
        false,
    ),
    sub(
        "capture-widths",
        "enqueue multi-width Weles capture batches",
        false,
    ),
    sub(
        "audit-reference-accessibility",
        "run axe audits over captured references",
        true,
    ),
    sub(
        "capture-cli-references",
        "pty-capture the CLI products of a capture plan into records",
        true,
    ),
    sub(
        "curate-catalog",
        "write a catalog's capture-pending records from a declared selector file",
        true,
    ),
    sub(
        "serve",
        "loopback HTTP/JSON backend for Spis Desktop (--port PORT, 0 picks one)",
        true,
    ),
];
