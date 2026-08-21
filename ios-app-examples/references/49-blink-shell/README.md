# Blink Shell — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://blink.sh/](https://blink.sh/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Blink Shell product owner; recording published by Blink Shell, Build & Code

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [SSH and Mosh on iOS: First time access to remote machines](https://www.youtube.com/watch?v=cnfNDjIokvw)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 360×516, 81.933 s, 1229 frames, 834647 bytes
- SHA-256: `d47e4df83033da319b2a13869c78487ccd4a8d87706a9b541e0501c93bc6df15`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `9d7396d060cc7ee0ab75aa3751ff8501fdddca5f36425901e80a6ae6fc9dc94a` |
| action in progress | [media/state-02.png](media/state-02.png) | `e32dcfc3a3805ed8076cc5630c254d8c126bb9e093f6a02a4a302c48db12f3b2` |
| result / established state | [media/state-03.png](media/state-03.png) | `ddad8a186240f23fd55e005c79c730c425bc3baaf2127928004266042158bbb2` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Connect to a remote host in Blink Shell

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Blink Shell** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:04.1`
2. **create the first host connection** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:20.5`
3. **enter SSH/Mosh host and credentials** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:36.9`
4. **connect** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:53.3`
5. **see the remote shell prompt** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:09.6`

**Completion evidence:** `media/motion.mp4 at 01:09.6 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:26.2–00:47.5`
- Recovery route: Continue from the retained incomplete state and connect. The confirmation transition resumes from the same observed flow. Then see the remote shell prompt; The product shows the documented first-success result: see the remote shell prompt. Evidence: `media/motion.mp4 00:55.7–01:17.0`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | create the first host connection | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:09.8–00:22.9` |
| focus / selection | enter SSH/Mosh host and credentials | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:21.3–00:36.1` |
| navigation | Open Blink Shell | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:03.3–00:19.7` |
| confirmation | connect | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:47.5–01:02.3` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:31.1–00:47.5` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:50.8–01:12.1` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:26.2–00:45.1` |
| recovery | Continue from the incomplete state and connect. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:42.6–01:15.4` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** create the first host connection
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
