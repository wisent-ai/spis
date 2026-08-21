# Signal — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://signal.org/](https://signal.org/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Signal Messenger LLC / Signal Technology Foundation

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [usernames-chat-via-username.mp4](https://signal.org/blog/videos/usernames-chat-via-username.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 720×722, 26.800 s, 402 frames, 156182 bytes
- SHA-256: `5b6f8f99451c6c55a988d07c6dfc834a92476cb8bbfe57e3b9d4b129cf52c5fc`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `07836be9fff9dac81bd35151da2ab91039342b1212d761ab2181d2ce5df07d0e` |
| action in progress | [media/state-02.png](media/state-02.png) | `2b86c8e8583be09c7509348a98c5f82a8e8c6de75f75c7cfe6ca0f9c6498f81c` |
| result / established state | [media/state-03.png](media/state-03.png) | `96e4e76ba22dc4c1504dfea4903b39c0ade022e486acc1e40150c0cccaa97b62` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Start a Signal chat via username

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Signal** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:01.3`
2. **choose Find by Username** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:06.7`
3. **enter and select a Signal username** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:12.1`
4. **confirm the conversation** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:17.4`
5. **see the private chat ready for messaging** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:22.8`

**Completion evidence:** `media/motion.mp4 at 00:22.8 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:08.6–00:15.5`
- Recovery route: Continue from the retained incomplete state and confirm the conversation. The confirmation transition resumes from the same observed flow. Then see the private chat ready for messaging; The product shows the documented first-success result: see the private chat ready for messaging. Evidence: `media/motion.mp4 00:18.2–00:25.2`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose Find by Username | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:03.2–00:07.5` |
| focus / selection | enter and select a Signal username | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:07.0–00:11.8` |
| navigation | Open Signal | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:01.1–00:06.4` |
| confirmation | confirm the conversation | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:15.5–00:20.4` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:10.2–00:15.5` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:16.6–00:23.6` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:08.6–00:14.7` |
| recovery | Continue from the incomplete state and confirm the conversation. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:13.9–00:24.7` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose Find by Username
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
