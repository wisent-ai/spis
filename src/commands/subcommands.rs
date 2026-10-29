//! The subcommand table `spis` dispatches by and lists in its usage.

#[derive(Clone, Copy)]
pub(super) enum Activation {
    Invocation,
    Corpus,
}

/// A registered command declares its own help and execution context. The
/// dispatcher answers help for modules that do not, without running them.
pub(super) struct Subcommand {
    pub(super) name: &'static str,
    pub(super) description: &'static str,
    pub(super) own_help: bool,
    pub(super) activation: Activation,
}

const fn sub(
    name: &'static str,
    description: &'static str,
    own_help: bool,
    activation: Activation,
) -> Subcommand {
    Subcommand {
        name,
        description,
        own_help,
        activation,
    }
}

pub(super) const SUBCOMMANDS: &[Subcommand] = &[
    sub(
        "onboarding",
        "show or reset the first-use walkthrough",
        true,
        Activation::Invocation,
    ),
    sub(
        "corpus",
        "adopt or inspect an existing canonical reference corpus",
        true,
        Activation::Invocation,
    ),
    sub(
        "docs",
        "read or import document corpora and generate product documentation",
        true,
        Activation::Invocation,
    ),
    sub(
        "crawl",
        "plan, submit, track, resume and import every crawler",
        true,
        Activation::Corpus,
    ),
    sub(
        "crawler",
        "run a declared crawl engine over real products",
        true,
        Activation::Invocation,
    ),
    sub(
        "discover",
        "discover important pages behind a start URL",
        false,
        Activation::Corpus,
    ),
    sub(
        "reference-record",
        "manage numbered reference records in a catalog",
        false,
        Activation::Corpus,
    ),
    sub(
        "verify-reference-evidence",
        "measure and verify evidence fields of records",
        false,
        Activation::Corpus,
    ),
    sub(
        "check-upstream-drift",
        "detect drift between corpus and upstream sources",
        false,
        Activation::Corpus,
    ),
    sub(
        "catalog-type",
        "manage typed catalogs (add/edit/rename/remove)",
        true,
        Activation::Corpus,
    ),
    sub(
        "generate-example-catalogs",
        "validate catalogs and write the JSON index",
        true,
        Activation::Corpus,
    ),
    sub(
        "analyze-example-structures",
        "structural analysis of example screenshots",
        false,
        Activation::Corpus,
    ),
    sub(
        "collect-example-images",
        "collect cover images for examples",
        false,
        Activation::Corpus,
    ),
    sub(
        "capture-widths",
        "enqueue multi-width Weles capture batches",
        false,
        Activation::Corpus,
    ),
    sub(
        "audit-reference-accessibility",
        "run axe audits over captured references",
        true,
        Activation::Corpus,
    ),
    sub(
        "capture-cli-references",
        "pty-capture the CLI products of a capture plan into records",
        true,
        Activation::Corpus,
    ),
    sub(
        "curate-catalog",
        "write a catalog's capture-pending records from a declared selector file",
        true,
        Activation::Corpus,
    ),
    sub(
        "serve",
        "loopback HTTP/JSON backend for Spis Desktop (--port PORT, 0 picks one)",
        true,
        Activation::Invocation,
    ),
];
