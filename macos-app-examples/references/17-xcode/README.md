# 17. Xcode — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://developer.apple.com/xcode/](https://developer.apple.com/xcode/)  
**Motion source:** [https://www.youtube.com/watch?v=XapwQYZwmic](https://www.youtube.com/watch?v=XapwQYZwmic)  
**Upstream owner / recording owner:** Apple Developer  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `c06b10237ea861b343fde979ad2fd4c7278b1eaacf514fbee339b44e7d19c127` (116522 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Xcode user  
**Goal:** Move from an app idea to an Xcode development path  
**Prerequisites:** Xcode available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Start from the app-development entry point | Xcode advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose the setup path | Xcode advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Move into the development stage | Xcode advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Select the build or modeling step | Xcode advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the path toward a runnable app | Xcode advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Start from the app-development entry point | `media/state-01.png` | 1.44s | `15d277c63e07f0b26a3d38f53926182ebca992069ba1734c4d8dc8e3adaa4c88` |
| 2 | Invocation state: Choose the setup path | `media/state-02.png` | 5.04s | `4d649b468df819aecc6d494321fec8209b0b23abb9ba0f3d7f627ba62feed307` |
| 3 | Focused intermediate state: Move into the development stage | `media/state-03.png` | 8.64s | `e537ec3171000c6048623f642f601a736a55a82e674df25b53239b9407695333` |
| 4 | Committed transition: Select the build or modeling step | `media/state-04.png` | 12.24s | `2fc813e1b9066a6381f3e43bbaa8acd14763223e98225402c266e1670da541c3` |
| 5 | First-success result: Observe the path toward a runnable app | `media/state-05.png` | 15.84s | `63c3041716690a65685f81bcf744645809f0b4291a073870857b4aba1faf1c9d` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Start from the app-development entry point | Xcode exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose the setup path. |
| Focus and selection | Choose the setup path | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Move into the development stage | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Select the build or modeling step. |
| Confirmation | Select the build or modeling step | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the path toward a runnable app | The recording reaches the first meaningful result for “Move from an app idea to an Xcode development path”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Start from the app-development entry point.
- **Start state:** Start from the app-development entry point.
- **End state:** Observe the path toward a runnable app.
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

- **Product page:** https://developer.apple.com/xcode/
- **Original motion:** https://www.youtube.com/watch?v=XapwQYZwmic
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 116522 bytes
- **SHA-256:** `c06b10237ea861b343fde979ad2fd4c7278b1eaacf514fbee339b44e7d19c127`
- **Ownership:** Apple Developer. Product and recording rights remain with their respective upstream owners.
