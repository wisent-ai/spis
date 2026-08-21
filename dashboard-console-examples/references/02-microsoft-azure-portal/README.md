# Microsoft Azure Portal — observed product reference

- **Evidence status:** partial — accessibility was never measured against the live product, and the excerpt shows no interruption or reversal
- **Product:** [https://portal.azure.com/](https://portal.azure.com/)
- **Official motion source:** [Getting started in the Azure Portal](https://www.youtube.com/watch?v=leJRc0JWzSY)
- **Upstream owner/channel:** Microsoft Azure Developers
- **Captured:** 2026-08-16
- **Local motion:** [play `media/product-motion.mp4`](media/product-motion.mp4) (640×360, 14.967s, 449 frames, 38336 bytes, SHA-256 `98df9a44b1dd181887f21482f471f57076dddd01f3eabf2672c62f84f6688b8d`)
- **Observed source interval:** 68.55–83.52s
- **Capture method:** static official-video retrieval through the Invidious companion proxy, then ffmpeg excerpt/transcode; no browser or local GUI used.

## Retained states

| State | Evidence | Observed state | Relationship | Dimensions | Bytes | SHA-256 |
|---|---|---|---|---:|---:|---|
| entry context | [`media/state-1.png`](media/state-1.png) | Azure Quickstart Center on portal.azure.com with the "Get started" tab underlined: "Start a project" showing seven cards (Create a web app, Deploy a virtual machine, Deploy and run a container-based app, Set up a database, Get started with data analytics/machine learning and intelligence, Store back up or archive data, Build deploy and operate a serverless app) and "Setup guides" showing Azure setup guide, Azure migration guide, Azure innovation guide and Send us your feedback; no mouse pointer is drawn anywhere in the frame | frame of `media/product-motion.mp4` at 8s (mean abs diff 0.2422/255); re-checked at full resolution, state-1 is pixel-identical (max channel difference 0/255) to the frame at 3.0s, while the frame at 8.0s already carries the mouse pointer at x17,y50 that state-1 does not have | 640×360 | 67973 | `d81aa521017da21d1982c4c9abff3d899019278d5840b7bd2b30de1550920454` |
| focused working state | [`media/state-2.png`](media/state-2.png) | The same fully loaded Quickstart Center page with the mouse pointer now parked at x17,y50 immediately left of the "Quickstart Center" heading; that pointer is the only difference from state-1 (mean absolute difference 0.05/255, 18 pixels differing by more than 25, every one inside x16–19 y48–54) | frame of `media/product-motion.mp4` at 6.5s (mean abs diff 0.2656/255); re-checked at full resolution, state-2 is pixel-identical (max channel difference 0/255) to the frame at 8.0s | 640×360 | 68366 | `4cbc4906311e8b4008247abba19a9f74bb3464eedc4298dbee56824ecc53cc7d` |
| first-success result | [`media/state-3.png`](media/state-3.png) | The same Quickstart Center page with the same seven project cards, the same four setup-guide cards and the pointer still at x17,y50; it differs from state-2 only by low-amplitude encoder noise (mean absolute difference 0.81/255, 18 pixels above 25 scattered singly, no cluster) | frame of `media/product-motion.mp4` at 8.5s (mean abs diff 0.2148/255); confirmed pixel-identical (max channel difference 0/255) at full 640×360 resolution | 640×360 | 66808 | `cceff627927b3cda0de45fb7d127c27cbf7282667b85cfae8880260bd1a5fd11` |

## First-success journey

**Actor:** Operator using Microsoft Azure Portal  
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

- **Trigger:** No control is activated anywhere in the excerpt. The only input visible is the mouse pointer, which is not drawn at all in any frame from 0.00s to 5.75s, first appears at 6.00s at x57,y25 in the "Home >" breadcrumb row, and travels to x17,y50 beside the "Quickstart Center" heading by 6.50s, where it stays for the remaining 8.4 seconds.
- **Start → end:** Quickstart Center at 0.00s already fully loaded — "Get started" tab underlined next to "Take an online course", the "Start a project" blurb, all seven project cards each with a "Start >" link, and the "Setup guides" section with Azure setup guide, Azure migration guide, Azure innovation guide ("Open >") and "Send us your feedback", with no spinner, skeleton, or empty placeholder → the same page at 14.90s with identical headings, card titles and body copy; the only change is the pointer resting at x17,y50.
- **Continuity:** The layout is spatially frozen. Comparing the frame at 3.00s with the frame at 14.90s, only 20 pixels differ by more than 30/255 and 17 of those are the pointer glyph itself; the remaining three are isolated single pixels at (78,124), (140,16) and (230,17). No card, heading, tab, or shell element is repositioned or redrawn.
- **Timing:** sub-second (the only timed movement is the 0.50s pointer travel from 6.00s to 6.50s).
- **Interruption/reversal:** Not shown. The excerpt never interrupts, cancels, undoes, or navigates back, so this stays an open gap.
- **Feedback:** No feedback of any kind appears, because nothing is activated: no hover highlight on the card the pointer crosses, no focus ring, no spinner, and no status text changes in any of the 60 frames sampled at 4 fps.
- **Reduced-motion/nonanimated equivalent:** Not demonstrated — no motion-preference control appears in the frame, and the excerpt contains no animation for a reduced-motion mode to replace; the nonanimated equivalent retained here is the ordered three-frame set.

## Accessibility

### Observed

- Every card in "Start a project" pairs its isometric blue icon with a text title and a sentence of body copy ("Create a web app / Build and deploy web apps that can scale", "Deploy a virtual machine / Run your workloads in the cloud and reduce the redundancy and maintenance of physical hardware", "Set up a database / Explore options for managing relational or nonrelational databases in the cloud"), so no card is identified by its icon alone.
- The selected tab is marked by both its own text and a blue underline: "Get started" is underlined beside the unstyled "Take an online course", so the active tab is not signalled by colour alone.
- Every card carries a written action link rather than a bare arrow or coloured chevron: "Start >" on all seven project cards, "Open >" on Azure setup guide / Azure migration guide / Azure innovation guide, and "Feedback >" on "Send us your feedback".
- Contrast measured from `media/state-1.png`: the "Quickstart Center" heading glyphs (78,78,80) against the page background (255,255,255) give 8.3:1 over box x28–120 y47–62; the "Create a web app" card title (116,116,116) against the white card (255,255,255) gives 4.67:1 over box x42–120 y116–126.
- The shell stays put across all three retained frames at identical coordinates: the hamburger and "Microsoft Azure" wordmark on the blue bar at y27–38, the search field spanning x188–455, the account address at the top right, and the "Home >" breadcrumb at y24 — between state-1 and state-2 the only differing pixels in the whole 640×360 frame are the 18 belonging to the pointer.

### Unknown

- Keyboard focus order and keyboard operability were not exercised: no focus ring appears in any of the 60 sampled frames because nothing is focused or activated during the excerpt.
- Screen-reader names, roles, live-region announcements, and error announcement timing were not inspected.
- Zoom/reflow behaviour and high-contrast mode were not inspected, and only the two contrast pairs listed above were measured, both from the 640×360 retained frames.
- The placeholder text inside the global search field is present but too small to transcribe reliably at 640×360, so its wording was not recorded.
- A product-level reduced-motion preference was not demonstrated in the source excerpt.

## Provenance

The motion is an excerpt of an owner-published real-product recording, not a synthesized animation. Source: [Getting started in the Azure Portal](https://www.youtube.com/watch?v=leJRc0JWzSY), published by **Microsoft Azure Developers**. Local state files are direct frame extractions from the local MP4 and can be inspected offline. All byte counts and SHA-256 values are recorded in [`reference.json`](reference.json).
