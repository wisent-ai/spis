# Pocket Casts — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://pocketcasts.com/](https://pocketcasts.com/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Pocket Casts product owner; recording published by Pocket Casts

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [Pocket Casts - Multiplatform podcast player](https://www.youtube.com/watch?v=CHAvjgr9g5Y)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 32.067 s, 481 frames, 371008 bytes
- SHA-256: `dd4731944b511b136e34e7278fba0968a9f0f051fb158ab439ad9a9de496b2ef`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `32f4faa3c67d6692083100c582951e8df0cebb199c3a2e1804d578fa851f0195` |
| action in progress | [media/state-02.png](media/state-02.png) | `ca5df88b1b6b75d798750dcee228ac3919480bc77151f4f70918bd57e7479651` |
| result / established state | [media/state-03.png](media/state-03.png) | `d31c1a588bb48e8445d67e069576523e0d74534a8e2b1e24780ca4736d922aa7` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Play a podcast in Pocket Casts

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Pocket Casts** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:01.6`
2. **discover or choose a podcast** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:08.0`
3. **select an episode** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:14.4`
4. **start playback** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:20.8`
5. **see Now Playing and queue state** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:27.3`

**Completion evidence:** `media/motion.mp4 at 00:27.3 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:10.3–00:18.6`
- Recovery route: Continue from the retained incomplete state and start playback. The confirmation transition resumes from the same observed flow. Then see Now Playing and queue state; The product shows the documented first-success result: see Now Playing and queue state. Evidence: `media/motion.mp4 00:21.8–00:30.1`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | discover or choose a podcast | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:03.8–00:09.0` |
| focus / selection | select an episode | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:08.3–00:14.1` |
| navigation | Open Pocket Casts | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:01.3–00:07.7` |
| confirmation | start playback | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:18.6–00:24.4` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:12.2–00:18.6` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:19.9–00:28.2` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:10.3–00:17.6` |
| recovery | Continue from the incomplete state and start playback. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:16.7–00:29.5` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** discover or choose a podcast
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
