# 41. Base — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://menial.co.uk/base/](https://menial.co.uk/base/)  
**Motion source:** [https://www.youtube.com/watch?v=wNJ682XfzwU](https://www.youtube.com/watch?v=wNJ682XfzwU)  
**Upstream owner / recording owner:** Jimmy a Geek recording; Menial / Base  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `13be7e16cd8f4239dbf6f8169f89cbebe541c6b79ad69b1b6b79b9040457c4ba` (419672 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Base user  
**Goal:** Open and browse a SQLite database  
**Prerequisites:** Base available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Base | Base advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose a SQLite file | Base advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Select a database table | Base advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Inspect its fields or rows | Base advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the browsable database result | Base advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Base | `media/state-01.png` | 1.44s | `806ff3e3d7839ec54b413ec9bec0819410fa89fa6904f4bdb92b74492db79325` |
| 2 | Invocation state: Choose a SQLite file | `media/state-02.png` | 5.04s | `570ae331632559ef89979ca382ad3c815c2be677fe5358afe07aa757223de13c` |
| 3 | Focused intermediate state: Select a database table | `media/state-03.png` | 8.64s | `a4da1de745516c5e8f033fa2d0f9d492327054669a659e6777581fc8f0318043` |
| 4 | Committed transition: Inspect its fields or rows | `media/state-04.png` | 12.24s | `c1cb62ceaea873efe9dc75225171bc8ea8a9d578861b3be4057efb03a06d5409` |
| 5 | First-success result: Observe the browsable database result | `media/state-05.png` | 15.84s | `abcc14f1fea76a7914c8a51f74ffc2f6753d77ad85ff2eae0b1e0d82ecc173de` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Base | Base exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose a SQLite file. |
| Focus and selection | Choose a SQLite file | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Select a database table | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Inspect its fields or rows. |
| Confirmation | Inspect its fields or rows | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the browsable database result | The recording reaches the first meaningful result for “Open and browse a SQLite database”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Base.
- **Start state:** Open Base.
- **End state:** Observe the browsable database result.
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

- **Product page:** https://menial.co.uk/base/
- **Original motion:** https://www.youtube.com/watch?v=wNJ682XfzwU
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 419672 bytes
- **SHA-256:** `13be7e16cd8f4239dbf6f8169f89cbebe541c6b79ad69b1b6b79b9040457c4ba`
- **Ownership:** Jimmy a Geek recording; Menial / Base. Product and recording rights remain with their respective upstream owners.
