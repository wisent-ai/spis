# 06. Raycast — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.raycast.com/](https://www.raycast.com/)  
**Motion source:** [https://www.youtube.com/watch?v=NuIpZoQwuVY](https://www.youtube.com/watch?v=NuIpZoQwuVY)  
**Upstream owner / recording owner:** Raycast  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `3d0e73dca15db71ca3ea00a17756891d310cc68a890175826b34af1225aa4e2f` (392774 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Raycast user  
**Goal:** Run a command from the launcher  
**Prerequisites:** Raycast available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Invoke Raycast | Raycast advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Enter the command or search terms | Raycast advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Move focus to the intended result | Raycast advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Confirm the selected command | Raycast advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the command result | Raycast advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Invoke Raycast | `media/state-01.png` | 1.44s | `6427acbe8f566a4562cd62365262fc2564f183ff7121773b884e4a23f2e729f4` |
| 2 | Invocation state: Enter the command or search terms | `media/state-02.png` | 5.04s | `5063b2b6b28e4159b4e478e97e8d358957c96213db4ef932adda42f12c1df4c0` |
| 3 | Focused intermediate state: Move focus to the intended result | `media/state-03.png` | 8.64s | `fc8954c38d8160fd638c99ca80224ce8055afb83860258d4f6e858a21524aefd` |
| 4 | Committed transition: Confirm the selected command | `media/state-04.png` | 12.24s | `bd9739d8ca34e2be8cbdb7529be310a7c455f071ad79350838f2f07f07c965ea` |
| 5 | First-success result: Observe the command result | `media/state-05.png` | 15.84s | `375bc006587c39646275c1b7e95c7bbb161cc8fe720bf0c6ec50dfabd9d5b37a` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Invoke Raycast | Raycast exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Enter the command or search terms. |
| Focus and selection | Enter the command or search terms | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Move focus to the intended result | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm the selected command. |
| Confirmation | Confirm the selected command | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the command result | The recording reaches the first meaningful result for “Run a command from the launcher”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Invoke Raycast.
- **Start state:** Invoke Raycast.
- **End state:** Observe the command result.
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

- **Product page:** https://www.raycast.com/
- **Original motion:** https://www.youtube.com/watch?v=NuIpZoQwuVY
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 392774 bytes
- **SHA-256:** `3d0e73dca15db71ca3ea00a17756891d310cc68a890175826b34af1225aa4e2f`
- **Ownership:** Raycast. Product and recording rights remain with their respective upstream owners.
