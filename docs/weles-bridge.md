# Official Weles bridge and receipt provenance

How Spis reaches Weles through its one checked-in bridge, and how every receipt is bound to it.
Moved here from the [README](../README.md), which keeps the build, the crawlers, the layout and
the rules.


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

## Network configuration and public trust

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

## Public service and request identity

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

## Receipt-bound evidence and attempt envelope

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

