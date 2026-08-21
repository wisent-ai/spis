# 40. SQLPro Studio — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.sqlprostudio.com/](https://www.sqlprostudio.com/)  
**Motion source:** [https://www.youtube.com/watch?v=syAXSSAl6iE](https://www.youtube.com/watch?v=syAXSSAl6iE)  
**Upstream owner / recording owner:** Hankinsoft Development / SQLPro  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `b55f00def3c386d3f3145334549b197089232f8e18320f3772a3887c347ee0a1` (538707 bytes, 960×562, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning SQLPro Studio user  
**Goal:** Inspect and export database rows  
**Prerequisites:** SQLPro Studio available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open a database connection | SQLPro Studio advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Select a table | SQLPro Studio advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Select the intended rows | SQLPro Studio advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Invoke the export or data action | SQLPro Studio advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the resulting operation state | SQLPro Studio advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open a database connection | `media/state-01.png` | 1.44s | `f435d4f77fe9cfa626ba0369c967a36573af805ce01e47667c983353f4fd929f` |
| 2 | Invocation state: Select a table | `media/state-02.png` | 5.04s | `10a45ee57423ad506158e1e60af8a43742a8fd0480c5b0bc4f0ebd66ba1bb716` |
| 3 | Focused intermediate state: Select the intended rows | `media/state-03.png` | 8.64s | `e55a26a7c7faeabb653f6bac97507c0bdaf32079c566ed5275caf11a1643af93` |
| 4 | Committed transition: Invoke the export or data action | `media/state-04.png` | 12.24s | `917670638e51b57feb8875f531ac39019169792955641b6dae4ace95beae78aa` |
| 5 | First-success result: Observe the resulting operation state | `media/state-05.png` | 15.84s | `fb91ef5e3f478f604e18200ac564fdf7f21537f715b0c1f7d291aa4b3d30650b` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open a database connection | SQLPro Studio exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Select a table. |
| Focus and selection | Select a table | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Select the intended rows | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Invoke the export or data action. |
| Confirmation | Invoke the export or data action | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the resulting operation state | The recording reaches the first meaningful result for “Inspect and export database rows”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open a database connection.
- **Start state:** Open a database connection.
- **End state:** Observe the resulting operation state.
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

- **Product page:** https://www.sqlprostudio.com/
- **Original motion:** https://www.youtube.com/watch?v=syAXSSAl6iE
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×562; 18.000s; 270 frames; 538707 bytes
- **SHA-256:** `b55f00def3c386d3f3145334549b197089232f8e18320f3772a3887c347ee0a1`
- **Ownership:** Hankinsoft Development / SQLPro. Product and recording rights remain with their respective upstream owners.
