# 08. Keyboard Maestro — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.keyboardmaestro.com/](https://www.keyboardmaestro.com/)  
**Motion source:** [https://www.keyboardmaestro.com/img/v11/KeyboardMaestro11.mp4](https://www.keyboardmaestro.com/img/v11/KeyboardMaestro11.mp4)  
**Upstream owner / recording owner:** Stairways Software / Keyboard Maestro  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized. Offline identity: `8a20ddb4b2e0d3e906df4cfebbb447cf87d33220503e1949d62b3ff657beeec7` (125996 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Keyboard Maestro user  
**Goal:** Build and run a macro  
**Prerequisites:** Keyboard Maestro available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the macro editor | Keyboard Maestro advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Select or create a macro | Keyboard Maestro advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Add and configure an action | Keyboard Maestro advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Run or confirm the macro | Keyboard Maestro advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the macro’s resulting action | Keyboard Maestro advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open the macro editor | `media/state-01.png` | 1.44s | `88b476748f4a54e01e3cae0b9cc964d106f6f8d733e04db9570315e72c52570e` |
| 2 | Invocation state: Select or create a macro | `media/state-02.png` | 5.04s | `196fb18668dc67e47d9b48d6618eabcc213b549dde5b93353e16d76f7cbdc116` |
| 3 | Focused intermediate state: Add and configure an action | `media/state-03.png` | 8.64s | `a56595fc774631afb495d4148c16ef7b6fb198bdfa894ca05a0e083e12fb2290` |
| 4 | Committed transition: Run or confirm the macro | `media/state-04.png` | 12.24s | `85d87d7c1cf29968ce5c39a190d822eca770804d1fec8347ab5aba826d0ad873` |
| 5 | First-success result: Observe the macro’s resulting action | `media/state-05.png` | 15.84s | `06613d78c7f20ad03d77442f2f0593caa268f1821c9b6948afa2fb9864a15bb6` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the macro editor | Keyboard Maestro exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Select or create a macro. |
| Focus and selection | Select or create a macro | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Add and configure an action | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Run or confirm the macro. |
| Confirmation | Run or confirm the macro | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the macro’s resulting action | The recording reaches the first meaningful result for “Build and run a macro”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open the macro editor.
- **Start state:** Open the macro editor.
- **End state:** Observe the macro’s resulting action.
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

- **Product page:** https://www.keyboardmaestro.com/
- **Original motion:** https://www.keyboardmaestro.com/img/v11/KeyboardMaestro11.mp4
- **Capture method:** Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 125996 bytes
- **SHA-256:** `8a20ddb4b2e0d3e906df4cfebbb447cf87d33220503e1949d62b3ff657beeec7`
- **Ownership:** Stairways Software / Keyboard Maestro. Product and recording rights remain with their respective upstream owners.
