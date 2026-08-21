# The Reference Pipeline

How a reference record comes to exist, in order. Every stage is one `reference`
command; the underlying tool keeps its own flags and passes them through.

```
capture ──► collect-images ──► verify ──► catalogs
   │              │
   │              └──► analyze-structures
   └──► audit-accessibility

sync-readmes ──► analyze-readmes        (README corpus, separate lane)
drift                                    (upstream monitoring, any time)
```

## 1. Capture — `reference capture`

Own products only. The tool runs the installed binaries on this workstation
through a real pseudo-terminal and keeps the recording. Flags: `--list`,
`--product <name>`, `--catalog-only`.

Third-party products are never captured here; their records cite
owner-published media or a Weles-driven browser run on a Stado host, and the
provenance class in the record states which.

## 2. Collect imagery — `reference collect-images`

Acquires official imagery for third-party records, recording dimensions,
hashes, provenance, and capture method. A retained vendor video is never
recorded as a local run.

## 3. Analyze — `reference analyze-structures`

Deterministic panel regions, layout model, separators, density, and confidence
from every stored overview image. Runs after imagery exists.

## 4. Accessibility — `reference audit-accessibility`

Audits captured references for accessibility evidence. Records move to
`complete` only when this stage has no failed or pending findings.

## 5. Verify — `reference verify`

The honesty gate. Re-probes media kinds, durations, frame counts, hashes, and
provenance classes from the retained bytes, then rewrites each record to what
the files actually prove. `--apply` writes the corrections; without it the run
is a dry report.

## 6. Regenerate catalogs — `reference catalogs`

Renders `example-catalogs.json` and the catalog pages from the records. This
is the consistency gate: it refuses to render when the index, records, and
files disagree, and it always prints measured numbers (complete/partial,
provenance mix), never intentions.

## Separate lane — the README corpus

- `reference sync-readmes` refreshes the fifty verbatim upstream README
  snapshots and their blob SHAs.
- `reference analyze-readmes` regenerates the structural analysis
  (`readme-examples/analysis.json`) behind the README guidance.

## Upstream monitoring — `reference drift`

Rechecks local hashes, current upstream README blobs, and every recorded URL.
Distinguishes a dead source from an authenticated, rate-limited, or
bot-guarded one. `--strict` turns any drift into a non-zero exit for CI use.

## Rules that do not bend

1. A record exists only with its evidence: source URL, hashes, provenance
   class, retained bytes where the contract requires them.
2. A missing observation is recorded as an evidence gap, never promoted into
   prose.
3. Interpretation lives in `wisent-ai/product-guidelines`, never here.
