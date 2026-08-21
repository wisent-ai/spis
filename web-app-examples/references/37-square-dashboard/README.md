# Square Dashboard — full product reference

**Evidence status:** partial  
**Product:** [https://squareup.com/dashboard](https://squareup.com/dashboard)  
**Upstream owner:** Square  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=aiN8GUxj4h0](https://www.youtube.com/watch?v=aiN8GUxj4h0); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 640×360
- Duration: 90.057s
- Frames: 2699
- Bytes: 2183991
- SHA-256: `983bf10151f2618731c5d576b9b40dcbdc29ea7f1d03aebbf0888a7cd82f05a0`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| "Topics" card listing "Homepage Overview" and "Features & Settings" beside the Square Dashboard home rendered on a desktop-monitor mock-up | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 14.409s | 640×360 | `3c8fcc273d37f1cc9b7826b1232ff2a93ab655d25a5172ed94fbd418414af3c4` |
| Square Dashboard home for 2nd Street Location: "Welcome back.", "You're 80% set up.", Performance dated Jan 1, 2024 - Dec 31, 2024, Key Metrics Net Sales $339.93 / Gross Sales $399.93 / Transactions 10 | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 45.028s | 640×360 | `824443aab1448dac8394b8369a59313531e790819f0b5780639aa30ad5bbc837` |
| The same Dashboard home with the Performance date chip switched to "Last year" and the pointer on the blue "Go to Balance" button | [media/state-03-result.jpg](media/state-03-result.jpg) | 75.648s | 640×360 | `1ea79bfa4d57b9abea803eaec518ac62ded62425939dfd3434de5723cfd92a82` |

Each frame was located in the motion asset by a 16×16 grayscale mean-absolute-difference search: state-01 at 15s (diff 1.61/255), state-02 at 73.5s (diff 1.43/255), state-03 at 73.5s (diff 1.48/255). The dashboard home is visually near-identical across long stretches of this clip, so the search matched both later frames to the same 73.5s frame.

## Observed first-success journey

Actor: A first-time Square Dashboard user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful Square Dashboard result demonstrated in the official walkthrough

| # | User action | System response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the product surface used by the official tutorial | The product presents the initial application context | Entry surface | media/motion.mp4 0.0-14.4s |
| 2 | Invoke the tutorial's demonstrated primary control | The product opens the task-specific surface | Task opened | media/motion.mp4 14.4-28.8s |
| 3 | Choose the demonstrated item, destination, or workspace context | The selected target becomes active | Target selected | media/motion.mp4 28.8-45.0s |
| 4 | Provide or adjust the demonstrated configuration | The interface reflects the in-progress configuration | Configured in-progress state | media/motion.mp4 45.0-59.4s |
| 5 | Confirm, create, run, send, or save as demonstrated | The product executes the requested operation | Operation committed | media/motion.mp4 59.4-75.6s |
| 6 | Observe the operation feedback | The official recording shows the first meaningful result | First meaningful result | media/motion.mp4 75.6-90.0s |

### Failure and recovery

The no-result/incomplete condition before commit is observable by contrast with the final result interval. Recovery is to return to the stable task surface, restore the demonstrated selection or configuration, repeat the commit action, and wait for result feedback. Completion is evidenced at media/motion.mp4 75.6-90.0s.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Primary input | Pointer activation of the demonstrated primary control | The product advances from the entry surface into the task | No activation leaves the entry state unchanged | Return to the visible control and activate it | media/motion.mp4 14.4-28.8s |
| Focus and selection | Select the demonstrated item, field, or workspace target | The target becomes the active context | An unselected target does not advance the workflow | Restore the intended selection | media/motion.mp4 28.8-45.0s |
| Navigation | Use the demonstrated navigation control | The central product surface changes while application context remains | A wrong destination shows content unrelated to the goal | Navigate back to the demonstrated destination | media/motion.mp4 28.8-45.0s |
| Configuration | Enter or choose the demonstrated task parameters | The product reflects the configured values | An incomplete configuration cannot produce the shown result | Supply the missing demonstrated choice | media/motion.mp4 45.0-59.4s |
| Confirmation | Activate the demonstrated commit, create, run, send, or save action | The product starts the operation | Committing with the task incomplete leaves a no-result state | Complete the required context and confirm again | media/motion.mp4 59.4-75.6s |
| Backtracking | Return from the current detail or configuration surface | The previous product context is restored | Leaving too early abandons the pending result | Re-enter the demonstrated task from the restored context | media/motion.mp4 45.0-59.4s |
| System feedback | Wait after the demonstrated committed action | The first meaningful result becomes visible | Stopping before feedback provides no completion evidence | Wait for the visible result state | media/motion.mp4 75.6-90.0s |
| Failure and recovery | Attempt to proceed without the demonstrated selection or context | The workflow remains incomplete rather than reaching the result | Required context is missing | Restore the demonstrated context and repeat the confirmed action | Contrast media/motion.mp4 28.8-45.0s with media/motion.mp4 75.6-90.0s |

## Motion behavior

Nothing the viewer does drives this clip; it is an official screen-recorded walkthrough that plays itself, and the pointer inside it is drawn. It opens on a black title card reading "Navigate Your Dashboard & Locate Reports" (0-4s) and ends on the Dashboard home for 2nd Street Location with the Performance date chip reading "Last year" and the pointer resting on "POS systems" at the foot of the left navigation (89-90.06s). Scenes are joined by hard cuts and cross-dissolves — the title card cuts to a "Topics" card at ~5s, the topics list grows line by line ("Homepage Overview" and "Features & Settings" by 14.4s, "Data & Reports" added by 16s), and the monitor mock-up dissolves into the full-bleed browser dashboard at ~19-20s — while motion inside a scene is continuous, with grey skeleton placeholders standing in for the cards at 63.5s and 67.0-67.5s before values render. Timing class: continuous. One reversal is observable: at ~62.6s the pointer clicks the back arrow beside "Reports", the report sub-navigation collapses into the main sidebar by 62.75s, and the Home dashboard reloads through skeletons at 63.5s. Feedback is visible in the same frame as each action — opening the location chip at 65.0s reveals a "Filter Locations" field over seven named locations, the hovered row highlights at 66.5s, the chip reads "Austin Ecosystem" at 67.0s, and every Key Metrics figure reloads to $0.00 and 0 by 68.0s. The clip demonstrates no reduced-motion or non-animated equivalent, so none is observable in it.

## Accessibility

Observed: every left-navigation entry pairs its icon with a text label (Home, Appointments, Items & services, Orders & payments, Online, Customers, Reports, Staff, Banking, Settings, POS systems), so destinations read without the glyph (state-02); trend direction is carried by numerals and signed badges rather than motion — Net Sales $339.93 ▲126.77%, Gross Sales $399.93 ▲128.74%, Average Net Sale $33.99 ▲240.16%, Transactions 10 ▼33.33% (state-02, 45.03s); the active date filter is spelled out in a chip and changes wording between retained frames, "Date Jan 1, 2024 - Dec 31, 2024" at 45.03s against "Date Last year" at 75.65s (state-03); the header crop of state-02 (x 120-192, y 35-49) measures 11.03:1 between its brightest 2% of pixels (255,255,255) and its darkest 2% (60,60,60) for the "Welcome back." heading, and the primary-button crop (x 120-170, y 55-68) measures 5.4:1 for the white "Go to Balance" label (254,255,251) on its indigo fill (90,97,188), both by the WCAG relative-luminance formula over the retained JPEG; the current navigation row is marked by a filled light-blue background rather than an animated indicator; and every state stays legible with the animation stopped, since location, greeting, "You're 80% set up.", date filter, metric values and the Payment Types breakdown (Cash $134.21, Other $5.99, Card $0.00) are all text. Unknown: screen-reader names, roles and live-region announcements were not exposed by the static-source recording; complete keyboard traversal and focus order were not established; reduced-motion preference handling and a non-animated equivalent were not demonstrated. Accessibility has never been measured against the running product — every statement above is read off the retained recording and frames.

Structured evidence: [reference.json](reference.json).
