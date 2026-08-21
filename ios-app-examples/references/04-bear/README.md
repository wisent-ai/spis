# Bear — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://bear.app/](https://bear.app/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Shiny Frog Ltd.

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [bear_2_markdown@2x.gif](https://bear.app/images/home/bear_2_markdown@2x.gif)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 720×444, 20.667 s, 310 frames, 53317 bytes
- SHA-256: `639e45baf3bdbacdf09097a2c26760eacf9428fd1bdfb0f14b63b8b9d37c2bc1`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `1ccce1344789c86d2eaae12b4cacc9f51225ff4b413621ed272da6d1af15da19` |
| action in progress | [media/state-02.png](media/state-02.png) | `c4a3949e1c9636b4e63c0bcef77773b00d653dfd064d4fcd1fae91929b752420` |
| result / established state | [media/state-03.png](media/state-03.png) | `d533f606fc7a86c942ee927d78db251cbf4a750c10e58b4441037924166794d5` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Write a formatted note

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open the notes list** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:01.0`
2. **create a note** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:05.2`
3. **type and format Markdown content** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:09.3`
4. **leave the editor** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:13.4`
5. **see the saved note** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:17.6`

**Completion evidence:** `media/motion.mp4 at 00:17.6 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:06.6–00:12.0`
- Recovery route: Continue from the retained incomplete state and leave the editor. The confirmation transition resumes from the same observed flow. Then see the saved note; The product shows the documented first-success result: see the saved note. Evidence: `media/motion.mp4 00:14.1–00:19.4`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | create a note | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:02.5–00:05.8` |
| focus / selection | type and format Markdown content | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:05.4–00:09.1` |
| navigation | Open the notes list | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.8–00:05.0` |
| confirmation | leave the editor | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:12.0–00:15.7` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:07.9–00:12.0` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:12.8–00:18.2` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:06.6–00:11.4` |
| recovery | Continue from the incomplete state and leave the editor. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:10.7–00:19.0` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** create a note
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
