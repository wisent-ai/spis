# Examples

The examples are executable command recipes backed by retained runs. They deliberately distinguish safe local inspection from mutation and network/operator effects.

## Executed walkthroughs

- [Reference record lifecycle](walkthrough-record-lifecycle.md) — isolated catalog creation, partial record scaffolding, reconciliation, joined lookup, dry measurement, and removal; includes the observed post-mutation regeneration defect.
- [Offline corpus audit](walkthrough-offline-audit.md) — read-only catalog gate, dry evidence measurement, local media integrity, and a guideline draft written outside the repository.

## Copyable recipes

- [`examples/record-lifecycle.md`](../examples/record-lifecycle.md) — compact isolated lifecycle commands and expected checkpoints.
- [`examples/offline-integrity-check.md`](../examples/offline-integrity-check.md) — compact non-network integrity sequence.

## Other useful commands

List compiled Wisent product captures (this executes version probes):

```bash
spis capture-wisent-references --list
```

Inspect documentation inventory before a crawl:

```bash
HOME="$(mktemp -d)" spis docs-corpus status
```

Draft evidence-counted observations without changing a catalog:

```bash
spis guidelines cli-examples --out /tmp/cli-guidelines.md
```

Before copying any command, check its safety class in the [runbook](runbook.md) and complete signature in the [CLI reference](cli-reference.md). In particular, never probe `sync-readme-examples` with `--help`; it ignores arguments and performs the network refresh.
