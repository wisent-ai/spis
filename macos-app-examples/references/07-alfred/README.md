# 07. Alfred — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.alfredapp.com/](https://www.alfredapp.com/)  
**Motion source:** [https://www.youtube.com/watch?v=3mg5jA4uBH8](https://www.youtube.com/watch?v=3mg5jA4uBH8)  
**Upstream owner / recording owner:** Alfred App  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `90872930b67f1d11392d4ac5383975a686afbb6d87bd635ff9119b77ad678df2` (386200 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Alfred user  
**Goal:** Find and act on an item with Alfred  
**Prerequisites:** Alfred available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Invoke Alfred | Alfred advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Enter a query | Alfred advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Browse the focused results | Alfred advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Open the relevant action or workflow | Alfred advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the resulting item or action surface | Alfred advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Invoke Alfred | `media/state-01.png` | 1.44s | `92a4ba06f21826367c19745e43a20f3efaa2cd8b436ab3b0f762e9946cb39fa8` |
| 2 | Invocation state: Enter a query | `media/state-02.png` | 5.04s | `c54ee5012745daac24da957e473b0744abc44571c71e729cd9beff92204752d5` |
| 3 | Focused intermediate state: Browse the focused results | `media/state-03.png` | 8.64s | `0909f5f138c507ef174f0ed3b1383f70adbbddfcf887b48a33d2c7fda2e30b1e` |
| 4 | Committed transition: Open the relevant action or workflow | `media/state-04.png` | 12.24s | `e6ab2785c07606e901a54a80c086ba73b296dca76d5336f7be0896ef586cf157` |
| 5 | First-success result: Observe the resulting item or action surface | `media/state-05.png` | 15.84s | `f33e920d41915b5c79adc792acca5db39b1d12a58cd9dd545bca7bf734f6007e` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Invoke Alfred | Alfred exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Enter a query. |
| Focus and selection | Enter a query | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Browse the focused results | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Open the relevant action or workflow. |
| Confirmation | Open the relevant action or workflow | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the resulting item or action surface | The recording reaches the first meaningful result for “Find and act on an item with Alfred”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Invoke Alfred.
- **Start state:** Invoke Alfred.
- **End state:** Observe the resulting item or action surface.
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

- **Product page:** https://www.alfredapp.com/
- **Original motion:** https://www.youtube.com/watch?v=3mg5jA4uBH8
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 386200 bytes
- **SHA-256:** `90872930b67f1d11392d4ac5383975a686afbb6d87bd635ff9119b77ad678df2`
- **Ownership:** Alfred App. Product and recording rights remain with their respective upstream owners.
