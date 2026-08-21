# YouTube — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://apps.apple.com/us/app/youtube-watch-listen-stream/id544007664](https://apps.apple.com/us/app/youtube-watch-listen-stream/id544007664)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** YouTube product owner; recording published by YouTube Viewers

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [View and delete your history on the YouTube iOS app](https://www.youtube.com/watch?v=bysL2PnAt1o)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 49.333 s, 740 frames, 170251 bytes
- SHA-256: `46cb6edffba759e91bc0d5931616155c23109f67850ceb1dedbedf1736e0e0ef`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `312c6c0c09fbea55835672637613be7959d96b6408a740c2fe0e31a343e88f7b` |
| action in progress | [media/state-02.png](media/state-02.png) | `d1b9bbe8ce8d577c23bda0681d0a0e741cd866a2cc48f2ea91de58109aebd50e` |
| result / established state | [media/state-03.png](media/state-03.png) | `15ce060169128d4175c27aedc608780b799051f133d87da43c28330a2e811961` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Review and clear YouTube history

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open the YouTube iOS app** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:02.5`
2. **navigate to history** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:12.3`
3. **select a history item or menu** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:22.2`
4. **confirm deletion** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:32.1`
5. **see the updated history list** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:41.9`

**Completion evidence:** `media/motion.mp4 at 00:41.9 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:15.8–00:28.6`
- Recovery route: Continue from the retained incomplete state and confirm deletion. The confirmation transition resumes from the same observed flow. Then see the updated history list; The product shows the documented first-success result: see the updated history list. Evidence: `media/motion.mp4 00:33.5–00:46.4`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | navigate to history | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:05.9–00:13.8` |
| focus / selection | select a history item or menu | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:12.8–00:21.7` |
| navigation | Open the YouTube iOS app | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:02.0–00:11.8` |
| confirmation | confirm deletion | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:28.6–00:37.5` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:18.7–00:28.6` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:30.6–00:43.4` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:15.8–00:27.1` |
| recovery | Continue from the incomplete state and confirm deletion. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:25.7–00:45.4` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** navigate to history
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
