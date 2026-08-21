# 11. Bear — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://bear.app/](https://bear.app/)  
**Motion source:** [https://player.vimeo.com/video/838679626?h=fecf494d8f](https://player.vimeo.com/video/838679626?h=fecf494d8f)  
**Upstream owner / recording owner:** Shiny Frog / Bear  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `9ef0bd9d8e641b944d6a37a38e529ea2e6cbd1eb832d3db86261a916be9a4bae` (351977 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Bear user  
**Goal:** Create a formatted Bear note  
**Prerequisites:** Bear available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open Bear’s note surface | Bear advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Create or choose a note | Bear advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Enter note content | Bear advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Apply markup, task, or style treatment | Bear advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the formatted note | Bear advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open Bear’s note surface | `media/state-01.png` | 1.44s | `96a6016100423bc6c3d8d8b08fc2e7ae041b19c01f4c024bf4737fd531aedc5a` |
| 2 | Invocation state: Create or choose a note | `media/state-02.png` | 5.04s | `c759f28cb5a790422bc1662794d258504ccf8e6732fea6711b2af424b7759401` |
| 3 | Focused intermediate state: Enter note content | `media/state-03.png` | 8.64s | `4effb067212e042b4aade5e15c2fa304629d5e0efedd6daa8ad17ef26cb04145` |
| 4 | Committed transition: Apply markup, task, or style treatment | `media/state-04.png` | 12.24s | `bc1a50d03138a2f7dca5072a039257082ce9f17163e929452c3b59692cffe59d` |
| 5 | First-success result: Observe the formatted note | `media/state-05.png` | 15.84s | `09108d74bf3555b3ba7b64ce4d998fd22712292d7afad831d5a672a2ca22d3f9` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open Bear’s note surface | Bear exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Create or choose a note. |
| Focus and selection | Create or choose a note | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Enter note content | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Apply markup, task, or style treatment. |
| Confirmation | Apply markup, task, or style treatment | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the formatted note | The recording reaches the first meaningful result for “Create a formatted Bear note”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open Bear’s note surface.
- **Start state:** Open Bear’s note surface.
- **End state:** Observe the formatted note.
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

- **Product page:** https://bear.app/
- **Original motion:** https://player.vimeo.com/video/838679626?h=fecf494d8f
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 351977 bytes
- **SHA-256:** `9ef0bd9d8e641b944d6a37a38e529ea2e6cbd1eb832d3db86261a916be9a4bae`
- **Ownership:** Shiny Frog / Bear. Product and recording rights remain with their respective upstream owners.
