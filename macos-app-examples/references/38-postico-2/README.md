# 38. Postico 2 — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://eggerapps.at/postico2/](https://eggerapps.at/postico2/)  
**Motion source:** [https://www.youtube.com/watch?v=7ROh8Mel6Cs](https://www.youtube.com/watch?v=7ROh8Mel6Cs)  
**Upstream owner / recording owner:** Just Another Dang How To recording; Jakob Egger / Postico  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `b353dd251798899b07ec8e620f7a9ee2ec8d84d6d6fc98208c822bbdfc15a201` (82345 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Postico 2 user  
**Goal:** Connect to PostgreSQL with Postico  
**Prerequisites:** Postico 2 available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Postico | Postico 2 advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Enter or select connection details | Postico 2 advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Connect to the server | Postico 2 advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Choose a database object | Postico 2 advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the database content surface | Postico 2 advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Postico | `media/state-01.png` | 1.44s | `df295aa35f78ba3b0e738f07ad8c78fdabdcbaffdf2e2e88a7b8a72d4380842f` |
| 2 | Invocation state: Enter or select connection details | `media/state-02.png` | 5.04s | `2899a089ecb1e7d156dd8b613123a94bae3ca63edeefca6b3ed3db48455e68bd` |
| 3 | Focused intermediate state: Connect to the server | `media/state-03.png` | 8.64s | `a3cedd90f0c46265c97473465c2056d187d1c5af0739a66f65bcaf956bf09891` |
| 4 | Committed transition: Choose a database object | `media/state-04.png` | 12.24s | `0c7259431415f23b1220bd40cf0c495b0062c8f714fe87b44b491441a9fd4b39` |
| 5 | First-success result: Observe the database content surface | `media/state-05.png` | 15.84s | `0b30d4e1d2c291dd5cfe515c37028cd8febc9a1bb79e320b788e4fb74ef78c54` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Postico | Postico 2 exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Enter or select connection details. |
| Focus and selection | Enter or select connection details | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Connect to the server | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Choose a database object. |
| Confirmation | Choose a database object | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the database content surface | The recording reaches the first meaningful result for “Connect to PostgreSQL with Postico”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Postico.
- **Start state:** Open Postico.
- **End state:** Observe the database content surface.
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

- **Product page:** https://eggerapps.at/postico2/
- **Original motion:** https://www.youtube.com/watch?v=7ROh8Mel6Cs
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 82345 bytes
- **SHA-256:** `b353dd251798899b07ec8e620f7a9ee2ec8d84d6d6fc98208c822bbdfc15a201`
- **Ownership:** Just Another Dang How To recording; Jakob Egger / Postico. Product and recording rights remain with their respective upstream owners.
