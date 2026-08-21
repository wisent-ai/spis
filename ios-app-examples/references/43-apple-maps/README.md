# Apple Maps — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.apple.com/maps/](https://www.apple.com/maps/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Apple Maps product owner; recording published by Apple Support

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [How to create and share guides in Maps on iPhone and iPad | Apple Support](https://www.youtube.com/watch?v=i4fuAVWE55I)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 113.867 s, 1708 frames, 477384 bytes
- SHA-256: `ea8f45871a2fcfd11b198848ce2bc4e4e29695b50f71ab0d984be564b7a0a52d`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `e04dbeb7476473b1a02c51d6abcf82cec6b3291d4628a2ef0e17957868618fc7` |
| action in progress | [media/state-02.png](media/state-02.png) | `484f1edf24a844bca22828f54dd28d8dfcf321870e8a92ca57a8460ab870f1c7` |
| result / established state | [media/state-03.png](media/state-03.png) | `f57dfc6bd3c40c51b737624b71170daa87e3ed288d5a46ed0b1ed08d21189d39` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Create and share an Apple Maps guide

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Maps** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:05.7`
2. **open the Guides surface** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:28.5`
3. **add places to a guide** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:51.2`
4. **confirm and share** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 01:14.0`
5. **see the populated guide** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:36.8`

**Completion evidence:** `media/motion.mp4 at 01:36.8 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:36.4–01:06.0`
- Recovery route: Continue from the retained incomplete state and confirm and share. The confirmation transition resumes from the same observed flow. Then see the populated guide; The product shows the documented first-success result: see the populated guide. Evidence: `media/motion.mp4 01:17.4–01:47.0`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | open the Guides surface | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:13.7–00:31.9` |
| focus / selection | add places to a guide | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:29.6–00:50.1` |
| navigation | Open Maps | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:04.6–00:27.3` |
| confirmation | confirm and share | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 01:06.0–01:26.5` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:43.3–01:06.0` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 01:10.6–01:40.2` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:36.4–01:02.6` |
| recovery | Continue from the incomplete state and confirm and share. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:59.2–01:44.8` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** open the Guides surface
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
