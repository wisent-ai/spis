# GitHub Mobile — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://github.com/mobile](https://github.com/mobile)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** GitHub Mobile product owner; recording published by GitHub

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [What is GitHub Mobile?](https://www.youtube.com/watch?v=ObPdcm6jWoQ)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 73.067 s, 1096 frames, 1532640 bytes
- SHA-256: `3ca957c438eaa232ba0733954d307190d728ffd25d3e6e38a2311664d0e87981`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `af1f37cd42f7fce92f84aa0355ddb4a80164f65d68bc559630f1cc20a2d58477` |
| action in progress | [media/state-02.png](media/state-02.png) | `82f8717eb58db1f0bbdcdb6c06f380d2c1c91d9ef4d6ef5327510519734de5cb` |
| result / established state | [media/state-03.png](media/state-03.png) | `5dae18d3a8ad95688ce7218b3775d4b9477dd4b33a51cb5eafdeb91c5f69ce12` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Use GitHub Mobile on a repository

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open GitHub Mobile** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:03.7`
2. **select a repository or notification** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:18.3`
3. **open the work item** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:32.9`
4. **perform and confirm an action** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:47.5`
5. **see the repository state update** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:02.1`

**Completion evidence:** `media/motion.mp4 at 01:02.1 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:23.4–00:42.4`
- Recovery route: Continue from the retained incomplete state and perform and confirm an action. The confirmation transition resumes from the same observed flow. Then see the repository state update; The product shows the documented first-success result: see the repository state update. Evidence: `media/motion.mp4 00:49.7–01:08.7`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | select a repository or notification | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:08.8–00:20.5` |
| focus / selection | open the work item | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:19.0–00:32.1` |
| navigation | Open GitHub Mobile | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:02.9–00:17.5` |
| confirmation | perform and confirm an action | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:42.4–00:55.5` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:27.8–00:42.4` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:45.3–01:04.3` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:23.4–00:40.2` |
| recovery | Continue from the incomplete state and perform and confirm an action. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:38.0–01:07.2` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** select a repository or notification
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
