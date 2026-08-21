# Netflix — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://help.netflix.com/en/node/23927](https://help.netflix.com/en/node/23927)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Netflix product owner; recording published by Netflix

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [How To Use The New Netflix Experience](https://www.youtube.com/watch?v=mLRbltNrEBk)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 77.267 s, 1159 frames, 1695697 bytes
- SHA-256: `2565db8f93fb1abde2a25f1855735e2775af476ed27c454c04bf19330bc6d051`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `67e2a7f5ce348310ffff8355db95cc39b5b8152090a09ef59aaff27568dd4e02` |
| action in progress | [media/state-02.png](media/state-02.png) | `3bd5c64ace402988b779f379fd039b3f46c631ca26dc660f61a756fcd2fb91f5` |
| result / established state | [media/state-03.png](media/state-03.png) | `4c52d1b7ae8d3897c3e075f1458183b92f9a5a5335118a886c9ac445374f08c4` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Use the redesigned Netflix mobile experience

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Netflix** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:03.9`
2. **choose a profile and browse** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:19.3`
3. **open a title detail** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:34.8`
4. **start playback or save the title** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:50.2`
5. **see the selected content state** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:05.7`

**Completion evidence:** `media/motion.mp4 at 01:05.7 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:24.7–00:44.8`
- Recovery route: Continue from the retained incomplete state and start playback or save the title. The confirmation transition resumes from the same observed flow. Then see the selected content state; The product shows the documented first-success result: see the selected content state. Evidence: `media/motion.mp4 00:52.5–01:12.6`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose a profile and browse | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:09.3–00:21.6` |
| focus / selection | open a title detail | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:20.1–00:34.0` |
| navigation | Open Netflix | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:03.1–00:18.5` |
| confirmation | start playback or save the title | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:44.8–00:58.7` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:29.4–00:44.8` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:47.9–01:08.0` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:24.7–00:42.5` |
| recovery | Continue from the incomplete state and start playback or save the title. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:40.2–01:11.1` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** choose a profile and browse
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
