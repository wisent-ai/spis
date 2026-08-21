# Dropbox — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.dropbox.com/mobile](https://www.dropbox.com/mobile)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Dropbox product owner; recording published by Dropbox

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [How the Dropbox mobile app keeps projects moving | Dropbox Tutorials | Dropbox](https://www.youtube.com/watch?v=fhqQk2vMDp0)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 59.267 s, 889 frames, 863327 bytes
- SHA-256: `98204985170ac67ad1bb908108923d2d34d77bbd2d08ebef9d9e246091f344b4`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `4f74deb82405d1b192dfa373bfa516a4c83d9336d7a468952ea34e181c615290` |
| action in progress | [media/state-02.png](media/state-02.png) | `fd6af5fa8afe03c362cdbeaef256f69f55373fff68c0e284737d131f312fb660` |
| result / established state | [media/state-03.png](media/state-03.png) | `0a81aec783d7bc9a90bf400d591e319b30d0f5f63a2703d9594db404fec83e5e` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Move a file through Dropbox

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open the mobile home/files view** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:03.0`
2. **choose upload or a file action** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:14.8`
3. **select the file and destination** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:26.7`
4. **confirm the operation** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:38.5`
5. **see the file available in Dropbox** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:50.4`

**Completion evidence:** `media/motion.mp4 at 00:50.4 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:19.0–00:34.4`
- Recovery route: Continue from the retained incomplete state and confirm the operation. The confirmation transition resumes from the same observed flow. Then see the file available in Dropbox; The product shows the documented first-success result: see the file available in Dropbox. Evidence: `media/motion.mp4 00:40.3–00:55.7`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose upload or a file action | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:07.1–00:16.6` |
| focus / selection | select the file and destination | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:15.4–00:26.1` |
| navigation | Open the mobile home/files view | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:02.4–00:14.2` |
| confirmation | confirm the operation | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:34.4–00:45.0` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:22.5–00:34.4` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:36.7–00:52.2` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:19.0–00:32.6` |
| recovery | Continue from the incomplete state and confirm the operation. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:30.8–00:54.5` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose upload or a file action
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
