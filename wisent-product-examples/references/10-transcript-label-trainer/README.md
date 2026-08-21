# Transcript Label Trainer

Small models for your custom harness needs.

A Wisent product, measured by running it here. Repository [`wisent-ai/transcript-label-trainer`](https://github.com/wisent-ai/transcript-label-trainer); binary `transcript-label-trainer` resolved to `/Users/lukaszbartoszcze/.local/bin/transcript-label-trainer`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/transcript-label-trainer`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `transcript-label-trainer --version` | 2 | 4 | 77 |
| 2 | top-level help | `transcript-label-trainer --help` | 0 | 25 | 78 |
| 3 | subcommand help surface | `transcript-label-trainer info --help` | 0 | 5 | 50 |
| 4 | invalid flag refusal | `transcript-label-trainer --wisent-reference-probe` | 2 | 4 | 81 |
| 5 | Ctrl-C on an unsubmitted line | `transcript-label-trainer --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `transcript-label-trainer --help` | 0 | 25 | 78 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 transcript-label-trainer --help` | 0 | 25 | 78 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ transcript-label-trainer --version
usage: transcript-label-trainer [-h] [--training-root PATH]
                                [--storage-root PATH]
                                {train,run,evaluate,infer,info,autolabel} ...
transcript-label-trainer: error: unrecognized arguments: --version
exit-status=2
```

Transcript Label Trainer has no version flag. The refusal above is the measurement: this product cannot be asked what version it is from its own CLI.

## The refusal and the recovery

```
$ transcript-label-trainer --wisent-reference-probe
usage: transcript-label-trainer [-h] [--training-root PATH]
                                [--storage-root PATH]
                                {train,run,evaluate,infer,info,autolabel} ...
transcript-label-trainer: error: unrecognized arguments: --wisent-reference-probe
exit-status=2
```

The refusal names the next action ('usage:'). `transcript-label-trainer --help` then answers again with status 0.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 3.167 s, 185 events, 9520 bytes, `52bab386a096b751…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 19 at t=0.221065 s, 924x728, 21314 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 43 at t=0.645892 s, 924x728, 71001 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 75 at t=1.06654 s, 924x728, 70006 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 126 at t=1.48989 s, 924x728, 71895 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 158 at t=2.74532 s, 924x728, 70615 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `transcript-label-trainer --help` emitted 0 ANSI SGR sequences, so this product does not colour its help output.
- NO_COLOR: `NO_COLOR=1 transcript-label-trainer --help` printed 25 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is byte-identical, so every state the help communicates survives without colour.
- Refusal wording: the refusal for `transcript-label-trainer --wisent-reference-probe` names the next action ('usage:') in the same output.
- Terminal width: the widest line of `transcript-label-trainer --help` is 78 characters, so the help fits an 80-column terminal without wrapping.
- Input: the whole recorded journey is keyboard-only — typed commands, Enter and one Ctrl-C — and every state is emitted as selectable terminal text, not as a drawn widget.
- State without colour: success and refusal differ by exit status in the cast (0 against 2), which the shell prints as text.

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
| 2 | Run `transcript-label-trainer --version`. | Transcript Label Trainer refuses the version flag with status 2 and prints `usage: transcript-label-trainer [-h] [--training-root PATH]`. | version identity |
| 3 | Run `transcript-label-trainer --help`. | 25 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `transcript-label-trainer info --help`. | argparse gives every subcommand its own help. | subcommand help surface |
| 5 | Run `transcript-label-trainer --wisent-reference-probe`. | The option is refused: `usage: transcript-label-trainer [-h] [--training-root PATH]`, status 2. | observed refusal |
| 6 | Type `transcript-label-trainer --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `transcript-label-trainer --help` again. | Valid help is printed again with status 0: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 transcript-label-trainer --help`. | The same help text appears with colour removed, so no state was carried by colour. | colour-free help |

## Boundary

An argparse CLI that states its own boundary in the help body — 'Never writes to the lake.' Study a product that publishes what it will not touch above its command list.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
