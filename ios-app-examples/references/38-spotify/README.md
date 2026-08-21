# Spotify — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.spotify.com/us/download/other/](https://www.spotify.com/us/download/other/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Spotify product owner; recording published by Spotify Bulgaria

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [Introducing Jam on Spotify](https://www.youtube.com/watch?v=tla5uF2pM7c)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 30.067 s, 451 frames, 1074918 bytes
- SHA-256: `0082eeb09b36b35c07af53ba1379d473f51e6415348d3b15911a325466cbb921`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `b6cb6817e47449db07502cf131ff2af91644d7dc1b93c8292ff9d92b830a4c47` |
| action in progress | [media/state-02.png](media/state-02.png) | `4ceb5a85a370404eed660ace90b3d63cd861d61141aa3918899e03b3e940d374` |
| result / established state | [media/state-03.png](media/state-03.png) | `ba8ece102ad84dc934d59ff8dd88b5b8bd81e495fc91225a7e04cf1599fcee88` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Start a Spotify Jam

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Spotify** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:01.5`
2. **choose music and the Jam feature** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:07.5`
3. **invite or join participants** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:13.5`
4. **confirm the shared session** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:19.5`
5. **see the collaborative queue** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:25.6`

**Completion evidence:** `media/motion.mp4 at 00:25.6 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:09.6–00:17.4`
- Recovery route: Continue from the retained incomplete state and confirm the shared session. The confirmation transition resumes from the same observed flow. Then see the collaborative queue; The product shows the documented first-success result: see the collaborative queue. Evidence: `media/motion.mp4 00:20.4–00:28.3`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose music and the Jam feature | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:03.6–00:08.4` |
| focus / selection | invite or join participants | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:07.8–00:13.2` |
| navigation | Open Spotify | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:01.2–00:07.2` |
| confirmation | confirm the shared session | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:17.4–00:22.9` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:11.4–00:17.4` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:18.6–00:26.5` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:09.6–00:16.5` |
| recovery | Continue from the incomplete state and confirm the shared session. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:15.6–00:27.7` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose music and the Jam feature
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
