# What is Spis?

Spis is an evidence-grade reference corpus and the command-line machinery that maintains it. Its subject is product interface behavior: what a real product shows, how it changes state, how people interact with it, how failure and recovery work, and which accessibility properties were actually measured.

A conventional inspiration gallery is optimized for browsing screenshots. Spis is optimized for auditability. A selected example points to an attributable source and a retained evidence record. The record names its capture method and provenance, hashes local bytes, describes observed states and transitions, maps interactions and a journey, and records unknowns as evidence gaps. A generated index can say a record is `complete` only when the retained material passes the machine-enforced floor.

## What Spis owns

- interface-family catalogs and their source selections;
- full per-product evidence records and retained media;
- deterministic measurement, validation, structure analysis, and catalog rendering;
- real-product PTY capture for selected Wisent CLIs;
- Weles/Stado plans for browser width and accessibility evidence;
- a curated README snapshot corpus and its structural analysis;
- a resumable full-text documentation corpus and read-only JSON query surface;
- local and upstream drift checks.

## What Spis does not own

- design prescriptions or taste judgments — those belong in `wisent-ai/product-guidelines`;
- third-party copyrights — sources remain attributable and are covered by the [takedown policy](takedown.md);
- fleet placement or browser execution — Stado selects hosts and Weles performs authorized browser work;
- secrets — Spis consumes authenticated tools from its environment but is not a credential store;
- fabricated completion — missing evidence remains a named gap.

## The trust model

Spis trusts neither prose nor a filename by itself. It checks file presence, dimensions, bytes, SHA-256, media kind, duration, provenance vocabulary, cross-file indexes, and completeness floors. The local corpus can still be partial or stale; `verify-reference-evidence`, `generate-example-catalogs --check`, and `check-upstream-drift` answer different parts of that risk.

The maintained runtime is the Rust crate in `src/`. Some checked-in Python-era launch/release artifacts remain and are called out explicitly in the [runbook](runbook.md).
