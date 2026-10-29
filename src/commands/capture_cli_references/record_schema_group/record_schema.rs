use super::*;

pub(crate) use crate::commands::reference_contract::{
    CATALOG_SCHEMA as SOURCES_SCHEMA, INDEX_SCHEMA, RECORD_SCHEMA,
};

pub(crate) const PROMPT: &str = "wisent-ref$ ";

pub(crate) const PROBE_FLAG: &str = "--wisent-reference-probe";

pub(crate) const SHELL: &str = "/bin/bash";

// The transient scratch tree is confined to ~/.spis so it moves with the
// product; the catalog itself lives under the adopted corpus root.
pub(crate) fn root() -> PathBuf {
    crate::commands::corpus::data_root()
}

pub(crate) fn catalog_dir() -> PathBuf {
    root().join(&plan().catalog)
}

/// The scratch tree under `~/.spis`; an unset HOME is refused by the shared
/// helper rather than putting the tree under `/tmp`.
pub(crate) fn scratch_root() -> anyhow::Result<PathBuf> {
    Ok(crate::commands::docs_corpus::home_dir()?
        .join(".spis")
        .join("work")
        .join("cli-capture"))
}

// ---------------------------------------------------------------- the plan

pub(crate) const PLAN_SCHEMA: &str = "spis.cli-capture-plan.v1";

/// One product with a runnable CLI, as the capture plan declares it.
/// `repository` is the repository the binary comes from; `version_cmd` is the
/// product's own version form, which several products do not have — the
/// refusal is then the measurement.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Product {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) binary: String,
    pub(crate) repository: String,
    pub(crate) product_url: String,
    pub(crate) category: String,
    pub(crate) one_line: String,
    pub(crate) selection_note: String,
    pub(crate) version_cmd: String,
    pub(crate) help_cmd: String,
    pub(crate) sub_cmd: String,
    pub(crate) sub_note: String,
}

/// A binary deliberately left out of the catalog, with the reason, so the
/// catalog scope is a statement that can be checked rather than a claim
/// about what happened to be found.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Exclusion {
    pub(crate) binary: String,
    pub(crate) resolved: String,
    pub(crate) reason: String,
}

/// The pseudo-terminal every capture runs in and the font and colours its
/// frames are rendered with. The plan states it; no size or colour is
/// compiled into the binary.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Terminal {
    pub(crate) columns: usize,
    pub(crate) rows: usize,
    pub(crate) font_px: usize,
    /// `#rrggbb` the frame is filled with.
    pub(crate) background: String,
    /// `#rrggbb` the text is drawn in.
    pub(crate) foreground: String,
}

/// The capture plan: which catalog the records belong to, which products are
/// captured, which binaries are excluded and the terminal they run in. It is a
/// JSON file the operator names with `--plan`; nothing about a product or the
/// terminal is compiled into the binary.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Plan {
    pub(crate) schema: String,
    pub(crate) catalog: String,
    pub(crate) title: String,
    pub(crate) terminal: Terminal,
    pub(crate) products: Vec<Product>,
    pub(crate) exclusions: Vec<Exclusion>,
}

static PLAN: std::sync::OnceLock<Plan> = std::sync::OnceLock::new();

/// Read and check the plan, then make it the one this run captures from.
pub(crate) fn load_plan(path: &Path) -> Result<&'static Plan> {
    let bytes =
        std::fs::read(path).with_context(|| format!("read capture plan {}", path.display()))?;
    let plan: Plan = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse capture plan {}", path.display()))?;
    if plan.schema != PLAN_SCHEMA {
        bail!(
            "capture plan {} has schema {:?}; this command reads {PLAN_SCHEMA}",
            path.display(),
            plan.schema
        );
    }
    if plan.catalog.is_empty() || plan.catalog.contains('/') {
        bail!("capture plan {} names catalog {:?}; a catalog is one directory name under the corpus root", path.display(), plan.catalog);
    }
    if plan.products.is_empty() {
        bail!("capture plan {} declares no products", path.display());
    }
    let positive = |value: usize| value != 0;
    if !positive(plan.terminal.columns)
        || !positive(plan.terminal.rows)
        || !positive(plan.terminal.font_px)
    {
        bail!(
            "capture plan {} states terminal {}x{} at {} px; columns, rows and font_px must each be at least 1",
            path.display(),
            plan.terminal.columns,
            plan.terminal.rows,
            plan.terminal.font_px
        );
    }
    let hex = |value: &str| {
        value.strip_prefix('#').is_some_and(|digits| {
            // Six hex digits after '#': https://www.w3.org/TR/css-color-4/#hex-notation
            digits.len() == 6 && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
    };
    if !hex(&plan.terminal.background) || !hex(&plan.terminal.foreground) {
        bail!(
            "capture plan {} states terminal background {:?} and foreground {:?}; each must be a #rrggbb colour",
            path.display(),
            plan.terminal.background,
            plan.terminal.foreground
        );
    }
    let mut seen = std::collections::BTreeSet::new();
    for product in &plan.products {
        if !seen.insert(product.slug.as_str()) {
            bail!(
                "capture plan {} declares product {} twice",
                path.display(),
                product.slug
            );
        }
    }
    if PLAN.set(plan).is_err() {
        bail!("a capture plan is already loaded in this process");
    }
    Ok(PLAN
        .get()
        .expect("the capture plan was set on the line above"))
}

/// The loaded plan. `run` loads it before anything reads a product, so a
/// read before that is a programming error, not an operator one.
pub(crate) fn plan() -> &'static Plan {
    PLAN.get()
        .expect("the capture plan is loaded by run before any product is read")
}

pub(crate) fn products() -> &'static [Product] {
    &plan().products
}

/// The terminal the loaded plan states.
pub(crate) fn terminal() -> &'static Terminal {
    &plan().terminal
}
