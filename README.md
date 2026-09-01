# Spis

**Spis** is the evidence-grade reference corpus and corpus-maintenance CLI for people building interfaces. It holds measured records across 15 interface families; exact counts are generated into [`catalog-stats.json`](catalog-stats.json), never maintained in this prose. Every record ties claims to retained bytes, source URLs, hashes, provenance, observed states, interactions, journeys, motion, and accessibility evidence.

Spis owns the corpus data and the machinery that acquires, measures, validates, searches, and monitors it. Interpretation and prescriptive guidance belong in [`wisent-ai/product-guidelines`](https://github.com/wisent-ai/product-guidelines). Own-product captures and operational monitoring metadata may live in a private companion repository and are not published here.

Licensed Apache-2.0. Third-party content remains attributable to its owners; see [the takedown policy](https://spis.wisent.com/docs/takedown).

## Start here

- [Quick start](https://spis.wisent.com/docs/quick-start)
- [Command-line reference](https://spis.wisent.com/docs/cli)
- [Reference pipeline](https://spis.wisent.com/docs/pipeline)
- [Configuration](https://spis.wisent.com/docs/configuration)
- [Architecture](https://spis.wisent.com/docs/architecture)
- [Runbook](https://spis.wisent.com/docs/runbook)
- [Examples and executed walkthroughs](https://spis.wisent.com/docs/examples)

Core concepts:

- [Catalog](https://spis.wisent.com/docs/concept-catalog)
- [Reference record](https://spis.wisent.com/docs/concept-reference-record)
- [Evidence and completeness](https://spis.wisent.com/docs/concept-evidence)
- [Crawled documentation corpus](https://spis.wisent.com/docs/concept-docs-corpus)

## Build and invoke

The maintained implementation is the Rust binary in `src/`:

```bash
cargo build --release
./target/release/spis --help
./target/release/spis generate-example-catalogs --check
```

`scripts/build-release.sh` builds and ships that same binary, so the release archive and the `stado-release` install are the Rust command surface. The checked-in `bin/spis` is not: it is the retired Python dispatcher, and every script it dispatches to was deleted in the Rust rewrite. It stays only because `.wisent-release.json` reads the release version out of it, so do not use it as the authority for the current command surface. See the [runbook](https://spis.wisent.com/docs/runbook#rustpython-cutover-mismatch) for the remaining cutover mismatches.

## Real product crawlers

Every crawler that opens a product runs as an exact-revision job on a host explicitly selected through Stado; the coordinator never opens a local browser, simulator, terminal application, or native application.

Use `spis crawl start` as the durable public coordinator. It preflights every selected family before the first submission, resolves engine placement through Stado, retains the exact argv and job IDs, and exposes `status`, `resume`, and `import`. Terminal successful jobs are downloaded and imported idempotently; verifier/apply and the catalog generator run after import. The surface-specific commands below are execution engines.

| Product surface | Command | Real execution boundary |
|---|---|---|
| iOS applications | `spis crawl-mobile ios-app-examples --host <host>` | installed app via Appium and XCUITest |
| Android applications | `spis crawl-mobile android-app-examples --host <host>` | installed app via Appium and UiAutomator2 |
| macOS applications | `spis crawl-desktop macos-app-examples --host <host>` | installed app via Cua Driver |
| Cross-platform desktop applications | `spis crawl-desktop desktop-app-examples --host <host>` | installed app via Cua Driver |
| Web applications | `spis crawl-web web-app-examples --host <host> --admission-url <url>` | signed-in product via Weles |
| Dashboards and consoles | `spis crawl-web dashboard-console-examples --host <host> --admission-url <url>` | signed-in console via Weles |
| Terminal applications | `spis crawl-tui --host <host>` | installed app in an isolated real tmux PTY |
| Command-line applications | `spis crawl-cli --host <host>` | installed binary in an isolated real tmux PTY |
| Onboarding and authentication | `spis crawl-web onboarding-auth-examples --host <host> --admission-url <url>` | account-bound journey via Weles |
| Documentation sites | `spis crawl-docs --all --host <host>` | bounded HTTP crawl on Stado |
| App-store listings | `spis crawl-web app-store-listing-examples --host <host> --admission-url <url>` | live store listing via Weles |
| Design systems | `spis crawl-web design-system-examples --host <host> --admission-url <url>` | live docs and component explorer via Weles |
| Reports and evidence | `spis crawl-web report-evidence-examples --host <host> --admission-url <url>` | interactive report via Weles |
| Pricing pages | `spis crawl-web pricing-page-examples --host <host> --admission-url <url>` | live plan-selection surface via Weles |
| Landing pages | `spis crawl-web landing-page-examples --host <host> --admission-url <url>` | live responsive page via Weles |

Mobile and desktop crawlers accept fixture files whose values can come from environment variables. `--secret-env NAME=SKARBIEC_ITEM` asks Stado to inject those values from Skarbiec without placing credentials in a command line or artifact. CLI crawls accept declared non-destructive journeys; Weles account bindings select an existing product identity. Weles crawls wait for every queued action and retain the sanitized job result, receipt, and artifact pointers. Destructive paths stop at the final confirmation and retain that state without committing it.

## Repository layout

| Path | Owns |
|---|---|
| `src/commands/` | Rust implementations of acquisition, measurement, validation, query, and monitoring commands |
| `src/commands/crawl_mobile.rs` | real iOS and Android application state-graph crawler |
| `src/commands/crawl_desktop.rs` | real macOS and desktop application state-graph crawler |
| `src/commands/crawl_web.rs` | Weles plan builder, completion wait, and Stado coordinator for browser products |
| `src/commands/crawl_tui.rs` | terminal-application PTY crawler |
| `src/commands/crawl_cli.rs` | recursive CLI command and journey crawler |
| `src/commands/crawl_docs.rs` | documentation inventory and full-text crawler |
| `https://spis.wisent.com/docs` | product, contributor, evidence-contract, and operations documentation; source lives in `wisent-ai/spis-landing` |
| `example-catalogs.json` | generated cross-catalog index |
| `*-examples/sources.json` | selected examples and their visual/structure metadata |
| `*-examples/references.json` | generated per-catalog reference index |
| `*-examples/references/*/reference.json` | evidence record for one product |
| `readme-examples/` | curated README source metadata and measured records |
| `documentation-site-examples/content-structure/` | documentation-site inventory definitions |

## Non-negotiable rules

1. A missing observation is an evidence gap, never inferred prose.
2. Retained bytes, their measured hashes, and their provenance must agree with metadata.
3. Own-product captures run the real installed product in a pseudo-terminal; browser captures go through Weles on a Stado-selected host.
4. Generated indexes are regenerated from records, not hand-edited.
5. A `partial` record is useful but is not silently presented as `complete`.
6. Third-party content is referenced, never claimed.

## Status

The landing-page and pricing-page reference sets are intentionally empty while their attributable Weles recaptures are rebuilt. The former landing set and the auto-discovered Stripe capital/connect/atlas/customers records were not family evidence and were removed rather than preserved beside valid work. `spis crawl start` now rejects an empty family and rejects pricing/landing category or URL mismatches before submitting any job.
