# Spis

**Spis** is the evidence-grade reference corpus and corpus-maintenance CLI for people building interfaces. The repository holds roughly 700 third-party and locally captured records across interface families. Each record ties claims to retained bytes, source URLs, hashes, provenance, observed states, interactions, journeys, motion, and accessibility evidence.

Spis owns the corpus data and the machinery that acquires, measures, validates, searches, and monitors it. Interpretation and prescriptive guidance belong in [`wisent-ai/product-guidelines`](https://github.com/wisent-ai/product-guidelines). Own-product captures and operational monitoring metadata may live in a private companion repository and are not published here.

Licensed Apache-2.0. Third-party content remains attributable to its owners; see [the takedown policy](docs/takedown.md).

## Start here

- [Quick start](docs/quick-start.md)
- [Command-line reference](docs/cli-reference.md)
- [Reference pipeline](docs/pipeline.md)
- [Configuration](docs/configuration.md)
- [Architecture](docs/architecture.md)
- [Runbook](docs/runbook.md)
- [Examples and executed walkthroughs](docs/examples.md)

Core concepts:

- [Catalog](docs/concepts/catalog.md)
- [Reference record](docs/concepts/reference-record.md)
- [Evidence and completeness](docs/concepts/evidence.md)
- [Crawled documentation corpus](docs/concepts/docs-corpus.md)

## Build and invoke

The maintained implementation is the Rust binary in `src/`:

```bash
cargo build --release
./target/release/spis --help
./target/release/spis generate-example-catalogs --check
```

The checked-in `bin/spis` and release scripts still describe the retired Python tool layout. Do not use them as the authority for the current Rust command surface. See the [runbook](docs/runbook.md#rustpython-cutover-mismatch) for the known cutover mismatches.

## Repository layout

| Path | Owns |
|---|---|
| `src/commands/` | Rust implementations of acquisition, measurement, validation, query, and monitoring commands |
| `full-reference-contract.md` | human-readable evidence floor for a record |
| `example-catalogs.json` / `example-catalogs.md` | generated cross-catalog index and synthesis |
| `*-examples/sources.json` | selected examples and their visual/structure metadata |
| `*-examples/references.json` | generated per-catalog reference index |
| `*-examples/references/*/reference.json` | evidence record for one product |
| `readme-examples/` | curated README snapshots and source metadata |
| `documentation-site-examples/content-structure/` | documentation-site inventory definitions |
| `docs/` | operator and contributor corpus documentation |
| `kronika.sync.json` | source-to-document consistency manifest |

## Non-negotiable rules

1. A missing observation is an evidence gap, never inferred prose.
2. Retained bytes, their measured hashes, and their provenance must agree with metadata.
3. Own-product captures run the real installed product in a pseudo-terminal; browser captures go through Weles on a Stado-selected host.
4. Generated indexes are regenerated from records, not hand-edited.
5. A `partial` record is useful but is not silently presented as `complete`.
6. Third-party content is referenced, never claimed.

The former landing-page catalog was removed on 2026-08-21 because its observations were single-pass model guesses without a defensible contract. A replacement starts by defining an evidence contract, not by scraping screenshots.
