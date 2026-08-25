# The Reference Pipeline

How a reference record comes to exist. Product capture happens first; evidence
verification and catalog generation remain separate deterministic stages.

```text
real product crawler ──► retained state graph / recording
                                │
                                ├──► verify-reference-evidence
                                ├──► audit-reference-accessibility
                                └──► generate-example-catalogs

sync-readme-examples ──► analyze-readme-examples
check-upstream-drift                              (any time)
```

## 1. Capture the real product

The surface chooses the crawler:

- `crawl-mobile`: installed iOS/Android app through Appium.
- `crawl-desktop`: installed native app through Cua Driver.
- `crawl-web`: browser product through Weles.
- `crawl-tui`: installed terminal application inside a real PTY.
- `crawl-cli`: installed command-line application inside a real PTY.
- `crawl-docs`: documentation inventory and full text.
- `sync-readme-examples`: exact upstream README blobs.

The coordinator submits an exact source revision to a host selected through
Stado. No crawler opens a browser, simulator, TUI, CLI, or desktop application
on the coordinator workstation. TUI and CLI crawlers use isolated homes and
fixtures; their default worker environment cannot reach the selected host's
Docker, Kubernetes or user configuration. Login values are injected from
Skarbiec through Stado and referenced in artifacts only by fixture name. A
destructive journey is explored through its confirmation screen and stops
before the final commit.

`crawl-web` covers web apps, dashboards, onboarding and authentication,
app-store listings, design systems, reports, pricing pages, and landing pages.
Each catalog has its own mandatory coverage contract while Weles remains the
shared browser execution boundary. The worker waits for every Weles action and
retains sanitized results, receipts and artifact pointers instead of treating
queue acceptance as a completed crawl.

## 2. Collect imagery — `spis collect-example-images`

Acquires official imagery for third-party records, recording dimensions,
hashes, provenance, and capture method. A retained vendor video is never
recorded as a local run.

## 3. Analyze — `spis analyze-example-structures`

Deterministic panel regions, layout model, separators, density, and confidence
from every stored overview image. Runs after imagery exists.

## 4. Accessibility — `spis audit-reference-accessibility`

Audits captured references for accessibility evidence. Records move to
`complete` only when this stage has no failed or pending findings.

## 5. Verify — `spis verify-reference-evidence`

The honesty gate. Re-probes media kinds, durations, frame counts, hashes, and
provenance classes from the retained bytes, then rewrites each record to what
the files actually prove. `--apply` writes the corrections; without it the run
is a dry report.

## 6. Regenerate catalogs — `spis generate-example-catalogs`

Renders `example-catalogs.json` and the catalog pages from the records. This
is the consistency gate: it refuses to render when the index, records, and
files disagree, and it always prints measured numbers (complete/partial,
provenance mix), never intentions.

## Separate lane — the README corpus

- `spis sync-readme-examples --host <host>` refreshes the fifty verbatim
  upstream README snapshots and their blob SHAs through Stado.
- `spis analyze-readme-examples` regenerates the structural analysis
  (`readme-examples/analysis.json`) behind the README guidance.

## Upstream monitoring — `spis check-upstream-drift`

Rechecks local hashes, current upstream README blobs, and every recorded URL.
Distinguishes a dead source from an authenticated, rate-limited, or
bot-guarded one. `--strict` turns any drift into a non-zero exit for CI use.

## Rules that do not bend

1. A record exists only with its evidence: source URL, hashes, provenance
   class, retained bytes where the contract requires them.
2. A missing observation is recorded as an evidence gap, never promoted into
   prose.
3. Interpretation lives in `wisent-ai/product-guidelines`, never here.
