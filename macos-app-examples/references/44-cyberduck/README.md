# 44. Cyberduck — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://cyberduck.io/](https://cyberduck.io/)  
**Motion source:** [https://www.youtube.com/watch?v=YcDC3eo8URs](https://www.youtube.com/watch?v=YcDC3eo8URs)  
**Upstream owner / recording owner:** Doteasy recording; iterate GmbH / Cyberduck  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `4b1346911725f9cd0fa8b8f0f7c2e7a3a3a56daf7877fa51a178891a402265fc` (121957 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Cyberduck user  
**Goal:** Configure and use a Cyberduck transfer  
**Prerequisites:** Cyberduck available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Cyberduck | Cyberduck advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose the connection or bookmark | Cyberduck advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Enter server and path details | Cyberduck advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Confirm the transfer action | Cyberduck advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the remote file or transfer state | Cyberduck advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Cyberduck | `media/state-01.png` | 1.44s | `09af1ab69d5ea521442cde0686ecd04cb8eede6ac530792781a4ea047cc583de` |
| 2 | Invocation state: Choose the connection or bookmark | `media/state-02.png` | 5.04s | `77e7f2ab90547a3b80fd263cc54d1a2fad7776244f63ec7a59184dee46c6b6c2` |
| 3 | Focused intermediate state: Enter server and path details | `media/state-03.png` | 8.64s | `d4138a6ac70d854ddb7d45741dd6cc5f4cadb6f1552ee8c7e5b78ca6464b6003` |
| 4 | Committed transition: Confirm the transfer action | `media/state-04.png` | 12.24s | `f16ef31749100960e5ce5790ba26c25b49ff7e01770fb661879f8a7a6d065b40` |
| 5 | First-success result: Observe the remote file or transfer state | `media/state-05.png` | 15.84s | `e6a66022a481ebc1b9dc418848b6ed00e54f891167f7f02813511e3f5e02842d` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Cyberduck | Cyberduck exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose the connection or bookmark. |
| Focus and selection | Choose the connection or bookmark | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Enter server and path details | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm the transfer action. |
| Confirmation | Confirm the transfer action | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the remote file or transfer state | The recording reaches the first meaningful result for “Configure and use a Cyberduck transfer”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Cyberduck.
- **Start state:** Open Cyberduck.
- **End state:** Observe the remote file or transfer state.
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

- **Product page:** https://cyberduck.io/
- **Original motion:** https://www.youtube.com/watch?v=YcDC3eo8URs
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 121957 bytes
- **SHA-256:** `4b1346911725f9cd0fa8b8f0f7c2e7a3a3a56daf7877fa51a178891a402265fc`
- **Ownership:** Doteasy recording; iterate GmbH / Cyberduck. Product and recording rights remain with their respective upstream owners.
