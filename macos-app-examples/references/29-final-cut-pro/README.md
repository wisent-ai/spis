# 29. Final Cut Pro — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.apple.com/final-cut-pro/](https://www.apple.com/final-cut-pro/)  
**Motion source:** [https://www.youtube.com/watch?v=281mCfA-umM](https://www.youtube.com/watch?v=281mCfA-umM)  
**Upstream owner / recording owner:** Jenn Jager Pro Tutorials recording; Apple / Final Cut Pro  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `ac9440f2049bb9fccd3e3a479cc708451b07574059dfbf9fa7ce46b04e6e5bcb` (86013 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Final Cut Pro user  
**Goal:** Create a Final Cut Pro project  
**Prerequisites:** Final Cut Pro available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the library workspace | Final Cut Pro advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose New Project | Final Cut Pro advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Set the project details | Final Cut Pro advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Confirm project creation | Final Cut Pro advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the new project in the library | Final Cut Pro advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open the library workspace | `media/state-01.png` | 1.44s | `54b7311c5c6fb833ec482c279b289d35e2c20f60a55dc53b5da381220800d397` |
| 2 | Invocation state: Choose New Project | `media/state-02.png` | 5.04s | `3e64d5d132b84c8351589077ee4c6c9271787c5b5e8848d403b6251fc5591b33` |
| 3 | Focused intermediate state: Set the project details | `media/state-03.png` | 8.64s | `cff43d7d66e7ef73a36d583c43f6d6c66b454f0b5c981cc8e7d38ed106aff1ab` |
| 4 | Committed transition: Confirm project creation | `media/state-04.png` | 12.24s | `ed15397bef48b5af9ba2b9042444bbf5c8df888207446f6a5c27d80e33f48698` |
| 5 | First-success result: Observe the new project in the library | `media/state-05.png` | 15.84s | `a214da7c1858d3b731dacf548a7857e56f6e16f1df6be1ce195c5113825fdbed` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the library workspace | Final Cut Pro exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose New Project. |
| Focus and selection | Choose New Project | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Set the project details | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm project creation. |
| Confirmation | Confirm project creation | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the new project in the library | The recording reaches the first meaningful result for “Create a Final Cut Pro project”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open the library workspace.
- **Start state:** Open the library workspace.
- **End state:** Observe the new project in the library.
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

- **Product page:** https://www.apple.com/final-cut-pro/
- **Original motion:** https://www.youtube.com/watch?v=281mCfA-umM
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 86013 bytes
- **SHA-256:** `ac9440f2049bb9fccd3e3a479cc708451b07574059dfbf9fa7ce46b04e6e5bcb`
- **Ownership:** Jenn Jager Pro Tutorials recording; Apple / Final Cut Pro. Product and recording rights remain with their respective upstream owners.
