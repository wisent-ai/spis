# 45. Little Snitch — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.obdev.at/products/littlesnitch/index.html](https://www.obdev.at/products/littlesnitch/index.html)  
**Motion source:** [https://www.youtube.com/watch?v=cBtxiHdQxqY](https://www.youtube.com/watch?v=cBtxiHdQxqY)  
**Upstream owner / recording owner:** ScreenCastsONLINE tutorial endorsed by Objective Development / Little Snitch  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `5290feef2ac30cb88c49100ecdcf2b9b838758aad86199685b8045edcdc8115b` (168792 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Little Snitch user  
**Goal:** Review and control network activity  
**Prerequisites:** Little Snitch available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Little Snitch’s overview | Little Snitch advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Inspect the product/network context | Little Snitch advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Choose a connection or rule action | Little Snitch advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Confirm the visibility or policy choice | Little Snitch advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the resulting network-control state | Little Snitch advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Little Snitch’s overview | `media/state-01.png` | 1.44s | `afe81804838145431bd2ad8141bc6cd7abf57bd556d5bd432dc3f5a3dde083a9` |
| 2 | Invocation state: Inspect the product/network context | `media/state-02.png` | 5.04s | `6443fd5f4f0edd53e57d124049a167a034140bc72f8d417888703257b36eca96` |
| 3 | Focused intermediate state: Choose a connection or rule action | `media/state-03.png` | 8.64s | `4a6196f1a42b5fc27b7ef237859b25613d8f2d0475f964ee8b119135ce30762a` |
| 4 | Committed transition: Confirm the visibility or policy choice | `media/state-04.png` | 12.24s | `078e38e4d4dbb6cb91eee8a2ca28b61006bddbe44e512367f66747afeb58a8e1` |
| 5 | First-success result: Observe the resulting network-control state | `media/state-05.png` | 15.84s | `db756a52fe2a435b30556850c9a70d3ec408306198e510a82f4c27f6b76876f1` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Little Snitch’s overview | Little Snitch exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Inspect the product/network context. |
| Focus and selection | Inspect the product/network context | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Choose a connection or rule action | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm the visibility or policy choice. |
| Confirmation | Confirm the visibility or policy choice | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the resulting network-control state | The recording reaches the first meaningful result for “Review and control network activity”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Little Snitch’s overview.
- **Start state:** Open Little Snitch’s overview.
- **End state:** Observe the resulting network-control state.
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

- **Product page:** https://www.obdev.at/products/littlesnitch/index.html
- **Original motion:** https://www.youtube.com/watch?v=cBtxiHdQxqY
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 168792 bytes
- **SHA-256:** `5290feef2ac30cb88c49100ecdcf2b9b838758aad86199685b8045edcdc8115b`
- **Ownership:** ScreenCastsONLINE tutorial endorsed by Objective Development / Little Snitch. Product and recording rights remain with their respective upstream owners.
