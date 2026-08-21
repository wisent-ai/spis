# Strava — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.strava.com/mobile](https://www.strava.com/mobile)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Strava product owner; recording published by Strava

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [How to Use Strava: Upload an activity and dive into the data](https://www.youtube.com/watch?v=XMgOkAxCyn4)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 29.000 s, 435 frames, 141414 bytes
- SHA-256: `2c1f113729b94fbe3fd0dbbbf03918ee82d25a7d2e1981539ad6312dc308fd2c`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `3f9d629f5b111f11b7308f8274000b8b7d2679729d146b86a636b9b8b44ed5ee` |
| action in progress | [media/state-02.png](media/state-02.png) | `22cd77d52442d05d99b542398f055a4e4bd7e1a38456dbece0407412d472639c` |
| result / established state | [media/state-03.png](media/state-03.png) | `ec5a074cb72c498b323f3aec0378b0c649e08c5d4dc2847af575d3317236ab7e` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Upload and inspect a Strava activity

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Strava** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:01.5`
2. **choose an activity upload** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:07.2`
3. **select the activity data** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:13.1`
4. **save the upload** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:18.9`
5. **inspect the completed activity analysis** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:24.6`

**Completion evidence:** `media/motion.mp4 at 00:24.6 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:09.3–00:16.8`
- Recovery route: Continue from the retained incomplete state and save the upload. The confirmation transition resumes from the same observed flow. Then inspect the completed activity analysis; The product shows the documented first-success result: inspect the completed activity analysis. Evidence: `media/motion.mp4 00:19.7–00:27.3`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose an activity upload | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:03.5–00:08.1` |
| focus / selection | select the activity data | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:07.5–00:12.8` |
| navigation | Open Strava | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:01.2–00:07.0` |
| confirmation | save the upload | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:16.8–00:22.0` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:11.0–00:16.8` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:18.0–00:25.5` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:09.3–00:16.0` |
| recovery | Continue from the incomplete state and save the upload. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:15.1–00:26.7` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose an activity upload
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
