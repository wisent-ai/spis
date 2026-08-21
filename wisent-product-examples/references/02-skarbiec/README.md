# Skarbiec

Credential and authentication management for the AI era.

A Wisent product, measured by running it here. Repository [`wisent-ai/skarbiec`](https://github.com/wisent-ai/skarbiec); binary `skarbiec` resolved to `/Users/lukaszbartoszcze/.local/bin/skarbiec`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/skarbiec`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `skarbiec --version` | 0 | 6 | 55 |
| 2 | top-level help | `skarbiec help` | 0 | 75 | 27 |
| 3 | subcommand help surface | `skarbiec status --help` | 1 | 1 | 109 |
| 4 | invalid flag refusal | `skarbiec --wisent-reference-probe` | 1 | 1 | 48 |
| 5 | Ctrl-C on an unsubmitted line | `skarbiec --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `skarbiec help` | 0 | 75 | 27 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 skarbiec help` | 0 | 75 | 27 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ skarbiec --version
{
  "commit": "3a4af8cb9d656e5e699fa13341a98b415550bb95",
  "provenance": "source build",
  "release": null,
  "version": "0.2.4"
}
exit-status=0
```

## The refusal and the recovery

```
$ skarbiec --wisent-reference-probe
Error: unknown command: --wisent-reference-probe
exit-status=1
```

The refusal names no next action. `skarbiec help` then answers again with status 0.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 3.145 s, 142 events, 8854 bytes, `9108eb403fe6e3d4…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 21 at t=0.211839 s, 924x728, 20079 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 42 at t=0.628827 s, 924x728, 38144 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 53 at t=1.05052 s, 924x728, 43744 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 76 at t=1.46857 s, 924x728, 49003 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 111 at t=2.72037 s, 924x728, 38144 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `skarbiec help` emitted 0 ANSI SGR sequences, so this product does not colour its help output.
- NO_COLOR: `NO_COLOR=1 skarbiec help` printed 75 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is byte-identical, so every state the help communicates survives without colour.
- Refusal wording: the refusal for `skarbiec --wisent-reference-probe` does not name a next action anywhere in its output.
- Terminal width: the widest line of `skarbiec help` is 27 characters, so the help fits an 80-column terminal without wrapping.
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
| 2 | Run `skarbiec --version`. | Skarbiec prints `{` and exits 0. | version identity |
| 3 | Run `skarbiec help`. | 75 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `skarbiec status --help`. | Skarbiec has no per-subcommand help; the subcommand is reached before argument parsing and answers with the vault state gate instead. | subcommand help surface |
| 5 | Run `skarbiec --wisent-reference-probe`. | The option is refused: `Error: unknown command: --wisent-reference-probe`, status 1. | observed refusal |
| 6 | Type `skarbiec --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `skarbiec help` again. | Valid help is printed again with status 0: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 skarbiec help`. | The same help text appears with colour removed, so no state was carried by colour. | colour-free help |

## Boundary

The only product here whose help is machine-readable JSON rather than prose, and the only one whose first refusal is a state gate rather than a parse error. Study what a credential CLI is willing to say before a vault exists.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
