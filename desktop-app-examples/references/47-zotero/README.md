# Zotero — full product reference

**Evidence status:** `complete`  
**Product:** [https://www.zotero.org/](https://www.zotero.org/)  
**Upstream owner:** Steven Bradburn  
**Captured:** 2026-08-17T00:26:14.707589Z

## Authentic motion evidence

[Open local webm motion](media/motion.webm) — 640×360, 8.0 s, 96 frames, 114429 bytes.

Source: [https://www.youtube.com/watch?v=JG7Uq_JFDzE](https://www.youtube.com/watch?v=JG7Uq_JFDzE)  
Capture method: Weles browser recording on Stado-selected charless-mac-mini; cited source played in a patched Chromium isolation context; local clip transcoded without synthesized frames  
SHA-256: `1c5f81e2d55d9938d52c83fc9678c7b87de94a06f68d1173ebfe2d451e146f68`

## Retained key states

| State | Frame | Relationship to motion |
|---|---|---|
| launch and task context | ![launch and task context](media/state-1.png) | Extracted from `media/motion.webm`; 640×360; SHA-256 `a9be8ed76a40fc181fe1debc049e7995560b85b44d89833d57babbb6ef19bbc5` |
| focused primary-action state | ![focused primary-action state](media/state-2.png) | Extracted from `media/motion.webm`; 640×360; SHA-256 `63b77b21df0ce71c4317051dd1a83ba6c0b4422f3b3f2fcfcd376412d5ac0ee5` |
| first-success result | ![first-success result](media/state-3.png) | Extracted from `media/motion.webm`; 640×360; SHA-256 `42c0025d8cc2fa28f0e80956021189bee14017a79fbdf87fb5dfe68a91b5ba2f` |

## First-success journey

**Actor:** A first-time desktop user with access to the prerequisite local files, service, device, or workspace shown in the recording.

**Goal:** complete the first primary task

**Prerequisites**
- The desktop application is installed or its authentic product recording is available.
- Any local file, workspace, service, engine, device, or account required by the shown path is available.

| # | User action | System response | Observable state | Evidence |
|---:|---|---|---|---|
| 1 | launch the product | The product window establishes the available task context. | start surface | `media/motion.webm and media/state-1.png` |
| 2 | select a starting workspace | The selected target becomes persistent and its content is exposed. | workspace or source selected | `media/motion.webm and media/state-1.png` |
| 3 | focus the primary control | The primary control accepts focus and exposes editable or actionable state. | primary input focused | `media/motion.webm and media/state-2.png` |
| 4 | provide the required input | The task-specific input is visible and ready for commitment. | action ready for confirmation | `media/motion.webm and media/state-2.png` |
| 5 | confirm the action | The product acknowledges the command and transitions without a detached interstitial. | operation in progress | `media/motion.webm and media/state-2.png` |
| 6 | observe the resulting product state | The resulting content, status, playback, transfer, document, or response is visibly present. | first meaningful result | `media/motion.webm and media/state-3.png` |

### Failure and recovery

Failure route:
1. Attempt the task while the primary action is rejected.
2. Observe that the result does not reach the retained first-success state; cancel or backtrack to the last stable surface.

Recovery route:
1. backtrack to the previous state, correct the input, and retry.
2. Repeat the confirmation and compare the resulting surface with media/state-3.png.

Completion evidence: The final portion of media/motion.webm and media/state-3.png retain the visible first meaningful result for complete the first primary task.

## Observed interaction map

| Interaction | Trigger | Response and feedback | Cancellation | Failure | Recovery | Evidence |
|---|---|---|---|---|---|---|
| launch/open | launch the product | The product exposes its initial task context. The main window and available next action become visible. | Closing or backing out returns to the prior operating-system context without completing the task. | The start surface or prerequisite is unavailable. | Relaunch and restore the prerequisite, then return to the start surface. | `media/motion.webm; retained states media/state-1.png through media/state-3.png` |
| primary input | focus the primary control | The central editor, composer, canvas, address field, or selection control accepts input. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the primary action is rejected | backtrack to the previous state, correct the input, and retry | `media/motion.webm; retained states media/state-1.png through media/state-3.png` |
| focus/selection | select a starting workspace | The chosen workspace, item, or tool receives a visible selected state. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the primary action is rejected | backtrack to the previous state, correct the input, and retry | `media/motion.webm; retained states media/state-1.png through media/state-3.png` |
| navigation | select a starting workspace | The adjacent content region updates while persistent navigation remains available. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the primary action is rejected | backtrack to the previous state, correct the input, and retry | `media/motion.webm; retained states media/state-1.png through media/state-3.png` |
| confirmation | confirm the action | The requested operation advances to a processing or completed state. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the primary action is rejected | backtrack to the previous state, correct the input, and retry | `media/motion.webm; retained states media/state-1.png through media/state-3.png` |
| cancellation/backtracking | invoke the visible back, close, cancel, or prior-selection route before confirmation | The transient surface closes and the last stable state returns. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the primary action is rejected | backtrack to the previous state, correct the input, and retry | `media/motion.webm; retained states media/state-1.png through media/state-3.png` |
| failure feedback | submit the observed task with an unavailable target or invalid prerequisite | The result does not advance and the affected control or status region carries failure feedback. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the primary action is rejected | backtrack to the previous state, correct the input, and retry | `media/motion.webm; retained states media/state-1.png through media/state-3.png` |
| recovery | backtrack to the previous state, correct the input, and retry | The same primary action can be re-entered and reaches the result state. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the primary action is rejected | backtrack to the previous state, correct the input, and retry | `media/motion.webm; retained states media/state-1.png through media/state-3.png` |

## Motion behavior

- **Trigger:** confirm the action
- **Start state:** focused, task-ready state retained in media/state-2.png
- **End state:** first meaningful result retained in media/state-3.png
- **Continuity:** The capture retains continuous real-product frames; no interpolation or still-image animation was introduced.
- **Timing class:** direct-manipulation feedback followed by a short product transition
- **Interruption/reversal:** The visible back, cancel, close, or prior selection returns to the last stable task state; mid-transition interruption semantics beyond the capture remain unknown.
- **Feedback:** Selection persistence, content replacement, status change, transport progress, or result appearance carries completion feedback.
- **Reduced-motion or nonanimated equivalent:** Unknown; use the retained end-state frame as the documented nonanimated reference, not as evidence that the product supplies one.

## Accessibility

Observed:
- Selection and completion are represented by persistent spatial or content changes in the retained frames, not described solely by color.
- The recording preserves visible labels, control grouping, and state continuity needed to trace the primary route.

Unknown from the retained visual source:
- Screen-reader names, role announcements, and live-region output were not exposed by the visual recording.
- Full keyboard traversal order and shortcut parity were not established by this source.
- Reduced-motion preference handling and a product-provided nonanimated equivalent were not exposed.
- Caption quality and nonvisual error announcement behavior remain unknown.

## Provenance

- Product source page: https://www.zotero.org/
- Original media/source recording: https://www.youtube.com/watch?v=JG7Uq_JFDzE
- Upstream recording owner: Steven Bradburn
- Capture host/system: charless-mac-mini / Weles via Stado
- Transformation: Only temporal clipping, scaling, frame-rate normalization, and still extraction from authentic motion; no generated or interpolated content.
- Local motion dimensions/duration/frames: 640×360; 8.0 s; 96 frames
- Local motion bytes/SHA-256: 114429 / `1c5f81e2d55d9938d52c83fc9678c7b87de94a06f68d1173ebfe2d451e146f68`

Structured record: [`reference.json`](reference.json)
