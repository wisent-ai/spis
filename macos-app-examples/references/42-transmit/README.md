# 42. Transmit — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://panic.com/transmit/](https://panic.com/transmit/)  
**Motion source:** [https://apptrailers.itunes.apple.com/itunes-assets/PurpleVideo124/v4/b7/5c/db/b75cdb66-b015-c6ee-31b3-bc7a27352bd0/P210923183_default.m3u8](https://apptrailers.itunes.apple.com/itunes-assets/PurpleVideo124/v4/b7/5c/db/b75cdb66-b015-c6ee-31b3-bc7a27352bd0/P210923183_default.m3u8)  
**Upstream owner / recording owner:** Panic, Inc. / Apple App Store  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized. Offline identity: `4c16b1cd895d772e98a10622111b3805e82c71e2b5484573d414422685e5c2b3` (49246 bytes, 960×540, 4.867s, 73 frames).

## First-success journey

**Actor:** A first-time or returning Transmit user  
**Goal:** Connect to a remote server  
**Prerequisites:** Transmit available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Transmit’s server setup | Transmit advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 0.39s` |
| 2 | Choose a protocol | Transmit advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 1.36s` |
| 3 | Enter server credentials and path | Transmit advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 2.34s` |
| 4 | Confirm Connect | Transmit advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 3.31s` |
| 5 | Observe the remote file browser | Transmit advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 4.28s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 4.28s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Transmit’s server setup | `media/state-01.png` | 0.39s | `6ef843fbacbfd8ef877ae27dfac109186e4efb11337e6e6aeb96a04419334691` |
| 2 | Invocation state: Choose a protocol | `media/state-02.png` | 1.36s | `1415a90a9a0b2cc16f69b92955d1a1906aa4ba559a7e3554166d7ec22d61203c` |
| 3 | Focused intermediate state: Enter server credentials and path | `media/state-03.png` | 2.34s | `15faaeea7ce1109299181899ae046ab95e6d5960ed117c17a0a276f31d86cf66` |
| 4 | Committed transition: Confirm Connect | `media/state-04.png` | 3.31s | `a280795c921a8004f4427719ba570c8142daca7fbc7778d7fa2b9798dc7ca0c0` |
| 5 | First-success result: Observe the remote file browser | `media/state-05.png` | 4.28s | `c249474984f8ebb33246199dd3d2742f5f4c27bf3aa4a2aec720f729287d1d95` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Transmit’s server setup | Transmit exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose a protocol. |
| Focus and selection | Choose a protocol | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Enter server credentials and path | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm Connect. |
| Confirmation | Confirm Connect | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the remote file browser | The recording reaches the first meaningful result for “Connect to a remote server”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Transmit’s server setup.
- **Start state:** Open Transmit’s server setup.
- **End state:** Observe the remote file browser.
- **Continuity:** the MP4 preserves recorded temporal order; the five PNGs are decoded directly from it.
- **Timing class:** brief native animation (4.867s retained).
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

- **Product page:** https://panic.com/transmit/
- **Original motion:** https://apptrailers.itunes.apple.com/itunes-assets/PurpleVideo124/v4/b7/5c/db/b75cdb66-b015-c6ee-31b3-bc7a27352bd0/P210923183_default.m3u8
- **Capture method:** Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 4.867s; 73 frames; 49246 bytes
- **SHA-256:** `4c16b1cd895d772e98a10622111b3805e82c71e2b5484573d414422685e5c2b3`
- **Ownership:** Panic, Inc. / Apple App Store. Product and recording rights remain with their respective upstream owners.
