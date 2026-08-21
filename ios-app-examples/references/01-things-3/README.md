# Things 3 — observed iOS product reference

**Evidence status:** partial — everything below was measured from the retained files; accessibility has never been audited against the running app, and the asset shows no interruption, reversal, or reduced-motion variant  
**Product:** [https://culturedcode.com/things/](https://culturedcode.com/things/)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Cultured Code

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [meettheallnewthings.mp4](https://static.culturedcode.com/things/videos/2026-06-25-meet-things-remastered-2/meettheallnewthings.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 720×406, 70.867 s, 1063 frames, 970667 bytes
- SHA-256: `b2b137b3b69b35ee619166a4a6f83071dfcda08b269e7f0adf94a6998148fd36`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| "Vacation in Rome" project open, seven to-dos listed, "Book a hotel room" row highlighted | [media/state-01.png](media/state-01.png) | `390be6683aebb09314aa1dc18008d354637e1f182c886720b323d2ceeaab792e` |
| to-do "Pack suitcase" lifted as a floating card while being dragged toward the "Pack" heading | [media/state-02.png](media/state-02.png) | `58631bf68e54dbe95483c06ee03e3233f4e311bdf18701d7efd158656e8a5c1d` |
| blue "and much more" title card between two product segments | [media/state-03.png](media/state-03.png) | `371a71ea67d1b3289a5b4b8f5f98986f77db2b99f283a068cfcc05b6dfdae4b4` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Create and organize a task

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Today** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:03.5`
2. **tap the new-task control** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:17.7`
3. **enter a task title and details** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:31.9`
4. **confirm the task** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:46.1`
5. **see the task in Today** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:00.2`

**Completion evidence:** `media/motion.mp4 at 01:00.2 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:22.7–00:41.1`
- Recovery route: Continue from the retained incomplete state and confirm the task. The confirmation transition resumes from the same observed flow. Then see the task in Today; The product shows the documented first-success result: see the task in Today. Evidence: `media/motion.mp4 00:48.2–01:06.6`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | tap the new-task control | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:08.5–00:19.8` |
| focus / selection | enter a task title and details | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:18.4–00:31.2` |
| navigation | Open Today | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:02.8–00:17.0` |
| confirmation | confirm the task | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:41.1–00:53.9` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:26.9–00:41.1` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:43.9–01:02.4` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:22.7–00:39.0` |
| recovery | Continue from the incomplete state and confirm the task. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:36.9–01:05.2` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** The blue circular + button parked at the bottom right of the to-do list is pressed and dragged up into the list (`media/motion.mp4` 29.05s–30.55s).
- **Start → end:** "Things to buy" list with the + at rest in the corner (28.05s) → an empty "New To-Do" row open between two rows, "Notes" placeholder, row toolbar and keyboard suggestion strip "I / The / Milk" (30.55s).
- **Continuity:** Unbroken. The + stays on screen for the whole gesture, a grey insertion gap opens between "Ask Sarah for travel guide" and "Book a hotel" as the button passes them, and the button resolves into the new row without a cut. Transcoding to H.264 changed encoding only; no frames were synthesized.
- **Timing:** one-to-three-seconds (1.5 s measured between 29.05s and 30.55s)
- **Interruption / reversal:** Not shown. A ✕ appears at the position the button left, but the retained asset never shows it pressed or the drag released onto it, so nothing is recorded here.
- **Feedback:** The lifted button tracks the drag, a ✕ appears where it was, the rows part to show where the item will land, and the finished row takes text entry with the keyboard suggestion strip beneath it (29.05s–30.80s).
- **Reduced motion / nonanimated equivalent:** Not shown by this asset; nothing is claimed.

## Accessibility

Observed:
- In `media/state-01.png` every to-do carries a text label beside its checkbox, and the scheduled ones carry text date chips ("Mon", "Tue", "18. Apr", "11. May"), so the schedule is readable without colour.
- The label "Book a hotel room" measures 15.59:1 inside its own row: darkest pixel `#20242a` against lightest `#ffffff` in the crop 216,182 130×18 of `media/state-01.png`.
- That row's selection is carried by fill alone: the highlight measures `#e9f0fc` against the `#fefefe` page beside it, a ratio of 1.14:1, with no border, checkmark, or other non-colour marker in the frame.
- The move in `media/state-02.png` reads from one still: the lifted "Pack suitcase" row is drawn as a white card with a drop shadow above the list and a gap is already open where it will land, so the drag is not communicated by movement alone.
- The new to-do opened at 30.55s of `media/motion.mp4` shows a text "Notes" placeholder and a visible row toolbar, and the caret sits in the title field, so the focused target is identifiable in a still frame.

Unknown from this evidence:
- VoiceOver names, hints, rotor order, and focus return were not exposed by the source recording.
- Dynamic Type behavior and text truncation at accessibility sizes were not exposed.
- Reduce Motion behavior and a nonanimated equivalent were not exposed.
- Switch Control, keyboard navigation, contrast ratios, and haptic/audio-only feedback were not measured.

## Provenance

The source URL, local path, capture method, dimensions, duration, frame count, byte size, SHA-256, capture date, and upstream ownership are recorded in [`reference.json`](reference.json). All three state images are frames of `media/motion.mp4`: state-01 at 10.5s (mean abs diff 2.0352/255), state-02 at 35.5s (5.1992/255), and state-03 at 60.267s (1.9258/255, found by re-running the same 16×16 grayscale search at 30 fps over 59–62s because the card is on screen only from 60.0s to 60.4s).
