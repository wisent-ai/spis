# Reference pipeline

Spis has three related lanes: interface evidence, README research, and documentation full text.

```text
catalog/record scaffold
        |
        +--> real product crawler --> retained state graph / recording
        |                               +--> analyze-example-structures
        |                               +--> audit-reference-accessibility
        |
        +--> verify-reference-evidence [--apply]
        +--> generate-example-catalogs [--check]
        +--> check-upstream-drift

sync-readme-examples --> analyze-readme-examples --> guidelines
crawl-docs --> docs-corpus {status,search,show}
```

## 1. Select and scaffold

`catalog-type` creates or edits a `*-examples` family. `reference-record add` introduces one selected product with source, category, rationale, and a measured overview image. New records are partial and list what has not been observed.

## 2. Acquire authentic evidence

The surface chooses the crawler:

- `crawl-mobile`: installed iOS/Android app through Appium.
- `crawl-desktop`: installed native app through Cua Driver.
- `crawl-web`: browser product through Weles.
- `crawl-tui`: installed terminal application inside a real PTY.
- `crawl-cli`: installed command-line application inside a real PTY.
- `crawl-docs`: documentation inventory and full text.
- `sync-readme-examples`: exact upstream README blobs.

The coordinator submits an exact source revision to a host selected through Stado. No crawler opens a browser, simulator, TUI, CLI, or desktop application on the coordinator workstation. TUI and CLI crawlers use isolated homes and fixtures; their default worker environment cannot reach the selected host's Docker, Kubernetes, or user configuration. Login values are injected from Skarbiec through Stado and referenced in artifacts only by fixture name. A destructive journey is explored through its confirmation screen and stops before the final commit.

`crawl-web` covers web apps, dashboards, onboarding and authentication, app-store listings, design systems, reports, pricing pages, and landing pages. Each catalog has its own mandatory coverage contract while Weles remains the shared browser execution boundary. The worker waits for every Weles action and retains sanitized results, receipts, and artifact pointers instead of treating queue acceptance as a completed crawl.

Owner-published material remains valid third-party evidence where the contract calls for it. `collect-example-images` acquires qualifying interface imagery; `capture-widths` and `audit-reference-accessibility` submit Weles work through Stado rather than direct SSH.

Acquisition method and provenance class must agree. Retain bytes under the record, not merely a remote URL.

## 3. Analyze — `spis analyze-example-structures`

`analyze-example-structures [catalog ...]` deterministically writes image layout regions, separators, density, and confidence into `sources.json`. It writes `structure-analysis-failures.json` while unresolved entries remain.

## 4. Accessibility — `spis audit-reference-accessibility`

`audit-reference-accessibility` plans, submits, polls, retrieves, verifies, and merges Weles axe results. Planning refusal exits 2; execution or retrieval failure exits 3. `--dry-run` stops after validating and writing the plan.


## 5. Verify — `spis verify-reference-evidence`

`verify-reference-evidence` re-probes retained media, hashes, dimensions, durations, provenance, state matches, journey/interactions, motion analysis, and accessibility. Without `--apply` it is a report only. With `--apply` it rewrites records and matching reference-index statuses to what the bytes prove.


## 6. Regenerate catalogs — `spis generate-example-catalogs`

`generate-example-catalogs --check` validates without writing. Without `--check`, the same validation precedes rendering every catalog `README.md` plus `example-catalogs.json` and `example-catalogs.md`. A partial record is valid only when its gaps are explicit and consistent.

## 7. Monitor drift

`check-upstream-drift` always verifies local media. Unless `--skip-network` is used, it also checks README blob SHAs and recorded URLs, distinguishing reachable, gone, guarded, and unresolved sources. `--strict` exits 1 if any drift exists. `--write-report` creates `upstream-drift.json`, which operational policy may keep private.

## README research lane

- `spis sync-readme-examples --host <host>` refreshes the fifty verbatim upstream README snapshots and their blob SHAs through Stado.
- `spis analyze-readme-examples` regenerates the structural analysis (`readme-examples/analysis.json`) behind the README guidance.
- `spis guidelines <catalog>` drafts counted observations for human review; interpretation remains outside Spis.

## Documentation full-text lane

`crawl-docs` resolves sitemap/override inventories, obeys robots rules, and writes resumable local archives. `docs-corpus` exposes read-only JSON status, search, and exact-page lookup.
