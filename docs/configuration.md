# Configuration reference

The maintained Rust binary does not load a Spis config file. Configuration comes from command flags, three model-router variables, and process location variables. Run from the repository root because catalog and inventory paths are relative to the current working directory.

## Environment variables read by Spis

| Variable | Readers | Default / absence behavior |
|---|---|---|
| `HOME` | crawl/query storage, capture/audit work directories | `.` for crawl/docs/audit/widths; `/tmp` for Wisent capture; empty string for product scraping |
| `PATH` | `stado`, `gh`, and capturable product discovery | no fallback; commands refuse or omit products when required executables cannot be resolved |
| `MODEL_ROUTER_URL` | `discover` Brama ranking | absent/empty disables model ranking and uses deterministic keyword fallback |
| `MODEL_ROUTER_MODEL` | `discover` | `gpt-4o-mini` |
| `MODEL_ROUTER_TOKEN` | `discover` | absent/empty sends no bearer authorization |

No other user environment keys are read by `src/`. `capture-wisent-references` deliberately constructs its PTY child environment: `TERM=xterm-256color`, `SHELL=/bin/bash`, `PAGER=cat`, `LESS=-FRX`, `LINES=32`, `COLUMNS=100`, and `PS1='wisent-ref$ '`. Its no-color probe adds `NO_COLOR=1`; inherited color/shell/pager values are filtered so recordings are comparable.

## Important flag defaults

| Command | Setting | Default |
|---|---|---|
| `crawl-docs` | `--workers` | 64 |
| `crawl-docs` | `--host-delay` | 0.3 seconds per origin |
| `docs-corpus search` | `--limit` | 20 |
| `discover` | `--limit` | 6 selected pages |
| `discover` | `--max-links` | 120 extracted same-origin links |
| `verify-reference-evidence` | `--jobs` | 8 |
| `verify-reference-evidence` | state matching | enabled; disable with `--no-state-match` |
| `collect-example-images` | candidates per record | 14, compiled constant |
| `capture-widths` | `--host` | compiled fleet target; override explicitly outside the owning environment |
| `audit-reference-accessibility` | `--target` | compiled fleet target; override explicitly outside the owning environment |
| `audit-reference-accessibility` | `--poll-seconds` | 15 |
| `audit-reference-accessibility` | `--timeout-minutes` | 120 |

## Network constants

- Shared crawl GET: user agent `WisentKronikaCorpus/0.1 (documentation writing-style research; +https://wisent.com)`, 45-second deadline per attempt, two attempts at call sites, two-second retry sleep, 256 MiB read ceiling.
- Robots fetch: `/robots.txt`, 15-second timeout, cached per origin for the process.
- `discover`: 25-second page fetch, 40-second thumbnail fetch, 60-second Brama request.
- README sync: GitHub API version `2022-11-28`, bearer from `gh auth token`, 30-second request timeout, 64 MiB body limit.
- Product scraping: 30-second request timeout.
- Image collection: 15-second page timeout, 8 MiB page limit, 16 MiB image limit.

## Local paths

| Path | Content |
|---|---|
| `$HOME/.spis/docs-corpus/<slug>/` | `pages.jsonl.gz`, `state.json`, and resume data |
| `$HOME/.spis/work/landing-width-plans/` | Weles width-capture plans |
| `$HOME/.spis/work/wisent-capture/run/<product>/` | PTY cast and rendered screenshots |
| catalog directories in CWD | source/index records and retained evidence |
| `upstream-drift.json` in CWD | only when `check-upstream-drift --write-report` is used |

## External executable requirements

`stado` is mandatory for width capture and accessibility audit; those commands explicitly refuse direct SSH. `gh` and an authenticated token are mandatory for README refresh and used for README drift checks. `capture-wisent-references` requires the selected product binaries on `PATH`, `/bin/bash`, and a Python with Pillow for PNG rendering. Media verification may call local media tooling such as `ffprobe` as described by its errors.
