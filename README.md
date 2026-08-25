# Spis

**Spis** — the evidence-grade reference corpus for people building interfaces.
655 records across 15 interface families, every record with its source, hashes,
provenance class, and measured state. Where other libraries show you screenshots,
Spis shows you what can be proven about them.

Licensed Apache-2.0. Own-product captures and operational monitoring metadata
live in a private companion repository and are intentionally not here.

Guidelines that interpret this data live in
[`wisent-ai/product-guidelines`](https://github.com/wisent-ai/product-guidelines).
This repository owns the data and the machinery that produces it.


## Command line

`spis` owns one crawler per product surface. Every crawler that opens a
product runs as an exact-revision job on a host explicitly selected through
Stado; the coordinator never opens a local browser, simulator, terminal
application, or native application.

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
| README files | `spis sync-readme-examples --host <host>` | exact GitHub source blobs on Stado |

Mobile and desktop crawlers accept fixture files whose values can come from
environment variables. `--secret-env NAME=SKARBIEC_ITEM` asks Stado to inject
those values from Skarbiec without placing credentials in a command line or
artifact. CLI crawls accept declared non-destructive journeys; Weles account
bindings select an existing product identity. Weles crawls wait for every
queued action and retain the sanitized job result, receipt and artifact
pointers. Destructive paths stop at the final confirmation and retain that
state without committing it.

The full flow is in [`docs/pipeline.md`](docs/pipeline.md).

## Layout

| Path | Owns |
|---|---|
| `full-reference-contract.md` | the evidence floor every record must meet |
| `src/commands/crawl_mobile.rs` | real iOS and Android application state-graph crawler |
| `src/commands/crawl_desktop.rs` | real macOS and desktop application state-graph crawler |
| `src/commands/crawl_web.rs` | Weles plan builder and Stado coordinator for browser products |
| `src/commands/crawl_tui.rs` | terminal-application PTY crawler |
| `src/commands/crawl_cli.rs` | recursive CLI command and journey crawler |
| `src/commands/crawl_docs.rs` | documentation inventory and full-text crawler |
| `src/commands/sync_readme_examples.rs` | README source-blob crawler |
| `example-catalogs.json` / `example-catalogs.md` | the index of all catalogs and records |
| `*-examples/` | one directory per catalog: sources, references, retained media |
| `docs/pipeline.md` | the capture → verify → catalogs flow |
| `docs/takedown.md` | content rights and takedown policy |
| `LICENSE` | Apache-2.0 |

## Credential recovery

Before declaring any credential unavailable: search transcript-lake for it,
then check .env and config files in the product repo. See
[docs/credential-recovery-rule.md](docs/credential-recovery-rule.md).

## Rules
1. A record exists only with its evidence: source URL, hashes, provenance class, and retained bytes where required by the contract.
2. A missing observation is recorded as an evidence gap, never promoted into prose.
3. Captures of real products run through the real product or through Weles on a Stado-selected host; no ad-hoc scraping.
4. Catalogs are generated, never hand-edited: change records, then run `spis generate-example-catalogs`.
5. Interpretation belongs in `product-guidelines`, not here.
6. Third-party content is referenced, never claimed; owners can remove or relink
   their media at any time via [`docs/takedown.md`](docs/takedown.md).

## Status

The landing-page reference set remains empty after its unverifiable predecessor was removed on 2026-08-21. `crawl-web landing-page-examples` is now the real-product capture path once new records satisfying `full-reference-contract.md` are added; it does not revive the discarded model-guessed observations.