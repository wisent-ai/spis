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

`weles-bridge/spis-weles-bridge.mjs` is the only supported Node boundary for
Spis submission, exact status reads, cancellation, and receipt verification.
It loads the pinned official `WelesClient` and `verifyReceipt` implementation
from `weles-bridge/vendor/weles-client/index.mjs`. `UPSTREAM.json` records the
upstream commit, source digest, license, and license digest. The bridge rejects
a symlink/non-file, opens one inode with no-follow where available, checks that
inode against the pre-open identity, hashes its bytes, and imports those exact
verified bytes through a data URL. The module loader never reopens the path.

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

Every operation this repository runs — `submit`, `get`, `cancel` and `verify` —
goes through one invoker, `weles_provenance::run_bridge_command`. The operation
is carried by the command document, and the invocation names only the validated
trust, the working directory, the output destination, whether the protected
config is in play, and the wall-clock budget. Rust opens and validates the
canonical trust document itself, then passes its already-read public bytes, its
canonical path, a minimal `PATH`, and — for the three network operations — the
config path, with no inherited environment. `NODE_OPTIONS`, `NODE_PATH`,
network credentials, caller-selected trust, and a second trust-file read are
excluded; the secretless `verify` path is handed no config at all and so cannot
reach the network. The command travels on the child's stdin, so no bridge
command is ever left on disk. Child stdin/stdout/stderr are bounded. The bridge
SHA-256 pin is derived at build time by `build.rs` from
`weles-bridge/spis-weles-bridge.mjs` in the source tree, so the compiled
constant cannot drift from the checked-in bridge. Rust lstat-checks and hashes
the bridge against that pin, then executes those verified bytes from a data URL
in a new process group rather than reopening the path. The whole group is killed
and drained after 30 seconds for local verification and 60 seconds for a network
round trip.

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
`product_url`, nonempty `objective`, `constraints`, and the full `spisBinding`.
`constraints` is not a Spis vocabulary: the service admits only the exact typed
browser-evidence withholding policy array it enforces while capturing, in that
order, so the worker refuses to submit unless the immutable runtime constraints
request exactly that withholding and then submits the policy verbatim. The
binding schema is
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
idempotency key and service-added execution constraints. Canonicalization is
the shared RFC 8785/JCS subset: keys sort by UTF-16 code units, arrays preserve
order, and strings use JSON escaping and reject lone surrogates. The result is
compact UTF-8 JSON with no newline.

Numbers have one contract for both layers, stated on the parsed JSON value and
never on the incoming spelling. A number canonicalizes only when it is an
integer whose magnitude is at most 2^53-1, and it always canonicalizes to its
shortest integer text. Fractional and out-of-range numbers are rejected. The
spelling is therefore not significant: `1`, `1.0`, and `-0.0` are the same
document, canonicalizing to `1`, `1`, and `0` on both sides. This is the
resolvable direction, because the JSON parsers on both sides discard the
original token — `JSON.parse("1.0")` returns the JS number `1`, and serde_json
without `arbitrary_precision` returns the double `1.0` — so a raw-text rule
could not be enforced on values that either layer constructs itself. Rust
accordingly canonicalizes an integral double to the same integer text the
bridge emits, instead of rejecting it as floating-point. Submit retains the
complete request as `requestDocument` and requires the service-returned
`requestIdentity` to contain the same digest and binding.

`get` and `cancel` require exact known task identity and service identity. Every
response must include the server-derived service identity and request identity.
`queued`, `leased`, `running`, and `pending_review` are nonterminal. Terminal
statuses normalize to the typed outcome; `completed` and `succeeded` normalize
to `completed`. A terminal response must contain a fresh official receipt,
`resultDigest`, and the same signed request identity. Empty `artifactRefs` and
empty anonymous `credentialRefs` are valid; nonempty arrays reject empty or
duplicate entries.

### Receipt-bound evidence and attempt envelope

Terminal receipts contain exactly the core task claims, `requestDigest`,
`resultDigest`, `spisBinding`, key ID, signature, and signed payload. The
displayed extended claims must equal the freshly verified signed claims and are
included in the provenance ID. The retained evidence manifest is
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
stado://spis-crawls/runs/{run_id}/{catalog}/{record}/{record_key}/attempts/{attempt}/{attempt_id}
```

Portable components use `[A-Za-z0-9._-]+` and are neither `.` nor `..`;
`record_key` is lowercase 64-hex and attempt is a positive `u32`. Signed
pre-submit binding URIs are exactly `{base}/artifacts.tar.gz` and
`{base}/worker-output.log`.

Every producer and every verifier derives that base through
`crawl_attempt_base_uri` in `src/lib.rs`, so the two sides of a digest
comparison can never spell it differently. The fixed `runs/` segment exists so
Stado can authorize the namespace by prefix: object policies match a prefix
only when it ends in `/`, so a key beginning with a per-run identifier could be
granted by nothing narrower than the whole namespace. Everything a crawl
publishes therefore lives in one namespace, `spis-crawls`, under two named
roots — `runs/` for attempt trees and `inputs/` for the digest-addressed
runtime-bindings document — each granted exactly `get`, `put` and `stat`. One
namespace, not two, because a caller sends exactly one bearer per request and
the service compares it against the credential item of the namespace being
addressed.

Neither the shape of those components nor the shape of the URIs is the binding
contract by itself. `record_key` and `attempt_id` are derived values, and both
verification layers re-derive them exactly as the Weles public admission
runtime does. All inputs come from the binding itself; `\0` is a single NUL
byte and every digest is lowercase hex SHA-256 over UTF-8:

```
catalog_key = sha256(source_revision \0 run_id \0 catalog)
record_key  = sha256(catalog_key \0 record \0 source_input_sha256)
attempt_id  = "attempt-" attempt "-" sha256(record_key \0 attempt \0 service.host)[0:16]
```

`attempt` is rendered as its decimal integer text and the attempt fingerprint
is the first 16 characters of the hex digest. The bridge and the Rust verifier
each reject a binding whose `record_key` or `attempt_id` is not this exact
derivation, so neither side can accept a weaker attempt binding than the
runtime issues. Post-submit coordinates are distinct:

- official evidence manifest:
  `stado://weles/recordings/{weles_task_id}/evidence-manifest.json`
- artifact document:
  `{base}/weles/artifacts/{artifact_document_sha256}.json`
- observation document:
  `{base}/weles/observations/{observation_document_sha256}.json`

Digest path components are bare lowercase SHA-256; signed request/result claims
retain the `sha256:` prefix. No pre-submit URI is projected onto a post-submit
artifact or observation coordinate.

Import merges exactly the `weles/` and `recordings/` subtrees of an attempt into
the shared record directory, and refuses any retained name that already holds
different bytes, so every object in those two subtrees is addressed by its own
content or by its Weles task. The role-named operational documents of an attempt
— submission, task status, cancellation, official provenance, attempt envelope
and failure diagnostic — differ per attempt by construction and therefore stay in
the attempt root, reaching the record only under `crawl/{attempt_id}`.

Only `verify` creates `wisent.spis-weles-provenance.v1`. It needs no network
secret. `artifact.bytes` is required, positive, and at most 4 MiB. The bridge
re-runs the official verifier, requires every signed claim and the typed
evidence manifest to match, hashes the manifest itself, and requires
`claims.evidenceDigest == artifact.sha256`. Rust independently enforces the
artifact size before hashing, then repeats trust, claim, JCS request digest,
URL, envelope, manifest, inventory, and retained-byte checks.

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
