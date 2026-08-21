# Flighty — observed iOS product reference

**Evidence status:** complete  
**Product:** [https://www.flightyapp.com/](https://www.flightyapp.com/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Flighty LLC

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [SYTv7nXIQy8CViCphfLJSTNfjq0.mp4](https://framerusercontent.com/assets/SYTv7nXIQy8CViCphfLJSTNfjq0.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 360×202, 15.400 s, 231 frames, 75684 bytes
- SHA-256: `687798b207777db260a4f389bd45ca2fc959c4d8304ae1055aaca8e0882be6ea`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| entry / orientation | [media/state-01.png](media/state-01.png) | `850ec52f8fd611d2215ce77e6208c524b2e2b6a2b0a3878c7f88ed34e9d0cb89` |
| action in progress | [media/state-02.png](media/state-02.png) | `2ca9c5437af4cca411ab1ea60d133a859eeee897b30cc505fa9aa0be039f2456` |
| result / established state | [media/state-03.png](media/state-03.png) | `ce675478ed68a1322bf3f2bbc32eeebc9fbf78b15535c3df58f2d9f7d3fbf8fc` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Track a flight in Flighty

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Flighty** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:00.8`
2. **add a flight** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:03.9`
3. **select the matching itinerary** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:06.9`
4. **confirm tracking** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:10.0`
5. **see the live flight timeline** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:13.1`

**Completion evidence:** `media/motion.mp4 at 00:13.1 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:04.9–00:08.9`
- Recovery route: Continue from the retained incomplete state and confirm tracking. The confirmation transition resumes from the same observed flow. Then see the live flight timeline; The product shows the documented first-success result: see the live flight timeline. Evidence: `media/motion.mp4 00:10.5–00:14.5`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | add a flight | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:01.8–00:04.3` |
| focus / selection | select the matching itinerary | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:04.0–00:06.8` |
| navigation | Open Flighty | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.6–00:03.7` |
| confirmation | confirm tracking | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:08.9–00:11.7` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:05.9–00:08.9` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:09.5–00:13.6` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:04.9–00:08.5` |
| recovery | Continue from the incomplete state and confirm tracking. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:08.0–00:14.2` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** add a flight
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
