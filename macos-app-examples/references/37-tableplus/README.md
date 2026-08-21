# 37. TablePlus — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://tableplus.com/](https://tableplus.com/)  
**Motion source:** [https://www.youtube.com/watch?v=6WDnuMvqe_U](https://www.youtube.com/watch?v=6WDnuMvqe_U)  
**Upstream owner / recording owner:** CK Data Tech real-product recording; TablePlus Inc.  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `0343c5efdd80892334600e4dd3cfac53fb103671a95546ad8f3e3887c6bae5d8` (63415 bytes, 960×624, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning TablePlus user  
**Goal:** Open and inspect a database in TablePlus  
**Prerequisites:** TablePlus available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open TablePlus | TablePlus advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose or create a connection | TablePlus advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Connect to the database | TablePlus advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Select a table or query context | TablePlus advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe database rows or query results | TablePlus advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open TablePlus | `media/state-01.png` | 1.44s | `78f6447e7a4c6a01ccefbed8de2582d326563b798af027107ff51788e88f0019` |
| 2 | Invocation state: Choose or create a connection | `media/state-02.png` | 5.04s | `19914ede3036c27555589122df93b761f93826e3b54508dc09f8a5b6a0244ab3` |
| 3 | Focused intermediate state: Connect to the database | `media/state-03.png` | 8.64s | `9157f64fe1306f22beddd9dc5ceba7f000a60959e7a92cb0f90450ee8af136f2` |
| 4 | Committed transition: Select a table or query context | `media/state-04.png` | 12.24s | `10123868fbd7a50c5280513da0cc216d6efb1e9e62a8f7f0be8fd525295d1e36` |
| 5 | First-success result: Observe database rows or query results | `media/state-05.png` | 15.84s | `8680f559eb6211cac5812b9fd8bb544aa496bdda637129af3b64930d1418ddc1` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open TablePlus | TablePlus exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose or create a connection. |
| Focus and selection | Choose or create a connection | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Connect to the database | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Select a table or query context. |
| Confirmation | Select a table or query context | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe database rows or query results | The recording reaches the first meaningful result for “Open and inspect a database in TablePlus”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open TablePlus.
- **Start state:** Open TablePlus.
- **End state:** Observe database rows or query results.
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

- **Product page:** https://tableplus.com/
- **Original motion:** https://www.youtube.com/watch?v=6WDnuMvqe_U
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×624; 18.000s; 270 frames; 63415 bytes
- **SHA-256:** `0343c5efdd80892334600e4dd3cfac53fb103671a95546ad8f3e3887c6bae5d8`
- **Ownership:** CK Data Tech real-product recording; TablePlus Inc.. Product and recording rights remain with their respective upstream owners.
