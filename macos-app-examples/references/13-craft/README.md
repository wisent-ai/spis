# 13. Craft — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://www.craft.do/](https://www.craft.do/)  
**Motion source:** [https://www.youtube.com/watch?v=KB7NyeqUiwk](https://www.youtube.com/watch?v=KB7NyeqUiwk)  
**Upstream owner / recording owner:** Craft Docs  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `8a65a8cb860916c609fd282f87d08ddea25f7d3a056b8f896e4b327e1dfef5a1` (439018 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Craft user  
**Goal:** Create and organize a Craft task document  
**Prerequisites:** Craft available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open a Craft document | Craft advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Enter task content | Craft advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Convert or structure the content | Craft advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Use the visible task controls | Craft advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the organized document result | Craft advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open a Craft document | `media/state-01.png` | 1.44s | `d020e3c0d4b1a0d2c962f3035206bbedb561155ae4e5c33019b8e7635bd8b19a` |
| 2 | Invocation state: Enter task content | `media/state-02.png` | 5.04s | `8f07995b736cf3850768b959a821426c4890d8caabaa2415eea6c97dcb005148` |
| 3 | Focused intermediate state: Convert or structure the content | `media/state-03.png` | 8.64s | `4e26138bf64e6db20e188ed10091f58b6983c09799e4d0245443ea2cb772c0e6` |
| 4 | Committed transition: Use the visible task controls | `media/state-04.png` | 12.24s | `2666eee61a039f274df82a2089ff2a7d86204d39c42907a3823bfa0d94e2a663` |
| 5 | First-success result: Observe the organized document result | `media/state-05.png` | 15.84s | `485b1ca74ac4e4f6313511a5c49179a7c1f792913088d65bd1799d3f7fdb71bf` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open a Craft document | Craft exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Enter task content. |
| Focus and selection | Enter task content | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Convert or structure the content | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Use the visible task controls. |
| Confirmation | Use the visible task controls | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the organized document result | The recording reaches the first meaningful result for “Create and organize a Craft task document”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open a Craft document.
- **Start state:** Open a Craft document.
- **End state:** Observe the organized document result.
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

- **Product page:** https://www.craft.do/
- **Original motion:** https://www.youtube.com/watch?v=KB7NyeqUiwk
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 439018 bytes
- **SHA-256:** `8a65a8cb860916c609fd282f87d08ddea25f7d3a056b8f896e4b327e1dfef5a1`
- **Ownership:** Craft Docs. Product and recording rights remain with their respective upstream owners.
