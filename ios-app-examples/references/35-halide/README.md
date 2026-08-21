# Halide — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://halide.cam/](https://halide.cam/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Halide product owner; recording published by Halide – The best pro camera for iPhone & iPad

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [FAST ProRAW on iPhone with Halide Mark II](https://www.youtube.com/watch?v=dIeBiSknb0c)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 14.467 s, 217 frames, 300779 bytes
- SHA-256: `c90d9660ba4fc4bb15cf48d5cdafb7481212db98d9aab648acf85b59431f6cc0`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `82c95d4e8fc0c67b25dc685afa61c13febac60cb2ebd11f2bee1862e9049f8d0` |
| action in progress | [media/state-02.png](media/state-02.png) | `491c9eb1fb06f43ec5610c0c5eabdd3af8bdb63e8918149118753ae5806982f0` |
| result / established state | [media/state-03.png](media/state-03.png) | `887647241dc4d0aa39eaff04b20cab7ef8fb8c998d621e721ef8f76b0369ff82` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Capture with Halide

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Halide camera** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:00.7`
2. **select exposure or focus controls** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:03.6`
3. **adjust the capture settings** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:06.5`
4. **press the shutter** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:09.4`
5. **see capture feedback and review** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:12.3`

**Completion evidence:** `media/motion.mp4 at 00:12.3 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:04.6–00:08.4`
- Recovery route: Continue from the retained incomplete state and press the shutter. The confirmation transition resumes from the same observed flow. Then see capture feedback and review; The product shows the documented first-success result: see capture feedback and review. Evidence: `media/motion.mp4 00:09.8–00:13.6`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | select exposure or focus controls | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:01.7–00:04.1` |
| focus / selection | adjust the capture settings | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:03.8–00:06.4` |
| navigation | Open Halide camera | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.6–00:03.5` |
| confirmation | press the shutter | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:08.4–00:11.0` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:05.5–00:08.4` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:09.0–00:12.7` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:04.6–00:08.0` |
| recovery | Continue from the incomplete state and press the shutter. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:07.5–00:13.3` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** select exposure or focus controls
- **Start → end:** entry / orientation → result / established state
- **Continuity:** The recording retains the real product transition sequence; transcoding changes encoding only and does not synthesize intermediate frames.
- **Timing:** short guided sequence
- **Interruption / reversal:** The retained pre-confirmation state supplies the reversible boundary: withholding or reversing commit leaves the result absent; continuing through confirmation reaches the end state. Exact gesture-interruption semantics are unknown unless visibly demonstrated.
- **Feedback:** Selection emphasis, surface transition, content change, or result placement provides visible acknowledgment.
- **Reduced motion / nonanimated equivalent:** Unknown from the retained source; no reduced-motion claim is inferred.

## Accessibility

Observed:
- Visible text labels and state changes supplement iconography in the retained recording.
- Selection, progress, or completion is communicated by a changed product state rather than motion alone.
- The retained frames preserve the visual context before, during, and after the principal action.

Unknown from this evidence:
- VoiceOver names, hints, rotor order, and focus return were not exposed by the source recording.
- Dynamic Type behavior and text truncation at accessibility sizes were not exposed.
- Reduce Motion behavior and a nonanimated equivalent were not exposed.
- Switch Control, keyboard navigation, contrast ratios, and haptic/audio-only feedback were not measured.

## Provenance

The source URL, local path, capture method, dimensions, duration, frame count, byte size, SHA-256, capture date, and upstream ownership are recorded in [`reference.json`](reference.json). The three state images are direct frames from the local motion asset.
