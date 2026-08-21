# 47. Things — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://culturedcode.com/things/](https://culturedcode.com/things/)  
**Motion source:** [https://static.culturedcode.com/things/videos/2026-06-25-meet-things-remastered-2/meettheallnewthings.mp4](https://static.culturedcode.com/things/videos/2026-06-25-meet-things-remastered-2/meettheallnewthings.mp4)  
**Upstream owner / recording owner:** Cultured Code / Things  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized. Offline identity: `7daedb3fff7f64cb0590ef730908867ebfdbceb2771c9413c4ff651a34f9d94c` (323210 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Things user  
**Goal:** Capture and organize a to-do  
**Prerequisites:** Things available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Things | Things advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Create a to-do | Things advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Enter its title or notes | Things advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Place it in the desired list or schedule | Things advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the organized task | Things advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Things | `media/state-01.png` | 1.44s | `c5c3f29244bb8bb4ca9b75cd24b7a464942c44f6cf5b4ad6d99c22d0059b4503` |
| 2 | Invocation state: Create a to-do | `media/state-02.png` | 5.04s | `a929cdd2844c4e9f6e31688e65e373cab15ce510870cef3371bae84bca1962e6` |
| 3 | Focused intermediate state: Enter its title or notes | `media/state-03.png` | 8.64s | `ff399526b18b92181f77f054bbcba5a3000ac1f0d7b23dbb9827456f01b04666` |
| 4 | Committed transition: Place it in the desired list or schedule | `media/state-04.png` | 12.24s | `2a0abb4a672fa242a60ea1b8228da87cb5882751754e61cd1530d8c4c380a06c` |
| 5 | First-success result: Observe the organized task | `media/state-05.png` | 15.84s | `b2acc9beb757e76de03d543fe8d80fde1689323046bf2380c660c8be62d461dc` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Things | Things exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Create a to-do. |
| Focus and selection | Create a to-do | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Enter its title or notes | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Place it in the desired list or schedule. |
| Confirmation | Place it in the desired list or schedule | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the organized task | The recording reaches the first meaningful result for “Capture and organize a to-do”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Things.
- **Start state:** Open Things.
- **End state:** Observe the organized task.
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

- **Product page:** https://culturedcode.com/things/
- **Original motion:** https://static.culturedcode.com/things/videos/2026-06-25-meet-things-remastered-2/meettheallnewthings.mp4
- **Capture method:** Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 323210 bytes
- **SHA-256:** `7daedb3fff7f64cb0590ef730908867ebfdbceb2771c9413c4ff651a34f9d94c`
- **Ownership:** Cultured Code / Things. Product and recording rights remain with their respective upstream owners.
