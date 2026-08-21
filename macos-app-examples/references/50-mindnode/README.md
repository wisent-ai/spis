# 50. MindNode — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.mindnode.com/](https://www.mindnode.com/)  
**Motion source:** [https://www.youtube.com/watch?v=wsnhReyt0ms](https://www.youtube.com/watch?v=wsnhReyt0ms)  
**Upstream owner / recording owner:** IdeasOnCanvas / MindNode  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `3d527572d34e9cef2cfcb57b004594f10d227309aaae2889fe6117ee61532aee` (428191 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning MindNode user  
**Goal:** Create and style a mind map  
**Prerequisites:** MindNode available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open a map | MindNode advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Create or select the central node | MindNode advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Add branches and child nodes | MindNode advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Organize or style the selected nodes | MindNode advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the structured mind map | MindNode advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open a map | `media/state-01.png` | 1.44s | `462b18b58158cbfe7cb560e6bb81fdbc61e158faa7695ea72a987c4a951d1abc` |
| 2 | Invocation state: Create or select the central node | `media/state-02.png` | 5.04s | `787b1892bb4cafb21795f943f1805c600245d9ef6f24be1b602350a67e2b0c2c` |
| 3 | Focused intermediate state: Add branches and child nodes | `media/state-03.png` | 8.64s | `0beb66b88838c488a2af2a672d12ee355ccdbe4b84891e4232ae61a4e310cb22` |
| 4 | Committed transition: Organize or style the selected nodes | `media/state-04.png` | 12.24s | `ddc6f2231bda2e50db94fcff15a2408ba543246865152f52612cad7429a6da06` |
| 5 | First-success result: Observe the structured mind map | `media/state-05.png` | 15.84s | `8e504d02d0b6219e1b2c88f1c24d250d05bff254513f05e332b90bdfd7e23bef` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open a map | MindNode exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Create or select the central node. |
| Focus and selection | Create or select the central node | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Add branches and child nodes | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Organize or style the selected nodes. |
| Confirmation | Organize or style the selected nodes | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the structured mind map | The recording reaches the first meaningful result for “Create and style a mind map”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open a map.
- **Start state:** Open a map.
- **End state:** Observe the structured mind map.
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

- **Product page:** https://www.mindnode.com/
- **Original motion:** https://www.youtube.com/watch?v=wsnhReyt0ms
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 428191 bytes
- **SHA-256:** `3d527572d34e9cef2cfcb57b004594f10d227309aaae2889fe6117ee61532aee`
- **Ownership:** IdeasOnCanvas / MindNode. Product and recording rights remain with their respective upstream owners.
