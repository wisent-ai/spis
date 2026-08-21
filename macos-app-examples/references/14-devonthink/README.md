# 14. DEVONthink — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.devontechnologies.com/apps/devonthink](https://www.devontechnologies.com/apps/devonthink)  
**Motion source:** [https://www.youtube.com/watch?v=Bsgi0eEDU8Q](https://www.youtube.com/watch?v=Bsgi0eEDU8Q)  
**Upstream owner / recording owner:** MyProductiveMac real-product recording; DEVONtechnologies  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `2aafb714f6b814cee51a0dbb6787555f13ea4840819e0fa2473cddb2e04fa7f3` (90828 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning DEVONthink user  
**Goal:** Reach a usable DEVONthink database view  
**Prerequisites:** DEVONthink available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Start from the product introduction or launch state | DEVONthink advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Proceed into installation or setup | DEVONthink advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Open a database or content view | DEVONthink advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Select an item in the hierarchy | DEVONthink advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the item in DEVONthink’s detail surface | DEVONthink advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Start from the product introduction or launch state | `media/state-01.png` | 1.44s | `e2c20c5273be0f0c304dab33fbc9672eaae375e8d1548da09f4ea73b1c582146` |
| 2 | Invocation state: Proceed into installation or setup | `media/state-02.png` | 5.04s | `73eb40d8f37a5d3c50b79bd6ea23bb501d2f6ba6942dd40262867066fb38a067` |
| 3 | Focused intermediate state: Open a database or content view | `media/state-03.png` | 8.64s | `e9934ac7bfeb2f25421491bcd9238d23a78868d6e31bd2eeb856b3193a86632f` |
| 4 | Committed transition: Select an item in the hierarchy | `media/state-04.png` | 12.24s | `f1e2757465ee527c7fec334720fece756a5a2c6cfeaf0dc54e64f1f49449583f` |
| 5 | First-success result: Observe the item in DEVONthink’s detail surface | `media/state-05.png` | 15.84s | `850a3bee91a704e58182c14d71ad5985a9cb9f14ddd114a74a7a79e9007f0693` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Start from the product introduction or launch state | DEVONthink exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Proceed into installation or setup. |
| Focus and selection | Proceed into installation or setup | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Open a database or content view | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Select an item in the hierarchy. |
| Confirmation | Select an item in the hierarchy | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the item in DEVONthink’s detail surface | The recording reaches the first meaningful result for “Reach a usable DEVONthink database view”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Start from the product introduction or launch state.
- **Start state:** Start from the product introduction or launch state.
- **End state:** Observe the item in DEVONthink’s detail surface.
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

- **Product page:** https://www.devontechnologies.com/apps/devonthink
- **Original motion:** https://www.youtube.com/watch?v=Bsgi0eEDU8Q
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 90828 bytes
- **SHA-256:** `2aafb714f6b814cee51a0dbb6787555f13ea4840819e0fa2473cddb2e04fa7f3`
- **Ownership:** MyProductiveMac real-product recording; DEVONtechnologies. Product and recording rights remain with their respective upstream owners.
