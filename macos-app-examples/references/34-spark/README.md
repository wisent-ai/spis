# 34. Spark — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://sparkmailapp.com/](https://sparkmailapp.com/)  
**Motion source:** [https://www.youtube.com/watch?v=Wn0wuFWuLpY](https://www.youtube.com/watch?v=Wn0wuFWuLpY)  
**Upstream owner / recording owner:** Readdle / Spark  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `48b2cd971593aa08f5581e958b820dc13d0722249f3bdfd2781392f1e46d1429` (117657 bytes, 960×600, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Spark user  
**Goal:** Open and act on email in Spark  
**Prerequisites:** Spark available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Spark’s inbox | Spark advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Select a message or category | Spark advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Review the message surface | Spark advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Invoke a mail action | Spark advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the mailbox update | Spark advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Spark’s inbox | `media/state-01.png` | 1.44s | `6e7e3b43c4db4db430479da69c4bdb47f7f7c1023dfcd31a8877968c6d5cc0cd` |
| 2 | Invocation state: Select a message or category | `media/state-02.png` | 5.04s | `b75ecdef7469551397f782a9f636c5ef5b30c0abfc05565d1223804cc805ac80` |
| 3 | Focused intermediate state: Review the message surface | `media/state-03.png` | 8.64s | `63f4fed72f175dabcf0c93c541867e62aadb02010465b34fc41e185cf4ac2d57` |
| 4 | Committed transition: Invoke a mail action | `media/state-04.png` | 12.24s | `619f9ea851b75a34044ed9ee3feb0be5ec67838a7b294c4fb0f035f188fdb817` |
| 5 | First-success result: Observe the mailbox update | `media/state-05.png` | 15.84s | `36d015aa565f94bfef196c8ecef4975f0fcae50b530e530f45e08d7ee2ece312` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Spark’s inbox | Spark exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Select a message or category. |
| Focus and selection | Select a message or category | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Review the message surface | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Invoke a mail action. |
| Confirmation | Invoke a mail action | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the mailbox update | The recording reaches the first meaningful result for “Open and act on email in Spark”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Spark’s inbox.
- **Start state:** Open Spark’s inbox.
- **End state:** Observe the mailbox update.
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

- **Product page:** https://sparkmailapp.com/
- **Original motion:** https://www.youtube.com/watch?v=Wn0wuFWuLpY
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×600; 18.000s; 270 frames; 117657 bytes
- **SHA-256:** `48b2cd971593aa08f5581e958b820dc13d0722249f3bdfd2781392f1e46d1429`
- **Ownership:** Readdle / Spark. Product and recording rights remain with their respective upstream owners.
