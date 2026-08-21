# Wise — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://wise.com/](https://wise.com/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Wise Payments Limited

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [video-mobile-53fd156ed48933ab3e19a026f7457022.webm](https://wise.com/static-assets/app/_next/static/media/video-mobile-53fd156ed48933ab3e19a026f7457022.webm)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 720×960, 13.467 s, 202 frames, 344705 bytes
- SHA-256: `4d4b99e510d808ed64103a210357ae37c83da4985e277a433133e25f4755dc9d`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `39035e6c7c101024b69f83845bfa8037af2721f2425973c2df3087cf87a728f1` |
| action in progress | [media/state-02.png](media/state-02.png) | `3913d9b72d786d6d642f7a91ec987f757093c94fa70443e715be70680f5438ca` |
| result / established state | [media/state-03.png](media/state-03.png) | `90d80885d0ae75d3d84f3651e9842b7005e1ccaba924ce0dbb9ef06f0994e5b5` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Send money with Wise

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open the Wise account** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:00.7`
2. **choose Send** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:03.4`
3. **enter amount, currency, and recipient** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:06.1`
4. **review fees and confirm** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:08.8`
5. **see the transfer result** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:11.4`

**Completion evidence:** `media/motion.mp4 at 00:11.4 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:04.3–00:07.8`
- Recovery route: Continue from the retained incomplete state and review fees and confirm. The confirmation transition resumes from the same observed flow. Then see the transfer result; The product shows the documented first-success result: see the transfer result. Evidence: `media/motion.mp4 00:09.2–00:12.7`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose Send | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:01.6–00:03.8` |
| focus / selection | enter amount, currency, and recipient | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:03.5–00:05.9` |
| navigation | Open the Wise account | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.5–00:03.2` |
| confirmation | review fees and confirm | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:07.8–00:10.2` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:05.1–00:07.8` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:08.3–00:11.9` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:04.3–00:07.4` |
| recovery | Continue from the incomplete state and review fees and confirm. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:07.0–00:12.4` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose Send
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
