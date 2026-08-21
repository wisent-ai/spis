# Yaak — full product reference

**Evidence status:** `complete`  
**Product:** [https://yaak.app/](https://yaak.app/)  
**Upstream owner:** Yaak  
**Captured:** 2026-08-17T00:26:08.567344Z

## Authentic motion evidence

[Open local mp4 motion](media/motion.mp4) — 480×300, 18.0 s, 180 frames, 58002 bytes.

Source: [https://assets.yaak.app/uploads/template-functions-aGrrS_1362x852.mp4](https://assets.yaak.app/uploads/template-functions-aGrrS_1362x852.mp4)  
Capture method: official product-site MP4 downloaded and transcoded without synthesized frames  
SHA-256: `6384146b82bb14198cdcf6c347268725fbfba4fa256a14ea6ca83e3949c10747`

## Retained key states

| State | Frame | Relationship to motion |
|---|---|---|
| launch and task context | ![launch and task context](media/state-1.png) | Extracted from `media/motion.mp4`; 480×300; SHA-256 `56e18e5a12b8636d25f54190bb2937eb95a918c1690d8fd48a1eb7f4f0bb2125` |
| focused primary-action state | ![focused primary-action state](media/state-2.png) | Extracted from `media/motion.mp4`; 480×300; SHA-256 `9051ad49d17715939f88241fb7b688eafe8769e94e6c45988565d2f8fb7304e3` |
| first-success result | ![first-success result](media/state-3.png) | Extracted from `media/motion.mp4`; 480×300; SHA-256 `3e23ebd19b4c94a52ea635e27d777a586a90a612fe6b8850501b3b5ebfecc871` |

## First-success journey

**Actor:** A first-time desktop user with access to the prerequisite local files, service, device, or workspace shown in the recording.

**Goal:** send a first request and inspect its response

**Prerequisites**
- The desktop application is installed or its authentic product recording is available.
- Any local file, workspace, service, engine, device, or account required by the shown path is available.

| # | User action | System response | Observable state | Evidence |
|---:|---|---|---|---|
| 1 | launch and choose a workspace | The product window establishes the available task context. | start surface | `media/motion.mp4 and media/state-1.png` |
| 2 | create or select a request | The selected target becomes persistent and its content is exposed. | workspace or source selected | `media/motion.mp4 and media/state-1.png` |
| 3 | focus the URL or request editor | The primary control accepts focus and exposes editable or actionable state. | primary input focused | `media/motion.mp4 and media/state-2.png` |
| 4 | enter the request target | The task-specific input is visible and ready for commitment. | action ready for confirmation | `media/motion.mp4 and media/state-2.png` |
| 5 | invoke Send | The product acknowledges the command and transitions without a detached interstitial. | operation in progress | `media/motion.mp4 and media/state-2.png` |
| 6 | observe the response panel and status | The resulting content, status, playback, transfer, document, or response is visibly present. | first meaningful result | `media/motion.mp4 and media/state-3.png` |

### Failure and recovery

Failure route:
1. Attempt the task while the request cannot resolve or returns an error.
2. Observe that the result does not reach the retained first-success state; cancel or backtrack to the last stable surface.

Recovery route:
1. correct the target or environment, resend, and inspect the updated response.
2. Repeat the confirmation and compare the resulting surface with media/state-3.png.

Completion evidence: The final portion of media/motion.mp4 and media/state-3.png retain the visible first meaningful result for send a first request and inspect its response.

## Observed interaction map

| Interaction | Trigger | Response and feedback | Cancellation | Failure | Recovery | Evidence |
|---|---|---|---|---|---|---|
| launch/open | launch and choose a workspace | The product exposes its initial task context. The main window and available next action become visible. | Closing or backing out returns to the prior operating-system context without completing the task. | The start surface or prerequisite is unavailable. | Relaunch and restore the prerequisite, then return to the start surface. | `media/motion.mp4; retained states media/state-1.png through media/state-3.png` |
| primary input | focus the URL or request editor | The central editor, composer, canvas, address field, or selection control accepts input. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the request cannot resolve or returns an error | correct the target or environment, resend, and inspect the updated response | `media/motion.mp4; retained states media/state-1.png through media/state-3.png` |
| focus/selection | create or select a request | The chosen workspace, item, or tool receives a visible selected state. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the request cannot resolve or returns an error | correct the target or environment, resend, and inspect the updated response | `media/motion.mp4; retained states media/state-1.png through media/state-3.png` |
| navigation | create or select a request | The adjacent content region updates while persistent navigation remains available. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the request cannot resolve or returns an error | correct the target or environment, resend, and inspect the updated response | `media/motion.mp4; retained states media/state-1.png through media/state-3.png` |
| confirmation | invoke Send | The requested operation advances to a processing or completed state. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the request cannot resolve or returns an error | correct the target or environment, resend, and inspect the updated response | `media/motion.mp4; retained states media/state-1.png through media/state-3.png` |
| cancellation/backtracking | invoke the visible back, close, cancel, or prior-selection route before confirmation | The transient surface closes and the last stable state returns. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the request cannot resolve or returns an error | correct the target or environment, resend, and inspect the updated response | `media/motion.mp4; retained states media/state-1.png through media/state-3.png` |
| failure feedback | submit the observed task with an unavailable target or invalid prerequisite | The result does not advance and the affected control or status region carries failure feedback. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the request cannot resolve or returns an error | correct the target or environment, resend, and inspect the updated response | `media/motion.mp4; retained states media/state-1.png through media/state-3.png` |
| recovery | correct the target or environment, resend, and inspect the updated response | The same primary action can be re-entered and reaches the result state. The recording shows an immediate visible state change or persistent selected state. | Back, close, or returning to the prior selection leaves the previous durable result unchanged. | the request cannot resolve or returns an error | correct the target or environment, resend, and inspect the updated response | `media/motion.mp4; retained states media/state-1.png through media/state-3.png` |

## Motion behavior

- **Trigger:** invoke Send
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

- Product source page: https://yaak.app/
- Original media/source recording: https://assets.yaak.app/uploads/template-functions-aGrrS_1362x852.mp4
- Upstream recording owner: Yaak
- Capture host/system: static HTTP acquisition / official direct media acquisition
- Transformation: Only temporal clipping, scaling, frame-rate normalization, and still extraction from authentic motion; no generated or interpolated content.
- Local motion dimensions/duration/frames: 480×300; 18.0 s; 180 frames
- Local motion bytes/SHA-256: 58002 / `6384146b82bb14198cdcf6c347268725fbfba4fa256a14ea6ca83e3949c10747`

Structured record: [`reference.json`](reference.json)
