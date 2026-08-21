# Oko (oko-cli)

Understand your team's interactions with AI.

A Wisent product, measured by running it here. Repository [`wisent-ai/oko`](https://github.com/wisent-ai/oko); binary `oko-cli` resolved to `/Users/lukaszbartoszcze/.local/bin/oko-cli`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/oko-cli`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `oko-cli --version` | 1 | 51 | 195 |
| 2 | top-level help | `oko-cli --help` | 0 | 50 | 195 |
| 3 | subcommand help surface | `oko-cli diff --help` | 1 | 50 | 195 |
| 4 | invalid flag refusal | `oko-cli --wisent-reference-probe` | 1 | 51 | 195 |
| 5 | Ctrl-C on an unsubmitted line | `oko-cli --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `oko-cli --help` | 0 | 50 | 195 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 oko-cli --help` | 0 | 50 | 195 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ oko-cli --version
unknown command: --version
oko-cli — Oko headless tools

USAGE:
  oko-cli agent {claude|codex|kimi} <prompt> [--route-url STADO_URL]
  oko-cli diff <base> <branch> [--in PATH]
  oko-cli proxy-sign --ts <unix> --body <str>
  oko-cli proxy-call --prompt <str> [--router URL] [--model NAME]
  oko-cli proxy-serve [--port N] [--router URL]
  oko-cli proxy-translate --body <anthropic-json>
  oko-cli slack post --channel C [--thread-ts TS] {--text TEXT | --text-file PATH}
  oko-cli slack autoconfigure [--config PATH]
exit-status=1
```

Oko (oko-cli) has no version flag. The refusal above is the measurement: this product cannot be asked what version it is from its own CLI.

## The refusal and the recovery

```
$ oko-cli --wisent-reference-probe
unknown command: --wisent-reference-probe
oko-cli — Oko headless tools

USAGE:
  oko-cli agent {claude|codex|kimi} <prompt> [--route-url STADO_URL]
  oko-cli diff <base> <branch> [--in PATH]
  oko-cli proxy-sign --ts <unix> --body <str>
  oko-cli proxy-call --prompt <str> [--router URL] [--model NAME]
  oko-cli proxy-serve [--port N] [--router URL]
  oko-cli proxy-translate --body <anthropic-json>
exit-status=1
```

The refusal names the next action ('USAGE:'). `oko-cli --help` then answers again with status 0.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 3.189 s, 198 events, 26401 bytes, `9d7dec2e9a3cd0e6…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 29 at t=0.2243 s, 924x728, 100095 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 58 at t=0.646736 s, 924x728, 97003 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 88 at t=1.07254 s, 924x728, 96805 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 126 at t=1.4985 s, 924x728, 100095 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 157 at t=2.75573 s, 924x728, 97003 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `oko-cli --help` emitted 0 ANSI SGR sequences, so this product does not colour its help output.
- NO_COLOR: `NO_COLOR=1 oko-cli --help` printed 50 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is byte-identical, so every state the help communicates survives without colour.
- Refusal wording: the refusal for `oko-cli --wisent-reference-probe` names the next action ('USAGE:') in the same output.
- Terminal width: the widest line of `oko-cli --help` is 195 characters, so the help does not fit an 80-column terminal and will wrap.
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
| 2 | Run `oko-cli --version`. | Oko (oko-cli) refuses the version flag with status 1 and prints `unknown command: --version`. | version identity |
| 3 | Run `oko-cli --help`. | 50 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `oko-cli diff --help`. | Oko answers a subcommand help request with the whole top-level usage block. | subcommand help surface |
| 5 | Run `oko-cli --wisent-reference-probe`. | The option is refused: `unknown command: --wisent-reference-probe`, status 1. | observed refusal |
| 6 | Type `oko-cli --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `oko-cli --help` again. | Valid help is printed again with status 0: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 oko-cli --help`. | The same help text appears with colour removed, so no state was carried by colour. | colour-free help |

## Boundary

A headless companion CLI with one flat usage block and no version form at all. Study the cost of that: the same text answers help, an unknown flag, and a subcommand help request, and only the exit status distinguishes them.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
