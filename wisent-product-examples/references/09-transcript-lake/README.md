# Transcript Lake

Nothing you ever told an AI is lost again.

A Wisent product, measured by running it here. Repository [`wisent-ai/transcript-lake`](https://github.com/wisent-ai/transcript-lake); binary `transcript-lake` resolved to `/Users/lukaszbartoszcze/.local/bin/transcript-lake`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/transcript-lake`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `transcript-lake --version` | 0 | 1 | 5 |
| 2 | top-level help | `transcript-lake --help` | 0 | 52 | 101 |
| 3 | subcommand help surface | `transcript-lake paths --help` | 0 | 2 | 65 |
| 4 | invalid flag refusal | `transcript-lake --wisent-reference-probe` | 1 | 54 | 101 |
| 5 | Ctrl-C on an unsubmitted line | `transcript-lake --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `transcript-lake --help` | 0 | 52 | 101 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 transcript-lake --help` | 0 | 52 | 101 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ transcript-lake --version
0.2.0
exit-status=0
```

## The refusal and the recovery

```
$ transcript-lake --wisent-reference-probe
error: unknown command: --wisent-reference-probe

Transcript Lake creates a privacy-masked local event archive from coding-agent transcripts.

Usage: transcript-lake [--data-dir <path>] <command> [flags]

Start safely:
  transcript-lake paths                         show every local product path
  transcript-lake sources                       discover supported transcript stores
  transcript-lake status                        inspect Lake and stream state
exit-status=1
```

The refusal names the next action ('Usage:'). `transcript-lake --help` then answers again with status 0.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 3.146 s, 171 events, 16407 bytes, `3e4e02d1ad868a06…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 12 at t=0.213863 s, 924x728, 11468 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 31 at t=0.63068 s, 924x728, 70678 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 53 at t=1.05756 s, 924x728, 70361 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 71 at t=1.48349 s, 924x728, 70478 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 130 at t=2.72479 s, 924x728, 70678 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `transcript-lake --help` emitted 0 ANSI SGR sequences, so this product does not colour its help output.
- NO_COLOR: `NO_COLOR=1 transcript-lake --help` printed 52 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is byte-identical, so every state the help communicates survives without colour.
- Refusal wording: the refusal for `transcript-lake --wisent-reference-probe` names the next action ('Usage:') in the same output.
- Terminal width: the widest line of `transcript-lake --help` is 101 characters, so the help does not fit an 80-column terminal and will wrap.
- Input: the whole recorded journey is keyboard-only — typed commands, Enter and one Ctrl-C — and every state is emitted as selectable terminal text, not as a drawn widget.
- State without colour: success and refusal differ by exit status in the cast (0 against 1), which the shell prints as text.

Not measured:

- Screen-reader behaviour was not observed: no screen reader was attached to this PTY.
- Colour contrast of any emitted colours was not measured, and no WCAG or terminal-accessibility audit was performed.
- Behaviour in a terminal narrower than 80 columns was not observed; only the emitted line widths were measured.
- High-contrast themes, non-UTF-8 locales and alternative fonts were not exercised.
- Authenticated and target-selected paths were deliberately not run, so nothing here describes the product's accessibility once a host, vault, queue or credential is involved.

## Journey

| # | action | response | state |
|---:|---|---|---|
| 1 | Open the recorded pseudo-terminal at `wisent-ref$` in an empty scratch directory. | A clean prompt appears; no project, credential, host or queue target is selected. | ready prompt |
| 2 | Run `transcript-lake --version`. | Transcript Lake prints `0.2.0` and exits 0. | version identity |
| 3 | Run `transcript-lake --help`. | 52 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `transcript-lake paths --help`. | Per-command usage exists and prints one line of purpose with its flags. | subcommand help surface |
| 5 | Run `transcript-lake --wisent-reference-probe`. | The option is refused: `error: unknown command: --wisent-reference-probe`, status 1. | observed refusal |
| 6 | Type `transcript-lake --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `transcript-lake --help` again. | Valid help is printed again with status 0: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 transcript-lake --help`. | The same help text appears with colour removed, so no state was carried by colour. | colour-free help |

## Boundary

The only help here that opens with a `Start safely:` section and names the read-only commands first. Study help text that is ordered by risk rather than alphabetically.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
