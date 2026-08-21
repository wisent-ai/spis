# PayPal — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.paypal.com/us/digital-wallet](https://www.paypal.com/us/digital-wallet)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** PayPal product owner; recording published by PayPal

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [How to Use Tap to Pay on iPhone](https://www.youtube.com/watch?v=Bq0AQHnQO0w)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 30.000 s, 450 frames, 722644 bytes
- SHA-256: `a35d0bb49313d68ce1dc3e0b2e80e603d381543d455b0c0e6ff600396e2d1fc9`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `6d98c8c010e9b7feafa6b10d56c8427926dec5c3520faa059ccfea166627be39` |
| action in progress | [media/state-02.png](media/state-02.png) | `ea540f2a88b510b6b39bca0148c93a4255db46adca6ef372fe0d4e07475fd9dd` |
| result / established state | [media/state-03.png](media/state-03.png) | `50edd4f51f942304bd3587e6310ef5c590e6302f9ad0c32e4062bf0af64eab43` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Accept an in-person PayPal payment

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open PayPal Tap to Pay** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:01.5`
2. **enter the sale amount** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:07.5`
3. **present the contactless payment screen** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:13.5`
4. **confirm payment** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:19.5`
5. **see successful payment feedback** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:25.5`

**Completion evidence:** `media/motion.mp4 at 00:25.5 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:09.6–00:17.4`
- Recovery route: Continue from the retained incomplete state and confirm payment. The confirmation transition resumes from the same observed flow. Then see successful payment feedback; The product shows the documented first-success result: see successful payment feedback. Evidence: `media/motion.mp4 00:20.4–00:28.2`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | enter the sale amount | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:03.6–00:08.4` |
| focus / selection | present the contactless payment screen | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:07.8–00:13.2` |
| navigation | Open PayPal Tap to Pay | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:01.2–00:07.2` |
| confirmation | confirm payment | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:17.4–00:22.8` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:11.4–00:17.4` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:18.6–00:26.4` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:09.6–00:16.5` |
| recovery | Continue from the incomplete state and confirm payment. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:15.6–00:27.6` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** enter the sale amount
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
