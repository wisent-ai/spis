# Singularity

An open-source framework for autonomous agents that execute tasks and manage resources.

A Wisent product, measured by running it here. Repository [`wisent-ai/singularity`](https://github.com/wisent-ai/singularity); binary `singularity` resolved to `/Library/Frameworks/Python.framework/Versions/3.12/bin/singularity`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/singularity`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `singularity --version` | 1 | 3 | 73 |
| 2 | top-level help | `singularity --help` | 1 | 3 | 73 |
| 3 | subcommand help surface | `singularity onboarding --help` | 1 | 3 | 73 |
| 4 | invalid flag refusal | `singularity --wisent-reference-probe` | 1 | 3 | 73 |
| 5 | Ctrl-C on an unsubmitted line | `singularity --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `singularity --help` | 1 | 3 | 73 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 singularity --help` | 1 | 3 | 73 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ singularity --version
<coroutine object main at 0x100d0e740>
sys:1: RuntimeWarning: coroutine 'main' was never awaited
RuntimeWarning: Enable tracemalloc to get the object allocation traceback
exit-status=1
```

Singularity has no version flag. The refusal above is the measurement: this product cannot be asked what version it is from its own CLI.

## The refusal and the recovery

```
$ singularity --wisent-reference-probe
<coroutine object main at 0x105152740>
sys:1: RuntimeWarning: coroutine 'main' was never awaited
RuntimeWarning: Enable tracemalloc to get the object allocation traceback
exit-status=1
```

The refusal names no next action. `singularity --help` then answers again with status 1.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 113.372 s, 147 events, 5054 bytes, `7e7bcd0c8ec73d0e…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 15 at t=14.2271 s, 924x728, 20424 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 37 at t=34.2087 s, 924x728, 36167 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 67 at t=55.2597 s, 924x728, 51967 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 94 at t=77.9838 s, 924x728, 68027 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 132 at t=98.4095 s, 924x728, 86177 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `singularity --help` emitted 0 ANSI SGR sequences, so this product does not colour its help output.
- NO_COLOR: `NO_COLOR=1 singularity --help` printed 3 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is not identical, so the two runs are recorded as different rather than equivalent.
- Refusal wording: the refusal for `singularity --wisent-reference-probe` does not name a next action anywhere in its output.
- Terminal width: the widest line of `singularity --help` is 73 characters, so the help fits an 80-column terminal without wrapping.
- Input: the whole recorded journey is keyboard-only — typed commands, Enter and one Ctrl-C — and every state is emitted as selectable terminal text, not as a drawn widget.
- State without colour: success and refusal differ by exit status in the cast (1 against 1), which the shell prints as text.

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
| 2 | Run `singularity --version`. | Singularity refuses the version flag with status 1 and prints `<coroutine object main at 0x100d0e740>`. | version identity |
| 3 | Run `singularity --help`. | 3 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `singularity onboarding --help`. | argparse gives every subcommand its own help; `onboarding` is the only one. | subcommand help surface |
| 5 | Run `singularity --wisent-reference-probe`. | The option is refused: `<coroutine object main at 0x105152740>`, status 1. | observed refusal |
| 6 | Type `singularity --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `singularity --help` again. | Valid help is printed again with status 1: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 singularity --help`. | The help text changes once colour is removed, which is recorded as a difference, not a claim of parity. | colour-free help |

## Boundary

The narrowest installed surface in the catalog: one subcommand, `onboarding`. Study a product whose CLI deliberately exposes only the first-use journey.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
