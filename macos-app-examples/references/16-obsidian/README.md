# 16. Obsidian — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://obsidian.md/](https://obsidian.md/)  
**Motion source:** [https://www.youtube.com/watch?v=_QFUOyIB1nY](https://www.youtube.com/watch?v=_QFUOyIB1nY)  
**Upstream owner / recording owner:** Obsidian  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `9fe6c2307b18c1c09047b3614baf5ed12e6967b0963adc353c74102fec9e84ba` (108559 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Obsidian user  
**Goal:** Create and inspect an Obsidian note or plugin view  
**Prerequisites:** Obsidian available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the vault workspace | Obsidian advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Select a file or command | Obsidian advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Edit the note or configuration | Obsidian advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Review the live rendered result | Obsidian advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the completed note or view | Obsidian advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open the vault workspace | `media/state-01.png` | 1.44s | `47d9b8a0f7038c556a3e47e3b9e9273d4f511992f63536b9d982b5885340d0c5` |
| 2 | Invocation state: Select a file or command | `media/state-02.png` | 5.04s | `6a6279e7b60bcf27bb3aff0da7ef96988848b53f4db0e365fe67581f1696a32a` |
| 3 | Focused intermediate state: Edit the note or configuration | `media/state-03.png` | 8.64s | `52a98dfc087edcb1bef1f5c901b6aae0cf38713c0bab34bbd26d0436460e665a` |
| 4 | Committed transition: Review the live rendered result | `media/state-04.png` | 12.24s | `79944ba9539a3873563539ccd891b8c185df57563e5d6fda8dce76e87125672a` |
| 5 | First-success result: Observe the completed note or view | `media/state-05.png` | 15.84s | `1d5e8b1674e144d3db2a62ecb2de4cf6c80ace6aa2543ea5f784606f22306ff0` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the vault workspace | Obsidian exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Select a file or command. |
| Focus and selection | Select a file or command | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Edit the note or configuration | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Review the live rendered result. |
| Confirmation | Review the live rendered result | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the completed note or view | The recording reaches the first meaningful result for “Create and inspect an Obsidian note or plugin view”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open the vault workspace.
- **Start state:** Open the vault workspace.
- **End state:** Observe the completed note or view.
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

- **Product page:** https://obsidian.md/
- **Original motion:** https://www.youtube.com/watch?v=_QFUOyIB1nY
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 108559 bytes
- **SHA-256:** `9fe6c2307b18c1c09047b3614baf5ed12e6967b0963adc353c74102fec9e84ba`
- **Ownership:** Obsidian. Product and recording rights remain with their respective upstream owners.
