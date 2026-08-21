# 43. ForkLift — observed macOS product reference

**Evidence status:** complete  
**Product:** [https://binarynights.com/](https://binarynights.com/)  
**Motion source:** [https://binarynights.com/videos/remote.mp4](https://binarynights.com/videos/remote.mp4)  
**Upstream owner / recording owner:** BinaryNights / ForkLift  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized. Offline identity: `9006afb592863f451a3812f7e9fdbfb02bb2550cf8f3c7e8307c16e3cc8aef0e` (52901 bytes, 960×720, 6.933s, 104 frames).

## First-success journey

**Actor:** A first-time or returning ForkLift user  
**Goal:** Configure and use a remote connection  
**Prerequisites:** ForkLift available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open ForkLift’s connection surface | ForkLift advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 0.55s` |
| 2 | Choose a service or protocol | ForkLift advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 1.94s` |
| 3 | Enter connection options | ForkLift advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 3.33s` |
| 4 | Adjust visible connection preferences | ForkLift advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 4.71s` |
| 5 | Observe the configured remote browser | ForkLift advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 6.10s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 6.10s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Entry state: Open ForkLift’s connection surface | `media/state-01.png` | 0.55s | `6b73992d03b186ad9a50fbbbbdb9242880b29906f8c89cea677037757df41d6c` |
| 2 | Invocation state: Choose a service or protocol | `media/state-02.png` | 1.94s | `803495137b1a38de8030b8502da899b9201623b6e4f330ad3a76a06bfd6305bb` |
| 3 | Focused intermediate state: Enter connection options | `media/state-03.png` | 3.33s | `d4adba83aee14d9e898d1632b846b0397c5fb9fee7be5917a08369f5ab3c57e6` |
| 4 | Committed transition: Adjust visible connection preferences | `media/state-04.png` | 4.71s | `938ad58a6f6db8572876ef6b7dc4e36674a2ea6a896023e7a8c4b57fb834d5cc` |
| 5 | First-success result: Observe the configured remote browser | `media/state-05.png` | 6.10s | `00dabf7c09d09b030a67c81057a86dcc532189155dc26942f04f8255ade0cf2e` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open ForkLift’s connection surface | ForkLift exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose a service or protocol. |
| Focus and selection | Choose a service or protocol | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Enter connection options | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Adjust visible connection preferences. |
| Confirmation | Adjust visible connection preferences | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the configured remote browser | The recording reaches the first meaningful result for “Configure and use a remote connection”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** Open ForkLift’s connection surface.
- **Start state:** Open ForkLift’s connection surface.
- **End state:** Observe the configured remote browser.
- **Continuity:** the MP4 preserves recorded temporal order; the five PNGs are decoded directly from it.
- **Timing class:** brief native animation (6.933s retained).
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

- **Product page:** https://binarynights.com/
- **Original motion:** https://binarynights.com/videos/remote.mp4
- **Capture method:** Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×720; 6.933s; 104 frames; 52901 bytes
- **SHA-256:** `9006afb592863f451a3812f7e9fdbfb02bb2550cf8f3c7e8307c16e3cc8aef0e`
- **Ownership:** BinaryNights / ForkLift. Product and recording rights remain with their respective upstream owners.
