# 28. Affinity Designer — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://affinity.serif.com/en-us/designer/](https://affinity.serif.com/en-us/designer/)  
**Motion source:** [https://www.youtube.com/watch?v=CzPzRxDoirM](https://www.youtube.com/watch?v=CzPzRxDoirM)  
**Upstream owner / recording owner:** Canva / Affinity  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `8a1509297a66ab6a98ba8ead42c674cf351369f954a9ec9e6c3de273c968328e` (494718 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Affinity Designer user  
**Goal:** Create vector artwork in Affinity  
**Prerequisites:** Affinity Designer available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Affinity’s design surface | Affinity Designer advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose the vector persona or tool | Affinity Designer advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Select an object or canvas region | Affinity Designer advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Apply a vector edit | Affinity Designer advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the artwork update | Affinity Designer advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Affinity’s design surface | `media/state-01.png` | 1.44s | `93dd9780a918f70ec0fad5fad56c59731aef16356317cc3703ed01bd6f2fea57` |
| 2 | Invocation state: Choose the vector persona or tool | `media/state-02.png` | 5.04s | `b928574e918236c9cd7b69ad4b3148f5b0cf03b52d9dee83a532b3913c5706a9` |
| 3 | Focused intermediate state: Select an object or canvas region | `media/state-03.png` | 8.64s | `8d36b81ac568e7fd8f4894fd40ffd5566379130089679c181dfa9afb015a7667` |
| 4 | Committed transition: Apply a vector edit | `media/state-04.png` | 12.24s | `a2fd0f5c63f8d0cb4a897ea0b336041ccf3007b8c78aeb2a3c6f8b2654cf517d` |
| 5 | First-success result: Observe the artwork update | `media/state-05.png` | 15.84s | `21919b61b6269b3b486dc314dc121ad6ac2a7a694b0fec0a1591d20e27f01cf3` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Affinity’s design surface | Affinity Designer exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose the vector persona or tool. |
| Focus and selection | Choose the vector persona or tool | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Select an object or canvas region | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Apply a vector edit. |
| Confirmation | Apply a vector edit | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the artwork update | The recording reaches the first meaningful result for “Create vector artwork in Affinity”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Affinity’s design surface.
- **Start state:** Open Affinity’s design surface.
- **End state:** Observe the artwork update.
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

- **Product page:** https://affinity.serif.com/en-us/designer/
- **Original motion:** https://www.youtube.com/watch?v=CzPzRxDoirM
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 494718 bytes
- **SHA-256:** `8a1509297a66ab6a98ba8ead42c674cf351369f954a9ec9e6c3de273c968328e`
- **Ownership:** Canva / Affinity. Product and recording rights remain with their respective upstream owners.
