# Splitwise — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.splitwise.com/](https://www.splitwise.com/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Splitwise product owner; recording published by Splitwise

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [Splitwise: What exactly is "simplify debts"?](https://www.youtube.com/watch?v=R2CBrFq9KAI)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 48.067 s, 721 frames, 339476 bytes
- SHA-256: `8ed9c50e0422c28aebaaa014aea62fd7723b6d587eaeb3be2b835d8b2663fc21`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `28195d5e5d4dc968a644df54057c637029e330f99c0b5cb72449d656179a4d9e` |
| action in progress | [media/state-02.png](media/state-02.png) | `994808d7849c487c85d09b4579a2df7ec75089082e093205213357bf124eb183` |
| result / established state | [media/state-03.png](media/state-03.png) | `cb2c6ba1788c1e91abd362af97a2834180ed3027edc3003b26e2a64684243d66` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Simplify shared debts

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open a Splitwise group** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:02.4`
2. **open debt simplification** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:12.0`
3. **review the recalculated obligations** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:21.6`
4. **confirm the setting** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:31.2`
5. **see the simplified balances** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:40.9`

**Completion evidence:** `media/motion.mp4 at 00:40.9 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:15.4–00:27.9`
- Recovery route: Continue from the retained incomplete state and confirm the setting. The confirmation transition resumes from the same observed flow. Then see the simplified balances; The product shows the documented first-success result: see the simplified balances. Evidence: `media/motion.mp4 00:32.7–00:45.2`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | open debt simplification | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:05.8–00:13.5` |
| focus / selection | review the recalculated obligations | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:12.5–00:21.1` |
| navigation | Open a Splitwise group | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:01.9–00:11.5` |
| confirmation | confirm the setting | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:27.9–00:36.5` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:18.3–00:27.9` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:29.8–00:42.3` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:15.4–00:26.4` |
| recovery | Continue from the incomplete state and confirm the setting. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:25.0–00:44.2` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** open debt simplification
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
