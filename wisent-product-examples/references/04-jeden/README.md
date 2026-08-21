# Jeden

The autonomous agent for building software and running the loop around it.

A Wisent product, measured by running it here. Repository [`wisent-ai/jeden`](https://github.com/wisent-ai/jeden); binary `jeden` resolved to `/Users/lukaszbartoszcze/.nvm/versions/node/v22.20.0/bin/jeden`.

## What was run

One `/bin/bash --norc --noprofile -i` session on a real 100x32 pseudo-terminal, cwd `/Users/lukaszbartoszcze/.stado/work/wisent-capture/run/jeden`, on macOS 26.4.1 (25E253) arm64. Seven commands, all read-only:

| # | step | command | exit | lines | widest line |
|---:|---|---|---:|---:|---:|
| 1 | version identity | `jeden --version` | 0 | 1 | 27 |
| 2 | top-level help | `jeden --help` | 0 | 66 | 161 |
| 3 | subcommand help surface | `jeden version` | 0 | 1 | 27 |
| 4 | invalid flag refusal | `jeden --wisent-reference-probe` | 1 | 67 | 161 |
| 5 | Ctrl-C on an unsubmitted line | `jeden --wisent-reference-probe` | — | 0 | 0 |
| 6 | recovery help | `jeden --help` | 0 | 66 | 161 |
| 7 | help with NO_COLOR=1 | `NO_COLOR=1 jeden --help` | 0 | 66 | 161 |

Nothing else was issued. No host was contacted, no credential minted, no vault written, no job submitted, no service restarted, and no test run.

## Identity as installed today

```
$ jeden --version
jeden 0.1.1+dev.341.e2d6a22
exit-status=0
```

## The refusal and the recovery

```
$ jeden --wisent-reference-probe
Error: unknown option: --wisent-reference-probe
Usage:
  jeden [--cwd path] [--model name] [--max-tokens n] [--allow-write] [--allow-command] [--yolo|--auto-approve] [--max-steps n]
  jeden --version | -V
  jeden run "task" [--json] [--model-only] [--cwd path] [--model name] [--max-tokens n] [--allow-write] [--allow-command] [--yolo|--auto-approve] [--max-steps n]
  jeden pursue "rough objective" [--json] [--cwd path] [--model name] [--allow-write] [--allow-command] [--yolo|--auto-approve] [--max-steps n]
  jeden rpc              serve newline-delimited JSON RPC on stdio
  jeden headless <addr> <server-cert.pem> <server-key.pem> <client-ca.pem> <identity-map.json> [revoked-serials.txt]
  jeden acp              serve ACP on stdio
  jeden sessions [limit]
exit-status=1
```

The refusal names the next action ('Usage:'). `jeden --help` then answers again with status 0.

## Motion evidence

- [`media/session.cast`](media/session.cast) — asciinema v2, 8.852 s, 163 events, 18354 bytes, `8d29d137d2f0df24…`
- Play it with `asciinema play media/session.cast`, or read it as JSON: one header line, then `[time, "o", output]` events.

## States

Each PNG is a deterministic render of the cast's own text at a named point in the sequence — not a separate screenshot.

- [`media/01-version-identity.png`](media/01-version-identity.png) — version-identity, cast event 14 at t=1.53742 s, 924x728, 12617 bytes
- [`media/02-help-surface.png`](media/02-help-surface.png) — help-surface, cast event 39 at t=2.84382 s, 924x728, 64647 bytes
- [`media/03-subcommand-help.png`](media/03-subcommand-help.png) — subcommand-help, cast event 67 at t=4.15873 s, 924x728, 63550 bytes
- [`media/04-refusal.png`](media/04-refusal.png) — refusal, cast event 90 at t=5.50231 s, 924x728, 62134 bytes
- [`media/05-recovery.png`](media/05-recovery.png) — recovery, cast event 131 at t=7.54481 s, 924x728, 64647 bytes

## Accessibility, measured

- Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, `jeden --help` emitted 0 ANSI SGR sequences, so this product does not colour its help output.
- NO_COLOR: `NO_COLOR=1 jeden --help` printed 66 lines with 0 SGR sequences; the ANSI-stripped text of the two runs is byte-identical, so every state the help communicates survives without colour.
- Refusal wording: the refusal for `jeden --wisent-reference-probe` names the next action ('Usage:') in the same output.
- Terminal width: the widest line of `jeden --help` is 161 characters, so the help does not fit an 80-column terminal and will wrap.
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
| 2 | Run `jeden --version`. | Jeden prints `jeden 0.1.1+dev.341.e2d6a22` and exits 0. | version identity |
| 3 | Run `jeden --help`. | 66 lines of the product's own top-level help are printed. | top-level help surface |
| 4 | Run `jeden version`. | Jeden's help is one usage block covering every subcommand; there is no per-subcommand help. `jeden run --help` is not a safe probe: when probed once outside this recording it resolved credentials through Skarbiec before parsing `--help` and failed with an HTTP 403, so this record measures `jeden version` instead and reports that finding rather than re-running it. | subcommand help surface |
| 5 | Run `jeden --wisent-reference-probe`. | The option is refused: `Error: unknown option: --wisent-reference-probe`, status 1. | observed refusal |
| 6 | Type `jeden --wisent-reference-probe` again and press Ctrl-C before Enter. | The unsubmitted line is discarded and the prompt returns; nothing ran. | cancelled pending command |
| 7 | Run `jeden --help` again. | Valid help is printed again with status 0: first success is recovered. | recovered first success |
| 8 | Run `NO_COLOR=1 jeden --help`. | The same help text appears with colour removed, so no state was carried by colour. | colour-free help |

## Boundary

A single-block usage synopsis for an agent runtime whose flags are permission grants (`--allow-write`, `--allow-command`, `--yolo`). Study how a dangerous capability set is presented in first-run help.

This record evidences first-look grammar, help discoverability, refusal wording, safe cancellation, recovery and colour independence. It evidences nothing about authenticated behaviour, remote calls, queue submission, vault writes or destructive commands: those paths were deliberately not run.
