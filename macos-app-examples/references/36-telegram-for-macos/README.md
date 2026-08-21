# 36. Telegram for macOS — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://macos.telegram.org/](https://macos.telegram.org/)  
**Motion source:** [https://www.youtube.com/watch?v=nwFGP37pEkc](https://www.youtube.com/watch?v=nwFGP37pEkc)  
**Upstream owner / recording owner:** Pixelfriedhof real-product comparison; Telegram FZ-LLC  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `ff47975e3eaf3a0a008d7d0ffcf1543ac599803527c7ceef3ae8327eeb8adb2e` (626780 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Telegram for macOS user  
**Goal:** Install and reach Telegram for macOS  
**Prerequisites:** Telegram for macOS available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the official download page | Telegram for macOS advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Start the macOS download | Telegram for macOS advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Open the downloaded installer or image | Telegram for macOS advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Move Telegram into the install destination | Telegram for macOS advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Return to the product entry point ready to launch | Telegram for macOS advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open the official download page | `media/state-01.png` | 1.44s | `df77eb6bd6db5b9c3e6e5a52b61c15ed6a7d484064dee808dc3bb1c3397304db` |
| 2 | Invocation state: Start the macOS download | `media/state-02.png` | 5.04s | `94d2dd7ebee39a0924df6e8ea004868ba5aa104b8ae10d51d4e7c7338bf67e51` |
| 3 | Focused intermediate state: Open the downloaded installer or image | `media/state-03.png` | 8.64s | `e7798352103ea731014f54cfae507734638ec78bb54327546c31bd22dbd78ee7` |
| 4 | Committed transition: Move Telegram into the install destination | `media/state-04.png` | 12.24s | `eea8285a4a783c93039c8b0b8553ac35a1d7a648cb98b8781208d272dceae9b8` |
| 5 | First-success result: Return to the product entry point ready to launch | `media/state-05.png` | 15.84s | `cdacec52cce49ff2ad210efeb3f5be97a240486f94e3552a9c4e3ca36f9c0815` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the official download page | Telegram for macOS exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Start the macOS download. |
| Focus and selection | Start the macOS download | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Open the downloaded installer or image | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Move Telegram into the install destination. |
| Confirmation | Move Telegram into the install destination | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Return to the product entry point ready to launch | The recording reaches the first meaningful result for “Install and reach Telegram for macOS”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open the official download page.
- **Start state:** Open the official download page.
- **End state:** Return to the product entry point ready to launch.
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

- **Product page:** https://macos.telegram.org/
- **Original motion:** https://www.youtube.com/watch?v=nwFGP37pEkc
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 626780 bytes
- **SHA-256:** `ff47975e3eaf3a0a008d7d0ffcf1543ac599803527c7ceef3ae8327eeb8adb2e`
- **Ownership:** Pixelfriedhof real-product comparison; Telegram FZ-LLC. Product and recording rights remain with their respective upstream owners.
