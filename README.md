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

`scripts/build-release.sh` builds and ships that same binary, so the release archive and the `stado-release` install are the Rust command surface. `Cargo.toml` is the single package and release version source.

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

## Official Weles bridge and receipt provenance

`weles-bridge/spis-weles-bridge.mjs` is the only supported Node boundary for
Spis task submission, exact task-status reads, official cancellation, and
receipt verification. It loads the official `WelesClient` and `verifyReceipt`
implementation vendored at `weles-bridge/vendor/weles-client/index.mjs`; it
does not implement a second HTTP client. The vendored file is byte-for-byte
source from commit `37798a26022a040fbd0a4a4a25c99b5559d95a32`.
`UPSTREAM.json` records the repository, commit, source digest, license, and
license digest. At runtime the bridge checks the source SHA-256 before loading
it. A clean exact-revision checkout needs only Node: there is no package
install, network fetch, or SSH dependency.

Commands use schema `wisent.spis-weles-bridge-command.v1`. Pass command JSON
through stdin or `--input <file>` and persist output with `--output <file>`.
Only those two path arguments are accepted. File output is an atomic immutable
create: the bridge fsyncs bytes, hard-links a unique temporary inode into
place, and fsyncs the parent directory. Process death cannot leave a stale lock
that blocks recovery. Identical bytes are a no-op; different bytes at the same
path fail with `output-conflict`. File and stdout output share the same 4 MiB
bound. Submit requires a file because stdout is not durable storage.

Network authorization and public receipt trust are intentionally separate.
`submit`, `get`, and `cancel` require `SPIS_WELES_CONFIG_FILE`, an owner-only,
non-symlink regular file with mode `0600` and schema
`wisent.spis-weles-bridge-config.v1`. It contains the endpoint, bearer,
organization ID, exact origin/action allowlists, terminal outcomes, receipt
public keys, and key-set version. The environment-only alternative uses
`WELES_API_BASE`, `WELES_TOKEN`, `WISENT_ORGANIZATION_ID`,
`SPIS_WELES_ALLOWED_ORIGINS_JSON`, `SPIS_WELES_ALLOWED_ACTIONS_JSON`,
`SPIS_WELES_TERMINAL_OUTCOMES_JSON`, `SPIS_WELES_RECEIPT_KEYS_JSON`, and
`SPIS_WELES_KEY_SET_VERSION`.

`verify` does not read or require that secret configuration. It requires
`SPIS_WELES_TRUST_FILE`, pointing to a regular non-symlink public document with
schema `wisent.spis-weles-receipt-trust.v1`. The document contains only the
organization ID, one exact `allowedAction`, trusted key-ID/public-key mapping,
and `keySetVersion`; the exact origin is derived from the current record.
This file may be tracked and world-readable. The repository deliberately has
no placeholder trust document: verification fails closed until onboarding
produces the real public keys and core commits that document. The Rust verifier
also removes all network-auth configuration from the verification subprocess.

Submission requires only information known before submission. It never asks
for a dummy future task ID, outcome, evidence digest, or artifact:

```json
{
  "schema": "wisent.spis-weles-bridge-command.v1",
  "operation": "submit",
  "idempotencyKey": "caller-retained-operation-id",
  "request": {
    "origin": "https://console.example.com",
    "action": "generic_browser_task",
    "input": {"record": "01-example"},
    "credentialRefs": ["approved-account"],
    "evidencePolicy": "receipt",
    "justification": "Capture the approved reference."
  }
}
```

The result is `wisent.spis-weles-submission.v1` with the task identity,
idempotency key, and `requestDigest`, a SHA-256 over canonical normalized
request, organization, and idempotency input. Before loading the official
client or making a network request, submit reuses an existing output only when
that digest and every retained request identity field match; any difference is
an `output-conflict`. If Weles returns a receipt immediately,
`receiptCheckpoint` retains signed material, independently verified claims,
and key-set version. It is not terminal artifact provenance.

`get` requires `taskId` and exact `expectedTask`. Its
`wisent.spis-weles-task-status.v1` result carries only typed task identity,
`status`, `terminal`, `outcome`, exact `resultRef`/`artifactRefs`, and the
receipt checkpoint. `queued`, `running`, and `pending_review` are nonterminal
and have no outcome. Any other status is accepted only when it exactly equals
a configured terminal outcome and a freshly verified receipt claim. Terminal
status without that receipt fails closed; the raw service response is never
retained.

`cancel` uses the official `WelesClient.cancel` operation with `taskId`,
`expectedTask`, a nonsecret `reason`, and retained `idempotencyKey`. It returns
the same typed status contract under `wisent.spis-weles-cancellation.v1`, so a
Spis/Stado cancellation is coupled to the Weles task rather than abandoning it.
`get` remains the reconciliation operation if cancellation is still
nonterminal.

Only `verify` creates `wisent.spis-weles-provenance.v1`. The caller supplies
the terminal expectations and already retained JSON artifact; the bridge uses
only the public trust document, re-runs the official verifier, compares every
bound claim exactly, hashes the artifact itself, and requires
`claims.evidenceDigest == artifact.sha256`:

```json
{
  "schema": "wisent.spis-weles-bridge-command.v1",
  "operation": "verify",
  "receipt": {
    "schema": "weles.receipt.current",
    "taskId": "known-task-id",
    "organizationId": "00000000-0000-4000-8000-000000000000",
    "origin": "https://console.example.com",
    "action": "generic_browser_task",
    "outcome": "completed",
    "evidenceDigest": "lowercase-64-character-sha256",
    "keyId": "current-signing-key",
    "signature": "base64-signature",
    "signedPayload": "{\"taskId\":\"known-task-id\",\"organizationId\":\"00000000-0000-4000-8000-000000000000\",\"origin\":\"https://console.example.com\",\"action\":\"generic_browser_task\",\"outcome\":\"completed\",\"evidenceDigest\":\"lowercase-64-character-sha256\"}"
  },
  "expectedClaims": {
    "taskId": "known-task-id",
    "organizationId": "00000000-0000-4000-8000-000000000000",
    "origin": "https://console.example.com",
    "action": "generic_browser_task",
    "outcome": "completed",
    "evidenceDigest": "lowercase-64-character-sha256"
  },
  "artifact": {
    "path": "evidence/weles-result.json",
    "sha256": "lowercase-64-character-sha256",
    "bytes": 1234
  }
}
```

### Record and observation provenance

A record references bridge-produced provenance documents by retained path and
file digest:

```json
{
  "provenance_documents": [
    {
      "schema": "wisent.spis-weles-provenance-document-ref.v1",
      "path": "evidence/weles-provenance.json",
      "sha256": "lowercase-64-character-document-sha256"
    }
  ]
}
```

The record's imported `crawl_runs` entry must identify the same completed
attempt with matching `run_id`, `attempt_id`, `job_id`, and a typed
`weles_attempt`. The receipt-bound JSON artifact must repeat that object
exactly as top-level `spisBinding`:

```json
{
  "schema": "wisent.spis-weles-attempt-binding.v1",
  "catalog": "web-app-examples",
  "record": "01-example",
  "runId": "crawl-run-id",
  "attemptId": "attempt-id",
  "taskId": "known-task-id",
  "source": "https://console.example.com/product/path",
  "inputDigest": "sha256:lowercase-64-character-sha256",
  "referenceDigest": "sha256:lowercase-64-character-sha256"
}
```

Rust requires the task ID to be that imported attempt, the receipt origin to
equal the current `product_url` origin, the action to equal
`generic_browser_task`, and the signed catalog/record/run/attempt/source/input/
reference binding to match the attempt. A valid receipt for any unrelated task
therefore leaves the record raw.


Each supported record value carries its own
`wisent.spis-provenance-link.v1`; a valid record-level receipt never blesses
every observation. For a semantic observation, `artifactPointer` is an RFC
6901 pointer into the receipt-bound JSON artifact and `valueSha256` is the
SHA-256 of that member's canonical JSON:

```json
{
  "observation": "The submit control exposes an accessible name.",
  "provenance": {
    "schema": "wisent.spis-provenance-link.v1",
    "kind": "observation",
    "documentId": "sha256:derived-provenance-document-id",
    "artifactPath": "evidence/weles-result.json",
    "artifactSha256": "lowercase-64-character-sha256",
    "artifactPointer": "/observations/0",
    "valueSha256": "lowercase-64-character-canonical-json-sha256"
  }
}
```

Canonical JSON recursively sorts object keys, preserves array order, emits
compact UTF-8 JSON, and hashes those bytes. Rust reopens the signed artifact,
resolves the pointer, recomputes that digest, and requires the current value
(after recursively removing its `provenance` links) to equal the pointed-to
member. If an observation member contains `local_path` and `sha256`, Rust also
contains, reopens, and hashes that local file; a signed manifest entry alone is
not proof of the member bytes. `kind: "artifact"` is reserved for an exact
media value whose `local_path` and `sha256` equal the receipt-bound artifact
path and digest; it must omit `artifactPointer` and `valueSha256`.

Both `spis verify-reference-evidence` and catalog generation construct one
cached `VerifiedProvenanceSet::verify_record(record, reference_dir)` per record,
so validation and generated provenance statistics use the same result. Only a
supported link receives `weles-signed-browser-evidence`; capture-method prose
cannot confer that class. Every referenced document is hashed, parsed through
the typed Rust schema, sent back through the pinned official bridge, and
independently checked again in Rust. Missing vendored client source, public
trust document, trusted key, supported claim, terminal outcome, attempt
binding, artifact, pointer, member digest, or exact current-value match leaves
the value unverified and adds an evidence gap. Persisted `verified: true`,
verifier names, receipt-provided keys, persisted claim copies, and standalone
correlation JSON are never authority.

## Repository layout

| Path | Owns |
|---|---|
| `src/commands/` | Rust implementations of acquisition, measurement, validation, query, and monitoring commands |
| `src/weles_provenance.rs` | typed, fail-closed Rust verification of receipt-bound record and observation provenance |
| `weles-bridge/` | self-contained Node ESM bridge and exact vendored official Weles client source, license, and commit metadata |
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

The pricing-page and landing-page selectors now generate 50 exact official candidates per family. Their source manifests remain explicitly `pending-weles` until Weles returns a retained screenshot and machine-readable surface proof: pricing requires a visible comparison of at least two plans or prices, while landing requires the normalized observed URL to match the requested page exactly. Former family mismatches were removed rather than preserved beside valid work. `spis crawl start` validates every selected record, runs one durable host-capability preflight per engine/host, records diagnostics per record, and refuses expensive submission when its Appium/device, Cua Driver, terminal binary, Weles admission, or docs network/storage prerequisites are absent.
