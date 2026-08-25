# Runbook

Use this page to diagnose refusals without weakening the evidence contract. Quoted messages below come from the Rust sources or retained executed output.

## Safety classification before running

- Read-only/local: `generate-example-catalogs --check`, default `verify-reference-evidence`, `check-upstream-drift --skip-network`, `docs-corpus ...`, `reference-record get`.
- Local mutation: catalog/record add-edit-remove, generator without `--check`, analyzers, guidelines, verify `--apply`.
- Network mutation: crawl, discovery, product scraping, image collection, README sync.
- Operator/fleet state: width capture, accessibility audit, real product capture.

Make a clean worktree or isolated temporary directory before mutation. Never use a help flag as a safety probe unless the command reference says it implements help.

## `sync-readme-examples --help` refreshed snapshots

**Cause:** `sync_readme_examples::run` ignores its `_rest` argument. Any invocation dispatches the GitHub refresh.

**Observed result:** a retained run of `spis sync-readme-examples --help` fetched 50 README snapshots, changed tracked snapshot/index files, and created `readme-examples/scrape-run.json`.

**Recovery:** preserve unrelated work, restore only the changed `readme-examples` files from version control, and remove only the newly created scrape-run if it was not intended. Do not rerun to confirm. To inspect this command, read [the CLI reference](cli-reference.md) or source.

## Record mutation reports `index regeneration refused the change`

Exact shape:

```text
error: index regeneration refused the change:
python3: can't open file '<working-directory>/generate-example-catalogs.py': [Errno 2] No such file or directory
```

**Cause:** `reference_contract::regenerate_index` still calls the removed `generate-example-catalogs.py`, although the generator was ported to Rust.

**Important:** add/remove writes its primary record/index change before the failing regeneration call. Do not blindly repeat the mutation.

**Recovery:** inspect with `reference-record get` or the relevant JSON, then run:

```bash
spis generate-example-catalogs
spis generate-example-catalogs --check
```

The isolated lifecycle run proved this reconciliation path. Remove likewise updated `references.json` to zero entries before returning 1.

## Rust/Python cutover mismatch

Symptoms include legacy README/release commands failing with missing `*.py`, `discover` errors such as `run catalog-type.py`, or release staging errors from `scripts/build-release.sh`.

**Cause:** the repository contains the Rust port, but `bin/spis`, the release manifest/scripts, `discover`, and the record-regeneration helper retain Python-era references.

**Action:** use `cargo build --release` and `target/release/spis`. For record index regeneration, use the explicit Rust subcommand. Treat `discover` catalog creation and the legacy release packaging path as broken until source is fixed; do not create shims or fake scripts in the corpus.

## Catalog gate refusal

Run the non-writing gate first:

```bash
spis generate-example-catalogs --check
```

Common source messages identify the violated invariant: schema mismatch, count mismatch, unavailable local evidence, byte count differs from file, SHA-256 differs from file, unknown provenance class, duration below the 0.2-second floor, too few states/interactions/journey steps, or status contradicts gaps.

Fix the source record or retained evidence. Do not hand-edit generated README/index totals to match a broken source.

## Verification prints gaps but exits 0

This is expected. Default verification says:

```text
dry run: records are measured and reported, nothing is written
```

It then prints completeness totals and grouped gaps. Exit 0 means measurement completed, not that every record is complete. Use `--apply` only when rewriting measurements is intended, then rerun the catalog gate through the normal evidence process.

## Local drift

```bash
spis check-upstream-drift --skip-network
```

- `local media missing` > 0: restore or intentionally remove/reclassify the record; inspect the listed path.
- `local media hash mismatch` > 0: determine whether bytes changed legitimately. Never update a hash merely to silence drift.
- `--strict`: exits 1 on either condition.

The retained baseline verified 3,306 local media files, with zero missing and zero mismatches.

## Network drift categories

The command separates:

- reachable/unchanged;
- gone;
- guarded (authentication, rate limit, bot wall);
- unresolved transport/network failure.

A guarded or unresolved source is not equivalent to deletion. Use `--skip-readme` to omit README SHA checks while still probing other URLs, or `--skip-network` for a local-only run. `--write-report` creates `upstream-drift.json`; do not publish it without reviewing operational metadata.

## Docs crawl

- `pass --site <NN-slug> or --all`: select a site or all.
- `unknown site: SLUG`: choose a stem present in `documentation-site-examples/content-structure/`.
- Interrupted crawl: rerun the same selection; `state.json` is periodically flushed and already-seen URL hashes are skipped.
- Search returns zero with empty `HOME`: expected until `pages.jsonl.gz` exists.
- `url not found in the SLUG corpus (or archive still being written)`: verify exact URL; a concurrent gzip member may be incomplete.

## Stado/Weles work

Exact refusal when Stado cannot be resolved:

```text
stado is not on PATH; hosts are reached through stado, never ssh
```

Add the trusted Stado binary to `PATH`; do not bypass with SSH. Use `--dry-run` to validate capture/audit plans. Accessibility values must satisfy `--poll-seconds: must be at least 1` and `--timeout-minutes: must be at least 1`. Planning refusal exits 2; execution/retrieval failure exits 3.

## PTY capture

`capture-wisent-references --list` executes each installed product's version probe; it is not a static list. Normal capture refuses `not on PATH: ...` for selected missing binaries and requires Pillow for screenshots. A session may report `session failed mid-run: ...`; keep the cast/run directory for diagnosis and do not represent it as complete evidence.

## Credential recovery

Follow the repository's [credential recovery rule](credential-recovery-rule.md). Do not place tokens in documentation, command lines committed to history, or corpus records.
