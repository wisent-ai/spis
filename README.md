# Spis

**Spis** — the evidence-grade reference corpus for people building interfaces.
700 records across 13 interface families, every record with its source, hashes,
provenance class, and measured state. Where other libraries show you screenshots,
Spis shows you what can be proven about them.

Licensed Apache-2.0. Own-product captures and operational monitoring metadata
live in a private companion repository and are intentionally not here.

Guidelines that interpret this data live in
[`wisent-ai/product-guidelines`](https://github.com/wisent-ai/product-guidelines).
This repository owns the data and the machinery that produces it.


## Command line

One entry point for every tool; flags pass through unchanged:

```bash
bin/spis capture --list          # what own-product capture can run here
bin/spis verify --apply          # rewrite records to what the bytes prove
bin/spis catalogs --check        # consistency gate over index and records
bin/spis drift --strict          # non-zero exit on upstream drift (CI)
```

The full flow, stage by stage, is in [`docs/pipeline.md`](docs/pipeline.md).

`collect-images` and `analyze-structures` additionally require Pillow
(`python3 -m pip install pillow`); every other command runs on the standard library.

## Layout

| Path | Owns |
|---|---|
| `full-reference-contract.md` | the evidence floor every record must meet |
| `reference_contract.py` | the machine-readable schema behind that contract |
| `example-catalogs.json` / `example-catalogs.md` | the index of all catalogs and records |
| `*-examples/` | one directory per catalog: sources, references, retained media |
| `capture-wisent-references.py` | captures own-product references through the real product |
| `audit-reference-accessibility.py` | accessibility audit of captured references |
| `collect-example-images.py` | attributable official imagery with hashes and provenance |
| `analyze-example-structures.py` | deterministic panel and layout analysis |
| `verify-reference-evidence.py` | re-probes media, hashes, durations, provenance |
| `check-upstream-drift.py` | upstream README and URL drift report (report stays private) |
| `generate-example-catalogs.py` | renders catalogs from records; refuses disagreement |
| `sync-readme-examples.py` / `analyze-readme-examples.py` | README corpus refresh and structural analysis |
| `bin/spis` | one CLI over all tools; flags pass through |
| `docs/pipeline.md` | the capture → verify → catalogs flow |
| `docs/takedown.md` | content rights and takedown policy |
| `LICENSE` | Apache-2.0 |

## Rules
1. A record exists only with its evidence: source URL, hashes, provenance class, and retained bytes where required by the contract.
2. A missing observation is recorded as an evidence gap, never promoted into prose.
3. Captures of real products run through the real product or through Weles on a Stado-selected host; no ad-hoc scraping.
4. Catalogs are generated, never hand-edited: change records, then run `generate-example-catalogs.py`.
5. Interpretation belongs in `product-guidelines`, not here.
6. Third-party content is referenced, never claimed; owners can remove or relink
   their media at any time via [`docs/takedown.md`](docs/takedown.md).

## Status

The landing-page catalog (50 first-viewport screenshots and model-read observations) was removed on 2026-08-21: its observations were single-pass model guesses without a defensible contract. Rebuilding a landing reference set starts with a new entry in `full-reference-contract.md`, not with a scraper.