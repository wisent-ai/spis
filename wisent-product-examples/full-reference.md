# Wisent product full interaction reference

This synthesis is derived from the 10 complete per-product records in [`references.json`](references.json). Every record is one real local run of an installed Wisent binary on the capture host, recorded through a pseudo-terminal as an asciinema v2 cast, with five deterministic renders of that cast, an eight-step observed journey, nine interaction records, a real refusal with its exit status, a Ctrl-C on an unsubmitted line, help-based recovery, and accessibility facts measured by running the product twice — once with colour available and once with `NO_COLOR=1`.

## What makes this catalog different

Every other catalog in this repository measures somebody else's product. This one measures ours, and it is the only one whose motion evidence was produced by driving the product rather than by collecting what its owner published. The cost of that honesty is size: it holds 10 records, one per installed Wisent CLI, not fifty curated families.

## Evidence method and boundary

Each product was driven through one `/bin/bash --norc --noprofile -i` session on a 100x32 PTY with `TERM=xterm-256color` and `PAGER=cat`, from an empty scratch directory under `~/.stado/work/wisent-capture/run/`, on macOS 26.4.1 (25E253) arm64. Seven commands were issued and no others: the version form, the top-level help, one subcommand help surface, `--wisent-reference-probe`, a Ctrl-C on an unsubmitted line, the recovering help, and the same help with `NO_COLOR=1`.

No host was contacted, no credential minted, no vault written, no queue job submitted, no service restarted and no test executed. The records therefore evidence first-look identity, discoverability, refusal wording, safe cancellation, recovery and colour independence. They evidence nothing about authenticated behaviour, remote calls, queue submission, vault contents or destructive commands.

## What the runs agree on

1. **Every product refuses an unknown flag, and every refusal is nonzero.** The observed statuses are 1, 2 — so automation may treat nonzero as refusal, but must not assume a shared numeric code across our own products.
2. **Every product recovers through its own help.** After the refusal, re-running the top-level help returned valid output in every record; no product needed a reset, a flag order change or a restart.
3. **Cancellation before submission is safe everywhere.** Ctrl-C on a typed but unsubmitted line discarded it and restored the prompt in every session, and the product never started.
4. **Text carries the state.** In 9 of 10 records the ANSI-stripped help text is byte-identical with and without `NO_COLOR=1`, so colour is decoration rather than the carrier.
5. **The shell's exit status is the portable success signal.** Each record prints the real status after each command, and that line is the only cross-product way to tell a refusal from an answer.

## What the runs disagree on, in our own products

- **Version identity.** Only 5 of 10 answer `--version`. The rest refuse the flag: a tool that wants to know which Wisent build it is talking to cannot ask uniformly.
- **Help spelling.** `--help` for most, bare `help` for Skarbiec, and Tama ignores a trailing `--help` after a subcommand and runs the command instead. A wrapper cannot guess.
- **Per-subcommand help.** Stado, Singularity, Transcript Lake and Transcript Label Trainer have it. Oko answers with the whole top-level usage; Probierz refuses `--help` as an unknown surface; Skarbiec reaches the vault state gate first; Weles and Jeden have none at all.
- **Refusal shape.** 7 of 10 refusals name a next action in their own output; the others print usage or a bare sentence. Probierz is alone in emitting a machine-readable failure envelope.
- **Terminal width.** 3 of 10 top-level helps fit 80 columns. The rest overflow, some far past it, so our own help text wraps on a default terminal.
- **Colour.** 1 of 10 colour their help output on a TTY. Colour is therefore not a convention here, it is a per-product choice.

## Applicability boundaries

Use these records to study first-contact behaviour of our own CLIs: identity, discoverability, refusal wording, exit-status contracts, cancellation and recovery. Do not use them as evidence of authenticated workflows, host operations, queue behaviour, credential handling, browser execution, model calls or anything that requires a target. Those need their own recordings, and they are not in this catalog.

## Complete record citations

| # | Product | Repository | Evidence | Version identity as installed | Invalid exit |
|---:|---|---|---|---|---:|
| 1 | Stado | [`wisent-ai/stado`](https://github.com/wisent-ai/stado) | [`references/01-stado/`](references/01-stado/) | `stado 0.7.9` | 2 |
| 2 | Skarbiec | [`wisent-ai/skarbiec`](https://github.com/wisent-ai/skarbiec) | [`references/02-skarbiec/`](references/02-skarbiec/) | `{` | 1 |
| 3 | Weles | [`wisent-ai/weles`](https://github.com/wisent-ai/weles) | [`references/03-weles/`](references/03-weles/) | `0.5.1` | 1 |
| 4 | Jeden | [`wisent-ai/jeden`](https://github.com/wisent-ai/jeden) | [`references/04-jeden/`](references/04-jeden/) | `jeden 0.1.1+dev.341.e2d6a22` | 1 |
| 5 | Probierz | [`wisent-ai/probierz`](https://github.com/wisent-ai/probierz) | [`references/05-probierz/`](references/05-probierz/) | _no version flag_ — `probierz --version` refused | 2 |
| 6 | Oko (oko-cli) | [`wisent-ai/oko`](https://github.com/wisent-ai/oko) | [`references/06-oko-cli/`](references/06-oko-cli/) | _no version flag_ — `oko-cli --version` refused | 1 |
| 7 | Singularity | [`wisent-ai/singularity`](https://github.com/wisent-ai/singularity) | [`references/07-singularity/`](references/07-singularity/) | _no version flag_ — `singularity --version` refused | 1 |
| 8 | Tama (tama-cli) | [`wisent-ai/hooks-rotator`](https://github.com/wisent-ai/hooks-rotator) | [`references/08-tama/`](references/08-tama/) | _no version flag_ — `tama --version` refused | 2 |
| 9 | Transcript Lake | [`wisent-ai/transcript-lake`](https://github.com/wisent-ai/transcript-lake) | [`references/09-transcript-lake/`](references/09-transcript-lake/) | `0.2.0` | 1 |
| 10 | Transcript Label Trainer | [`wisent-ai/transcript-label-trainer`](https://github.com/wisent-ai/transcript-label-trainer) | [`references/10-transcript-label-trainer/`](references/10-transcript-label-trainer/) | _no version flag_ — `transcript-label-trainer --version` refused | 2 |

