# Example: offline integrity check

Run from the repository root with the release-built Rust binary. These commands do not contact upstream services. The first three are read-only; the guideline output is directed to `/tmp`.

```bash
spis generate-example-catalogs --check
spis verify-reference-evidence --catalog pricing-page-examples
spis check-upstream-drift --skip-network
spis guidelines cli-examples --out /tmp/spis-guidelines-demo.md
```

Retained 2026-08-24 checkpoints:

```text
pricing-page-examples: 0 complete, 5 partial, no measured motion

measured 5 records, 0 complete, 5 partial

local media verified: 3306
local media missing: 0
local media hash mismatch: 0

wrote /tmp/spis-guidelines-demo.md
```

A zero exit from dry verification means measurement completed. It does not erase the listed evidence gaps. `--skip-network` verifies only local files and hashes. See the [full walkthrough](../docs/walkthrough-offline-audit.md).
