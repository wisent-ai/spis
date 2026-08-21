# 31. ScreenFlow — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.telestream.net/screenflow/overview.htm](https://www.telestream.net/screenflow/overview.htm)  
**Motion source:** [https://player.vimeo.com/video/561608818](https://player.vimeo.com/video/561608818)  
**Upstream owner / recording owner:** Telestream / ScreenFlow  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `f15dfc89801326d48c45fce3b9aa4639929a7bc2103c9135edce37739b2b75f8` (84930 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning ScreenFlow user  
**Goal:** Create a ScreenFlow recording project  
**Prerequisites:** ScreenFlow available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open ScreenFlow | ScreenFlow advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose the capture or project action | ScreenFlow advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Configure recording inputs | ScreenFlow advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Start or confirm the capture setup | ScreenFlow advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the project/capture workspace | ScreenFlow advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open ScreenFlow | `media/state-01.png` | 1.44s | `28c31aba45cc82910313a97527fedfe6be810b4d75aa928d3793c60715c438a9` |
| 2 | Invocation state: Choose the capture or project action | `media/state-02.png` | 5.04s | `8c030f2b7507a220ef1038b731207b439a119979f01199a437f7126199158928` |
| 3 | Focused intermediate state: Configure recording inputs | `media/state-03.png` | 8.64s | `068744c930461aa3014bdb69dd329c83e2bdcc7da4389fb4fed3cc4488bc4886` |
| 4 | Committed transition: Start or confirm the capture setup | `media/state-04.png` | 12.24s | `4952ca7de9291a23f4a3e37bd8f484c71b1a841325eff1a8503fea6c6655e036` |
| 5 | First-success result: Observe the project/capture workspace | `media/state-05.png` | 15.84s | `083e749805e11993f13fb91c826694cbd06ecf6a12f16fcfc869e99da3e5f496` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open ScreenFlow | ScreenFlow exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose the capture or project action. |
| Focus and selection | Choose the capture or project action | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Configure recording inputs | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Start or confirm the capture setup. |
| Confirmation | Start or confirm the capture setup | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the project/capture workspace | The recording reaches the first meaningful result for “Create a ScreenFlow recording project”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open ScreenFlow.
- **Start state:** Open ScreenFlow.
- **End state:** Observe the project/capture workspace.
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

- **Product page:** https://www.telestream.net/screenflow/overview.htm
- **Original motion:** https://player.vimeo.com/video/561608818
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 84930 bytes
- **SHA-256:** `f15dfc89801326d48c45fce3b9aa4639929a7bc2103c9135edce37739b2b75f8`
- **Ownership:** Telestream / ScreenFlow. Product and recording rights remain with their respective upstream owners.
