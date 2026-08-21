# 03. Maccy — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://maccy.app/](https://maccy.app/)  
**Motion source:** [https://maccy.app/img/maccy/Demo.mp4](https://maccy.app/img/maccy/Demo.mp4)  
**Upstream owner / recording owner:** Alex Rodionov / Maccy  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized. Offline identity: `2de1e3040113033f735068f2c0c7a48a203a79655852e4a5e173aa32441d8a20` (200741 bytes, 960×746, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Maccy user  
**Goal:** Retrieve an earlier clipboard item  
**Prerequisites:** Maccy available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Invoke Maccy | Maccy advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Type a query into clipboard history | Maccy advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Move focus through matching clips | Maccy advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Confirm the intended clip | Maccy advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the chosen item returned for use | Maccy advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Invoke Maccy | `media/state-01.png` | 1.44s | `e10b7096701c9cfa14767bc4dc2f25bdbbcd75e9c792cbdd8385f02c682a85a9` |
| 2 | Invocation state: Type a query into clipboard history | `media/state-02.png` | 5.04s | `aa13bd7e0419c3658757e05f4398e17b4f3c8b0c4da6abfd60640d47e73e3ebc` |
| 3 | Focused intermediate state: Move focus through matching clips | `media/state-03.png` | 8.64s | `69bca795e28958c33f9bdb40a0e44bab315ef7f90e108180de594e698dd0eae3` |
| 4 | Committed transition: Confirm the intended clip | `media/state-04.png` | 12.24s | `77650cf1320b2180cd9533eb72568c7aaf956679e15e0586a238a769c4622cbf` |
| 5 | First-success result: Observe the chosen item returned for use | `media/state-05.png` | 15.84s | `5a0cf1ff28f0dc78c9bf96b936278bb8689ab4ab6ee93bba8280560a46abd6ab` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Invoke Maccy | Maccy exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Type a query into clipboard history. |
| Focus and selection | Type a query into clipboard history | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Move focus through matching clips | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm the intended clip. |
| Confirmation | Confirm the intended clip | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the chosen item returned for use | The recording reaches the first meaningful result for “Retrieve an earlier clipboard item”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Invoke Maccy.
- **Start state:** Invoke Maccy.
- **End state:** Observe the chosen item returned for use.
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

- **Product page:** https://maccy.app/
- **Original motion:** https://maccy.app/img/maccy/Demo.mp4
- **Capture method:** Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×746; 18.000s; 270 frames; 200741 bytes
- **SHA-256:** `2de1e3040113033f735068f2c0c7a48a203a79655852e4a5e173aa32441d8a20`
- **Ownership:** Alex Rodionov / Maccy. Product and recording rights remain with their respective upstream owners.
