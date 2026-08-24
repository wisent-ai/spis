# Full per-example reference contract

A catalog is complete only when every one of its examples has an authentic, inspectable product reference. Fifty unrelated still images plus a type-level essay are not fifty full references.

Two Rust subcommands enforce this contract and share its definitions in
[`src/commands/reference_contract.rs`](src/commands/reference_contract.rs):

- `spis verify-reference-evidence` measures every record against the files beside it — ffprobe for video and animation, the asciinema header and event stream for casts, SHA-256 and byte counts for everything, and a 16x16 grayscale frame search that proves which timestamp of the motion a state frame came from. With `--apply` it writes the measurement back into the record, names every missing item in `evidence_gaps`, and sets `evidence_status` from that list alone; without `--apply` it writes nothing.
- `spis generate-example-catalogs` refuses to render a catalog whose index, records, and files disagree, and renders the measured counts instead of the intended ones. `--check` validates without writing.

`evidence_status` is therefore never an opinion: `complete` means `evidence_gaps` is empty, `partial` means the record itself lists what is still missing.

## How the motion was obtained is part of the evidence

Every motion asset carries a `provenance_class`, derived from its recorded capture method:

| Class | Meaning |
|---|---|
| `local-product-run` | the real product was installed and driven here, and the run was recorded (terminal cast, pseudo-terminal session, local application recording) |
| `local-browser-recording` | the product was driven in a browser we control on a Stado-selected host and screen-recorded |
| `upstream-owner-media` | a recording the product's owner published — official site media, documentation asset, store preview, publisher video |
| `unclassified` | the capture method does not say; this is a gap, never a pass |

The first three are admissible evidence, in that order of strength. What is not admissible is calling the third one the first: a page that says "authentic local motion" about downloaded vendor media is wrong even when every file it points at is real.

## Per-example completion floor

Every example must have:

1. **Motion evidence:** at least one local animated asset — GIF, animated WebP, MP4, WebM, or terminal cast — showing the real product, measured at 0.2 seconds or longer with at least two frames. A still image recorded as motion, a link to a static page, prose describing motion, or animation synthesized from stills does not qualify. `media_kind` is one of `video-mp4`, `video-webm`, `animated-gif`, `animated-webp`, `terminal-cast`, and it comes from the container, not from the filename and not from what the author typed.
2. **Onboarding or first-success sequence:** the actual product-native onboarding when one exists; otherwise the real first-run path from launch through the first meaningful result. The journey must expose at least five ordered observable steps.
3. **State visuals:** at least three distinct product states, retained as local frames or screenshots and tied to the motion source or recording.
4. **Interaction map:** primary input, focus/selection, navigation, confirmation, cancellation/backtracking, feedback, failure, and recovery as actually observed.
5. **User journey:** actor, prerequisite, ordered user actions, system responses, intermediate states, failure route, recovery route, and completion evidence.
6. **Motion analysis:** trigger, start and end state, continuity, timing class, interruption or reversal, feedback, and reduced-motion or nonanimated equivalent — each observed or explicitly null.
7. **Accessibility:** at least three observations made from the retained evidence, and every unknown named. `accessibility.measured` is true only when the product itself was driven and audited — a browser audit on the live product, or a real local run exercised with and without color. Observations read off a vendor's recording are observations, not a measurement, and the record keeps `accessibility never measured against the product` among its gaps until the audit exists.
8. **Provenance:** product and source URL, original media URL or recording environment, capture method, provenance class, capture time, dimensions, measured duration and frame count, byte size, SHA-256, and upstream ownership.
9. **Truthful evidence status:** inaccessible, authentication-gated, unavailable, or missing evidence keeps the record `partial` with the gap named. Inference and a marketing claim cannot be promoted to an observation.

## Accepted evidence

Preferred order:

1. a real run of the product captured here, in an isolated environment (`local-product-run`);
2. the product driven in a browser we control and screen-recorded (`local-browser-recording`);
3. an official product recording or animated asset published by its owner (`upstream-owner-media`);
4. an official documentation recording or animated asset;
5. an official multi-screen sequence only when the product has no motion surface, accompanied by a real interaction recording.

Unofficial compilations, generated mockups, interpolated animation, and one static screenshot are not accepted. Every accepted asset records which of these it is.

## Storage contract

Each catalog owns:

- `references/<NN-slug>/README.md` — the human-readable observed reference;
- `references/<NN-slug>/reference.json` — structured journey, interaction, motion, and provenance data;
- `references/<NN-slug>/media/` — local authentic motion plus key states;
- `references.json` — the record index, carrying per-record `evidence_status` and `evidence_gap_count`, the measured `complete_count` and `partial_count`, and the measured provenance mix;
- `full-reference.md` — synthesis derived from the records, never a substitute for them.

## Reference record

Every `reference.json` records:

- `schema` (`wisent.full-product-reference.v2`), `name`, `product_url`, `evidence_status`, and `evidence_gaps`;
- `motion[]` with local path, source URL, canonical media kind, provenance class, dimensions, measured duration and frame count, bytes, SHA-256, capture method, and the measurement method that produced those numbers;
- `motion_provenance` — the set of provenance classes present, so a reader sees at a glance whether anything here was driven by us;
- `states[]` with local path, state name, source relationship, and — where the frame was located inside the motion — `source_match` with the proven timestamp and the pixel distance that proves it;
- `interactions[]` with name, trigger, response, feedback, cancellation, failure, recovery, and evidence;
- `journey` with actor, goal, prerequisites, ordered observed steps, failure route, recovery route, and completion evidence;
- `motion_analysis` with trigger, start and end state, continuity, timing class, interruption or reversal, feedback, and reduced-motion equivalent — null where the asset genuinely does not show it;
- `accessibility` observations, unknowns, and whether it was measured;
- `captured_at`, `measured_at`, and upstream owner.

## Type-level completion

A type is complete only when:

- every example satisfies the per-example floor;
- all local media resolve and match their recorded hashes (`spis check-upstream-drift` re-verifies this, and also reports which upstream sources have since moved or died);
- every journey and motion statement points to observable evidence;
- the type synthesis cites the records and identifies recurring patterns, disagreements, and applicability boundaries;
- an example with missing evidence keeps the type partial, with the shortfall visible in `references.json` rather than described in prose.

## Acceptance check

Choose any numbered example, open its directory without network access, play the motion asset, follow the five-or-more-state first-success sequence, and trace every interaction and recovery claim to that evidence. If that cannot be done, the reference is incomplete.

## Family: landing pages

A landing-page reference records one marketing page as a *measured surface*,
not as a picture. The 2026-08-21 removal of the previous landing catalog is the
precedent: fifty first-viewport screenshots read once by a model, with counts
quoted from a single pass, were not evidence and were deleted rather than
repaired.

A landing record must have:

1. **Captures at all three review widths** — 390 × 844, 768 × 1024, and
   1440 × 1000 — taken in the same session through Weles on a Stado-selected
   host (`local-browser-recording`), or, when the owner publishes a complete,
   official responsive set, `upstream-owner-media` with every width present.
2. **DOM snapshot with computed styles** for each width. Line counts, colors,
   type sizes, and element boundaries are read from the page, never inferred
   from a picture. A screenshot without the DOM is a gap named in the record.
3. **First-viewport observations** limited to fields that survive a two-pass
   reliability check (agreement ≥ 70%). Headline line counts and visual-crop
   judgments are not recordable fields.
4. **Section inventory below the fold**, each section named with its evidence
   source; sections nobody captured stay in `evidence_gaps`, never in prose.
5. **Motion evidence** per the general floor when the page animates anything.
6. **Provenance and hashes** per the general floor.

What a landing record deliberately does not contain: quality verdicts, taste
rankings, or conversion claims. Spis measures what is on the page; whether it
is good is an interpretation that lives in `product-guidelines`.
