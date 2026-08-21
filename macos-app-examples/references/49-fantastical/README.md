# 49. Fantastical — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://flexibits.com/fantastical](https://flexibits.com/fantastical)  
**Motion source:** [https://cdn.flexibits.com/video/fantastical-promo-video-silent.mp4](https://cdn.flexibits.com/video/fantastical-promo-video-silent.mp4)  
**Upstream owner / recording owner:** Flexibits / Fantastical  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="768"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized. Offline identity: `9b21d7c5558ee255da14b41baebcb4bfedb99e23fc459a8f3cf6eca862dc90ea` (362366 bytes, 768×432, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Fantastical user  
**Goal:** Create an event with natural language  
**Prerequisites:** Fantastical available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Fantastical’s event input | Fantastical advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Type a natural-language event | Fantastical advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Review parsed date and time feedback | Fantastical advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Confirm the event | Fantastical advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe it placed on the calendar | Fantastical advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Fantastical’s event input | `media/state-01.png` | 1.44s | `06616ec57dce71fbacafde5e8b547500dd49bef7e471870639926da47c229bc9` |
| 2 | Invocation state: Type a natural-language event | `media/state-02.png` | 5.04s | `608b1ec5f3e8449fdedaa35fe577a791d57f72a6dfad471282570134660fd9e2` |
| 3 | Focused intermediate state: Review parsed date and time feedback | `media/state-03.png` | 8.64s | `7c704e0fb2dc91da44d727217453b3710498fa644b9b6ff439b713c3907ba44a` |
| 4 | Committed transition: Confirm the event | `media/state-04.png` | 12.24s | `6ecf5a2b2929659ff5f9be0b7255be15cc944e931c291f8d2ec74f30a92df681` |
| 5 | First-success result: Observe it placed on the calendar | `media/state-05.png` | 15.84s | `e4a8e4780f06de32b0cc884a390c51888aba288663a54fb3156ded14f7b05d64` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Fantastical’s event input | Fantastical exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Type a natural-language event. |
| Focus and selection | Type a natural-language event | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Review parsed date and time feedback | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm the event. |
| Confirmation | Confirm the event | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe it placed on the calendar | The recording reaches the first meaningful result for “Create an event with natural language”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Fantastical’s event input.
- **Start state:** Open Fantastical’s event input.
- **End state:** Observe it placed on the calendar.
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

- **Product page:** https://flexibits.com/fantastical
- **Original motion:** https://cdn.flexibits.com/video/fantastical-promo-video-silent.mp4
- **Capture method:** Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 768×432; 18.000s; 270 frames; 362366 bytes
- **SHA-256:** `9b21d7c5558ee255da14b41baebcb4bfedb99e23fc459a8f3cf6eca862dc90ea`
- **Ownership:** Flexibits / Fantastical. Product and recording rights remain with their respective upstream owners.
