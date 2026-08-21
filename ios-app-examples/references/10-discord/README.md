# Discord — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://discord.com/](https://discord.com/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Discord Inc.

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [6257adef93867e50d84d30e2%2F683a317f73a44ea577535c09_Homepage_Hop-In_EN-transcode.mp4](https://cdn.prod.website-files.com/6257adef93867e50d84d30e2%2F683a317f73a44ea577535c09_Homepage_Hop-In_EN-transcode.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 720×550, 7.400 s, 111 frames, 90188 bytes
- SHA-256: `c3c0b5c90a0ab51024e3d1891612b7be34a75fb30266ddc3561f10cdbea3be1f`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `14838d3b3cfd87e096286a5a98605584538353836d326fb737bc4426a3e87a47` |
| action in progress | [media/state-02.png](media/state-02.png) | `0773f7fe952dc2f6b39e2627e01de843e521794bcfcc74175ebcceb5546d1415` |
| result / established state | [media/state-03.png](media/state-03.png) | `d5bd67cb0edc3ecf865688a0a1c9f1ffc7ed0f81ec59693bf07b4c6e2282c3eb` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Join and participate in a Discord space

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Discord** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:00.4`
2. **select a server and channel** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:01.9`
3. **enter or react to the conversation** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:03.3`
4. **confirm the action** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:04.8`
5. **see live channel feedback** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:06.3`

**Completion evidence:** `media/motion.mp4 at 00:06.3 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:02.4–00:04.3`
- Recovery route: Continue from the retained incomplete state and confirm the action. The confirmation transition resumes from the same observed flow. Then see live channel feedback; The product shows the documented first-success result: see live channel feedback. Evidence: `media/motion.mp4 00:05.0–00:07.0`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | select a server and channel | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:00.9–00:02.1` |
| focus / selection | enter or react to the conversation | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:01.9–00:03.3` |
| navigation | Open Discord | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.3–00:01.8` |
| confirmation | confirm the action | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:04.3–00:05.6` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:02.8–00:04.3` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:04.6–00:06.5` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:02.4–00:04.1` |
| recovery | Continue from the incomplete state and confirm the action. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:03.8–00:06.8` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** select a server and channel
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
