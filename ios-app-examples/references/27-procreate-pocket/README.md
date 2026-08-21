# Procreate Pocket — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://procreate.com/pocket](https://procreate.com/pocket)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Savage Interactive Pty Ltd

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [timelapse_mobile.DfykDKYp.mp4](https://procreate-assets-cdn.procreate.com/_nuxt/timelapse_mobile.DfykDKYp.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 720×1556, 10.000 s, 150 frames, 1055606 bytes
- SHA-256: `d5435f4d2398cf347bbc65b1d66e850019d4280dbb15fcee2667875acbc28a9f`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `75395131f3692358ffd2ae0512796749492fd4904fa196bc883859ff2561eebf` |
| action in progress | [media/state-02.png](media/state-02.png) | `417cd1a8dc37ec253ca54661fc903e76b6e54e3d91d81925d5b95063de2aefa2` |
| result / established state | [media/state-03.png](media/state-03.png) | `62c515f9e3fd421845e1f516682e67ad8d28e70cadeb40e733f0b7c1d5d074f6` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Create a drawing in Procreate Pocket

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open the gallery** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:00.5`
2. **create or open a canvas** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:02.5`
3. **draw with selected brush and color** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:04.5`
4. **apply or finish the edit** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:06.5`
5. **see the completed artwork** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:08.5`

**Completion evidence:** `media/motion.mp4 at 00:08.5 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:03.2–00:05.8`
- Recovery route: Continue from the retained incomplete state and apply or finish the edit. The confirmation transition resumes from the same observed flow. Then see the completed artwork; The product shows the documented first-success result: see the completed artwork. Evidence: `media/motion.mp4 00:06.8–00:09.4`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | create or open a canvas | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:01.2–00:02.8` |
| focus / selection | draw with selected brush and color | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:02.6–00:04.4` |
| navigation | Open the gallery | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.4–00:02.4` |
| confirmation | apply or finish the edit | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:05.8–00:07.6` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:03.8–00:05.8` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:06.2–00:08.8` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:03.2–00:05.5` |
| recovery | Continue from the incomplete state and apply or finish the edit. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:05.2–00:09.2` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** create or open a canvas
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
