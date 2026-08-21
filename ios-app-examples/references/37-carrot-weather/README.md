# CARROT Weather — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.meetcarrot.com/weather/](https://www.meetcarrot.com/weather/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** CARROT Weather product owner; recording published by MeetCARROT

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [CARROT Weather Launch Trailer](https://www.youtube.com/watch?v=-STnUiuIhlw)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 31.267 s, 469 frames, 288279 bytes
- SHA-256: `85b50405b75ca6cb1f1890549ba08227dedcae7d398ec1ec714120da9cc9b7a0`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `2225d41865d514948cbc4e1c18cc4ae5a6b71a4d3f92199dad74cb11bc442d03` |
| action in progress | [media/state-02.png](media/state-02.png) | `5d78e44df8feb4b0c0e41960c33de14eb3906fde2ece6968d197bfe9057eaabe` |
| result / established state | [media/state-03.png](media/state-03.png) | `6ac160e51f0eb95b268f8978e4253cf91cc87dfe7655726bae0672e71a0f7f89` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Check a CARROT forecast

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open CARROT Weather** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:01.6`
2. **choose a location or forecast card** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:07.8`
3. **inspect forecast details** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:14.1`
4. **acknowledge the selected view** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:20.3`
5. **see updated weather detail** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:26.6`

**Completion evidence:** `media/motion.mp4 at 00:26.6 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:10.0–00:18.1`
- Recovery route: Continue from the retained incomplete state and acknowledge the selected view. The confirmation transition resumes from the same observed flow. Then see updated weather detail; The product shows the documented first-success result: see updated weather detail. Evidence: `media/motion.mp4 00:21.3–00:29.4`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose a location or forecast card | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:03.8–00:08.8` |
| focus / selection | inspect forecast details | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:08.1–00:13.8` |
| navigation | Open CARROT Weather | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:01.3–00:07.5` |
| confirmation | acknowledge the selected view | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:18.1–00:23.8` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:11.9–00:18.1` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:19.4–00:27.5` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:10.0–00:17.2` |
| recovery | Continue from the incomplete state and acknowledge the selected view. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:16.3–00:28.8` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose a location or forecast card
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
