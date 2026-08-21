# 23. Dash — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://kapeli.com/dash](https://kapeli.com/dash)  
**Motion source:** [https://www.youtube.com/watch?v=nwMfx8h0kdQ](https://www.youtube.com/watch?v=nwMfx8h0kdQ)  
**Upstream owner / recording owner:** Kapeli / Dash  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `bee84010491d629a74123d4792db2ed7c5b5b5451637ae6d11ddb27d98f58b69` (205602 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Dash user  
**Goal:** Find API documentation  
**Prerequisites:** Dash available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Focus Dash search | Dash advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Type a symbol or API query | Dash advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Review matching docset results | Dash advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Choose a class or symbol | Dash advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe its documentation page | Dash advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Focus Dash search | `media/state-01.png` | 1.44s | `6e4f6dedc22b8d1edea56cc848dcb8d6af169ad72f13ebcccbf94cd4d5a54924` |
| 2 | Invocation state: Type a symbol or API query | `media/state-02.png` | 5.04s | `4703ea1bed47d8327388a7b85c9b414744e77d975a2e67134fd697fa1a93642b` |
| 3 | Focused intermediate state: Review matching docset results | `media/state-03.png` | 8.64s | `e7e3804e9b7454d1f89fc1656311384d13b30666299c769b0db42727d7b81785` |
| 4 | Committed transition: Choose a class or symbol | `media/state-04.png` | 12.24s | `adbcb7c76dcb0ba35e0594ee7fb5df491f1c3576abad0666545ef1ab53d69bad` |
| 5 | First-success result: Observe its documentation page | `media/state-05.png` | 15.84s | `2df366b912fe2271e1e0511e9db5a6e09fd148088980448036904d6750ac66dd` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Focus Dash search | Dash exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Type a symbol or API query. |
| Focus and selection | Type a symbol or API query | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Review matching docset results | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Choose a class or symbol. |
| Confirmation | Choose a class or symbol | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe its documentation page | The recording reaches the first meaningful result for “Find API documentation”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Focus Dash search.
- **Start state:** Focus Dash search.
- **End state:** Observe its documentation page.
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

- **Product page:** https://kapeli.com/dash
- **Original motion:** https://www.youtube.com/watch?v=nwMfx8h0kdQ
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 205602 bytes
- **SHA-256:** `bee84010491d629a74123d4792db2ed7c5b5b5451637ae6d11ddb27d98f58b69`
- **Ownership:** Kapeli / Dash. Product and recording rights remain with their respective upstream owners.
