# Walkthrough: a reference record lifecycle

This walkthrough was executed on 2026-08-24 with the release-built Rust binary in an isolated temporary directory. It used a generated 4×4, 78-byte PNG and `https://example.com/product`; it did not touch the repository catalogs or the network.

## 1. Create a catalog

```bash
spis catalog-type add demo --title "Demo widgets" \
  --description "A scratch catalog for the walkthrough."
```

Observed:

```text
added demo-examples (Demo widgets); scaffolded with zero records
(exit 0)
```

The command created `demo-examples/sources.json`, `references.json`, `references/`, a generated catalog README, and root example-catalog indexes.

## 2. Add a visual scaffold

```bash
spis reference-record add demo-examples \
  --name "Sample Product" \
  --source-url https://example.com/product \
  --category "demo tool" \
  --selection-note "smallest possible record for the walkthrough" \
  --visual sample.png
```

Observed:

```text
error: index regeneration refused the change:
python3: can't open file '<temporary-directory>/generate-example-catalogs.py': [Errno 2] No such file or directory
(exit 1)
```

Despite exit 1, inspection showed the intended image, source entry, record directory, partial `reference.json`, and updated `references.json`. The failure occurred after mutation in the obsolete Python regeneration helper. Repeating `add` would have been wrong.

## 3. Reconcile through the maintained generator

```bash
spis generate-example-catalogs
spis generate-example-catalogs --check
```

Observed:

```text
(exit 0)

demo-examples: 0 complete, 1 partial, no measured motion
(exit 0)
```

The partial state is honest: the scaffold has an attributable visual but no journey, motion, interaction, or accessibility evidence.

## 4. Read the joined view

```bash
spis reference-record get demo-examples 1
```

The JSON returned `example`, `entry`, and `record`. Measured visual facts included:

```json
{
  "local_path": "images/01-sample-product.png",
  "capture_kind": "provided-file",
  "format": "png",
  "width": 4,
  "height": 4,
  "bytes": 78,
  "sha256": "641bd7067faa77f5042705e6c36b61303e26f6a1cf48d281a5401aa2f6baf230"
}
```

The index entry was `partial` with seven scaffold gaps. The record named missing motion, first-success sequence, state floor, interactions, journey, motion analysis, and product accessibility measurement.

## 5. Dry-measure the record

```bash
spis verify-reference-evidence --catalog demo-examples
```

Observed:

```text
dry run: records are measured and reported, nothing is written
demo-examples: 0/1 complete

measured 1 records, 0 complete, 1 partial
    1  fewer than 3 states
    1  journey exposes fewer than 5 observed steps
    1  journey missing actor
    1  journey missing goal
    1  journey missing prerequisites
    1  journey missing failure_route
    1  journey missing recovery_route
    1  journey missing completion_evidence
    1  fewer than 8 mapped interactions
    1  motion analysis absent
    1  fewer than three accessibility observations
    1  accessibility never measured against the product
    1  no measured motion evidence
(exit 0)
```

Exit 0 means the dry measurement completed; it does not mean the record is complete.

## 6. Remove the scaffold

```bash
spis reference-record remove demo-examples 1
```

The same obsolete regeneration helper returned exit 1 after removal. Inspection proved that `references.json` already contained `reference_count: 0` and an empty `references` array. The correct recovery is another explicit `spis generate-example-catalogs`, not a repeated remove.

## Lessons

- Mutating record commands are not transactional across their final regeneration step.
- Inspect the source files after a regeneration refusal.
- The explicit Rust generator reconciles generated views.
- A passing consistency check can correctly describe a partial record.
