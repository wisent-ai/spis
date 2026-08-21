# Slack — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://slack.com/downloads/ios](https://slack.com/downloads/ios)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Slack product owner; recording published by Slack

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [How to use Slack: Your quick start guide](https://www.youtube.com/watch?v=FTuOS8E1LZk)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 45.067 s, 676 frames, 486011 bytes
- SHA-256: `00e3c2532ab4027a9722a7c233cd93c1d5f8fd9cc275881742b128d4d9d266b7`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `ee7c4fda2af9c3e145810a2ac76f1fe457b4e8a691cba43645d20813fcb6cb74` |
| action in progress | [media/state-02.png](media/state-02.png) | `ed49682d1e2e43060a20c9f67d4d9947f575dc3f3e8d718edc9c5382f7779792` |
| result / established state | [media/state-03.png](media/state-03.png) | `787474faa78638c33746f316177b34f27ae1b7e7c522d5a869fd7937596eaf6d` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Send a workspace message

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Slack and choose a workspace** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:02.3`
2. **open a channel or conversation** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:11.3`
3. **compose a message** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:20.3`
4. **send it** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:29.3`
5. **see the message in the thread** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:38.3`

**Completion evidence:** `media/motion.mp4 at 00:38.3 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:14.4–00:26.1`
- Recovery route: Continue from the retained incomplete state and send it. The confirmation transition resumes from the same observed flow. Then see the message in the thread; The product shows the documented first-success result: see the message in the thread. Evidence: `media/motion.mp4 00:30.6–00:42.4`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | open a channel or conversation | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:05.4–00:12.6` |
| focus / selection | compose a message | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:11.7–00:19.8` |
| navigation | Open Slack and choose a workspace | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:01.8–00:10.8` |
| confirmation | send it | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:26.1–00:34.3` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:17.1–00:26.1` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:27.9–00:39.7` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:14.4–00:24.8` |
| recovery | Continue from the incomplete state and send it. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:23.4–00:41.5` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** open a channel or conversation
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
