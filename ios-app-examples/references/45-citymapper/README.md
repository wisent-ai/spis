# Citymapper — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://citymapper.com/](https://citymapper.com/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Citymapper product owner; recording published by Citymapper

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [Citymapper on Android Wear - Google I/O 2015](https://www.youtube.com/watch?v=kJ4UyNTzIi8)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 107.867 s, 1618 frames, 1025941 bytes
- SHA-256: `5c7701cf54d739dd50f80040f2d610b8ae394e397b9f12b681e0b5201c96577f`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `941004073e582c5c6a9b122642fb6b96d96b75f80f6918379fe685ae826d9985` |
| action in progress | [media/state-02.png](media/state-02.png) | `d2901c34c281f25101fb1d7b44a5505667233a050e44331d5d4a5ab2c527c607` |
| result / established state | [media/state-03.png](media/state-03.png) | `59ebc66402e70f3fb4ced53fa71d41c9a54c89edceb10c01b5e39398d294f1b5` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Follow a Citymapper trip

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Citymapper** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:05.4`
2. **choose the trip/navigation surface** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:27.0`
3. **select the transit instruction** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:48.5`
4. **advance through guidance** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 01:10.1`
5. **see the next live travel state** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:31.7`

**Completion evidence:** `media/motion.mp4 at 01:31.7 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:34.5–01:02.6`
- Recovery route: Continue from the retained incomplete state and advance through guidance. The confirmation transition resumes from the same observed flow. Then see the next live travel state; The product shows the documented first-success result: see the next live travel state. Evidence: `media/motion.mp4 01:13.3–01:41.4`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose the trip/navigation surface | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:12.9–00:30.2` |
| focus / selection | select the transit instruction | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:28.0–00:47.5` |
| navigation | Open Citymapper | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:04.3–00:25.9` |
| confirmation | advance through guidance | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 01:02.6–01:22.0` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:41.0–01:02.6` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 01:06.9–01:34.9` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:34.5–00:59.3` |
| recovery | Continue from the incomplete state and advance through guidance. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:56.1–01:39.2` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose the trip/navigation surface
- **Start → end:** entry / orientation → result / established state
- **Continuity:** The recording retains the real product transition sequence; transcoding changes encoding only and does not synthesize intermediate frames.
- **Timing:** extended guided walkthrough
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
