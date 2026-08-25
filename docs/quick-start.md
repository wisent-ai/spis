# Quick start

This path builds the maintained Rust CLI and runs the read-only corpus gate. Run it from the repository root.

## Prerequisites

- Rust/Cargo compatible with `Cargo.toml` and `Cargo.lock`.
- The repository checkout, including the `*-examples/` directories.
- Extra tools are command-specific: Pillow for rendered PTY screenshots, `gh` for README refresh/drift, and `stado` for remote Weles capture/audit. The read-only catalog gate below needs none of those.

## Build

```bash
cargo build --release
./target/release/spis --help
```

The top-level help identifies the program as `spis — evidence-grade reference corpus tooling` and lists the maintained subcommands. Use the Rust binary directly; `bin/spis` is a legacy Python dispatcher.

## Validate the checked-in catalogs

```bash
./target/release/spis generate-example-catalogs --check
```

This command validates only and writes nothing. An executed run on 2026-08-24 exited 0 and reported:

```text
ios-app-examples: 0 complete, 50 partial, 50 owner-published media
android-app-examples: 0 complete, 50 partial, 50 owner-published media
macos-app-examples: 0 complete, 50 partial, 50 owner-published media
desktop-app-examples: 0 complete, 50 partial, 46 browser driven here, 4 owner-published media
web-app-examples: 0 complete, 50 partial, 50 owner-published media
dashboard-console-examples: 0 complete, 50 partial, 50 owner-published media
tui-examples: 0 complete, 50 partial, 27 owner-published media, 23 product run here
cli-examples: 0 complete, 50 partial, 50 product run here
onboarding-auth-examples: 0 complete, 50 partial, 50 owner-published media
documentation-site-examples: 0 complete, 50 partial, 50 browser driven here
app-store-listing-examples: 0 complete, 50 partial, 50 browser driven here, 14 owner-published media
design-system-examples: 0 complete, 50 partial, 48 owner-published media, 2 browser driven here
report-evidence-examples: 0 complete, 50 partial, 50 owner-published media
pricing-page-examples: 0 complete, 5 partial, no measured motion
```

`partial` is not a failure of the gate. It means the record and its explicit gaps are internally consistent. The gate fails when schemas, indexes, hashes, paths, status, or required evidence fields contradict one another.

## Measure without changing data

```bash
./target/release/spis verify-reference-evidence --catalog pricing-page-examples
```

Without `--apply`, verification is a dry run. The retained run measured all five pricing records, wrote nothing, exited 0, and reported thirteen gap categories. See [the offline audit walkthrough](walkthrough-offline-audit.md).

## Next steps

- Learn the [reference pipeline](pipeline.md).
- Use the [CLI reference](cli-reference.md) before any mutating or networked command.
- Follow the [record lifecycle walkthrough](walkthrough-record-lifecycle.md) in an isolated directory.
- Read the [runbook](runbook.md), especially the warning that `sync-readme-examples --help` performs a real refresh because that command ignores all arguments.
