# 46. iTerm2 — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://iterm2.com/](https://iterm2.com/)  
**Motion source:** [https://www.youtube.com/watch?v=Ut9dOVxVdhs](https://www.youtube.com/watch?v=Ut9dOVxVdhs)  
**Upstream owner / recording owner:** Outside Open recording; George Nachman / iTerm2  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `6efb989bee640b7dbba16b50cf2c3d52b2b3263f12d1264f96bfdf9c0a27cb89` (345313 bytes, 960×590, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning iTerm2 user  
**Goal:** Run a command in iTerm2  
**Prerequisites:** iTerm2 available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open an iTerm2 session | iTerm2 advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Focus the command prompt | iTerm2 advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Enter a command | iTerm2 advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Execute it | iTerm2 advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the terminal output | iTerm2 advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open an iTerm2 session | `media/state-01.png` | 1.44s | `69676e670c5225c834997b3e3567dabf430ad9415b011ea640ccf9fd4b6c441c` |
| 2 | Invocation state: Focus the command prompt | `media/state-02.png` | 5.04s | `9783c4615797b6a473594662d77ce172468ded287a105dba0062c6a795d5c1b7` |
| 3 | Focused intermediate state: Enter a command | `media/state-03.png` | 8.64s | `8ea932561579b05653e47c0870cade1973fac13f2fb574cb8d45d0481f94bfb2` |
| 4 | Committed transition: Execute it | `media/state-04.png` | 12.24s | `a8374c9fccdc07ea7aa36ebacb648641188b192b34c0f39a82df2fdf8be342f1` |
| 5 | First-success result: Observe the terminal output | `media/state-05.png` | 15.84s | `639e288634e17510bd96951a19eb7b7f674ae5d4456a3eed6bee106e96c60840` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open an iTerm2 session | iTerm2 exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Focus the command prompt. |
| Focus and selection | Focus the command prompt | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Enter a command | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Execute it. |
| Confirmation | Execute it | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the terminal output | The recording reaches the first meaningful result for “Run a command in iTerm2”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open an iTerm2 session.
- **Start state:** Open an iTerm2 session.
- **End state:** Observe the terminal output.
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

- **Product page:** https://iterm2.com/
- **Original motion:** https://www.youtube.com/watch?v=Ut9dOVxVdhs
- **Capture method:** Downloaded as a real-product tutorial recording with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×590; 18.000s; 270 frames; 345313 bytes
- **SHA-256:** `6efb989bee640b7dbba16b50cf2c3d52b2b3263f12d1264f96bfdf9c0a27cb89`
- **Ownership:** Outside Open recording; George Nachman / iTerm2. Product and recording rights remain with their respective upstream owners.
