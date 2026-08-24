# Reference pipeline

Spis has three related lanes: interface evidence, README research, and documentation full text.

```text
catalog/record scaffold
        |
        +--> acquire owner media or real product/browser capture
        |        +--> analyze-example-structures
        |        +--> audit-reference-accessibility
        |
        +--> verify-reference-evidence [--apply]
        +--> generate-example-catalogs [--check]
        +--> check-upstream-drift

sync-readme-examples --> analyze-readme-examples --> guidelines
crawl-docs --> docs-corpus {status,search,show}
```

## 1. Select and scaffold

`catalog-type` creates or edits a `*-examples` family. `reference-record add` introduces one selected product with source/category/rationale and a measured overview image. New records are partial and list what has not been observed.

## 2. Acquire authentic evidence

Use owner-published material for third-party products, local Weles browser recording for browser evidence, or `capture-wisent-references` for real installed Wisent CLIs. `collect-example-images` acquires qualifying interface imagery; `capture-widths` and `audit-reference-accessibility` submit Weles work through Stado rather than direct SSH.

Acquisition method and provenance class must agree. Retain bytes under the record, not merely a remote URL.

## 3. Analyze

`analyze-example-structures [catalog ...]` deterministically writes image layout regions, separators, density, and confidence into `sources.json`. It writes `structure-analysis-failures.json` while unresolved entries remain.

`audit-reference-accessibility` plans, submits, polls, retrieves, verifies, and merges Weles axe results. Planning refusal exits 2; execution or retrieval failure exits 3. `--dry-run` stops after validating and writing the plan.

## 4. Measure

`verify-reference-evidence` re-probes retained media, hashes, dimensions, durations, provenance, state matches, journey/interactions, motion analysis, and accessibility. Without `--apply` it is a report only. With `--apply` it rewrites records and matching reference-index statuses to what the bytes prove.

## 5. Gate and render

`generate-example-catalogs --check` validates without writing. Without `--check`, the same validation precedes rendering every catalog `README.md` plus `example-catalogs.json` and `example-catalogs.md`. A partial record is valid only when its gaps are explicit and consistent.

## 6. Monitor drift

`check-upstream-drift` always verifies local media. Unless `--skip-network` is used, it also checks README blob SHAs and recorded URLs, distinguishing reachable, gone, guarded, and unresolved sources. `--strict` exits 1 if any drift exists. `--write-report` creates `upstream-drift.json`, which operational policy may keep private.

## README research lane

`sync-readme-examples` refreshes curated GitHub snapshots using `gh auth token`, then writes source metadata, the index, and `scrape-run.json`. It ignores every argument: even `--help` executes the refresh. `analyze-readme-examples` deterministically writes `readme-examples/analysis.json`. `guidelines <catalog>` drafts counted observations for human review; interpretation remains outside Spis.

## Documentation full-text lane

`crawl-docs` resolves sitemap/override inventories, obeys robots rules, and writes resumable local archives. `docs-corpus` exposes read-only JSON status, search, and exact-page lookup.
