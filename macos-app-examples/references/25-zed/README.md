# 25. Zed — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://zed.dev/](https://zed.dev/)  
**Motion source:** [https://www.youtube.com/watch?v=c6Bns1T77HM](https://www.youtube.com/watch?v=c6Bns1T77HM)  
**Upstream owner / recording owner:** Zed Industries  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `fc2c23e71f37502175880bb2b1bcf39336be87c32b34d1e4273107d65fba95ea` (1590926 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Zed user  
**Goal:** Open a project and edit collaboratively in Zed  
**Prerequisites:** Zed available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Launch Zed | Zed advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Open or join a project | Zed advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Select a source file | Zed advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Edit or navigate the code surface | Zed advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the updated project or collaboration state | Zed advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Launch Zed | `media/state-01.png` | 1.44s | `9b995eaef7d50d8cd6cb8e438045c8f1e71ef1628df4963990e71bb30b202605` |
| 2 | Invocation state: Open or join a project | `media/state-02.png` | 5.04s | `1c5e69388002910d3228f8c22b338c9a3b0aaf138ace00db491509785b5595f3` |
| 3 | Focused intermediate state: Select a source file | `media/state-03.png` | 8.64s | `ab3b9528754c29915a14a08e101e2eaa1c100a2abde5b12ca5719a3ad2de280c` |
| 4 | Committed transition: Edit or navigate the code surface | `media/state-04.png` | 12.24s | `3e6fa2e53755d7236a6952e356cf1dacf446e296c20ed8b27169f62b9ab6ac01` |
| 5 | First-success result: Observe the updated project or collaboration state | `media/state-05.png` | 15.84s | `09563e69a9a566ab014764137f3a29535fd08ccf840789f717a030a898036198` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Launch Zed | Zed exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Open or join a project. |
| Focus and selection | Open or join a project | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Select a source file | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Edit or navigate the code surface. |
| Confirmation | Edit or navigate the code surface | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the updated project or collaboration state | The recording reaches the first meaningful result for “Open a project and edit collaboratively in Zed”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Launch Zed.
- **Start state:** Launch Zed.
- **End state:** Observe the updated project or collaboration state.
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

- **Product page:** https://zed.dev/
- **Original motion:** https://www.youtube.com/watch?v=c6Bns1T77HM
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 1590926 bytes
- **SHA-256:** `fc2c23e71f37502175880bb2b1bcf39336be87c32b34d1e4273107d65fba95ea`
- **Ownership:** Zed Industries. Product and recording rights remain with their respective upstream owners.
