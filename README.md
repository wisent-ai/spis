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
./target/release/spis onboarding
./target/release/spis generate-example-catalogs --check
```

### Adopt an existing corpus

First use can begin with an unpacked canonical corpus already on disk:

```bash
./target/release/spis corpus adopt /absolute/path/to/corpus
./target/release/spis corpus status
```

`corpus adopt` validates `example-catalogs.json`, every catalog's
`sources.json` and `references.json`, and every indexed reference record before
atomically saving `~/.config/spis/corpus.json`. The corpus remains in place, so
its original provenance, screenshots, recordings, receipts, and other evidence
files remain the files subsequent commands read. Re-adopting the same path
reports every reference unchanged. Archives are refused and must be unpacked
before Spis can validate them. The desktop first-run and Manage controls call
this exact operation through the loopback API.

`scripts/build-release.sh` builds and ships that same binary, so the release archive and the `stado-release` install are the Rust command surface. `Cargo.toml` is the single package and release version source.

## Real product crawlers

Every crawler that opens a product runs as an exact-revision job on a host explicitly selected through Stado; the coordinator never opens a local browser, simulator, terminal application, or native application.

`spis crawl start` is the durable public coordinator and the only supported entry point. One record is one immutable attempt: its input digest, catalog and record keys, attempt id, correlation id, Stado run id and both `stado://` attempt URIs are derived from the record itself, and every state transition is persisted under a durable per-record lock *before* the external effect it authorizes. A record held by another process is skipped, never failed.

```
spis crawl bindings generate --weles-token-ref ITEM#FIELD --organization-ref ITEM#FIELD [--output PATH]
spis crawl start  [--host ENGINE=TARGET] [--catalog SLUG ...] [--record SLUG] [--run-id ID] [--bindings PATH]
spis crawl status [--run RUN_ID] [--record SLUG]
spis crawl cancel --run RUN_ID [--record SLUG] --reason TEXT
spis crawl resume --run RUN_ID [--record SLUG]
spis crawl import --run RUN_ID [--record SLUG]
```

`start` is idempotent: re-running the same request digest continues the existing run. `resume` never reruns a Stado job — a terminal `failed`, `cancelled`, `lost` or `submission_failed` attempt becomes attempt N+1 with a fresh execution identity and fully recomputed identity, while `queued` and `running` records are left alone and completed records are imported. `cancel` is status-first, durable and idempotent. `import` verifies the typed worker report, the retained Stado submission receipt, the attempt artifact digest and byte count and every retained evidence hash before a staged, fsynced, atomically installed record transaction; earlier attempts and their partial diagnostics are preserved. Only an exact typed `stado.submission-receipt.v3` for exactly one job is accepted, and the coordinator compares the child's reported artifact and output URIs against the values it derived itself rather than adopting them.

The surface-specific commands below are execution engines. Each takes exactly one `--record` and the immutable `--runtime-manifest-base64` that `spis crawl start` produced; they are not a second operator interface.

| Product surface | Engine command | Real execution boundary |
|---|---|---|
| iOS applications | `spis crawl-mobile ios-app-examples` | installed app via Appium and XCUITest |
| Android applications | `spis crawl-mobile android-app-examples` | installed app via Appium and UiAutomator2 |
| macOS applications | `spis crawl-desktop macos-app-examples` | installed app via Cua Driver |
| Cross-platform desktop applications | `spis crawl-desktop desktop-app-examples` | installed app via Cua Driver |
| Web applications | `spis crawl-web web-app-examples` | real browser session via the official Weles task API |
| Dashboards and consoles | `spis crawl-web dashboard-console-examples` | real browser session via the official Weles task API |
| Terminal applications | `spis crawl-tui` | installed app in an isolated real tmux PTY |
| Command-line applications | `spis crawl-cli` | installed binary in an isolated real tmux PTY |
| Onboarding and authentication | `spis crawl-web onboarding-auth-examples` | real browser session via the official Weles task API |
| Documentation sites | `spis crawl-docs` | bounded HTTP corpus crawl on Stado |
| App-store listings | `spis crawl-web app-store-listing-examples` | real browser session via the official Weles task API |
| Design systems | `spis crawl-web design-system-examples` | real browser session via the official Weles task API |
| Reports and evidence | `spis crawl-web report-evidence-examples` | real browser session via the official Weles task API |
| Pricing pages | `spis crawl-web pricing-page-examples` | real browser session via the official Weles task API |
| Landing pages | `spis crawl-web landing-page-examples` | real browser session via the official Weles task API |

`spis crawl bindings generate` writes the exact typed binding for every checked-in record; with `--output` an existing generated document is replaced atomically after validation and read-back, and the reported outcome is `created`, `replaced` or `unchanged`. `headless` is set only for the web engine. Native records without an explicit binding and an independently observed authorization proof stay explicitly unconfigured and surface one typed `unavailable` attempt diagnostic rather than disappearing from the run.

`SPIS_CRAWL_OBJECT_TOKEN_FILE` names the owner-only file holding the bearer of
the `spis-crawls` object namespace. One object request carries exactly one
bearer and Stado compares it against the credential item of the namespace being
addressed, so the coordinator needs two: this one for everything it publishes,
and the host's configured bearer for the queue plane a job submission reads and
writes. Spis injects it on exactly the invocations that address its own objects
and on no other. Unset, every call uses the configured bearer, which is correct
for a deployment where one credential covers both.

Browser crawls are anonymous: `credentialRefs` is always empty, `evidencePolicy` is always `full`, and the only secrets in play are the bearer and organization references that Stado injects into the pinned worker — the coordinator never holds either. The worker confirms the deployed Weles release against both the Stado service directory and the unauthenticated `{endpoint}/version` readback, requires the requested URL to be the exact committed `product_url` and the final URL to be same-origin, and requires a typed screenshot and accessibility-tree inventory whose signed digests match the bytes actually retained. A task that is still nonterminal when `--wait-seconds` runs out is cancelled through the bridge's `cancel` operation under an idempotency key derived from the same immutable attempt, and the typed cancellation is retained beside the status before the attempt fails, so no browser session outlives the attempt that can no longer publish evidence. Native crawls refuse first-run consent, system permission prompts, notifications, purchases and any final destructive action; destructive paths stop at the final confirmation and retain that state without committing it. Every crawler subprocess runs in its own process group under a hard timeout with capped output streams.

## Official Weles bridge and receipt provenance

The Weles bridge, its network configuration, public service identity and receipt-bound
evidence are described in [docs/weles-bridge.md](docs/weles-bridge.md).

## Repository layout

| Path | Owns |
|---|---|
| `src/commands/` | Rust implementations of acquisition, measurement, validation, query, and monitoring commands |
| `src/weles_provenance.rs` | typed, fail-closed Rust verification of receipt-bound record and observation provenance |
| `weles-bridge/` | self-contained Node ESM bridge and exact vendored official Weles client source, license, and commit metadata |
| `src/commands/crawl.rs` | durable per-record crawl coordinator: planning, runtime manifests, preflight, submission, cancellation, resumption and attempt import |
| `src/commands/crawl_mobile.rs` | real iOS and Android application state-graph crawler |
| `src/commands/crawl_desktop.rs` | real macOS and desktop application state-graph crawler |
| `src/commands/crawl_web.rs` | one-record official Weles browser-evidence task bridge and its Stado coordinator |
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

The pricing-page and landing-page selectors generate 50 exact official candidates per family. Their source manifests remain explicitly `pending-weles` until an imported browser attempt carries a signed evidence manifest whose retained screenshot and accessibility-tree bytes match the digests inside the receipt, and whose requested URL is the exact committed `product_url` with a same-origin final URL. Nothing is inferred from crawler prose: a claim is either bound to a verified receipt or it stays a declared gap. Former family mismatches were removed rather than preserved beside valid work. `spis crawl start` validates every selected record, runs one durable host-capability preflight per engine/host — including an independent `{endpoint}/api/v1/version` release confirmation for browser crawls — records one typed diagnostic per record, and refuses expensive submission when its Appium/device, Cua Driver, terminal binary, Weles admission, or docs network/storage prerequisites are absent.
