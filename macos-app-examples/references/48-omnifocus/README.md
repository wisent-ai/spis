# 48. OmniFocus — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.omnigroup.com/omnifocus/](https://www.omnigroup.com/omnifocus/)  
**Motion source:** [https://www.youtube.com/watch?v=PO2G_3mJ3Q0](https://www.youtube.com/watch?v=PO2G_3mJ3Q0)  
**Upstream owner / recording owner:** The Omni Group  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `cb421d9ff85430667c3b16b6b48d2c5d8cd985e1b0f670c169e8c5e2aa0987c3` (239740 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning OmniFocus user  
**Goal:** Create and configure an OmniFocus action  
**Prerequisites:** OmniFocus available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open an action or project | OmniFocus advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Enter the task details | OmniFocus advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Choose review, repeat, or scheduling options | OmniFocus advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Confirm the configuration | OmniFocus advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the task in its workflow | OmniFocus advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open an action or project | `media/state-01.png` | 1.44s | `b0e35209f2d7f2692419dfb1bedd58d5c0ff2221a87c10a9491563c465db52e2` |
| 2 | Invocation state: Enter the task details | `media/state-02.png` | 5.04s | `fc81d02daff614d2d98f3023c217c9cc2e925761fa8544258a3fc5615ddc6c44` |
| 3 | Focused intermediate state: Choose review, repeat, or scheduling options | `media/state-03.png` | 8.64s | `43a6c4fa81cf770c422eefb0931bada66532093611cb1230174a225a83a5c8c7` |
| 4 | Committed transition: Confirm the configuration | `media/state-04.png` | 12.24s | `b77719cfd7b41ff88beb16e0f875f1a0efa7179504542297df81ab001d936011` |
| 5 | First-success result: Observe the task in its workflow | `media/state-05.png` | 15.84s | `07fc787800807e5cf2e0566590ac46397c86f98e5f5424f4aeb615d73954f5a5` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open an action or project | OmniFocus exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Enter the task details. |
| Focus and selection | Enter the task details | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Choose review, repeat, or scheduling options | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm the configuration. |
| Confirmation | Confirm the configuration | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the task in its workflow | The recording reaches the first meaningful result for “Create and configure an OmniFocus action”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open an action or project.
- **Start state:** Open an action or project.
- **End state:** Observe the task in its workflow.
- **Continuity:** the MP4 preserves recorded temporal order; the five PNGs are decoded directly from it.
- **Timing class:** short edited product demonstration (18.000s retained).
- **Interruption / reversal:** stopping before state 4 leaves the journey incomplete. No reversal is claimed unless visible.
- **Feedback:** selection, content, layout, or result changes across the retained frames show progress and completion.
- **Reduced-motion equivalent:** `state-01.png` through `state-05.png` preserve the ordered evidence without playback; product-level Reduce Motion behavior is unknown.

## Accessibility

Observed:

- Active focus or selection is conveyed by a visible change between retained states.
- The excerpt preserves the native macOS/product visual hierarchy and pointer/keyboard-driven state changes where shown.
- The five extracted frames provide a nonanimated way to inspect the same sequence.

Unknown:

- VoiceOver announcements and accessible names are not audible in the retained excerpt.
- Full Keyboard Access order is not proven unless explicitly visible in the recording.
- The product’s Reduce Motion behavior and contrast preferences are not demonstrated.

## Provenance

- **Product page:** https://www.omnigroup.com/omnifocus/
- **Original motion:** https://www.youtube.com/watch?v=PO2G_3mJ3Q0
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 239740 bytes
- **SHA-256:** `cb421d9ff85430667c3b16b6b48d2c5d8cd985e1b0f670c169e8c5e2aa0987c3`
- **Ownership:** The Omni Group. Product and recording rights remain with their respective upstream owners.
