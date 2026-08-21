# 27. DigitalOcean CLI (doctl)

- **Official product source:** [https://github.com/digitalocean/doctl](https://github.com/digitalocean/doctl)
- **Upstream owner:** DigitalOcean, LLC
- **Evidence status:** complete
- **Captured:** 2026-08-16T23:06:54+00:00
- **Recording environment:** isolated temporary working directory on macOS 26.4 arm64

## Authentic local evidence

The local [`media/session.cast`](media/session.cast) is an asciinema v2 terminal event stream recorded from the real `doctl` executable. It contains a successful version query, a bounded real help stream, a rejected invocation with observed status `255`, a PTY-recorded Ctrl-C cancellation, and a valid help recovery. It does not contact or mutate a live target and contains no credentials.

- Motion metadata and SHA-256: [`reference.json`](reference.json)
- Key state 1: [version success](media/01-version-success.png)
- Key state 2: [help stream](media/02-help-stream.png)
- Key state 3: [failure and recovery](media/03-failure-recovery.png)

![DigitalOcean CLI (doctl) version success](media/01-version-success.png)

![DigitalOcean CLI (doctl) help stream](media/02-help-stream.png)

![DigitalOcean CLI (doctl) failure and recovery](media/03-failure-recovery.png)

## First-success journey

| State | User action | Observed response | Evidence |
|---:|---|---|---|
| 1 | Open the isolated PTY at `cli-ref$`. | A clean prompt appears with no credentials, project files, or live target selected. | media/session.cast at 0.00–0.55 s |
| 2 | Run `doctl version 2>&1 \| sed -n '1,14p'`. | DigitalOcean CLI (doctl) prints its installed version and exits 0. | media/session.cast at 0.55–2.20 s; media/01-version-success.png; observed exit 0 |
| 3 | Run `doctl help 2>&1 \| sed -n '1,14p'`. | The executable streams its authentic top-level help through the shown fourteen-line limiter. | media/session.cast at 2.20–5.10 s; media/02-help-stream.png |
| 4 | Run `doctl --definitely-invalid-option 2>&1 \| sed -n '1,14p'`. | The executable rejects the input and returns status 255. | media/session.cast at 5.10–7.10 s; observed exit 255; media/03-failure-recovery.png |
| 5 | Type `doctl --help` but press Ctrl-C before Enter. | The PTY accepts Ctrl-C, abandons the pending line, and restores the prompt; the cast labels this observed input action. | media/session.cast at 7.10–8.10 s; recorder annotation identifies the PTY-observed Ctrl-C, discarded pending input, and restored prompt |
| 6 | Rerun `doctl help 2>&1 \| sed -n '1,14p'`. | Valid help output appears again, proving recovery after both parser failure and cancellation and completing first success. | media/session.cast at 8.10–8.45 s; media/03-failure-recovery.png |

## Observed command and streams

### Successful version stream

```text
cli-ref$ doctl version 2>&1 | sed -n '1,14p'
doctl version 1.166.0-release
[exit status 0]
```

### Bounded help stream

```text
cli-ref$ doctl help 2>&1 | sed -n '1,14p'
doctl is a command line interface (CLI) for the DigitalOcean API.

Usage:
  doctl [command]

Manage DigitalOcean Resources:
  1-click              Display commands that pertain to 1-click applications
  account              Display commands that retrieve account details
  apps                 Displays commands for working with apps
  compute              Display commands that manage infrastructure
  databases            Display commands that manage databases
  dedicated-inference  Display commands for managing dedicated inference endpoints
  gradient             Manage Gradient AI resources
  kubernetes           Displays commands to manage Kubernetes clusters and configurations
```

### Failure and recovery

```text
cli-ref$ doctl --definitely-invalid-option 2>&1 | sed -n '1,14p'
Error: unknown flag: --definitely-invalid-option
Usage:
  doctl [command]

Manage DigitalOcean Resources:
  1-click              Display commands that pertain to 1-click applications
  account              Display commands that retrieve account details
  apps                 Displays commands for working with apps
  compute              Display commands that manage infrastructure
  databases            Display commands that manage databases
  dedicated-inference  Display commands for managing dedicated inference endpoints
  gradient             Manage Gradient AI resources
  kubernetes           Displays commands to manage Kubernetes clusters and configurations
  monitoring           Display commands to manage monitoring
[exit status 255]
cli-ref$ doctl --help
[recorder observation: Ctrl-C before Enter; pending input discarded]
cli-ref$
cli-ref$ doctl help 2>&1 | sed -n '1,14p'
doctl is a command line interface (CLI) for the DigitalOcean API.

Usage:
  doctl [command]
```

The error, exit status, Ctrl-C echo, and recovered output above are transcribed from the local cast; they are not example or synthesized command output.

## Interaction map

| Interaction | Trigger | Response / feedback | Cancellation | Failure / recovery | Evidence |
|---|---|---|---|---|---|
| command entry | Type `doctl version 2>&1 \| sed -n '1,14p'` and press Enter. | DigitalOcean CLI (doctl) starts and emits its installed version. doctl version 1.166.0-release | Ctrl-C before Enter clears the pending command in the recorded shell. | A misspelled option is rejected later in the same cast. Re-enter a documented help command. | media/session.cast at 0.55–2.20 s; media/01-version-success.png; observed exit 0 |
| argument parsing | Provide the documented version arguments. | The CLI accepts the option/subcommand grammar and exits successfully. Observed exit status 0. | No cancellation is required for the short version query. | `doctl --definitely-invalid-option 2>&1 \| sed -n '1,14p'` follows the same parser and is rejected. Use the grammar printed by help. | media/session.cast at 0.55–2.20 s; media/01-version-success.png; observed exit 0 |
| stdout stream | Run `doctl help 2>&1 \| sed -n '1,14p'`. | Help text streams into the terminal and is bounded to fourteen lines by the shown pipeline. doctl is a command line interface (CLI) for the DigitalOcean API. | The shell remains interruptible while the stream is active. | A closed downstream stream may stop display without changing the product help content. Run the bounded help command again. | media/session.cast at 2.20–5.10 s; media/02-help-stream.png |
| stderr and exit status | Run `doctl --definitely-invalid-option 2>&1 \| sed -n '1,14p'`. | The executable reports an invalid invocation and returns a nonzero status. Error: unknown flag: --definitely-invalid-option | Ctrl-C is available if an error path blocks; this observed path returned on its own. | Observed status 255. Run `doctl help 2>&1 \| sed -n '1,14p'`. | media/session.cast at 5.10–7.10 s; observed exit 255; media/03-failure-recovery.png |
| help navigation | Invoke `doctl help 2>&1 \| sed -n '1,14p'`. | The top-level usage, options, or command index is printed without opening a browser. The first fourteen nonblank output lines remain readable in the cast. | Ctrl-C/backtracking returns to the shell prompt. | An invalid help spelling is handled as an invalid invocation. Repeat the exact documented help form. | media/session.cast at 2.20–5.10 s; media/02-help-stream.png |
| completion | Wait for version or help output to finish. | Control returns to `cli-ref$`. A new prompt is visible and the exit status is recorded. | Not needed after completion. | A nonzero status distinguishes rejected input from successful completion. Issue a corrected command at the restored prompt. | media/session.cast at 0.85–2.20 s and after the recovery output at 8.45 s |
| cancellation | Type `doctl --help` without Enter, then press Ctrl-C. | The interactive PTY accepts Ctrl-C, discards the pending line, and restores `cli-ref$`. The cast labels the recorder-observed Ctrl-C; the canceled command emits no product help because it was not submitted. | Ctrl-C is the observed cancellation mechanism. | The pending command is intentionally abandoned rather than executed. Submit the bounded help command on the next prompt. | media/session.cast at 7.10–8.10 s; recorder annotation identifies the PTY-observed Ctrl-C, discarded pending input, and restored prompt |
| failure recovery | After the invalid invocation returns nonzero, enter the bounded help command again. | The same real executable prints valid help output and returns control. doctl is a command line interface (CLI) for the DigitalOcean API. | The recovery can itself be canceled with Ctrl-C at the prompt. | Repeating the invalid option reproduces the nonzero route. The valid help form completes the first-success journey. | media/session.cast at 5.10–8.45 s; media/03-failure-recovery.png |

## Motion analysis

- **Trigger:** Enter submits the version, help, invalid, and recovery commands; Ctrl-C interrupts the unsubmitted pending command.
- **Start / end:** the cast starts at a clean prompt and ends at a clean prompt after the explicit first-success line.
- **Continuity:** output appends linearly to one terminal stream; no hidden screen or synthetic tween is used.
- **Timing class:** immediate version response, short streamed help, immediate parser feedback, immediate Ctrl-C cancellation, and short recovery.
- **Interruption / reversal:** Ctrl-C visibly abandons a pending command and returns to the same prompt; the invalid invocation returns by itself with status `255`.
- **Feedback:** text, prompt return, and exit status—not color or animation—carry state.
- **Reduced motion / nonanimated equivalent:** the cast can be paused, and the three PNG frames plus the exact text streams above retain the same states without playback.

## Accessibility

Observed: keyboard-only operation, selectable text, no color dependency, and a pager-free bounded help stream. Not observed: screen-reader announcements, high-contrast themes, non-UTF-8 locales, or a product-specific reduced-motion preference. See [`reference.json`](reference.json) for the complete observations and unknowns.

## Provenance

The executable and output belong to DigitalOcean, LLC. Product/documentation URL: [https://github.com/digitalocean/doctl](https://github.com/digitalocean/doctl). Capture method: Real executable stdout/stderr captured from exact displayed commands in isolated temporary working directory on macOS 26.4 arm64 and serialized as a timed terminal cast; Ctrl-C cancellation was separately captured through an interactive PTY. Version, bounded help, invalid invocation, cancellation, and recovery help were recorded without credentials or live targets. Dimensions are `null` for the terminal cast by contract; its duration is 8.45 seconds with 11 timed output events. Exact byte sizes and SHA-256 digests for the cast and all three state frames are recorded in [`reference.json`](reference.json).
