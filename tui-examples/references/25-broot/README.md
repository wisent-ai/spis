# 25 — Broot

**Product:** [https://github.com/Canop/broot](https://github.com/Canop/broot)  
**Evidence:** complete — real isolated product recording  
**Captured:** 2026-08-16  
**Category:** File management

## Play the authentic motion

`agg media/product-motion.cast /tmp/broot.gif` then play `/tmp/broot.gif`

This is authentic product motion, not an animation synthesized from the catalog still. Source: [https://github.com/Canop/broot](https://github.com/Canop/broot). Local SHA-256: `f161adfea53d45cdf35dc8f992df3befb7eb17517da19386d92fab61d112646e`. Duration: 4.342s; frames/output events: 56; bytes: 87982; dimensions: terminal columns in cast header × terminal rows in cast header.

## Key states

### first stable surface

![Broot — first stable surface](media/state-1.png)

Derived directly from `media/product-motion.cast`; 983×694, 21348 bytes, SHA-256 `1be513bd95270a7ecdfe3e45dca6860e49a5ded2630b0fb640c01c87f12d2aeb`.

### selection or detail transition

![Broot — selection or detail transition](media/state-2.png)

Derived directly from `media/product-motion.cast`; 983×694, 25821 bytes, SHA-256 `76ff62cd3d103100602057663a54c91f0288d9951f9a8bbe57ff61a93e64907f`.

### help, recovery, or completion

![Broot — help, recovery, or completion](media/state-3.png)

Derived directly from `media/product-motion.cast`; 983×694, 13231 bytes, SHA-256 `994da82a721b5258d1d5be72d5f3ef540ca38cdf3cb85f6330c790f757038f70`.

## First-success journey

**Actor:** Keyboard user evaluating the real terminal application  
**Goal:** Reach the first inspectable file management result in Broot, discover controls, and recover from a blocked or transient state  

| State | User action | System response | Evidence |
|---:|---|---|---|
| 1 | Launch Broot with the recorded prerequisite/fixture | The application claims the terminal and begins its first render | isolated real-program terminal cast at the opening segment |
| 2 | Wait for the first stable product surface | Primary content, an onboarding gate, or an explicit prerequisite state becomes readable | isolated real-program terminal cast at the first retained key state |
| 3 | Move the active selection within the primary list or table | The highlight/cursor moves and selection-coupled content repaints | isolated real-program terminal cast in the early-to-middle segment |
| 4 | Change panel/context or inspect the selected item | A secondary region, detail, child view, or contextual status becomes active | isolated real-program terminal cast around the midpoint key state |
| 5 | Read the persistent key map or invoke the recorded help/discovery control | The visible footer, function-key map, prompt, or help layer exposes the next supported controls | isolated real-program terminal cast in the middle-to-late segment |
| 6 | Cancel/backtrack from the transient or nested state | The prior stable context returns with selection continuity where supported | isolated real-program terminal cast before the final key state |
| 7 | Finish the first inspection and leave or settle on the meaningful result | The result remains inspectable or terminal control is cleanly restored | isolated real-program terminal cast at the final retained state |

### Failure and recovery

- Launch without a required product-specific prerequisite or reach an unavailable/empty selection.
- Observe the explicit terminal error, empty state, disabled action, or unchanged boundary selection.
- Do not substitute a marketing claim for that recorded failure state.
- Recovery: Use the recorded help/footer/discovery affordance to identify the supported action.
- Recovery: Restore the named account, daemon, cluster, device, file, database, or selection prerequisite.
- Recovery: Relaunch or backtrack and repeat the successful navigation shown in the motion evidence.

## Interaction map

| Interaction | Trigger | Response / feedback | Cancellation | Failure → recovery |
|---|---|---|---|---|
| launch | Invoke the product in the isolated fixture | The terminal enters the product surface or its first-run gate; Full-screen redraw or explicit startup status | Quit key or terminal interrupt | Missing service, account, device, or configuration is surfaced in-terminal → Open help, supply the prerequisite, and relaunch |
| move selection | j or Down | Selection advances by one visible item; Highlight/cursor relocates without leaving the view | k or Up reverses the move | At a boundary the selection remains in place → Reverse direction or change region |
| change focus | Tab or the product panel key | Keyboard focus moves to another region; Active border, title, cursor, or highlight changes | Shift-Tab, Tab cycle, or Escape | Unavailable regions do not accept focus → Return to the populated primary region |
| confirm or inspect | Enter | The selected item opens, expands, or becomes the active detail; A detail, child view, dialog, or status repaint appears | Escape, h, or the parent-navigation key | An empty or unavailable selection produces no destructive action → Choose an available item and confirm again |
| open help | ? or the product help key | Shortcut discovery appears or help is requested; Help overlay, footer expansion, or help text | Escape or the same help key | Context may not bind ? → Use the persistent footer/function-key map |
| cancel/backtrack | Escape, h, or Back | The transient layer closes or parent context returns; Previous selection and primary surface reappear | Not applicable; this is the cancellation path | Root view cannot backtrack further → Resume navigation from the retained selection |
| recover from prerequisite failure | Read the visible error/empty/loading state and invoke help or relaunch after supplying the prerequisite | The product exposes its supported next action rather than silently proceeding; Explicit status text followed by a stable interactive surface | Quit leaves the external system unchanged | Account, daemon, cluster, device, or data remains unavailable → Restore that named prerequisite and repeat launch |
| quit | q, F10, or the product quit command | The full-screen surface closes; Terminal control and cursor are restored | Remain in the surface by not confirming a quit dialog | A modal may consume the first quit key → Dismiss the modal, then use the documented quit command |

## Motion analysis

- **Trigger:** launch followed by the recorded keyboard/control sequence.
- **Start/end:** terminal acquisition or upstream launch → first inspectable result and cleanly settled/returned terminal.
- **Continuity:** state changes are direct terminal repaints; selection continuity is spatial rather than animated easing.
- **Timing class:** immediate input feedback plus asynchronous refresh/service latency where the product owns live data.
- **Interruption/reversal:** Escape/back/parent navigation interrupts transient states; reverse navigation restores an earlier selection where supported.
- **Feedback:** cursor, highlight, active border, status line, table/detail repaint, and explicit errors.
- **Reduced motion/nonanimated equivalent:** terminal text and retained state PNGs provide pausable equivalents; a product-level reduced-motion preference was not observed.

## Accessibility

**Observed**
- The observed flow is operable from the keyboard.
- Selection/focus is communicated by a cursor, highlight, active border, title, or position change in the retained states.
- Status and errors are rendered as terminal text rather than motion alone.
- The terminal cast/recording can be paused and replayed independently of the live product.

**Unknown**
- Screen-reader behavior was not exposed by the upstream recording or terminal capture.
- Contrast ratios and color-only distinctions were not instrumentally measured.
- Behavior under user-defined reduced-motion settings is not documented in the observed evidence.
- Mouse parity, switch-control mappings, and nonvisual ordering remain unverified unless visible in the source recording.

## Provenance

- Upstream owner: `Canop`
- Source/capture URL: https://github.com/Canop/broot
- Capture method: Real installed product recorded through a pseudo-terminal on macOS 25.4.0 arm64 in an isolated temporary HOME/XDG environment with deterministic Git, filesystem, SQLite, Maildir, notmuch, or MPD fixtures as applicable; no still-image animation or generated product frames
- Structured record: [`reference.json`](reference.json)
