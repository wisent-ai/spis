# 12. iA Writer — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://ia.net/writer](https://ia.net/writer)  
**Motion source:** [https://static.ia.net/writer/landing/iAW-hero-auth.mp4](https://static.ia.net/writer/landing/iAW-hero-auth.mp4)  
**Upstream owner / recording owner:** Information Architects / iA Writer  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized. Offline identity: `ae6623418545eb95f87831ba9c2d909f7720a69b8ebb588a549a7e9a0bd7db26` (1217829 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning iA Writer user  
**Goal:** Write with iA Writer’s focused tools  
**Prerequisites:** iA Writer available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the editor | iA Writer advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Enter or select text | iA Writer advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Invoke a writing analysis or style tool | iA Writer advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Review the emphasized feedback | iA Writer advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the revised document state | iA Writer advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open the editor | `media/state-01.png` | 1.44s | `a420f0aac594dcb7912464f43802f6418d0bed568fe11da0898577b2eb544820` |
| 2 | Invocation state: Enter or select text | `media/state-02.png` | 5.04s | `a3140a0b01ab3bc1db396215bc840fd7ed84a8e046f6de8d427c66818a45c344` |
| 3 | Focused intermediate state: Invoke a writing analysis or style tool | `media/state-03.png` | 8.64s | `cf25605d18243c6fb4e3983af808bcded086c3cf30f11b9124ce79a15dd80bb3` |
| 4 | Committed transition: Review the emphasized feedback | `media/state-04.png` | 12.24s | `d738dd2f51ea38eaf591c40606f70658ddcf5b715b68be7c833801560e73741f` |
| 5 | First-success result: Observe the revised document state | `media/state-05.png` | 15.84s | `9475a77d3bb474187140f550d45bac5f3fdde466cc0f74fd22a11c0a623dc93e` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the editor | iA Writer exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Enter or select text. |
| Focus and selection | Enter or select text | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Invoke a writing analysis or style tool | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Review the emphasized feedback. |
| Confirmation | Review the emphasized feedback | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the revised document state | The recording reaches the first meaningful result for “Write with iA Writer’s focused tools”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open the editor.
- **Start state:** Open the editor.
- **End state:** Observe the revised document state.
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

- **Product page:** https://ia.net/writer
- **Original motion:** https://static.ia.net/writer/landing/iAW-hero-auth.mp4
- **Capture method:** Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 1217829 bytes
- **SHA-256:** `ae6623418545eb95f87831ba9c2d909f7720a69b8ebb588a549a7e9a0bd7db26`
- **Ownership:** Information Architects / iA Writer. Product and recording rights remain with their respective upstream owners.
