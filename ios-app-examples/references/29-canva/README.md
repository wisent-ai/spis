# Canva — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://apps.apple.com/us/app/canva-ai-photo-video-editor/id897446215](https://apps.apple.com/us/app/canva-ai-photo-video-editor/id897446215)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Canva product owner; recording published by Canva

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [How to use the Canva App on your Phone](https://www.youtube.com/watch?v=-cXyTnAldIo)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 202×360, 120.000 s, 1800 frames, 658579 bytes
- SHA-256: `7a2704c1298c842786fb8b9ce32133cff7b087f9b18a0524d286f4ff33ae1298`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `172ad6dcda4b375fc37e41916cf6a4518ef70fe30b86e52d19d004a3a19ebd51` |
| action in progress | [media/state-02.png](media/state-02.png) | `af12fb4ac44cf5e07948e10a95e0e9c681cf20e9530b55dbc8e81a12e31abcca` |
| result / established state | [media/state-03.png](media/state-03.png) | `041ada58878e94fd8dd0586b68c77a752cc2f9a441d503cbac40e055c97a6bea` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Create a design in Canva mobile

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Canva** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:06.0`
2. **choose a template or format** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:30.0`
3. **edit text and visual elements** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:54.0`
4. **confirm/share the design** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 01:18.0`
5. **see the finished design** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:42.0`

**Completion evidence:** `media/motion.mp4 at 01:42.0 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:38.4–01:09.6`
- Recovery route: Continue from the retained incomplete state and confirm/share the design. The confirmation transition resumes from the same observed flow. Then see the finished design; The product shows the documented first-success result: see the finished design. Evidence: `media/motion.mp4 01:21.6–01:52.8`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose a template or format | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:14.4–00:33.6` |
| focus / selection | edit text and visual elements | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:31.2–00:52.8` |
| navigation | Open Canva | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:04.8–00:28.8` |
| confirmation | confirm/share the design | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 01:09.6–01:31.2` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:45.6–01:09.6` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 01:14.4–01:45.6` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:38.4–01:06.0` |
| recovery | Continue from the incomplete state and confirm/share the design. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 01:02.4–01:50.4` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose a template or format
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
