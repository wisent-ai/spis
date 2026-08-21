# 33. Mimestream — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://mimestream.com/](https://mimestream.com/)  
**Motion source:** [https://www.youtube.com/watch?v=LmtKeKRd5kk](https://www.youtube.com/watch?v=LmtKeKRd5kk)  
**Upstream owner / recording owner:** Mimestream  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `bb1394062a027b59222ef3826621696a500a31b0e971f1b3701033656ccb3e8b` (759548 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Mimestream user  
**Goal:** Reach and use the Mimestream inbox  
**Prerequisites:** Mimestream available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Launch Mimestream | Mimestream advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Open an account or mailbox | Mimestream advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Select a message | Mimestream advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Review the message content | Mimestream advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the native mail workflow ready for action | Mimestream advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Launch Mimestream | `media/state-01.png` | 1.44s | `83658619a64b4a461beb3abd4fe2ff449bba6f1cd760edaf3c5b12d119f00e40` |
| 2 | Invocation state: Open an account or mailbox | `media/state-02.png` | 5.04s | `ea54b36ad1135314ad696d6c165eaf43159833ba2d1d89e22aac74b518748bd2` |
| 3 | Focused intermediate state: Select a message | `media/state-03.png` | 8.64s | `eb596a8ac33231019974dc077baee3975a94d439c46f06a35f6100b951c0950a` |
| 4 | Committed transition: Review the message content | `media/state-04.png` | 12.24s | `778e231a3ef8c41775045c074408d1f1ee09c81f391f32433dbdb564c724a8b1` |
| 5 | First-success result: Observe the native mail workflow ready for action | `media/state-05.png` | 15.84s | `c798231b52235528b527b2f94a2702c593a0d2dc4e87817f20e727fce4320352` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Launch Mimestream | Mimestream exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Open an account or mailbox. |
| Focus and selection | Open an account or mailbox | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Select a message | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Review the message content. |
| Confirmation | Review the message content | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the native mail workflow ready for action | The recording reaches the first meaningful result for “Reach and use the Mimestream inbox”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Launch Mimestream.
- **Start state:** Launch Mimestream.
- **End state:** Observe the native mail workflow ready for action.
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

- **Product page:** https://mimestream.com/
- **Original motion:** https://www.youtube.com/watch?v=LmtKeKRd5kk
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 759548 bytes
- **SHA-256:** `bb1394062a027b59222ef3826621696a500a31b0e971f1b3701033656ccb3e8b`
- **Ownership:** Mimestream. Product and recording rights remain with their respective upstream owners.
