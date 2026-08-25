# Concept: catalog

A **catalog** is one interface family stored in a directory whose name ends with `-examples`. Examples include `cli-examples` and `pricing-page-examples`. A catalog is both a selection set and an evidence index; it is not merely a screenshot folder.

## Files

| Path | Role |
|---|---|
| `sources.json` | `wisent.example-catalog.v2`: catalog metadata and selected examples |
| `images/` | overview images named for numbered records |
| `references.json` | `wisent.full-reference-catalog.v2`: generated status/index summary |
| `references/NN-slug/reference.json` | one full evidence record |
| `references/NN-slug/media/` | retained motion/state evidence |
| `README.md` | generated catalog page |

At repository root, `example-catalogs.json` and `example-catalogs.md` aggregate every discovered catalog.

## Source catalog (`sources.json`)

The source schema carries `catalog`, `slug`, `title`, `description`, `status`, `curated_at`, count fields, and `examples`. Every example has a unique name and source URL, a category, a selection note, visual metadata, and deterministic `interface_structure` metadata. The declared byte count and SHA-256 must match the referenced image.

## Reference index (`references.json`)

The index declares its schema, catalog name, generation time, total/complete/partial counts, and entries. Each entry contains the 1-based index, name, record path, evidence status, and number of gaps. Generated provenance totals count measured assets by class and distinguish motion driven locally.

## Lifecycle

```bash
spis catalog-type add <slug> --title <title> [--description <text>]
spis catalog-type edit <slug> [--title <text>] [--description <text>] [--status <value>] [--rename <slug>]
spis catalog-type remove <slug> [--force]
```

The command accepts either the base slug or the `-examples` form and normalizes to lowercase kebab-case plus the suffix. Add creates an empty, explicitly `scaffolded` catalog; it does not fabricate records. Remove refuses a non-empty references directory unless `--force` is supplied.

Every successful catalog mutation re-executes the current Rust binary with `generate-example-catalogs`. Record mutations currently have a separate cutover defect described in the [runbook](../runbook.md#record-mutation-reports-index-regeneration-refused).

## Invariants

`generate-example-catalogs --check` enforces, among other things:

- schema and directory/catalog-name agreement;
- count fields equal array lengths;
- unique example names and URLs;
- local file existence, byte counts, and SHA-256 values;
- agreement between `sources.json`, `references.json`, and record files;
- valid evidence status and provenance vocabulary;
- completeness floors for motion, states, interactions, journey, and accessibility.

The generated pages are views. Change source records, then regenerate; do not repair disagreement by hand-editing generated summaries.
