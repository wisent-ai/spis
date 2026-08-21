# Google Maps — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.google.com/maps/about/](https://www.google.com/maps/about/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Google LLC

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [maps-quick-action.mp4](https://www.gstatic.com/marketing-cms/09/9b/f0c6b82944059b402165a3f70b8e/maps-quick-action.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 616×820, 5.400 s, 81 frames, 208920 bytes
- SHA-256: `0facb891330ff20330179e31d5c7b7a51fa78f8ffd95097c0306cc88cb7e8c36`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `cec23813d1abc7ce177a98affa50248fd50594d71b033f59420b025ae445c500` |
| action in progress | [media/state-02.png](media/state-02.png) | `357a7378ac6063b2a88df9b4bff9e21bc0c98d8b6f165f0ae9158477741ee17b` |
| result / established state | [media/state-03.png](media/state-03.png) | `fc87f0e7a483cd222d88c4e8a3b7e97bb94d73f8860b06cffa44fbc35b4429a5` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Use a Google Maps quick action

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Google Maps** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:00.3`
2. **select a place or quick action** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:01.4`
3. **review the place/action sheet** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:02.4`
4. **confirm the action** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:03.5`
5. **see the resulting map state** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:04.6`

**Completion evidence:** `media/motion.mp4 at 00:04.6 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:01.7–00:03.1`
- Recovery route: Continue from the retained incomplete state and confirm the action. The confirmation transition resumes from the same observed flow. Then see the resulting map state; The product shows the documented first-success result: see the resulting map state. Evidence: `media/motion.mp4 00:03.7–00:05.1`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | select a place or quick action | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:00.6–00:01.5` |
| focus / selection | review the place/action sheet | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:01.4–00:02.4` |
| navigation | Open Google Maps | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.2–00:01.3` |
| confirmation | confirm the action | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:03.1–00:04.1` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:02.1–00:03.1` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:03.3–00:04.8` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:01.7–00:03.0` |
| recovery | Continue from the incomplete state and confirm the action. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:02.8–00:05.0` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** select a place or quick action
- **Start → end:** entry / orientation → result / established state
- **Continuity:** The recording retains the real product transition sequence; transcoding changes encoding only and does not synthesize intermediate frames.
- **Timing:** rapid microinteraction
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
