# Example: isolated record lifecycle

Run this with the built `spis` binary and a real local PNG from a temporary working directory. The retained execution used a 4×4 PNG named `sample.png`.

```bash
spis catalog-type add demo --title "Demo widgets" \
  --description "A scratch catalog for the walkthrough."

spis reference-record add demo-examples \
  --name "Sample Product" \
  --source-url https://example.com/product \
  --category "demo tool" \
  --selection-note "smallest possible record for the walkthrough" \
  --visual sample.png
```

At the captured revision, the second command writes the record and then exits 1 because its final helper looks for removed `generate-example-catalogs.py`. Do not repeat `add`. Reconcile and inspect:

```bash
spis generate-example-catalogs
spis generate-example-catalogs --check
spis reference-record get demo-examples 1
spis verify-reference-evidence --catalog demo-examples
```

Expected checkpoints from the run:

```text
demo-examples: 0 complete, 1 partial, no measured motion

dry run: records are measured and reported, nothing is written
demo-examples: 0/1 complete
```

Remove only if this is an isolated scratch catalog:

```bash
spis reference-record remove demo-examples 1
spis generate-example-catalogs
spis catalog-type remove demo
```

See the [full transcript and interpretation](../docs/walkthrough-record-lifecycle.md).
