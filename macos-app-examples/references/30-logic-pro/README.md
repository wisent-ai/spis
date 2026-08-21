# 30. Logic Pro — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.apple.com/logic-pro/](https://www.apple.com/logic-pro/)  
**Motion source:** [https://www.youtube.com/watch?v=b0gViHAhWB0](https://www.youtube.com/watch?v=b0gViHAhWB0)  
**Upstream owner / recording owner:** KC Sounds recording; Apple / Logic Pro  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `5d475eb13c9b53276ebf6894093f15a7d64aceaec94ff9dcb80d97c730ac5a15` (218735 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Logic Pro user  
**Goal:** Create and play a Logic project  
**Prerequisites:** Logic Pro available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the Logic workspace | Logic Pro advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose or add a track | Logic Pro advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Place or select musical material | Logic Pro advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Invoke playback or edit the track | Logic Pro advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the project responding in the timeline | Logic Pro advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open the Logic workspace | `media/state-01.png` | 1.44s | `532508ec55d0ec0264ea8feb78209c13994473e68ad7d5f2b73045a25e4d96e2` |
| 2 | Invocation state: Choose or add a track | `media/state-02.png` | 5.04s | `e5ea64a23352adc04b1655ac2daeee2f310b8579ec5e238c83e7fb8a4c100700` |
| 3 | Focused intermediate state: Place or select musical material | `media/state-03.png` | 8.64s | `fe6548e42ea23d467bbdb80750ae970718f24dc13f1dc3ac5fdd6f294fe4c3b3` |
| 4 | Committed transition: Invoke playback or edit the track | `media/state-04.png` | 12.24s | `80a5114a0910ad7bdb0b0bda9cd38a7ab29605ef829b618e1ee3695ff99fcd98` |
| 5 | First-success result: Observe the project responding in the timeline | `media/state-05.png` | 15.84s | `cf8f8d4cc7e003ba08649472e3f1717691c70d5d333c20ba20007d9f068c274a` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the Logic workspace | Logic Pro exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose or add a track. |
| Focus and selection | Choose or add a track | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Place or select musical material | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Invoke playback or edit the track. |
| Confirmation | Invoke playback or edit the track | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the project responding in the timeline | The recording reaches the first meaningful result for “Create and play a Logic project”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open the Logic workspace.
- **Start state:** Open the Logic workspace.
- **End state:** Observe the project responding in the timeline.
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

- **Product page:** https://www.apple.com/logic-pro/
- **Original motion:** https://www.youtube.com/watch?v=b0gViHAhWB0
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 218735 bytes
- **SHA-256:** `5d475eb13c9b53276ebf6894093f15a7d64aceaec94ff9dcb80d97c730ac5a15`
- **Ownership:** KC Sounds recording; Apple / Logic Pro. Product and recording rights remain with their respective upstream owners.
