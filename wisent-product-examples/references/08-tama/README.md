# Tama (tama-cli)

Your AI agent made a mistake? Tama creates rules so that it never happens again.

A Wisent product, measured by running it here. Repository [`wisent-ai/hooks-rotator`](https://github.com/wisent-ai/hooks-rotator); binary `tama` resolved to `/Users/lukaszbartoszcze/.nvm/versions/node/v22.20.0/bin/tama`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/tama`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `tama --version` | 2 | 16 | 115 |
| 2 | top-level help | `tama --help` | 0 | 16 | 115 |
| 3 | subcommand help surface | `tama install-plan --help` | 0 | 8 | 112 |
| 4 | invalid flag refusal | `tama --wisent-reference-probe` | 2 | 16 | 115 |
| 5 | Ctrl-C on an unsubmitted line | `tama --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `tama --help` | 0 | 16 | 115 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 tama --help` | 0 | 16 | 115 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ tama --version
tama <command>

Commands:
  list [--json]              List hooks from the catalog
  show <hook-id> [--json]    Show one hook
  validate [--json]          Validate catalog/source archive
  docs [--out <path>]        Render detailed hook documentation
  install-plan [--json]      Show runtime scopes and target paths
  install                    Install user-global Git dispatchers; use --set-git-config to write Git config
  verify                     Run catalog validation and hook block fixtures
  mcp-config                 Print MCP server config snippet
  find-violations (--repo <path> | --tree <dir> | --owner <gh-owner> | --me) [...] [--json]
exit-status=2
```

Tama (tama-cli) has no version flag. The refusal above is the measurement: this product cannot be asked what version it is from its own CLI.

## The refusal and the recovery

```
$ tama --wisent-reference-probe
tama <command>

Commands:
  list [--json]              List hooks from the catalog
  show <hook-id> [--json]    Show one hook
  validate [--json]          Validate catalog/source archive
  docs [--out <path>]        Render detailed hook documentation
  install-plan [--json]      Show runtime scopes and target paths
  install                    Install user-global Git dispatchers; use --set-git-config to write Git config
  verify                     Run catalog validation and hook block fixtures
exit-status=2
```

The refusal names the next action ('<command>'). `tama --help` then answers again with status 0.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 3.155 s, 164 events, 10355 bytes, `51846bedc0ca86b8…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 20 at t=0.216877 s, 924x728, 56377 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 34 at t=0.63774 s, 924x728, 74207 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 59 at t=1.05838 s, 924x728, 87629 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 91 at t=1.48143 s, 924x728, 76064 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 129 at t=2.73583 s, 924x728, 76158 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `tama --help` emitted 0 ANSI SGR sequences, so this product does not colour its help output.
- NO_COLOR: `NO_COLOR=1 tama --help` printed 16 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is byte-identical, so every state the help communicates survives without colour.
- Refusal wording: the refusal for `tama --wisent-reference-probe` names the next action ('<command>') in the same output.
- Terminal width: the widest line of `tama --help` is 115 characters, so the help does not fit an 80-column terminal and will wrap.
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
| 2 | Run `tama --version`. | Tama (tama-cli) refuses the version flag with status 2 and prints `tama <command>`. | version identity |
| 3 | Run `tama --help`. | 16 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `tama install-plan --help`. | Tama ignores a trailing `--help` and executes the subcommand; `install-plan` only reports the paths an install would touch, so nothing is written. | subcommand help surface |
| 5 | Run `tama --wisent-reference-probe`. | The option is refused: `tama <command>`, status 2. | observed refusal |
| 6 | Type `tama --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `tama --help` again. | Valid help is printed again with status 0: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 tama --help`. | The same help text appears with colour removed, so no state was carried by colour. | colour-free help |

## Boundary

A policy catalog CLI that answers `--help` after a subcommand by ignoring the flag and running the read-only command. Study how a hook installer separates a plan from an install.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
