# Architecture

Spis is a single Rust command-line process over a file-backed corpus. There is no database or resident service in the maintained path. `src/main.rs` collects arguments, `src/commands/mod.rs` dispatches subcommands, command modules own workflows, and `src/lib.rs` supplies shared HTTP, robots, sitemap, HTML extraction, hashing, time, and JSON helpers.

## Components

```text
                    +-----------------------+
                    |      Rust spis CLI    |
                    +-----------+-----------+
                                |
        +-----------------------+-----------------------+
        |                       |                       |
 catalog/reference lane    docs-crawl lane        acquisition adapters
        |                       |                       |
 *-examples/*.json        ~/.spis/docs-corpus     gh / Stado / Weles /
 retained media           pages.jsonl.gz          installed product PTYs
 generated indexes        state.json
```

### Catalog/reference lane

`catalog_type`, `reference_record`, and `reference_contract` manage source structures and shared vocabulary. `verify_reference_evidence` measures record bytes. `generate_example_catalogs` is the cross-file consistency gate and renderer. Structure, README, guideline, image, accessibility, and drift modules enrich or monitor those files.

### Documentation-crawl lane

`crawl_docs` reads the 50 checked-in inventory definitions, resolves URLs, and places work into a process-wide queue. A per-origin host gate controls request spacing. Worker threads fetch and extract pages; one writer thread per site appends gzip JSONL; a periodic flusher writes resumable state through a temporary file and rename. `docs_corpus` is a read-only streaming reader over the same files.

### HTTP layer

The shared GET path spawns each attempt in a dedicated thread and applies a 45-second receive deadline. It retries network failures and HTTP 429/500/502/503/504, sleeps two seconds between retries, and caps the body read. Robots rules are fetched and cached by origin. Gzip is detected by magic bytes rather than headers.

Some specialized modules use their own clients/timeouts: GitHub REST for README snapshots, Brama chat completion for optional discovery ranking, product pages, and candidate image fetches.

### Real-product capture

`capture_wisent_references` searches `PATH`, forks `/bin/bash --norc --noprofile -i` in a 100x32 pseudo-terminal, normalizes the environment, and executes a seven-step probe plan. Output is timed into an asciinema v2 cast. Five named points are rendered as PNG evidence. The capture is measured and written into `wisent-product-examples`.

### Fleet/browser boundary

Spis produces Weles capture/audit plans but does not SSH to hosts. It calls Stado to submit/poll work; Weles owns browser execution. Returned artifacts are verified before record metadata changes. This preserves provenance and keeps placement outside Spis.

## Data ownership and generated views

The source of truth is distributed deliberately:

- `sources.json` owns why an example is selected and its overview/structure facts;
- each `reference.json` owns product evidence and gaps;
- retained files own the bytes that hashes and durations describe;
- `references.json`, catalog `README.md`, and root catalog indexes are generated views;
- `$HOME/.spis/docs-corpus` is local resumable crawl state, not checked-in interface evidence.

The generator refuses disagreement rather than selecting one conflicting copy silently.

## Process and failure model

The CLI is synchronous at the command boundary. Internal commands may use worker threads, subprocesses, or long-running Stado jobs. Ordinary errors return through `anyhow` and main prints `error:` with exit 1. Unknown top-level commands return exit 2. Accessibility audit reserves exits 2/3 for planning versus execution/retrieval. Strict drift exits 1 directly when drift is observed.

## Legacy boundary

`bin/spis`, `scripts/build-release.sh`, `.wisent-release.json`, and comments in some modules still assume removed Python entry points. The Rust crate is the maintained runtime, but two Rust workflows (`reference-record` regeneration and `discover` scaffolding) still invoke removed Python filenames. These are operational defects, not supported alternate architecture; see the [runbook](runbook.md).

`bin/spis-serve` is a loopback Python/JSON backend retained for a desktop client. It is not used by the Rust CLI data path documented here.
