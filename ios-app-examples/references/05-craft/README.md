# Craft — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.craft.do/](https://www.craft.do/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Craft product owner; recording published by Craft Docs

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [Introducing Craft for iPhone, iPad, and Mac](https://www.youtube.com/watch?v=qD665WBmGhs)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 120.000 s, 1800 frames, 2730453 bytes
- SHA-256: `e2113ee5be7e874f2fa2e0cddcc75f35c1ec3d130b6bd945aec1a639e6859ca9`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `8adf788009da93674cbc9457e9c46235288f623dc120260829d12574b56274f2` |
| action in progress | [media/state-02.png](media/state-02.png) | `d0371b9a22d888b2cefde4a18390da795bb890f38e62c4e9fd543bff67b147e9` |
| result / established state | [media/state-03.png](media/state-03.png) | `56d85717cfacae9589515dfc0391bd111f2acf17ef9722b01cab7f404e53fe9e` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Create a document

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open the document space** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:06.0`
2. **start a new document** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:30.0`
3. **add and arrange content blocks** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:54.0`
4. **finish editing** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 01:18.0`
5. **see the document card** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:42.0`

**Completion evidence:** `media/motion.mp4 at 01:42.0 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:38.4–01:09.6`
- Recovery route: Continue from the retained incomplete state and finish editing. The confirmation transition resumes from the same observed flow. Then see the document card; The product shows the documented first-success result: see the document card. Evidence: `media/motion.mp4 01:21.6–01:52.8`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | start a new document | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:14.4–00:33.6` |
| focus / selection | add and arrange content blocks | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:31.2–00:52.8` |
| navigation | Open the document space | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:04.8–00:28.8` |
| confirmation | finish editing | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 01:09.6–01:31.2` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:45.6–01:09.6` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 01:14.4–01:45.6` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:38.4–01:06.0` |
| recovery | Continue from the incomplete state and finish editing. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 01:02.4–01:50.4` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** start a new document
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
