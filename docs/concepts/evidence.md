# Concept: evidence and completeness

Spis separates **what bytes prove** from **what a writer might infer**. Evidence is retained, measured, attributable material; completeness is a contract evaluated from that material.

## Status is derived, not aspirational

`complete` means the measured record has no gaps and passes every structural floor. `partial` means one or more gaps remain. The catalog gate rejects a complete record with gaps and a partial record with no gaps.

`verify-reference-evidence` is the measurement pass:

- default: measure an in-memory copy, print per-catalog totals and grouped gap keys, write nothing;
- `--apply`: rewrite record measurements and matching index status/counts;
- `--no-state-match`: skip deriving state images from motion frames;
- `--jobs N`: worker count, default 8 and clamped to at least 1.

It checks retained bytes and hashes, derives media kind/duration, validates provenance, compares states where enabled, and recalculates gaps. Measurement errors for an individual path are printed as `verify-reference-evidence: PATH: DETAIL`; other records continue.

## Provenance is about acquisition

A vendor video does not become a local product run merely because it was downloaded locally. The canonical classes distinguish real product execution, local browser driving, and owner-published media. `capture_method` is descriptive text; `provenance_class` is the controlled value derived from it and the media kind.

## Minimum complete journey

A complete record describes more than a happy-path screenshot:

1. at least three distinct states;
2. at least eight interaction mappings with success, failure, cancellation, and recovery detail;
3. at least five ordered journey steps;
4. motion analysis including continuity, canonical timing, interruption/reversal, feedback, and a reduced-motion equivalent;
5. measured accessibility with at least three observations.

The journey object also names actor, goal, prerequisites, failure route, recovery route, and completion evidence.

## Gate versus measurement

- `verify-reference-evidence` measures and can rewrite evidence metadata.
- `generate-example-catalogs --check` validates the whole cross-file contract without writing.
- `generate-example-catalogs` validates, then renders catalog READMEs and root indexes.
- `check-upstream-drift` asks whether retained bytes and upstream sources have changed since capture. It does not promote evidence status.

The distinction matters operationally: a dry measurement can exit 0 while reporting many explicit gaps, and the catalog consistency gate can exit 0 for internally honest partial records.
