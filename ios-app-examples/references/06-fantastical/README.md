# Fantastical — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://flexibits.com/fantastical](https://flexibits.com/fantastical)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Flexibits Inc.

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [fantastical-promo-video-silent.mp4](https://cdn.flexibits.com/video/fantastical-promo-video-silent.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 720×406, 67.000 s, 1005 frames, 1530343 bytes
- SHA-256: `b69e77f9ae4ed6e5e197707173e1e5a65469dfc093b801b483b1cfd79f96e4f0`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `b2b718c1e4e1e766b04774cad2c97de0df1cbf02152c85b9556bb24605f8f1e6` |
| action in progress | [media/state-02.png](media/state-02.png) | `d8db3884041ab42acbb311691333c03acd24e9ad03dd4327105f4582d11987d9` |
| result / established state | [media/state-03.png](media/state-03.png) | `63bb0f33c1a44633f62aab3de3782cf22ff54f069fd469c0a0c7141b5c510327` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Add a calendar event

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open the calendar timeline** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:03.4`
2. **open the event composer** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:16.8`
3. **enter title, time, and details** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:30.2`
4. **save the event** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:43.6`
5. **see it placed on the calendar** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:56.9`

**Completion evidence:** `media/motion.mp4 at 00:56.9 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:21.4–00:38.9`
- Recovery route: Continue from the retained incomplete state and save the event. The confirmation transition resumes from the same observed flow. Then see it placed on the calendar; The product shows the documented first-success result: see it placed on the calendar. Evidence: `media/motion.mp4 00:45.6–01:03.0`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | open the event composer | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:08.0–00:18.8` |
| focus / selection | enter title, time, and details | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:17.4–00:29.5` |
| navigation | Open the calendar timeline | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:02.7–00:16.1` |
| confirmation | save the event | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:38.9–00:50.9` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:25.5–00:38.9` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:41.5–00:59.0` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:21.4–00:36.9` |
| recovery | Continue from the incomplete state and save the event. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:34.8–01:01.6` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** open the event composer
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
