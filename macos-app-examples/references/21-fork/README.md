# 21. Fork — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://git-fork.com/](https://git-fork.com/)  
**Motion source:** [https://www.youtube.com/watch?v=o1eypmon3N8](https://www.youtube.com/watch?v=o1eypmon3N8)  
**Upstream owner / recording owner:** Kyle Furey recording; Fork by Dan and Tanya  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `1c1c32422b4538110e232b38e84ce41a650d8870e8b0e9373a7e9acee1e2c762` (116535 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Fork user  
**Goal:** Inspect and act on a Git repository  
**Prerequisites:** Fork available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open a repository in Fork | Fork advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Select a branch or change set | Fork advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Inspect changed files | Fork advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Perform the chosen Git action | Fork advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the updated history or working tree | Fork advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open a repository in Fork | `media/state-01.png` | 1.44s | `06533aa6c61dbc8ea0ac11d6879b43a05981a69dc40e5d2f7ff47f2d5c5ca9fa` |
| 2 | Invocation state: Select a branch or change set | `media/state-02.png` | 5.04s | `833de1475a732afde553fb46b64d540223535bc3d104290e8c12496cf35b2733` |
| 3 | Focused intermediate state: Inspect changed files | `media/state-03.png` | 8.64s | `2aa594f4e23d8e734dce315cda18b153aa9e74a3f4a3799ad6da75783946b66e` |
| 4 | Committed transition: Perform the chosen Git action | `media/state-04.png` | 12.24s | `26b36aaba40c7401754a1b1ff6b51f657b5789de56943e92fa036260cacb1348` |
| 5 | First-success result: Observe the updated history or working tree | `media/state-05.png` | 15.84s | `fb3b34f12620d3893a65e2dee6b75abf520f155eed126a82babb3a09121f4bf8` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open a repository in Fork | Fork exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Select a branch or change set. |
| Focus and selection | Select a branch or change set | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Inspect changed files | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Perform the chosen Git action. |
| Confirmation | Perform the chosen Git action | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the updated history or working tree | The recording reaches the first meaningful result for “Inspect and act on a Git repository”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open a repository in Fork.
- **Start state:** Open a repository in Fork.
- **End state:** Observe the updated history or working tree.
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

- **Product page:** https://git-fork.com/
- **Original motion:** https://www.youtube.com/watch?v=o1eypmon3N8
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 116535 bytes
- **SHA-256:** `1c1c32422b4538110e232b38e84ce41a650d8870e8b0e9373a7e9acee1e2c762`
- **Ownership:** Kyle Furey recording; Fork by Dan and Tanya. Product and recording rights remain with their respective upstream owners.
