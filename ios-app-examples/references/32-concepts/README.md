# Concepts — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://concepts.app/en/](https://concepts.app/en/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Concepts product owner; recording published by Concepts App

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [Part 1: Learn to Draw | Getting Started](https://www.youtube.com/watch?v=TOZxfVp_fSc)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 120.000 s, 1800 frames, 1387216 bytes
- SHA-256: `9e2ef83eb0c1b20f3d072188f5fc3528ff41cce77826e498190dd09618103d3c`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `45a6821d87e8125ecb8a4095729ede76b939b35febe4151242f385eea12fb27f` |
| action in progress | [media/state-02.png](media/state-02.png) | `9928329b209677bbe691e5cfe551c7a85e14754200600d2de78af84e102ead6e` |
| result / established state | [media/state-03.png](media/state-03.png) | `baaaf35f850b792ee7adeb1abd1ea4f92663d170566aba34f9181152996f3242` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Make a first sketch in Concepts

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Concepts** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:06.0`
2. **create a drawing** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:30.0`
3. **choose a tool and draw strokes** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:54.0`
4. **finish or leave the edit** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 01:18.0`
5. **see the sketch on the canvas** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:42.0`

**Completion evidence:** `media/motion.mp4 at 01:42.0 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:38.4–01:09.6`
- Recovery route: Continue from the retained incomplete state and finish or leave the edit. The confirmation transition resumes from the same observed flow. Then see the sketch on the canvas; The product shows the documented first-success result: see the sketch on the canvas. Evidence: `media/motion.mp4 01:21.6–01:52.8`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | create a drawing | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:14.4–00:33.6` |
| focus / selection | choose a tool and draw strokes | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:31.2–00:52.8` |
| navigation | Open Concepts | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:04.8–00:28.8` |
| confirmation | finish or leave the edit | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 01:09.6–01:31.2` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:45.6–01:09.6` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 01:14.4–01:45.6` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:38.4–01:06.0` |
| recovery | Continue from the incomplete state and finish or leave the edit. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 01:02.4–01:50.4` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** create a drawing
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
