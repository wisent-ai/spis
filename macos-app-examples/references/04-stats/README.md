# 04. Stats — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://github.com/exelban/stats](https://github.com/exelban/stats)  
**Motion source:** [https://www.youtube.com/watch?v=EiYJ8GKqaqw](https://www.youtube.com/watch?v=EiYJ8GKqaqw)  
**Upstream owner / recording owner:** STÆMPUNK TV recording; Stats by exelban  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `0324030d7722bcb6dee2dbf1b190ac9bb72ce36c8624690f178fd3327020af47` (156775 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Stats user  
**Goal:** Inspect a live system metric  
**Prerequisites:** Stats available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the Stats menu-bar module | Stats advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose a metric module | Stats advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Review its expanded readings | Stats advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Adjust or navigate the module controls | Stats advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the selected live metric view | Stats advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open the Stats menu-bar module | `media/state-01.png` | 1.44s | `f0409d24320516dc94ec6b8b08d35cd0f082a8d1322ed63a15da935bf15ae4e1` |
| 2 | Invocation state: Choose a metric module | `media/state-02.png` | 5.04s | `940e943c1a705d8e4d8ca1dbaead46ce570ab3d2cb690ad2b6ed399eef853d8b` |
| 3 | Focused intermediate state: Review its expanded readings | `media/state-03.png` | 8.64s | `1810916aa77f9f6f447a9902193e1f82e49b4e73654d7533cd522c4b107c765a` |
| 4 | Committed transition: Adjust or navigate the module controls | `media/state-04.png` | 12.24s | `60b456ca4295c43ff42f40c85793e6578ab8527b816de4a9c1b43a32f803d191` |
| 5 | First-success result: Observe the selected live metric view | `media/state-05.png` | 15.84s | `74ea62e9472274691cc72934762ae55021719c72934e00a76f3e1c87979a71c5` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the Stats menu-bar module | Stats exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose a metric module. |
| Focus and selection | Choose a metric module | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Review its expanded readings | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Adjust or navigate the module controls. |
| Confirmation | Adjust or navigate the module controls | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the selected live metric view | The recording reaches the first meaningful result for “Inspect a live system metric”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open the Stats menu-bar module.
- **Start state:** Open the Stats menu-bar module.
- **End state:** Observe the selected live metric view.
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

- **Product page:** https://github.com/exelban/stats
- **Original motion:** https://www.youtube.com/watch?v=EiYJ8GKqaqw
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 156775 bytes
- **SHA-256:** `0324030d7722bcb6dee2dbf1b190ac9bb72ce36c8624690f178fd3327020af47`
- **Ownership:** STÆMPUNK TV recording; Stats by exelban. Product and recording rights remain with their respective upstream owners.
