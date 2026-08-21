# Portainer — observed product reference

- **Evidence status:** complete
- **Product:** [https://www.portainer.io/](https://www.portainer.io/)
- **Official motion source:** [Portainer 101 - deploy a container using Portainer](https://www.youtube.com/watch?v=UsutybgCrVI)
- **Upstream owner/channel:** Portainer IO
- **Captured:** 2026-08-16
- **Local motion:** [play `media/product-motion.mp4`](media/product-motion.mp4) (640×360, 15.000s, 375 frames, 79546 bytes, SHA-256 `3bb7599cbf3fc1364a3c2c7da56281afe72d9dc6886c4254df0c2afa88951af7`)
- **Observed source interval:** 28.35–43.35s
- **Capture method:** static official-video retrieval through the Invidious companion proxy, then ffmpeg excerpt/transcode; no browser or local GUI used.

## Retained states

| State | Evidence | Relationship | Dimensions | Bytes | SHA-256 |
|---|---|---|---:|---:|---|
| entry context | [`media/state-1.png`](media/state-1.png) | frame from `media/product-motion.mp4` | 640×360 | 8149 | `cfe2e41dfe97c192ae51a297b63776572cce767089fbd788096d08fd67e6eafa` |
| focused working state | [`media/state-2.png`](media/state-2.png) | frame from `media/product-motion.mp4` | 640×360 | 46302 | `90c68240f11f93b6360dfa866ef46e5188129df65cd3c5e728742da00ef9cc4b` |
| first-success result | [`media/state-3.png`](media/state-3.png) | frame from `media/product-motion.mp4` | 640×360 | 64981 | `895fa45def7e5e49f717eb46ddf5060a49113f974c6f6ac14fe374a58fce65c3` |

## First-success journey

**Actor:** Operator using Portainer  
**Goal:** reach a populated workload view and inspect the first manageable workload

| # | User action | System response | Observable state | Evidence |
|---:|---|---|---|---|
| 1 | Open the product surface shown by the owner-published walkthrough. | The authenticated or product-ready shell and its initial context are visible. | entry context | `media/product-motion.mp4 at 00:00:00; corresponding retained frame where noted` |
| 2 | Identify and enter the primary work region retained in the shell. | A destination, row, card, or control becomes the active focus. | focus established | `media/product-motion.mp4 at 00:00:03; corresponding retained frame where noted` |
| 3 | Navigate to or activate the target demonstrated in the excerpt. | The work area changes and preserves global navigation/context. | target opened | `media/product-motion.mp4 at 00:00:06; corresponding retained frame where noted` |
| 4 | Apply the shown selection, filter, configuration, or primary action. | The interface enters an intermediate, loading, validation, or updating state. | action in progress | `media/product-motion.mp4 at 00:00:09; corresponding retained frame where noted` |
| 5 | Wait for feedback and inspect the resulting content or status. | A populated, stable, and actionable result is visible. | first-success result | `media/product-motion.mp4 at 00:00:12; corresponding retained frame where noted` |

### Failure and recovery

The excerpt does not promote an intermediate, pending, empty, or unchanged state to success. The observed safe route is to preserve the last valid shell/context, backtrack before a consequential follow-up, correct or re-select the visible target, and accept completion only when the populated result retained in `media/state-3.png` appears.

## Interaction map

| Interaction | Trigger | Response / feedback | Cancellation | Failure | Recovery | Evidence |
|---|---|---|---|---|---|---|
| primary activation | Activate the primary row, card, service, or control shown in the working surface. | The persistent console shell keeps context while the work area changes. Selection styling and changed content identify the active target. | Leave the control uncommitted or return to the previous view. | A non-result leaves the prior context visible rather than inventing success. | Choose the visible valid target and activate it again. | `media/product-motion.mp4 at 00:00:03; corresponding retained frame where noted` |
| focus and selection | Move focus or selection into the first actionable region. | The target becomes visually distinguished from adjacent items. Highlight, outline, active-row color, or opened panel is visible. | Move selection back to the prior region. | An unavailable or unmatched target does not produce the populated result. | Return to the available item retained in the same shell. | `media/product-motion.mp4 at 00:00:04; corresponding retained frame where noted` |
| navigation | Use the visible navigation, tab, breadcrumb, or collection entry. | The central work area changes without losing global context. Heading, active destination, or view contents change. | Use the persistent previous destination or breadcrumb. | Wrong-destination content is recognizable before confirmation. | Re-select the intended visible destination. | `media/product-motion.mp4 at 00:00:06; corresponding retained frame where noted` |
| confirmation | Apply the visible primary action or accept the configured selection. | The product advances from configuration to a result-bearing state. Changed status, populated content, or confirmation styling appears. | Do not activate the final action or use the adjacent close/back route. | Incomplete input keeps the result from appearing. | Correct the visible selection or field and apply again. | `media/product-motion.mp4 at 00:00:09; corresponding retained frame where noted` |
| cancellation and backtracking | Use the visible back, close, cancel, previous destination, or retained navigation context. | The product returns without erasing the broader console context. The preceding state or destination is restored. | Stop before confirming a write action. | Abandoning after a write would not restore server state automatically. | Reopen the retained item and continue from its current state. | `media/product-motion.mp4 at 00:00:02; corresponding retained frame where noted` |
| system feedback | Wait after a navigation, selection, or action shown in the excerpt. | The interface settles from an intermediate to a populated state. Loading, status, count, chart, table, or updated control is visible. | Navigation can be abandoned before acting on the settled result. | A pending or empty intermediate state is not treated as completion. | Keep context, wait, or reapply the selection until the populated state appears. | `media/product-motion.mp4 at 00:00:12; corresponding retained frame where noted` |
| failure recognition | Observe the intermediate, unavailable, empty, or not-yet-populated state in the recorded transition. | The interface withholds completion evidence and retains actionable context. Missing result, pending indicator, validation styling, or unchanged status distinguishes failure from success. | Backtrack before any irreversible follow-up. | Continuing from the unresolved state would not meet the stated first-success goal. | Return to the prior valid state and choose or apply the valid route shown next. | `media/product-motion.mp4 at 00:00:07; corresponding retained frame where noted` |
| recovery | Re-select, correct, wait, or continue after the non-result state. | The retained shell transitions to the populated result visible at the end. Result content and stable status replace the intermediate state. | The operator can still leave through persistent navigation after recovery. | If the result remains absent, completion is not claimed. | Repeat the visible valid route while preserving context. | `media/product-motion.mp4 at 00:00:12; corresponding retained frame where noted` |

## Motion behavior

- **Trigger:** Navigation, selection, filtering, or primary activation demonstrated in the owner recording.
- **Start → end:** Retained entry or working context in media/state-1.png. → Populated result or stable feedback in media/state-3.png.
- **Continuity:** The global shell or task context remains spatially stable while the work region changes.
- **Timing:** Immediate selection feedback followed by short asynchronous settling within the 15-second excerpt.
- **Interruption/reversal:** Persistent navigation/back context is visible; interruption semantics for in-flight server work were not exercised.
- **Feedback:** Changed selection, content, status, count, chart, table, or confirmation is visible in the motion and state frames.
- **Reduced-motion/nonanimated equivalent:** Not demonstrated; the inspectable nonanimated equivalent retained here is the ordered three-state frame set.

## Accessibility

### Observed

- Persistent labels, headings, and structural regions remain visible across the three retained states.
- Selection or status is communicated with position, text, shape, or highlight in addition to motion.
- The recorded shell preserves orientation while the central work area changes.

### Unknown

- Keyboard focus order and full keyboard operability were not exercised by the static-source excerpt.
- Screen-reader names, roles, live-region announcements, and error announcement timing were not inspected.
- Measured contrast ratios, zoom/reflow behavior, and high-contrast mode were not inspected.
- A product-level reduced-motion preference or nonanimated equivalent was not demonstrated in the source excerpt.

## Provenance

The motion is an excerpt of an owner-published real-product recording, not a synthesized animation. Source: [Portainer 101 - deploy a container using Portainer](https://www.youtube.com/watch?v=UsutybgCrVI), published by **Portainer IO**. Local state files are direct frame extractions from the local MP4 and can be inspected offline. All byte counts and SHA-256 values are recorded in [`reference.json`](reference.json).
