# 39. Sequel Ace — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://github.com/Sequel-Ace/Sequel-Ace](https://github.com/Sequel-Ace/Sequel-Ace)  
**Motion source:** [https://www.youtube.com/watch?v=Eg_ghwLNO6U](https://www.youtube.com/watch?v=Eg_ghwLNO6U)  
**Upstream owner / recording owner:** CK Data Tech recording; Sequel Ace project  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `7be2262254480e9b4a4d9309f4addca330491e2e5bd98c06ba65912ff7beb10c` (36728 bytes, 960×624, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Sequel Ace user  
**Goal:** Connect to MySQL with Sequel Ace  
**Prerequisites:** Sequel Ace available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the connection window | Sequel Ace advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Enter server and account details | Sequel Ace advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Test or confirm the connection | Sequel Ace advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Choose the database context | Sequel Ace advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the connected database view | Sequel Ace advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open the connection window | `media/state-01.png` | 1.44s | `59565f2771622b86b800606819cfdbae551e29428f9b696ce1540171fbf97e36` |
| 2 | Invocation state: Enter server and account details | `media/state-02.png` | 5.04s | `99c2a8bfc5f5841e4c5403f37ec2c7b5f57ecd3a3f1293fc982a217e840ff1d6` |
| 3 | Focused intermediate state: Test or confirm the connection | `media/state-03.png` | 8.64s | `d947f3686036f6590c80f9d62f0b42ff226e0da151b71dc49c50042a2d376bab` |
| 4 | Committed transition: Choose the database context | `media/state-04.png` | 12.24s | `44894a54b1f042f1accea59fc858062be6e30d739bbc495053f99237e86c4d54` |
| 5 | First-success result: Observe the connected database view | `media/state-05.png` | 15.84s | `fe2a9fa081702f55afce226af5a07bd3d26aaed5670483dcb7eb1830d2f41e8a` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the connection window | Sequel Ace exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Enter server and account details. |
| Focus and selection | Enter server and account details | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Test or confirm the connection | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Choose the database context. |
| Confirmation | Choose the database context | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the connected database view | The recording reaches the first meaningful result for “Connect to MySQL with Sequel Ace”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open the connection window.
- **Start state:** Open the connection window.
- **End state:** Observe the connected database view.
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

- **Product page:** https://github.com/Sequel-Ace/Sequel-Ace
- **Original motion:** https://www.youtube.com/watch?v=Eg_ghwLNO6U
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×624; 18.000s; 270 frames; 36728 bytes
- **SHA-256:** `7be2262254480e9b4a4d9309f4addca330491e2e5bd98c06ba65912ff7beb10c`
- **Ownership:** CK Data Tech recording; Sequel Ace project. Product and recording rights remain with their respective upstream owners.
