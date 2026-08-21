# Weles

Authorized browser execution for AI agents, with signed receipts.

A Wisent product, measured by running it here. Repository [`wisent-ai/weles`](https://github.com/wisent-ai/weles); binary `weles` resolved to `/Users/lukaszbartoszcze/.nvm/versions/node/v22.20.0/bin/weles`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/weles`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `weles --version` | 0 | 1 | 5 |
| 2 | top-level help | `weles --help` | 0 | 34 | 108 |
| 3 | subcommand help surface | `weles version` | 0 | 1 | 5 |
| 4 | invalid flag refusal | `weles --wisent-reference-probe` | 1 | 1 | 48 |
| 5 | Ctrl-C on an unsubmitted line | `weles --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `weles --help` | 0 | 34 | 108 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 weles --help` | 0 | 34 | 108 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ weles --version
0.5.1
exit-status=0
```

## The refusal and the recovery

```
$ weles --wisent-reference-probe
weles: unknown command: --wisent-reference-probe
exit-status=1
```

The refusal names no next action. `weles --help` then answers again with status 0.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 3.385 s, 162 events, 10323 bytes, `847b6e02ba214954…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 23 at t=0.255347 s, 924x728, 10792 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 46 at t=0.714335 s, 924x728, 89457 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 68 at t=1.17225 s, 924x728, 86778 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 95 at t=1.63637 s, 924x728, 91732 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 127 at t=2.92474 s, 924x728, 89457 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `weles --help` emitted 0 ANSI SGR sequences, so this product does not colour its help output.
- NO_COLOR: `NO_COLOR=1 weles --help` printed 34 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is byte-identical, so every state the help communicates survives without colour.
- Refusal wording: the refusal for `weles --wisent-reference-probe` does not name a next action anywhere in its output.
- Terminal width: the widest line of `weles --help` is 108 characters, so the help does not fit an 80-column terminal and will wrap.
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
| 2 | Run `weles --version`. | Weles prints `0.5.1` and exits 0. | version identity |
| 3 | Run `weles --help`. | 34 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `weles version`. | Weles exposes no per-subcommand help. `weles version` is the only subcommand that can be run here without authorizing a workflow or touching durable onboarding state, so that is the subcommand surface this record measures. | subcommand help surface |
| 5 | Run `weles --wisent-reference-probe`. | The option is refused: `weles: unknown command: --wisent-reference-probe`, status 1. | observed refusal |
| 6 | Type `weles --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `weles --help` again. | Valid help is printed again with status 0: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 weles --help`. | The same help text appears with colour removed, so no state was carried by colour. | colour-free help |

## Boundary

A CLI whose real work is gated on an authorization boundary, so its safe surface is help and identity only. Study how a product that refuses unauthorized work advertises that boundary in its own usage text.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
