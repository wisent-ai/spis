# AllTrails — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.alltrails.com/mobile](https://www.alltrails.com/mobile)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** AllTrails, LLC

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [navigator.mp4](https://cdn-assets.alltrails.com/assets/videos/homepage-app-feature/en-US/navigator.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 490×1016, 6.933 s, 104 frames, 610596 bytes
- SHA-256: `c627ae31fe523abd7a966137ca30a3b63a12143cd724f66c1ff71a3098889e08`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `9b0f9899d41889259122ffb101fcb2562f9d3199b0bbceb597ea9a3cf9a4ab34` |
| action in progress | [media/state-02.png](media/state-02.png) | `52f1598485d9f147bd708a2ebb100c18633d2cedac39869bba430abd4eba3e89` |
| result / established state | [media/state-03.png](media/state-03.png) | `dc5058bf6de89831581887fb776625f7766ae0a87b39bf03c1dc6d2a874c1fdc` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Start AllTrails navigation

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open AllTrails** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:00.3`
2. **choose a trail** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:01.7`
3. **open Navigator and review the route** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:03.1`
4. **start navigation** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:04.5`
5. **see live route progress** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:05.9`

**Completion evidence:** `media/motion.mp4 at 00:05.9 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:02.2–00:04.0`
- Recovery route: Continue from the retained incomplete state and start navigation. The confirmation transition resumes from the same observed flow. Then see live route progress; The product shows the documented first-success result: see live route progress. Evidence: `media/motion.mp4 00:04.7–00:06.5`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose a trail | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:00.8–00:01.9` |
| focus / selection | open Navigator and review the route | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:01.8–00:03.1` |
| navigation | Open AllTrails | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.3–00:01.7` |
| confirmation | start navigation | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:04.0–00:05.3` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:02.6–00:04.0` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:04.3–00:06.1` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:02.2–00:03.8` |
| recovery | Continue from the incomplete state and start navigation. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:03.6–00:06.4` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose a trail
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
