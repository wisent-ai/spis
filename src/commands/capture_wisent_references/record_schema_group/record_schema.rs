use super::*;

pub(crate) const RECORD_SCHEMA: &str = "wisent.full-product-reference.v2";

pub(crate) const INDEX_SCHEMA: &str = "wisent.full-reference-catalog.v2";

pub(crate) const SOURCES_SCHEMA: &str = "wisent.example-catalog.v2";

pub(crate) const COLS: usize = 100;

pub(crate) const ROWS: usize = 32;

pub(crate) const PROMPT: &str = "wisent-ref$ ";

pub(crate) const PROBE_FLAG: &str = "--wisent-reference-probe";

pub(crate) const SHELL: &str = "/bin/bash";

pub(crate) const FONT_PX: usize = 15;

// The former script wrote its scratch under ~/.stado/work/wisent-capture; this
// checkout is confined to the spis tree and ~/.spis, so the scratch moves with it.
pub(crate) fn root() -> PathBuf {
    super::corpus::data_root()
}

pub(crate) fn catalog_dir() -> PathBuf {
    root().join("wisent-product-examples")
}

pub(crate) fn scratch_root() -> PathBuf {
    home_dir().join(".spis").join("work").join("wisent-capture")
}

pub(crate) fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
}

// ------------------------------------------------------------------ products

pub(crate) struct Product {
    pub(crate) slug: &'static str,
    pub(crate) name: &'static str,
    pub(crate) binary: &'static str,
    pub(crate) repository: &'static str,
    pub(crate) product_url: &'static str,
    pub(crate) category: &'static str,
    pub(crate) one_line: &'static str,
    pub(crate) selection_note: &'static str,
    pub(crate) version_cmd: &'static str,
    pub(crate) help_cmd: &'static str,
    pub(crate) sub_cmd: &'static str,
    pub(crate) sub_note: &'static str,
}

// One entry per Wisent product with a runnable CLI on this host. `repository` is
// the Wisent repository the binary comes from; `version_cmd` is the product's own
// version form, which several of these do not have — the refusal is then the
// measurement.
pub(crate) const PRODUCTS: &[Product] = &[
    product!(
        "stado",
        "Stado",
        "stado",
        "wisent-ai/stado",
        "https://github.com/wisent-ai/stado",
        "Infrastructure / compute and queue control plane",
        "Policy-controlled queue and compute control plane for machines you own or authorize.",
        "The widest command surface we ship: a Clap-style noun tree over jobs, hosts, quota and \
         credits. Study how a large control-plane CLI keeps its top-level help to one screen of \
         verbs and pushes detail into per-command help.",
        "stado --version",
        "stado --help",
        "stado capabilities --help",
        "Per-command help exists and is the documented way down one level."
    ),
    product!(
        "skarbiec",
        "Skarbiec",
        "skarbiec",
        "wisent-ai/skarbiec",
        "https://github.com/wisent-ai/skarbiec",
        "Security / credential and authentication management",
        "Credential and authentication management for the AI era.",
        "The only product here whose help is machine-readable JSON rather than prose, and the \
         only one whose first refusal is a state gate rather than a parse error. Study what a \
         credential CLI is willing to say before a vault exists.",
        "skarbiec --version",
        "skarbiec help",
        "skarbiec status --help",
        "Skarbiec has no per-subcommand help; the subcommand is reached before argument parsing \
         and answers with the vault state gate instead."
    ),
    product!(
        "weles",
        "Weles",
        "weles",
        "wisent-ai/weles",
        "https://github.com/wisent-ai/weles",
        "Automation / authorized browser execution",
        "Authorized browser execution for AI agents, with signed receipts.",
        "A CLI whose real work is gated on an authorization boundary, so its safe surface is \
         help and identity only. Study how a product that refuses unauthorized work advertises \
         that boundary in its own usage text.",
        "weles --version",
        "weles --help",
        "weles version",
        "Weles exposes no per-subcommand help. `weles version` is the only subcommand that can \
         be run here without authorizing a workflow or touching durable onboarding state, so \
         that is the subcommand surface this record measures."
    ),
    product!(
        "jeden",
        "Jeden",
        "jeden",
        "wisent-ai/jeden",
        "https://github.com/wisent-ai/jeden",
        "Agents / autonomous coding and company building",
        "The autonomous agent for building software and running the loop around it.",
        "A single-block usage synopsis for an agent runtime whose flags are permission grants \
         (`--allow-write`, `--allow-command`, `--yolo`). Study how a dangerous capability set is \
         presented in first-run help.",
        "jeden --version",
        "jeden --help",
        "jeden version",
        "Jeden's help is one usage block covering every subcommand; there is no per-subcommand \
         help. `jeden run --help` is not a safe probe: when probed once outside this recording \
         it resolved credentials through Skarbiec before parsing `--help` and failed with an \
         HTTP 403, so this record measures `jeden version` instead and reports that finding \
         rather than re-running it."
    ),
    product!(
        "probierz",
        "Probierz",
        "probierz",
        "wisent-ai/probierz",
        "https://github.com/wisent-ai/probierz",
        "Quality / test execution and evidence boundary",
        "The quality-evidence boundary: selection, execution, evidence, and verdicts.",
        "The one product whose refusal is a structured machine-readable failure envelope rather \
         than a usage dump. Study a CLI that answers an unknown surface with a parseable \
         `probierz-failure` line plus one plain sentence.",
        "probierz --version",
        "probierz --help",
        "probierz specs --help",
        "Probierz has no per-subcommand help: `--help` after a subcommand is read as a surface \
         name and refused. That refusal is the observed subcommand surface."
    ),
    product!(
        "oko-cli",
        "Oko (oko-cli)",
        "oko-cli",
        "wisent-ai/oko",
        "https://github.com/wisent-ai/oko",
        "Observability / agent session inspection",
        "Understand your team's interactions with AI.",
        "A headless companion CLI with one flat usage block and no version form at all. Study \
         the cost of that: the same text answers help, an unknown flag, and a subcommand help \
         request, and only the exit status distinguishes them.",
        "oko-cli --version",
        "oko-cli --help",
        "oko-cli diff --help",
        "Oko answers a subcommand help request with the whole top-level usage block."
    ),
    product!(
        "singularity",
        "Singularity",
        "singularity",
        "wisent-ai/singularity",
        "https://github.com/wisent-ai/singularity",
        "Agents / autonomous agent runtime",
        "An open-source framework for autonomous agents that execute tasks and manage resources.",
        "The narrowest installed surface in the catalog: one subcommand, `onboarding`. Study a \
         product whose CLI deliberately exposes only the first-use journey.",
        "singularity --version",
        "singularity --help",
        "singularity onboarding --help",
        "argparse gives every subcommand its own help; `onboarding` is the only one."
    ),
    product!(
        "tama",
        "Tama (tama-cli)",
        "tama",
        "wisent-ai/hooks-rotator",
        "https://github.com/wisent-ai/hooks-rotator",
        "Policy / agent and Git hook catalog",
        "Your AI agent made a mistake? Tama creates rules so that it never happens again.",
        "A policy catalog CLI that answers `--help` after a subcommand by ignoring the flag and \
         running the read-only command. Study how a hook installer separates a plan from an \
         install.",
        "tama --version",
        "tama --help",
        "tama install-plan --help",
        "Tama ignores a trailing `--help` and executes the subcommand; `install-plan` only \
         reports the paths an install would touch, so nothing is written."
    ),
    product!(
        "transcript-lake",
        "Transcript Lake",
        "transcript-lake",
        "wisent-ai/transcript-lake",
        "https://github.com/wisent-ai/transcript-lake",
        "Data / local privacy-masked transcript archive",
        "Nothing you ever told an AI is lost again.",
        "The only help here that opens with a `Start safely:` section and names the read-only \
         commands first. Study help text that is ordered by risk rather than alphabetically.",
        "transcript-lake --version",
        "transcript-lake --help",
        "transcript-lake paths --help",
        "Per-command usage exists and prints one line of purpose with its flags."
    ),
    product!(
        "transcript-label-trainer",
        "Transcript Label Trainer",
        "transcript-label-trainer",
        "wisent-ai/transcript-label-trainer",
        "https://github.com/wisent-ai/transcript-label-trainer",
        "Models / local classifiers over transcript labels",
        "Small models for your custom harness needs.",
        "An argparse CLI that states its own boundary in the help body — 'Never writes to the \
         lake.' Study a product that publishes what it will not touch above its command list.",
        "transcript-label-trainer --version",
        "transcript-label-trainer --help",
        "transcript-label-trainer info --help",
        "argparse gives every subcommand its own help."
    ),
];

// Products deliberately excluded, with the reason. Kept here so the catalog scope
// is a statement that can be checked rather than a claim about what happened to be
// found.
pub(crate) const EXCLUSIONS: &[(&str, &str, &str)] = &[
    (
        "omp",
        "~/.local/bin/omp",
        "Not a Wisent repository: the binary's own build metadata names \
         github.com/can1357/oh-my-pi. It is the harness we run, not a product we ship.",
    ),
    (
        "stado_fleet",
        "~/.stado/bin/stado_fleet",
        "A second binary of the same product (Stado, wisent-ai/stado), not a separate product.",
    ),
    (
        "wc",
        "~/.local/bin/wc",
        "The legacy name of the Stado CLI; `wc --version` prints `stado 0.6.0`. Same product.",
    ),
];

// --------------------------------------------------------------- small utils

/// Python `%g`-style float rendering (shortest form, no trailing `.0`).
pub(crate) fn g(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    if x.abs() >= 1e16 || x.abs() < 1e-4 {
        return format!("{x:e}");
    }
    let s = format!("{x}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Python-style `repr()` of a string: single quotes unless it contains one.
pub(crate) fn py_repr(s: &str) -> String {
    if s.contains('\'') {
        format!("{s:?}")
    } else {
        format!("'{s}'")
    }
}
