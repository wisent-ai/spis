# 15. Scrivener — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.literatureandlatte.com/scrivener/overview](https://www.literatureandlatte.com/scrivener/overview)  
**Motion source:** [https://www.youtube.com/watch?v=jp3RRmoWBvQ](https://www.youtube.com/watch?v=jp3RRmoWBvQ)  
**Upstream owner / recording owner:** Scrivener App / Literature & Latte  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `51e40d01c6a805b75a8d4e843f92c08ce86f76842ce9f3653467419745b607f3` (130579 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Scrivener user  
**Goal:** Create a new writing project  
**Prerequisites:** Scrivener available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Scrivener’s project templates | Scrivener advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose a project category | Scrivener advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Select a template | Scrivener advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Provide the project setup details | Scrivener advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the created project entry point | Scrivener advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Scrivener’s project templates | `media/state-01.png` | 1.44s | `3864936e116eb7d11e35c8b037a5e0263dd5006d24858a74c8debba978adcefd` |
| 2 | Invocation state: Choose a project category | `media/state-02.png` | 5.04s | `084e210f351181114dbe381e4e9959d121e33abe50d4cebd2d3225925035ff14` |
| 3 | Focused intermediate state: Select a template | `media/state-03.png` | 8.64s | `e0fc8ce96ab158c032578e575a8c44c62d6ada331dd7af5fe1e81cdd475e760a` |
| 4 | Committed transition: Provide the project setup details | `media/state-04.png` | 12.24s | `8f9ce26eb9cf104d05ae9a2af0ec2dba9bcac6418b17aed5c4166e8820470e2c` |
| 5 | First-success result: Observe the created project entry point | `media/state-05.png` | 15.84s | `57de96e1ee965ce74acaceb2037a2f4fb0010d0d91034198733dc071007eac5c` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Scrivener’s project templates | Scrivener exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose a project category. |
| Focus and selection | Choose a project category | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Select a template | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Provide the project setup details. |
| Confirmation | Provide the project setup details | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the created project entry point | The recording reaches the first meaningful result for “Create a new writing project”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Scrivener’s project templates.
- **Start state:** Open Scrivener’s project templates.
- **End state:** Observe the created project entry point.
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

- **Product page:** https://www.literatureandlatte.com/scrivener/overview
- **Original motion:** https://www.youtube.com/watch?v=jp3RRmoWBvQ
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 130579 bytes
- **SHA-256:** `51e40d01c6a805b75a8d4e843f92c08ce86f76842ce9f3653467419745b607f3`
- **Ownership:** Scrivener App / Literature & Latte. Product and recording rights remain with their respective upstream owners.
