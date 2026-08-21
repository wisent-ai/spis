# 32. Acorn — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://flyingmeat.com/acorn/](https://flyingmeat.com/acorn/)  
**Motion source:** [https://www.youtube.com/watch?v=GMDOomqaNzg](https://www.youtube.com/watch?v=GMDOomqaNzg)  
**Upstream owner / recording owner:** Flying Meat / Acorn  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `df4bca9afecc108803b4d2df4f444f53bf18b069512508b4b7606a014cd28228` (229533 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Acorn user  
**Goal:** Open and edit an image in Acorn  
**Prerequisites:** Acorn available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open an image | Acorn advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Select an editing tool or layer | Acorn advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Apply an edit to the canvas | Acorn advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Choose or confirm the next image state | Acorn advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the edited image | Acorn advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open an image | `media/state-01.png` | 1.44s | `e16bc239c8b53794c163720bd0e204ae00759ac6f4ca8fe0b4aef1fa9b29a11e` |
| 2 | Invocation state: Select an editing tool or layer | `media/state-02.png` | 5.04s | `33bf7969d945221609cf00fc4392c523acf8646d44db232b016d35f715dc6a3f` |
| 3 | Focused intermediate state: Apply an edit to the canvas | `media/state-03.png` | 8.64s | `2a33ce185e418b909e3285a3ac77e017e55380bc12cb4487ec199ec4e92bae61` |
| 4 | Committed transition: Choose or confirm the next image state | `media/state-04.png` | 12.24s | `b603cd052865c3be47cfc5d6bd0f9db10b21d37d71bd36798b3a4ebd0ae5e78f` |
| 5 | First-success result: Observe the edited image | `media/state-05.png` | 15.84s | `82f2c3150ef8d52ee95d12d23d60be51a801118a61869b52fc3760390d16fe02` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open an image | Acorn exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Select an editing tool or layer. |
| Focus and selection | Select an editing tool or layer | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Apply an edit to the canvas | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Choose or confirm the next image state. |
| Confirmation | Choose or confirm the next image state | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the edited image | The recording reaches the first meaningful result for “Open and edit an image in Acorn”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open an image.
- **Start state:** Open an image.
- **End state:** Observe the edited image.
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

- **Product page:** https://flyingmeat.com/acorn/
- **Original motion:** https://www.youtube.com/watch?v=GMDOomqaNzg
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 229533 bytes
- **SHA-256:** `df4bca9afecc108803b4d2df4f444f53bf18b069512508b4b7606a014cd28228`
- **Ownership:** Flying Meat / Acorn. Product and recording rights remain with their respective upstream owners.
