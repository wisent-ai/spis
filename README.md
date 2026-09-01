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
Spis submission, exact status reads, cancellation, and receipt verification.
It loads the pinned official `WelesClient` and `verifyReceipt` implementation
from `weles-bridge/vendor/weles-client/index.mjs`. `UPSTREAM.json` records the
upstream commit, source digest, license, and license digest; the bridge verifies
the vendored source SHA-256 before loading it.

Commands use `wisent.spis-weles-bridge-command.v1`. Only `--input` and
`--output` are accepted. Submit requires durable file output. An existing
submission is reusable only when its complete canonical request, request
identity, public service identity, and idempotency key are identical. Other
operations may use bounded stdout; `get` must use stdout so Rust can persist
each poll at a new content-addressed immutable observation path. File output is
an atomic create with fsync; identical bytes are a no-op and different bytes at
the same path fail with `output-conflict`.

### Network configuration and public trust

Network authorization and public receipt trust are separate:

- `SPIS_WELES_CONFIG_FILE` is required only for `submit`, `get`, and `cancel`.
  It must be an owner-only, mode-`0600`, regular non-symlink file with schema
  `wisent.spis-weles-bridge-config.v1` and exactly `endpoint`, `bearer`, and
  `organizationId`. The endpoint is the canonical exact `/api/v1` base. There
  is no environment-JSON or per-field credential fallback.
- Every operation requires `SPIS_WELES_TRUST_FILE` to resolve to the one
  repository-controlled `weles-bridge/weles-receipt-trust.json`. Its schema is
  `wisent.spis-weles-receipt-trust.v1` and it contains exactly the public
  organization ID, `allowedAction`, receipt public keys, and key-set version.
  The repository intentionally carries no placeholder: onboarding must commit
  the real public trust before verification can succeed.

Rust computes the canonical trust path itself and passes only that path, a
minimal `PATH`, and no inherited environment to the verification child.
`NODE_OPTIONS`, `NODE_PATH`, network credentials, and caller-selected trust are
not inherited. Child stdin/stdout/stderr are bounded and the process is killed
after 30 seconds.

### Public service and request identity

Network commands carry an exact public `serviceIdentity`; service responses are
authoritative and must repeat it field-for-field:

```json
{
  "name": "weles-admission",
  "generation": 7,
  "consumer": "spis",
  "capability": "browser-evidence",
  "active_host": "worker.example",
  "endpoint": "https://worker.example/api/v1",
  "action": "generic_browser_task",
  "release_id": "weles-worker@0.5.56",
  "source_revision": "full-lowercase-40-hex-source-revision"
}
```

Core derives generation/host/endpoint from the service directory and derives
release/source identity from the checked `/api/v1/version` readback. The bridge
does not accept those values as unverified caller decoration. The signed
`spisBinding.service` repeats the corresponding name, consumer, capability,
`directory_generation`, host, endpoint, action, `release_id`, and
`source_revision`.

The browser request is anonymous and exact: `credentialRefs` is `[]`,
`evidencePolicy` is `full`, action is `generic_browser_task`, and origin equals
the canonical `input.product_url` origin. Input contains exactly canonical
`product_url`, nonempty `objective`, unique nonempty `constraints`, and the full
`spisBinding`. The binding schema is
`weles.spis-browser-evidence-binding.v1`; it binds run/catalog/record/record key,
attempt number and ID, full Spis source revision, source-input/reference
digests, immutable attempt artifact/output URIs, and the public service
identity.

The official request digest is:

```
sha256:<SHA-256(canonical JSON UTF-8 of the exact weles.task.current body)>
```

That body contains schema, organization, origin, action, exact input,
credential references, evidence policy, and justification. It excludes the
idempotency key and service-added execution constraints. Canonical JSON sorts
object keys recursively, preserves array order, emits compact JSON, and adds no
newline. Submit retains the complete request as `requestDocument` and requires
the service-returned `requestIdentity` to contain the same digest and binding.

`get` and `cancel` require exact known task identity and service identity. Every
response must include the server-derived service identity and request identity.
`queued`, `leased`, `running`, and `pending_review` are nonterminal. Terminal
statuses normalize to the typed outcome; `completed` and `succeeded` normalize
to `completed`. A terminal response must contain a fresh official receipt,
`resultDigest`, and the same signed request identity. Empty `artifactRefs` and
empty anonymous `credentialRefs` are valid; nonempty arrays reject empty or
duplicate entries.

### Receipt-bound evidence and attempt envelope

Terminal receipts sign the core task claims plus `requestDigest`,
`resultDigest`, and `spisBinding`. The retained evidence manifest is
`weles.browser-evidence-manifest.v1` and carries exact task/organization/origin/
action/completed outcome, request/result digests, binding, requested/effective/
final URLs, and `evidenceInventory`. Requested URL must equal the canonical
current record `product_url`; effective and final URLs must remain same-origin.
Inventory entries contain exact `kind`, immutable Weles recording `uri`,
lowercase SHA-256, and positive byte count. Required entries are:

- `screenshot` at
  `stado://weles/recordings/{taskId}/artifacts/browser_evidence_final.png`
- `accessibility_tree` at
  `stado://weles/recordings/{taskId}/artifacts/browser_evidence_accessibility_tree.txt`

Additional entries use unique `artifact:{relative-path}` kinds and the matching
`stado://weles/recordings/{taskId}/{relative-path}` URI. Spis retains them under
`recordings/{taskId}/...`; Rust reopens each file with a limit-plus-one reader
and verifies bytes and SHA-256. The serialized evidence manifest is at most
4 MiB and the retained inventory is at most 8 MiB total.

Each imported crawl run contains one
`wisent.spis-weles-attempt-envelope.v1`. Outer Stado `stado_job_id` is distinct
from inner `weles_task_id`; a receipt task ID is always the inner ID. The
envelope retains the exact `spis_binding`, canonical official
`weles_request_document`, request/result digests, requested/final URLs,
inventory, service identity, and every source/reference/post-submit coordinate.
Rust compares the envelope to the selected typed crawl run field-for-field,
including attempt/state/outcome and every URI/digest.

The canonical attempt base is:

```
stado://spis-crawls/{run_id}/{catalog}/{record}/{record_key}/attempts/{attempt}/{attempt_id}
```

Portable components use `[A-Za-z0-9._-]+` and are neither `.` nor `..`;
`record_key` is lowercase 64-hex and attempt is a positive `u32`. Signed
pre-submit binding URIs are exactly `{base}/artifacts.tar.gz` and
`{base}/worker-output.log`. Post-submit coordinates are distinct:

- official evidence manifest:
  `stado://weles/recordings/{weles_task_id}/evidence-manifest.json`
- artifact document:
  `{base}/weles/artifacts/{artifact_document_sha256}.json`
- observation document:
  `{base}/weles/observations/{observation_document_sha256}.json`

Digest path components are bare lowercase SHA-256; signed request/result claims
retain the `sha256:` prefix. No pre-submit URI is projected onto a post-submit
artifact or observation coordinate.

Only `verify` creates `wisent.spis-weles-provenance.v1`. It needs no network
secret. The bridge re-runs the official verifier, requires every signed claim
and the typed evidence manifest to match, hashes the manifest itself, and
requires `claims.evidenceDigest == artifact.sha256`. Rust then repeats trust,
claim, request digest, URL, envelope, manifest, inventory, and retained-byte
checks independently.

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
