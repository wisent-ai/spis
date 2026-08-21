# 26. Sketch — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.sketch.com/](https://www.sketch.com/)  
**Motion source:** [https://www.youtube.com/watch?v=qZwP2xrtVMU](https://www.youtube.com/watch?v=qZwP2xrtVMU)  
**Upstream owner / recording owner:** Sketch  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `223596fd1b87f15307e11fd09a2c88955dd56c5016082f3130df2be8a7b11b41` (351083 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Sketch user  
**Goal:** Create and edit a Sketch design  
**Prerequisites:** Sketch available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open a Sketch document | Sketch advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Select an artboard or layer | Sketch advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Insert or manipulate a design element | Sketch advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Adjust its properties | Sketch advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the composed design | Sketch advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open a Sketch document | `media/state-01.png` | 1.44s | `4a0ba84523a79e36309c84936c702ba82eb5f42f345005c088064c8ffd48dd8d` |
| 2 | Invocation state: Select an artboard or layer | `media/state-02.png` | 5.04s | `b63d14007d8f527c8cb66e040a9d696aca383f3e8f30ea80995109954c522bc9` |
| 3 | Focused intermediate state: Insert or manipulate a design element | `media/state-03.png` | 8.64s | `dd0218535846f8227af66b13d386ada0a95cbfd5985a9ef6bac1f8a81a8bee32` |
| 4 | Committed transition: Adjust its properties | `media/state-04.png` | 12.24s | `fbed98a81ecf6d95c73adb8c11a595f17cb2ccb6e6615cb318b77ee8830a0ac6` |
| 5 | First-success result: Observe the composed design | `media/state-05.png` | 15.84s | `f823754fb7ed526dfdc8697d8d2e629629a340b3c6eba25e3e8092da1a7228da` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open a Sketch document | Sketch exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Select an artboard or layer. |
| Focus and selection | Select an artboard or layer | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Insert or manipulate a design element | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Adjust its properties. |
| Confirmation | Adjust its properties | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the composed design | The recording reaches the first meaningful result for “Create and edit a Sketch design”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open a Sketch document.
- **Start state:** Open a Sketch document.
- **End state:** Observe the composed design.
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

- **Product page:** https://www.sketch.com/
- **Original motion:** https://www.youtube.com/watch?v=qZwP2xrtVMU
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 351083 bytes
- **SHA-256:** `223596fd1b87f15307e11fd09a2c88955dd56c5016082f3130df2be8a7b11b41`
- **Ownership:** Sketch. Product and recording rights remain with their respective upstream owners.
