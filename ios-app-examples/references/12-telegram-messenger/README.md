# Telegram Messenger — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://telegram.org/](https://telegram.org/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Telegram FZ-LLC

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [t_main_iOS_demo_2x.mp4](https://telegram.org/img/t_main_iOS_demo_2x.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 608×480, 2.000 s, 30 frames, 57773 bytes
- SHA-256: `2f038afa124f1253a7bf1726f96e8605ddb41b13ce79152322ec4dafdf69b544`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `ddf3887b8299cb6148f1be0a2ea0a183001bd2285483deb84687f00bee3f4cba` |
| action in progress | [media/state-02.png](media/state-02.png) | `e1c222d247aeb94809b17fbcd18e27a80984315f21b1e52c64e49dcaaff12dfd` |
| result / established state | [media/state-03.png](media/state-03.png) | `2f363fad6b4e01ebba27bde1a1b690e811ac4fdb8c88ce6089edce30ab9d6ba9` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Start a Telegram conversation

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Telegram chats** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:00.1`
2. **select a conversation** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:00.5`
3. **compose a message** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:00.9`
4. **send it** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:01.3`
5. **see the thread update** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:01.7`

**Completion evidence:** `media/motion.mp4 at 00:01.7 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:00.6–00:01.2`
- Recovery route: Continue from the retained incomplete state and send it. The confirmation transition resumes from the same observed flow. Then see the thread update; The product shows the documented first-success result: see the thread update. Evidence: `media/motion.mp4 00:01.4–00:01.9`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | select a conversation | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:00.2–00:00.6` |
| focus / selection | compose a message | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:00.5–00:00.9` |
| navigation | Open Telegram chats | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.1–00:00.5` |
| confirmation | send it | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:01.2–00:01.5` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:00.8–00:01.2` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:01.2–00:01.8` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:00.6–00:01.1` |
| recovery | Continue from the incomplete state and send it. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:01.0–00:01.8` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** select a conversation
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
