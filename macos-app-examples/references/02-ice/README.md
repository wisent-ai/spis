# 02. Ice — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://github.com/jordanbaird/Ice](https://github.com/jordanbaird/Ice)  
**Motion source:** [https://www.youtube.com/watch?v=V9IMIUGrPK4](https://www.youtube.com/watch?v=V9IMIUGrPK4)  
**Upstream owner / recording owner:** ScreenCastsONLINE recording; Ice by Jordan Baird  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `6bbff9b9e49c514ba9c40baa7ff38f0f68b2b3f1cec861c52e9434723720c3bd` (64518 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Ice user  
**Goal:** Configure and reveal hidden menu-bar items  
**Prerequisites:** Ice available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Ice and reach its controls | Ice advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Review the menu-bar sections | Ice advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Choose reveal behavior | Ice advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Apply the visibility configuration | Ice advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Return to the menu bar with the chosen items available | Ice advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Ice and reach its controls | `media/state-01.png` | 1.44s | `ef5503eb260708a1e4d1deeeb1c25389eaa999982ddc42db5fa4089187bf723d` |
| 2 | Invocation state: Review the menu-bar sections | `media/state-02.png` | 5.04s | `31fc1667179360701a78a45127d5268688856c01055eb9c01750c75e9662d16e` |
| 3 | Focused intermediate state: Choose reveal behavior | `media/state-03.png` | 8.64s | `1696f24af1bb24970b0b6c48a3214a9dd28ec2b1fc86304eb6f0a7db4117dd37` |
| 4 | Committed transition: Apply the visibility configuration | `media/state-04.png` | 12.24s | `5a7963057c7725f196fadf3cc1cb849ea582783a3f6c469f017249d069b58586` |
| 5 | First-success result: Return to the menu bar with the chosen items available | `media/state-05.png` | 15.84s | `c12acf5e4997accc9f72eb9cf7bd0a85861aaebf36d07061e97746e9cb82456b` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Ice and reach its controls | Ice exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Review the menu-bar sections. |
| Focus and selection | Review the menu-bar sections | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Choose reveal behavior | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Apply the visibility configuration. |
| Confirmation | Apply the visibility configuration | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Return to the menu bar with the chosen items available | The recording reaches the first meaningful result for “Configure and reveal hidden menu-bar items”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Ice and reach its controls.
- **Start state:** Open Ice and reach its controls.
- **End state:** Return to the menu bar with the chosen items available.
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

- **Product page:** https://github.com/jordanbaird/Ice
- **Original motion:** https://www.youtube.com/watch?v=V9IMIUGrPK4
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 64518 bytes
- **SHA-256:** `6bbff9b9e49c514ba9c40baa7ff38f0f68b2b3f1cec861c52e9434723720c3bd`
- **Ownership:** ScreenCastsONLINE recording; Ice by Jordan Baird. Product and recording rights remain with their respective upstream owners.
