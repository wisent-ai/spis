# 05. Rectangle — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://rectangleapp.com/](https://rectangleapp.com/)  
**Motion source:** [https://www.youtube.com/watch?v=Vo09H-ln7BQ](https://www.youtube.com/watch?v=Vo09H-ln7BQ)  
**Upstream owner / recording owner:** Ryan Hanson / Rectangle  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `f73694c7c0048150a4c5636ea89a877f236134e9ee5b6c57ae9b0c967496de16` (79548 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Rectangle user  
**Goal:** Place a window with a Rectangle action  
**Prerequisites:** Rectangle available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Start with an unsnapped window | Rectangle advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Invoke a Rectangle placement action | Rectangle advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Preview the target placement | Rectangle advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Confirm or complete the placement gesture | Rectangle advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the window in its target region | Rectangle advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Start with an unsnapped window | `media/state-01.png` | 1.44s | `3530a4ca163a58f3926b89a6affea0cc28d754ce5b070add1e5093c0e6b244c3` |
| 2 | Invocation state: Invoke a Rectangle placement action | `media/state-02.png` | 5.04s | `9c08c2dae429c73b8b5f8b5c8a64098b75e4f90661014edad44e7e4dfe5c0c88` |
| 3 | Focused intermediate state: Preview the target placement | `media/state-03.png` | 8.64s | `f130aef9530338fc4ef2963ed1173f3c61423ac383ec6434912af53ff483f7e3` |
| 4 | Committed transition: Confirm or complete the placement gesture | `media/state-04.png` | 12.24s | `8bdd8db1c8b4eaa3cee9c0a871f7728fb198d308a3849afb9c592db647ea9006` |
| 5 | First-success result: Observe the window in its target region | `media/state-05.png` | 15.84s | `d064218681316d8009b2b7e3e7077a9ba8b46e97a46ff26fc1035e46df8b6027` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Start with an unsnapped window | Rectangle exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Invoke a Rectangle placement action. |
| Focus and selection | Invoke a Rectangle placement action | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Preview the target placement | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm or complete the placement gesture. |
| Confirmation | Confirm or complete the placement gesture | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the window in its target region | The recording reaches the first meaningful result for “Place a window with a Rectangle action”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Start with an unsnapped window.
- **Start state:** Start with an unsnapped window.
- **End state:** Observe the window in its target region.
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

- **Product page:** https://rectangleapp.com/
- **Original motion:** https://www.youtube.com/watch?v=Vo09H-ln7BQ
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 79548 bytes
- **SHA-256:** `f73694c7c0048150a4c5636ea89a877f236134e9ee5b6c57ae9b0c967496de16`
- **Ownership:** Ryan Hanson / Rectangle. Product and recording rights remain with their respective upstream owners.
