# Concept: reference record

A **reference record** is the evidence package for one selected product. It lives at `CATALOG/references/NN-slug/reference.json` and uses schema `wisent.full-product-reference.v2`. The catalog's `sources.json` provides the corresponding selection/overview entry; `references.json` indexes the record.

## Record fields

| Field | Contract |
|---|---|
| `schema` | exactly `wisent.full-product-reference.v2` |
| `name`, `product_url`, `upstream_owner` | identity and attributable source |
| `captured_at`, `measured_at` | capture date and measurement timestamp |
| `evidence_status` | `partial` or `complete`; must agree with `evidence_gaps` |
| `motion` | measured retained motion assets and their hashes/provenance |
| `states` | retained state images with dimensions, bytes, and hashes |
| `interactions` | trigger, response, feedback, cancellation, failure, recovery, and evidence |
| `journey` | actor, goal, prerequisites, ordered steps, failure/recovery routes, completion evidence |
| `motion_analysis` | transition semantics, continuity, timing class, interruption, feedback, reduced-motion equivalent |
| `accessibility` | whether measured, observations, and unknowns |
| `motion_provenance` | the provenance classes actually present in `motion` |
| `evidence_gaps` | every missing or contradictory observation, stated explicitly |

## Evidence asset

A motion asset names a local path and source URL, canonical media kind, byte count, SHA-256, capture method, provenance class, `measured` flag, and duration. Raster/video assets also carry dimensions. Canonical motion kinds are `animated-gif`, `animated-webp`, `terminal-cast`, `video-mp4`, and `video-webm`.

Provenance classes are:

- `local-product-run` — a real locally installed product, including PTY casts;
- `local-browser-recording` — browser work driven locally through the capture path;
- `upstream-owner-media` — attributable media published by the owner;
- `unclassified` — allowed as an intermediate vocabulary value but rejected for measured evidence used by the catalog gate.

## Completeness floor

A complete record needs at least one measured motion asset of at least 0.2 seconds, three state images, eight mapped interactions, five ordered journey steps, a complete motion analysis, and at least three accessibility observations. Every required field must be present and local evidence must match its recorded bytes and digest. A partial record may omit these observations only when the omission is preserved in `evidence_gaps`.

## Record commands

```bash
spis reference-record add <catalog> \
  --name <name> --source-url <url> --category <category> \
  --selection-note <text> --visual <file> [--owner <url>]
spis reference-record get <catalog> <NN|slug>
spis reference-record remove <catalog> <NN|slug> [--force]
```

`add` copies the provided visual, probes dimensions, calculates bytes/SHA-256, creates an explicitly partial scaffold, and assigns the next number. `get` emits one JSON object containing `example`, index `entry`, and `record`. `remove` renumbers later entries and refuses to destroy motion or journey evidence unless `--force` is present.

After a mutation, run `spis generate-example-catalogs` explicitly until the known regeneration cutover defect is fixed; see the [record walkthrough](../walkthrough-record-lifecycle.md).
