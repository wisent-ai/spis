# Stado

Policy-controlled queue and compute control plane for machines you own or authorize.

A Wisent product, measured by running it here. Repository [`wisent-ai/stado`](https://github.com/wisent-ai/stado); binary `stado` resolved to `/Users/lukaszbartoszcze/.local/bin/stado`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/stado`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `stado --version` | 0 | 1 | 11 |
| 2 | top-level help | `stado --help` | 0 | 56 | 220 |
| 3 | subcommand help surface | `stado capabilities --help` | 0 | 10 | 73 |
| 4 | invalid flag refusal | `stado --wisent-reference-probe` | 2 | 5 | 59 |
| 5 | Ctrl-C on an unsubmitted line | `stado --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `stado --help` | 0 | 56 | 220 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 stado --help` | 0 | 56 | 220 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ stado --version
stado 0.7.9
exit-status=0
```

## The refusal and the recovery

```
$ stado --wisent-reference-probe
error: unexpected argument '--wisent-reference-probe' found

Usage: stado [COMMAND]

For more information, try '--help'.
exit-status=2
```

The refusal names the next action ('Usage:'). `stado --help` then answers again with status 0.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 3.17 s, 178 events, 21077 bytes, `74e027989c72a367…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 19 at t=0.2171 s, 924x728, 11987 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 51 at t=0.64137 s, 924x728, 87230 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 82 at t=1.06623 s, 924x728, 72176 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 111 at t=1.49301 s, 924x728, 62647 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 140 at t=2.74221 s, 924x728, 87230 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `stado --help` emitted 113 ANSI SGR sequences, so this product does colour its help output.
- NO_COLOR: `NO_COLOR=1 stado --help` printed 56 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is byte-identical, so every state the help communicates survives without colour.
- Refusal wording: the refusal for `stado --wisent-reference-probe` names the next action ('Usage:') in the same output.
- Terminal width: the widest line of `stado --help` is 220 characters, so the help does not fit an 80-column terminal and will wrap.
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
| 2 | Run `stado --version`. | Stado prints `stado 0.7.9` and exits 0. | version identity |
| 3 | Run `stado --help`. | 56 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `stado capabilities --help`. | Per-command help exists and is the documented way down one level. | subcommand help surface |
| 5 | Run `stado --wisent-reference-probe`. | The option is refused: `error: unexpected argument '--wisent-reference-probe' found`, status 2. | observed refusal |
| 6 | Type `stado --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `stado --help` again. | Valid help is printed again with status 0: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 stado --help`. | The same help text appears with colour removed, so no state was carried by colour. | colour-free help |

## Boundary

The widest command surface we ship: a Clap-style noun tree over jobs, hosts, quota and credits. Study how a large control-plane CLI keeps its top-level help to one screen of verbs and pushes detail into per-command help.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
