# Walkthrough: offline corpus audit

This walkthrough was executed on 2026-08-24 against the checked-in corpus. It used only local bytes: no upstream probes, README refresh, browser work, fleet calls, or record rewrites.

## 1. Validate catalog consistency

```bash
spis generate-example-catalogs --check
```

The gate exited 0. It reported 14 catalogs: thirteen 50-record catalogs plus five pricing-page records. Every record was partial, with measured provenance summarized per catalog. This establishes cross-file consistency, not full product-evidence completeness.

## 2. Measure the smallest catalog in memory

```bash
spis verify-reference-evidence --catalog pricing-page-examples
```

Observed:

```text
dry run: records are measured and reported, nothing is written
pricing-page-examples: 0/5 complete

measured 5 records, 0 complete, 5 partial
    5  fewer than 3 states
    5  journey exposes fewer than 5 observed steps
    5  journey missing actor
    5  journey missing goal
    5  journey missing prerequisites
    5  journey missing failure_route
    5  journey missing recovery_route
    5  journey missing completion_evidence
    5  fewer than 8 mapped interactions
    5  motion analysis absent
    5  fewer than three accessibility observations
    5  accessibility never measured against the product
    5  no measured motion evidence
(exit 0)
```

The worktree remained clean after this dry run.

## 3. Verify all retained media without network access

```bash
spis check-upstream-drift --skip-network
```

Observed:

```text
local media verified: 3306
local media missing: 0
local media hash mismatch: 0
(exit 0)
```

This covers local existence and digest agreement across retained corpus media. Because `--skip-network` was used, it says nothing about current upstream URLs or README blobs.

## 4. Draft counted observations to a temporary path

```bash
spis guidelines cli-examples --out /tmp/spis-guidelines-demo.md
```

Observed header:

```text
wrote /tmp/spis-guidelines-demo.md
# CLI examples — derived guidelines (DRAFT)

Machine-derived from `cli-examples` on 2026-08-24. Every line cites its record count; a line without a count is not from this corpus.

**This is a DRAFT.** It becomes guidelines only after a human reviews it, edits it, and moves the confirmed rules into product-guidelines. Counts below quote only what the records measure; the corpus does not score taste.
```

The generated coverage section reported 50 records, zero complete, and zero with accessibility measured on the product. This illustrates the ownership boundary: Spis can count evidence; humans review and move accepted interpretation into the guidelines repository.

## Audit conclusion

At the captured revision, 3,306 retained media files matched their recorded hashes and the catalog graph was internally consistent. The run did not claim upstream freshness, accessibility coverage, or complete journeys. Those remain separate measurements and explicit gaps.
