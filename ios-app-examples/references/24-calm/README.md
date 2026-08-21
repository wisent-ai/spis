# Calm — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.calm.com/](https://www.calm.com/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Calm product owner; recording published by Calm

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [The Calm App](https://www.youtube.com/watch?v=fTQ9CRl_XPM)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 76.400 s, 1146 frames, 1350847 bytes
- SHA-256: `0b70f2e8a0e7263fd8815bb4cd5e1db115a0406aa30c944ddcfaae6b4e9a0af5`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `9d4bfb4c5679547a01f79635172bc98202f75995f1b2dca2896cea9d19f19aea` |
| action in progress | [media/state-02.png](media/state-02.png) | `014bf499a31f87b957a0da8c0a196690b4145fcaedb95bc7e8a523b07746fb1c` |
| result / established state | [media/state-03.png](media/state-03.png) | `9eec23f4d0a6257cef32eed68f8916738de0f7c5b87fe91f2e2405f38302aada` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Begin a Calm session

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Calm** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:03.8`
2. **browse a goal or content collection** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:19.1`
3. **choose a session** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:34.4`
4. **start the session** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:49.7`
5. **see the player and progress state** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:04.9`

**Completion evidence:** `media/motion.mp4 at 01:04.9 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:24.4–00:44.3`
- Recovery route: Continue from the retained incomplete state and start the session. The confirmation transition resumes from the same observed flow. Then see the player and progress state; The product shows the documented first-success result: see the player and progress state. Evidence: `media/motion.mp4 00:52.0–01:11.8`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | browse a goal or content collection | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:09.2–00:21.4` |
| focus / selection | choose a session | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:19.9–00:33.6` |
| navigation | Open Calm | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:03.1–00:18.3` |
| confirmation | start the session | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:44.3–00:58.1` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:29.0–00:44.3` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:47.4–01:07.2` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:24.4–00:42.0` |
| recovery | Continue from the incomplete state and start the session. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:39.7–01:10.3` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** browse a goal or content collection
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
