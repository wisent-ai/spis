# AWS Management Console — observed product reference

- **Evidence status:** partial — accessibility was never measured against the live product, and the excerpt shows no interruption or reversal
- **Product:** [https://aws.amazon.com/console/](https://aws.amazon.com/console/)
- **Official motion source:** [Introduction to the AWS Management Console for New AWS Users](https://www.youtube.com/watch?v=i331jNgsL_4)
- **Upstream owner/channel:** Amazon Web Services
- **Captured:** 2026-08-16
- **Local motion:** [play `media/product-motion.mp4`](media/product-motion.mp4) (640×360, 15.000s, 450 frames, 48458 bytes, SHA-256 `d3e98666c9a81fd3599d3f3b68f6e39d5246d65334cce1472dac5a1bf1eac6da`)
- **Observed source interval:** 29.70–44.70s
- **Capture method:** static official-video retrieval through the Invidious companion proxy, then ffmpeg excerpt/transcode; no browser or local GUI used.

## Retained states

| State | Evidence | Observed state | Relationship | Dimensions | Bytes | SHA-256 |
|---|---|---|---|---:|---:|---|
| entry context | [`media/state-1.png`](media/state-1.png) | Console Home dashboard fully loaded — Recently visited, Applications (no applications), Welcome to AWS, AWS Health and Cost and usage widgets all populated | frame of `media/product-motion.mp4` at 3.5s (mean abs diff 1.1328/255); the excerpt is static throughout, so the exact timestamp is not distinguishable | 640×360 | 99008 | `0f118bfef13eeb7cf64266f1b9c976d1661eca9d5c51fe19f67bce1677e98bd7` |
| focused working state | [`media/state-2.png`](media/state-2.png) | Console Home dashboard unchanged from state-1: mean absolute pixel difference 0.02/255 and no pixel differing by more than 25 | frame of `media/product-motion.mp4` at 3.5s (mean abs diff 1.125/255); timestamp not distinguishable | 640×360 | 99360 | `6cc63d219424b3236185cf60471ccef97ef42d6f09167bb75887ddfdcdae3d47` |
| first-success result | [`media/state-3.png`](media/state-3.png) | Console Home dashboard still unchanged at the end of the excerpt; same widgets and same $762.44 / $1,077.29 figures as state-1 | frame of `media/product-motion.mp4` at 3.5s (mean abs diff 1.1406/255); timestamp not distinguishable | 640×360 | 96201 | `1aaedae6188ddeaaf2666a12018ca2f9a241a69a940515535a318b16ad6ac726` |

## First-success journey

**Actor:** Operator using AWS Management Console  
**Goal:** reach a populated control-plane resource view and expose the next safe resource action

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

- **Trigger:** No control is activated anywhere in the excerpt; the only input visible is pointer travel across the Applications widget between 14.25s and 14.75s, from x381,y200 to x332,y183 (measured on the 640×360 frames).
- **Start → end:** Console Home at 0.00s with all six widgets already populated (Recently visited 12 links, Applications "No applications", AWS Health 0/1/1, Cost and usage $762.44 / $1,077.29) → the same page at 14.98s with identical content; only the pointer has moved.
- **Continuity:** The layout is spatially frozen — consecutive frames sampled at 4 fps differ by 0.00/255 mean absolute grayscale for 14 of the 15 seconds, and no pixel changes by more than 30 until 14.25s.
- **Timing:** sub-second (the only timed movement is the 0.75s pointer travel).
- **Interruption/reversal:** Not shown. The excerpt never interrupts or reverses an action, so this stays an open gap.
- **Feedback:** None appears, because no control is activated: no hover highlight, focus ring, spinner, or status change is visible in any of the 60 sampled frames.
- **Reduced-motion/nonanimated equivalent:** Not demonstrated — no motion-preference control is shown, and the excerpt is already a static rendering with no animation to replace.

## Accessibility

### Observed

- Every service shortcut in the Recently visited widget carries a text label beside its coloured square icon (Billing and Cost Management, AWS Resilience Hub, AWS FIS, EC2, AWS Organizations, VPC, DynamoDB, S3, CloudTrail, Elastic Transcoder, Support, API Gateway), so the icon colour is never the only identifier.
- The AWS Health widget states its counts as digits with text labels ("Open issues 0", "Scheduled changes 1", "Other notifications 1"), so health status is readable without colour or motion.
- The Applications widget states its empty condition in words, "No applications / Get started by creating an application", beside a "Create application" button, rather than showing an empty table only.
- Contrast measured from `media/state-1.png`: the "Console Home" heading against the page background gives 14.0:1; the orange "Add widgets" button label against white gives 10.63:1; the "$762.44" current-month figure against the card background gives 5.5:1.
- The same persistent shell (AWS logo, Services menu, search field, region selector "N. Virginia", CloudShell and Feedback footer) is present in all three retained frames at the same coordinates.

### Unknown

- Keyboard focus order and full keyboard operability were not exercised: no focus ring appears in any sampled frame because nothing is focused during the excerpt.
- Screen-reader names, roles, live-region announcements, and error announcement timing were not inspected.
- Zoom/reflow behaviour and high-contrast mode were not inspected; only the three contrast pairs listed above were measured, and only from the 640×360 retained frames.
- A product-level reduced-motion preference was not demonstrated in the source excerpt.

## Provenance

The motion is an excerpt of an owner-published real-product recording, not a synthesized animation. Source: [Introduction to the AWS Management Console for New AWS Users](https://www.youtube.com/watch?v=i331jNgsL_4), published by **Amazon Web Services**. Local state files are direct frame extractions from the local MP4 and can be inspected offline. All byte counts and SHA-256 values are recorded in [`reference.json`](reference.json).
