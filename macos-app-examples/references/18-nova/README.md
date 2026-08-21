# 18. Nova — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://nova.app/](https://nova.app/)  
**Motion source:** [https://nova.app/images/features/all-in-one.mp4](https://nova.app/images/features/all-in-one.mp4)  
**Upstream owner / recording owner:** Panic / Nova  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="850"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized. Offline identity: `02ce9ebaeca38b53c6d7513d73306560e7d0d6536dd971fa8c514e9beffaddf9` (122302 bytes, 850×478, 7.200s, 108 frames).

## First-success journey

**Actor:** A first-time or returning Nova user  
**Goal:** Edit and inspect a code project  
**Prerequisites:** Nova available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open a project in Nova | Nova advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 0.58s` |
| 2 | Select a file or tool pane | Nova advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 2.02s` |
| 3 | Edit code or configuration | Nova advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 3.46s` |
| 4 | Invoke a run, debug, or inspection action | Nova advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 4.90s` |
| 5 | Observe the resulting output or debug state | Nova advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 6.34s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 6.34s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open a project in Nova | `media/state-01.png` | 0.58s | `8011d0c6dad3a8e56fdbf660188f5558f016ad20f7f25ee6d11743a0687dd452` |
| 2 | Invocation state: Select a file or tool pane | `media/state-02.png` | 2.02s | `510f6d041ad7e09676f88568f0cffbf073fb9fda71fabb77dfdc403afd47f3b4` |
| 3 | Focused intermediate state: Edit code or configuration | `media/state-03.png` | 3.46s | `4c4c9bf053be3da42ef203594619e4052d460b412bd3b36c2648bf8cf823479a` |
| 4 | Committed transition: Invoke a run, debug, or inspection action | `media/state-04.png` | 4.90s | `85b468953c753688395fc8509df0c4dec85f8b5dcbf12d765130cdd20ef7cc15` |
| 5 | First-success result: Observe the resulting output or debug state | `media/state-05.png` | 6.34s | `2a623219a4ec2453a2cf0c7c7b77752d13b5c8689f69fdc222d4aebcd973e08a` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open a project in Nova | Nova exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Select a file or tool pane. |
| Focus and selection | Select a file or tool pane | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Edit code or configuration | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Invoke a run, debug, or inspection action. |
| Confirmation | Invoke a run, debug, or inspection action | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the resulting output or debug state | The recording reaches the first meaningful result for “Edit and inspect a code project”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open a project in Nova.
- **Start state:** Open a project in Nova.
- **End state:** Observe the resulting output or debug state.
- **Continuity:** the MP4 preserves recorded temporal order; the five PNGs are decoded directly from it.
- **Timing class:** brief native animation (7.200s retained).
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

- **Product page:** https://nova.app/
- **Original motion:** https://nova.app/images/features/all-in-one.mp4
- **Capture method:** Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 850×478; 7.200s; 108 frames; 122302 bytes
- **SHA-256:** `02ce9ebaeca38b53c6d7513d73306560e7d0d6536dd971fa8c514e9beffaddf9`
- **Ownership:** Panic / Nova. Product and recording rights remain with their respective upstream owners.
