# 35. Slack — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://slack.com/downloads/mac](https://slack.com/downloads/mac)  
**Motion source:** [https://www.youtube.com/watch?v=6pvVQw03QZY](https://www.youtube.com/watch?v=6pvVQw03QZY)  
**Upstream owner / recording owner:** Slack  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `d2c216e250bb96ffafd918f34ad266fe3a12f3c6e930fb847b33d8dbbf8f5be2` (291099 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Slack user  
**Goal:** Navigate a channel and act on a message  
**Prerequisites:** Slack available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the workspace | Slack advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Select a channel | Slack advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Focus a message or composer | Slack advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Perform the visible message action | Slack advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the channel response | Slack advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open the workspace | `media/state-01.png` | 1.44s | `f55a099b8253c8327a17e30d19915a552f6e07cbd22a2a50155704c9b57b72b4` |
| 2 | Invocation state: Select a channel | `media/state-02.png` | 5.04s | `a20feb5058a3f91f73dc4d335869239a1e2892ec22f4f29191f03eaf59015903` |
| 3 | Focused intermediate state: Focus a message or composer | `media/state-03.png` | 8.64s | `f2818deb91d0df48d29e608a200f3269553e9cd48a3e0fb9ab9606ccc5bb2f6d` |
| 4 | Committed transition: Perform the visible message action | `media/state-04.png` | 12.24s | `0c0c4de54eaf17102a80c706e41e4ffbcfd480cc3b2d7a2459e3d4966c056b19` |
| 5 | First-success result: Observe the channel response | `media/state-05.png` | 15.84s | `c7373d36add59443f202f54e52c498be79f7c53d8e8a1cb0a56361e1748f8b84` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the workspace | Slack exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Select a channel. |
| Focus and selection | Select a channel | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Focus a message or composer | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Perform the visible message action. |
| Confirmation | Perform the visible message action | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the channel response | The recording reaches the first meaningful result for “Navigate a channel and act on a message”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open the workspace.
- **Start state:** Open the workspace.
- **End state:** Observe the channel response.
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

- **Product page:** https://slack.com/downloads/mac
- **Original motion:** https://www.youtube.com/watch?v=6pvVQw03QZY
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 291099 bytes
- **SHA-256:** `d2c216e250bb96ffafd918f34ad266fe3a12f3c6e930fb847b33d8dbbf8f5be2`
- **Ownership:** Slack. Product and recording rights remain with their respective upstream owners.
