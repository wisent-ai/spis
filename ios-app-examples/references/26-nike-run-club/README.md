# Nike Run Club — observed iOS product reference

**Evidence status:** partial — the states, timings and contrast figures below were measured from the retained files; accessibility has never been audited against the running app, the asset contains no Nike Run Club interface at all, and it shows no interruption, reversal, or reduced-motion variant  
**Product:** [https://www.nike.com/nrc-app](https://www.nike.com/nrc-app)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Nike Run Club product owner; recording published by Nike

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [Running Mechanics for Proper Form: Blue Benadum | NRC Tips in Stride | Nike](https://www.youtube.com/watch?v=Vxj88hohaac)
- Capture method: official publisher YouTube stream downloaded from the product/vendor channel and transcoded to H.264 MP4; no frames synthesized
- Media: 640×360, 114.800 s, 1722 frames, 1838328 bytes
- SHA-256: `00eb4e727c0e8000105d7646d1c53d91f76c9f4b5fdadd481df1b19d27b83e0b`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| no caption and no app interface on screen: cropped studio shot of a runner's legs in black full-length tights and white Nike shoes with a pink swoosh, mid-stride on the light-grey floor | [media/state-01.png](media/state-01.png) | `9e3379b64feca386648e4a12feabfac64a6e9e493bc95858061ea5bbc00c0150` |
| coaching point 1 card mid-retraction: the volt chip has already lost its leading digit and reads ". FOOT LANDING" above "_DON'T OVERSTRIDE / TRY TO LAND FEET BENEATH YOUR HIP" and "_RUNNING IN PLACE / KEEP FOOT UNDER YOU VS IN FRONT", beside a demonstrator shot from behind in black tights | [media/state-02.png](media/state-02.png) | `9349c98ea70a77dd06a87562bb4ea74f498b7f6bc8e8ca9af80670f1a6ecff35` |
| coaching point 3 card fully drawn: volt chip "3. ARM SWING" above "_REMOVE THE ROTATION OF THE TORSO" and "_ADD MOBILITY TO THE SHOULDERS", with coach Blue Benadum demonstrating the arm swing in the blue/red/white Nike jacket | [media/state-03.png](media/state-03.png) | `1cdb14b1b2f8f599362d3cd0768ed8ef2b25512f1c8424586dc190ce739579fc` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Prepare for a Nike Run Club run

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open Nike Run Club** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:05.7`
2. **choose a run or guidance** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:28.7`
3. **review running mechanics and run setup** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:51.7`
4. **start the guided activity** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 01:14.6`
5. **see active coaching feedback** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 01:37.6`

**Completion evidence:** `media/motion.mp4 at 01:37.6 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:36.7–01:06.6`
- Recovery route: Continue from the retained incomplete state and start the guided activity. The confirmation transition resumes from the same observed flow. Then see active coaching feedback; The product shows the documented first-success result: see active coaching feedback. Evidence: `media/motion.mp4 01:18.1–01:47.9`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose a run or guidance | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:13.8–00:32.1` |
| focus / selection | review running mechanics and run setup | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:29.8–00:50.5` |
| navigation | Open Nike Run Club | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:04.6–00:27.6` |
| confirmation | start the guided activity | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 01:06.6–01:27.2` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:43.6–01:06.6` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 01:11.2–01:41.0` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:36.7–01:03.1` |
| recovery | Continue from the incomplete state and start the guided activity. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:59.7–01:45.6` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** No Nike Run Club interface and no user interaction appear anywhere in the 114.800s asset, so nothing in it is triggered by a tap. The one transition it does contain is the retraction of a coaching-point card, which begins at 57.367s of `media/motion.mp4` and of which `media/state-02.png` is a frame (best 30 fps match 57.400s, mean abs diff 1.9492/255).
- **Start → end:** the complete coaching point 1 card at 57.333s — volt chip "1. FOOT LANDING" over "_DON'T OVERSTRIDE / TRY TO LAND FEET BENEATH YOUR HIP" and "_RUNNING IN PLACE / KEEP FOOT UNDER YOU VS IN FRONT" → bare light-grey studio wall at 57.700s with the shot cut to coach Blue Benadum, no chip or instruction line left.
- **Continuity:** Cut-edited coaching reel, not one continuous product session. The 1 fps sweep of all 114.800s shows a "TIPS IN STRIDE" title card at 1–4s, the "COACH / BLUE BENADUM / LOS ANGELES" lower third at 6s, a "RUNNING MECHANICS FOR IMPROVEMENT" card at 8–9s, a "MECHANICAL MOVEMENT PATTERNS" card at 25–28s, repeated cuts between the coach and two demonstrators, and a Nike swoosh on black from 112s to the end. The transition analysed here is nevertheless unbroken inside one shot: the chip collapses across 57.367–57.667s with the same black-tights background behind it, and the cut to the coach lands only afterwards at 57.700s. Transcoding to H.264 changed encoding only; no frames were synthesized.
- **Timing:** sub-second (0.367 s measured between 57.333s and 57.700s)
- **Interruption / reversal:** Not shown by this asset; nothing is claimed.
- **Feedback:** The card leaves in two separate steps, both readable frame by frame at 30 fps: the two instruction blocks vanish in a single frame between 57.467s and 57.500s, while the volt chip collapses from its left edge so its label shortens "1. FOOT LANDING" (57.333s), ". FOOT LANDING" (57.367–57.433s), "FOOT LANDING" (57.467s), "ING" (57.533s), "G" (57.600s), gone by 57.700s. Nothing here acknowledges a user action, because the asset shows none.
- **Reduced motion / nonanimated equivalent:** Not shown by this asset; nothing is claimed.

## Accessibility

Observed:
- Every coaching point in the asset is written on screen as well as spoken: `media/state-02.png` carries the volt chip ". FOOT LANDING" with "_DON'T OVERSTRIDE / TRY TO LAND FEET BENEATH YOUR HIP" and "_RUNNING IN PLACE / KEEP FOOT UNDER YOU VS IN FRONT", and `media/state-03.png` carries "3. ARM SWING" with "_REMOVE THE ROTATION OF THE TORSO" and "_ADD MOBILITY TO THE SHOULDERS", so each instruction is readable with the sound off, and state-02 shows the card's retraction as a truncated label in one still rather than only as movement.
- The white caption body text is laid straight onto the light-grey studio wall and fails ordinary text contrast: the crop 63,103 180×17 of `media/state-03.png` over "_REMOVE THE ROTATION" gives darkest `#a7a7a7` against lightest `#ffffff`, 2.41:1, and the crop 63,130 175×18 of `media/state-02.png` over "_DON'T OVERSTRIDE" gives darkest `#bbbac1` against `#ffffff`, 1.93:1.
- The numbered chip is the only high-contrast text in either caption: black glyphs on volt fill measure 19.37:1 in the crop 63,79 119×14 of `media/state-03.png` (`#000000` against `#f2ff7b`) and 19.69:1 in the crop 72,116 112×11 of `media/state-02.png` (`#000000` against `#f1ffb2`).
- Coaching-step order is carried by the digit inside the chip — "1.", "2.", "3." — and not by colour: the chips in both retained stills use the same volt fill, mean `#9fbb3f` in the state-02 chip crop and `#b2d142` in the state-03 chip crop.
- `media/state-01.png` carries no text of any kind at 17.0s, only the runner's legs against the grey floor, so that retained frame offers a screen reader or a muted viewer nothing to read.

Unknown from this evidence:
- VoiceOver names, hints, rotor order, and focus return were not exposed by the source recording.
- Dynamic Type behavior and text truncation at accessibility sizes were not exposed.
- Reduce Motion behavior and a nonanimated equivalent were not exposed.
- Switch Control, keyboard navigation, contrast ratios, and haptic/audio-only feedback were not measured.
- No Nike Run Club interface appears in the asset at all: a 1 fps sweep of the whole 114.800s (120 frames) shows only studio coaching footage, title cards, a "COACH / BLUE BENADUM / LOS ANGELES" lower third at 6s and a closing Nike swoosh on black from 112s, so no in-app control label, focus indicator, touch target, or caret can be observed.

## Provenance

The source URL, local path, capture method, dimensions, duration, frame count, byte size, SHA-256, capture date, and upstream ownership are recorded in [`reference.json`](reference.json). All three state images are frames of `media/motion.mp4`: state-01 at 17.0s (mean abs diff 5.1719/255), state-02 at 57.0s (4.5469/255, and 1.9492/255 against the 57.400s frame in a 30 fps re-search over 55–59s), and state-03 at 97.5s (1.7148/255).
