# Revolut — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.revolut.com/](https://www.revolut.com/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Revolut product owner; recording published by Revolut

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [Revolut Your Way In (Transfers)](https://www.youtube.com/watch?v=YPYvZEgRagc)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 20.000 s, 300 frames, 582895 bytes
- SHA-256: `7eab849802e765396985c0b7143e10ad205e6b88b95e64a5c51c2906264ffadb`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `c33ded63bacf73dd1e3bf0361a2311036e0165b8ccd996c427fc818f0ce00054` |
| action in progress | [media/state-02.png](media/state-02.png) | `9e2f5a779c1ea57d76bddea22e9a70e7b941b067537e2e95e43eef83c1d9819b` |
| result / established state | [media/state-03.png](media/state-03.png) | `88b1115b7b395709ba432b08864f68de90f222e685a0d92ae7282f6309f9493e` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Make a Revolut transfer

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Revolut balances** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:01.0`
2. **choose Transfers** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:05.0`
3. **select recipient and amount** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:09.0`
4. **review and confirm** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:13.0`
5. **see transfer completion** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:17.0`

**Completion evidence:** `media/motion.mp4 at 00:17.0 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:06.4–00:11.6`
- Recovery route: Continue from the retained incomplete state and review and confirm. The confirmation transition resumes from the same observed flow. Then see transfer completion; The product shows the documented first-success result: see transfer completion. Evidence: `media/motion.mp4 00:13.6–00:18.8`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose Transfers | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:02.4–00:05.6` |
| focus / selection | select recipient and amount | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:05.2–00:08.8` |
| navigation | Open Revolut balances | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.8–00:04.8` |
| confirmation | review and confirm | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:11.6–00:15.2` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:07.6–00:11.6` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:12.4–00:17.6` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:06.4–00:11.0` |
| recovery | Continue from the incomplete state and review and confirm. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:10.4–00:18.4` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose Transfers
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
