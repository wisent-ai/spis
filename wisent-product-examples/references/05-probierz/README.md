# Probierz

The quality-evidence boundary: selection, execution, evidence, and verdicts.

A Wisent product, measured by running it here. Repository [`wisent-ai/probierz`](https://github.com/wisent-ai/probierz); binary `probierz` resolved to `/Users/lukaszbartoszcze/.local/bin/probierz`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/probierz`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `probierz --version` | 2 | 50 | 444 |
| 2 | top-level help | `probierz --help` | 0 | 48 | 444 |
| 3 | subcommand help surface | `probierz specs --help` | 1 | 2 | 189 |
| 4 | invalid flag refusal | `probierz --wisent-reference-probe` | 2 | 50 | 444 |
| 5 | Ctrl-C on an unsubmitted line | `probierz --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `probierz --help` | 0 | 48 | 444 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 probierz --help` | 0 | 48 | 444 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ probierz --version
usage:
  probierz list                 every test surface + run script
  probierz apps                 registered products, targets, and journeys
  probierz app <appId>          validated product manifest
  probierz source-identity <appId>  exact harness and app source SHA-256
  probierz specs [surface]      spec files on disk (optional surface filter)
  probierz accessibility <appId>  validate stable IDs and native selectors
  probierz history [appId] [target] [--limit N]  stability by run, journey, and test
  probierz dashboard <appId> [limit]  product/version/journey evidence projection
  probierz status <appId> [--base ref] [--text]  journey coverage, freshness vs HEAD, and merge eligibility (exit 1 when blocked)
  probierz author-spec <appId> <journey> --target <t> --desc <goal> [--base-url u | --app-path p] [--paths glob] [--rounds N] [--dry-run]  draft through the authenticated Stado model router, verify with a real run, keep it green
  probierz author-manifest <appId> --desc <what> --repo <path> --target <t> [--base-url u | --app-path p] [--owner s] [--specs] [--dry-run]  draft through the authenticated Stado model router, then optionally cover every journey
exit-status=2
```

Probierz has no version flag. The refusal above is the measurement: this product cannot be asked what version it is from its own CLI.

## The refusal and the recovery

```
$ probierz --wisent-reference-probe
usage:
  probierz list                 every test surface + run script
  probierz apps                 registered products, targets, and journeys
  probierz app <appId>          validated product manifest
  probierz source-identity <appId>  exact harness and app source SHA-256
  probierz specs [surface]      spec files on disk (optional surface filter)
  probierz accessibility <appId>  validate stable IDs and native selectors
  probierz history [appId] [target] [--limit N]  stability by run, journey, and test
  probierz dashboard <appId> [limit]  product/version/journey evidence projection
  probierz status <appId> [--base ref] [--text]  journey coverage, freshness vs HEAD, and merge eligibility (exit 1 when blocked)
exit-status=2
```

The refusal names the next action ('usage:'). `probierz --help` then answers again with status 0.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 3.583 s, 186 events, 36144 bytes, `a160079d54f79141…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 25 at t=0.284399 s, 924x728, 101309 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 46 at t=0.799115 s, 924x728, 102030 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 73 at t=1.29096 s, 924x728, 97368 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 116 at t=1.7701 s, 924x728, 99695 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 158 at t=3.09398 s, 924x728, 102030 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `probierz --help` emitted 0 ANSI SGR sequences, so this product does not colour its help output.
- NO_COLOR: `NO_COLOR=1 probierz --help` printed 48 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is byte-identical, so every state the help communicates survives without colour.
- Refusal wording: the refusal for `probierz --wisent-reference-probe` names the next action ('usage:') in the same output.
- Terminal width: the widest line of `probierz --help` is 444 characters, so the help does not fit an 80-column terminal and will wrap.
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
| 2 | Run `probierz --version`. | Probierz refuses the version flag with status 2 and prints `usage:`. | version identity |
| 3 | Run `probierz --help`. | 48 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `probierz specs --help`. | Probierz has no per-subcommand help: `--help` after a subcommand is read as a surface name and refused. That refusal is the observed subcommand surface. | subcommand help surface |
| 5 | Run `probierz --wisent-reference-probe`. | The option is refused: `usage:`, status 2. | observed refusal |
| 6 | Type `probierz --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `probierz --help` again. | Valid help is printed again with status 0: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 probierz --help`. | The same help text appears with colour removed, so no state was carried by colour. | colour-free help |

## Boundary

The one product whose refusal is a structured machine-readable failure envelope rather than a usage dump. Study a CLI that answers an unknown surface with a parseable `probierz-failure` line plus one plain sentence.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
