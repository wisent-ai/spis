# Working Copy — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://workingcopy.app/](https://workingcopy.app/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Anders Borum / Working Copy

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [frontpage-24.mp4](https://workingcopy.app/video/frontpage-24.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 720×486, 65.733 s, 986 frames, 892086 bytes
- SHA-256: `a5044fc7f8205da3fd758f265319057cf5126a3d908d1f66f0e3cc03e541c1cf`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `e3cd2b9887235824757a253d07d20590373524ee422369100839651af9807303` |
| action in progress | [media/state-02.png](media/state-02.png) | `ec640fd4da3458e9be6b78b04ca9cf9d3fe0666e6f60bea6be5e4df62d8a045c` |
| result / established state | [media/state-03.png](media/state-03.png) | `365480c50652e2a7a581a97cc6a5fb48601b1391a4263fb0f64c3f7fb77b7117` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Clone and work with a repository in Working Copy

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Working Copy** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:03.3`
2. **clone or choose a repository** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:16.4`
3. **inspect/edit files and changes** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:29.6`
4. **commit or push** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:42.7`
5. **see synchronized repository status** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:55.9`

**Completion evidence:** `media/motion.mp4 at 00:55.9 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:21.0–00:38.1`
- Recovery route: Continue from the retained incomplete state and commit or push. The confirmation transition resumes from the same observed flow. Then see synchronized repository status; The product shows the documented first-success result: see synchronized repository status. Evidence: `media/motion.mp4 00:44.7–01:01.8`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | clone or choose a repository | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:07.9–00:18.4` |
| focus / selection | inspect/edit files and changes | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:17.1–00:28.9` |
| navigation | Open Working Copy | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:02.6–00:15.8` |
| confirmation | commit or push | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:38.1–00:50.0` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:25.0–00:38.1` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:40.8–00:57.8` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:21.0–00:36.2` |
| recovery | Continue from the incomplete state and commit or push. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:34.2–01:00.5` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** clone or choose a repository
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
