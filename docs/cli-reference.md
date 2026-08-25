# Command-line reference

The maintained entry point is the Rust binary built from `src/main.rs`:

```text
spis <subcommand> [flags]
```

Top-level help exits 0. An unknown subcommand prints `unknown subcommand: NAME`, prints usage, and exits 2. A command error is printed as `error: DETAIL` and exits 1 unless the command defines a specialized exit policy.

Subcommands documented with `--help` implement it without starting a crawl. Treat the signatures below as the authoritative interface.

## Corpus acquisition and query

### `crawl-docs`

```text
spis crawl-docs --host <target> (--site <NN-slug> | --all)
                [--exclude <slug>]... [--workers <n>]
                [--host-delay <seconds>] [--refresh]
```

Submits an exact-revision Stado job that crawls checked-in documentation inventories into the worker's `$HOME/.spis/docs-corpus`, then archives the result to Stado storage. Defaults: 64 workers and 0.3 seconds per host. `--exclude` applies with `--all`. `--refresh` is currently parsed but has no behavioral effect. Networked and mutating.

### `docs-corpus`

```text
spis docs-corpus status
spis docs-corpus search --query <text> [--site <slug>] [--limit <n>]
spis docs-corpus show --site <slug> --url <exact-url>
```

Prints exactly one JSON document. Search limit defaults to 20. The current parser accepts `--site` for search but the scan does not apply it; `show` does. Read-only.

### `discover`

```text
spis discover <start-url> --catalog <slug> [--limit <n>] [--max-links <n>]
```

Fetches a start page, extracts at most 120 same-origin links by default, ranks through Brama when configured or keyword heuristics otherwise, and selects 6 by default. It then scaffolds records and thumbnails. The current Rust port still invokes removed `catalog-type.py`/record helpers for parts of this flow; see the runbook. Networked and mutating.

### `scrape-products` (dispatched but omitted from top-level help)

```text
spis scrape-products --catalog <name> --refs-dir <path>
```

Reads the catalog's references, fetches each product URL sequentially, and writes its local docs-corpus page/state/done data. Unknown arguments are currently ignored. Networked and mutating.

## Catalog and record management

### `catalog-type`

```text
spis catalog-type add <slug> --title <title> [--description <text>]
spis catalog-type edit <slug> [--title <title>] [--description <text>]
                       [--status <status>] [--rename <slug>]
spis catalog-type remove <slug> [--force]
```

Creates, edits/renames, or removes a managed `*-examples` catalog. Slugs must be lowercase kebab-case. Add and mutations regenerate indexes through the current executable. Remove refuses stored reference directories unless forced. Mutating.

### `reference-record`

```text
spis reference-record add <catalog> --name <name> --source-url <url>
     --category <category> --selection-note <text> --visual <file>
     [--owner <url>]
spis reference-record get <catalog> <NN|slug>
spis reference-record remove <catalog> <NN|slug> [--force]
```

Adds a numbered partial scaffold, emits a joined JSON view, or removes/renumbers a record. Remove refuses motion or journey evidence unless forced. Add/remove currently write their primary change before a broken Python index-regeneration call; reconcile with `generate-example-catalogs`. Get is read-only; add/remove mutate.

### `generate-example-catalogs`

```text
spis generate-example-catalogs [--check]
```

Validates every discovered catalog, record, index, and evidence file. `--check` writes nothing. Without it, writes catalog `README.md` files and root `example-catalogs.{json,md}`. Unknown flags fail.

### `verify-reference-evidence`

```text
spis verify-reference-evidence [--catalog <catalog>] [--apply]
                               [--no-state-match] [--jobs <n>]
```

Measures all catalogs or one. Default is an explicit dry run; `--apply` rewrites records and index status/count fields. State matching is enabled by default. Jobs default to 8 and are clamped to at least 1. A path-level measurement error is logged and other records continue.

### `check-upstream-drift`

```text
spis check-upstream-drift [--skip-network] [--skip-readme]
                          [--write-report] [--strict]
```

Always verifies local evidence bytes. Network mode also checks README blob SHAs and source URLs; `--skip-readme` only matters when network is enabled. `--write-report` creates `upstream-drift.json`. `--strict` exits 1 when any local, README, or upstream drift exists. Default without strict reports drift and exits 0.

## Analysis and guidance

### `analyze-example-structures`

```text
spis analyze-example-structures [catalog ...]
```

Analyzes all compiled catalog names when none are supplied. Rewrites each `sources.json` with deterministic structure data; writes `structure-analysis-failures.json` while failures remain and fails if any are unresolved. Mutating.

### `analyze-readme-examples`

```text
spis analyze-readme-examples
```

Accepts no arguments. Analyzes `readme-examples/NN-*`, writes `readme-examples/analysis.json` with schema `wisent.readme-example-analysis`, and prints the summary JSON. Mutating.

### `guidelines`

```text
spis guidelines <catalog> [--out <file>]
```

Drafts counted, evidence-linked writing observations. Default output is `<catalog>/guidelines-draft.md`. Refuses a catalog with no measured records. The draft is not normative until human review. Mutating.

## Acquisition and auditing

### `sync-readme-examples`

```text
spis sync-readme-examples --host <target>
                           [--secret-env GH_TOKEN=<skarbiec-item>]
```

Submits an exact-revision Stado job that obtains GitHub credentials from its scoped environment, fetches each curated repository and README, and archives the refreshed snapshots and metadata to Stado storage. `--help` only prints usage. Networked and mutating.

### `collect-example-images`

```text
spis collect-example-images [--replace] [catalog ...]
```

Processes all compiled catalogs or selected names. Retains qualifying images with dimensions/hash/provenance. Existing images are preserved unless `--replace`. Writes `failures.json` and fails when unresolved entries remain. Networked and mutating.

### `capture-widths`

```text
spis capture-widths <catalog> [--record <NN|slug>]
                    [--host <target>] [--dry-run]
```

Builds a `wisent.weles-capture-plan.v1` under `$HOME/.spis/work/landing-width-plans/`. Dry run prints the plan; otherwise submits via `stado host weles-capture` with a fallback Stado job submission path. Never uses direct SSH. Network/operator-state affecting unless dry.

### `audit-reference-accessibility`

```text
spis audit-reference-accessibility [--catalog <name>]... [--records <selection>]
     [--batch <id>] [--target <host>] [--plan <path>] [--dry-run]
     [--poll-seconds <n>] [--timeout-minutes <n>]
```

Defaults to compiled catalogs/target, 15-second polls, and a 120-minute timeout. Plans Weles axe work, submits through Stado, polls, retrieves, verifies, merges results, and invokes evidence verification. Poll and timeout values must be at least 1. Planning refusal exits 2; execution/retrieval failure exits 3; success exits 0. Networked and mutating except dry plan mode.

### `capture-wisent-references`

```text
spis capture-wisent-references [--list] [--product <slug>]... [--catalog-only]
```

`--list` resolves and probes the ten compiled Wisent products; it executes their version commands. Normal mode records selected installed CLIs in a controlled 100x32 PTY through version, help, subcommand, invalid flag, cancellation, recovery help, and no-color help. It writes an asciinema v2 cast, five rendered screenshots, reference records, and a generated catalog. `--catalog-only` skips new captures and rebuilds the catalog from retained runs. Mutating; normal capture executes local products.

## Exit summary

| Condition | Exit |
|---|---:|
| top-level help or successful command | 0 |
| ordinary Rust command error | 1 |
| unknown top-level command | 2 |
| accessibility planning refusal | 2 |
| accessibility execution/retrieval failure | 3 |
| strict drift detected | 1 |
