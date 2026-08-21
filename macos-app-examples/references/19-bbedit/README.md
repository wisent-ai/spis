# 19. BBEdit — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.barebones.com/products/bbedit/](https://www.barebones.com/products/bbedit/)  
**Motion source:** [https://www.youtube.com/watch?v=0ohrJUgsf7w](https://www.youtube.com/watch?v=0ohrJUgsf7w)  
**Upstream owner / recording owner:** Bare Bones Software  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `832b2261d38802f0815c5ab84e8963e65b34fd8d83bbd0a52f076d9abd6902e1` (132824 bytes, 960×652, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning BBEdit user  
**Goal:** Edit text and see a live preview  
**Prerequisites:** BBEdit available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open a text document | BBEdit advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Place focus in the editor | BBEdit advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Change the source text | BBEdit advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Keep the live preview visible | BBEdit advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe source and rendered result together | BBEdit advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open a text document | `media/state-01.png` | 1.44s | `4c31a92c9c6a314c74b933dd5913c42c08b9712064fd2c1b08bd98bbe81d4f7b` |
| 2 | Invocation state: Place focus in the editor | `media/state-02.png` | 5.04s | `d3484cc86fd2924a0b9fbb79e961a0f1911729f45e2c3af30d5a1c8fc377f462` |
| 3 | Focused intermediate state: Change the source text | `media/state-03.png` | 8.64s | `ffdb7bac0d89d5a7d6fc3b3c99dce02fa2d2cd8ff8651c7d205ab22e585f8b02` |
| 4 | Committed transition: Keep the live preview visible | `media/state-04.png` | 12.24s | `667b524ca93350d1c4be4d8b35b0ff84407669109c7bf2f25e26335cffd765c3` |
| 5 | First-success result: Observe source and rendered result together | `media/state-05.png` | 15.84s | `6aa65c173a8f876ab71154b6160e8b6cb0b1b905c6d522de47d8f4ab2fbcaf9a` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open a text document | BBEdit exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Place focus in the editor. |
| Focus and selection | Place focus in the editor | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Change the source text | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Keep the live preview visible. |
| Confirmation | Keep the live preview visible | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe source and rendered result together | The recording reaches the first meaningful result for “Edit text and see a live preview”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open a text document.
- **Start state:** Open a text document.
- **End state:** Observe source and rendered result together.
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

- **Product page:** https://www.barebones.com/products/bbedit/
- **Original motion:** https://www.youtube.com/watch?v=0ohrJUgsf7w
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×652; 18.000s; 270 frames; 132824 bytes
- **SHA-256:** `832b2261d38802f0815c5ab84e8963e65b34fd8d83bbd0a52f076d9abd6902e1`
- **Ownership:** Bare Bones Software. Product and recording rights remain with their respective upstream owners.
