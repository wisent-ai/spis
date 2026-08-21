# Xero — full product reference

**Evidence status:** partial  
**Product:** [https://go.xero.com/](https://go.xero.com/)  
**Upstream owner:** Xero  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=D7NbL-DfnW8](https://www.youtube.com/watch?v=D7NbL-DfnW8); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 640×360
- Duration: 90.120s
- Frames: 2251
- Bytes: 2212372
- SHA-256: `946fd38ccd722dda44a145c361a0cf864aa4bd24f2da4f89cb030134228a22dc`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| Purchases menu open over the Foxglove Studios business overview, listing Purchases overview, Bills, Purchase orders, Expenses, Suppliers and Purchases settings | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 14.419s | 640×360 | `d5570c77387b43148b50a48f66952693e652d9a60e5ee234f5bd3d70a5505171` |
| New expense claim detail for StateBird Provisions, spent on 28 May 2025, account 420 - Entertainment, with the teal-outlined Optional block (Region, Assign to project or customer, Label) and Total USD 243.92 | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 45.060s | 640×360 | `0d82c20381287ac9f9137da7c343897408408b2423cddf9ac553776835a495b1` |
| Expenses > Your own after approval: Submitted 0.00, To be paid 267.02, and the two To be paid rows StateBird Provisions "Lunch with client" 243.92 and 33 mi "Travel to client meeting" 23.10 | [media/state-03-result.jpg](media/state-03-result.jpg) | 75.701s | 640×360 | `a73623c1b86409629924a50584abeb251490f54a7bd6bed0391fa5319db40969` |

Each frame was located in the motion asset by a 16×16 grayscale mean-absolute-difference search: 14s (diff 2.39/255), 43.5s (diff 2.19/255), 75.5s (diff 2.51/255).

## Observed first-success journey

Actor: A first-time Xero user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful Xero result demonstrated in the official walkthrough

| # | User action | System response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the product surface used by the official tutorial | The product presents the initial application context | Entry surface | media/motion.mp4 0.0-14.4s |
| 2 | Invoke the tutorial's demonstrated primary control | The product opens the task-specific surface | Task opened | media/motion.mp4 14.4-28.8s |
| 3 | Choose the demonstrated item, destination, or workspace context | The selected target becomes active | Target selected | media/motion.mp4 28.8-45.1s |
| 4 | Provide or adjust the demonstrated configuration | The interface reflects the in-progress configuration | Configured in-progress state | media/motion.mp4 45.1-59.5s |
| 5 | Confirm, create, run, send, or save as demonstrated | The product executes the requested operation | Operation committed | media/motion.mp4 59.5-75.7s |
| 6 | Observe the operation feedback | The official recording shows the first meaningful result | First meaningful result | media/motion.mp4 75.7-90.1s |

### Failure and recovery

The no-result/incomplete condition before commit is observable by contrast with the final result interval. Recovery is to return to the stable task surface, restore the demonstrated selection or configuration, repeat the commit action, and wait for result feedback. Completion is evidenced at media/motion.mp4 75.7-90.1s.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Primary input | Pointer activation of the demonstrated primary control | The product advances from the entry surface into the task | No activation leaves the entry state unchanged | Return to the visible control and activate it | media/motion.mp4 14.4-28.8s |
| Focus and selection | Select the demonstrated item, field, or workspace target | The target becomes the active context | An unselected target does not advance the workflow | Restore the intended selection | media/motion.mp4 28.8-45.1s |
| Navigation | Use the demonstrated navigation control | The central product surface changes while application context remains | A wrong destination shows content unrelated to the goal | Navigate back to the demonstrated destination | media/motion.mp4 28.8-45.1s |
| Configuration | Enter or choose the demonstrated task parameters | The product reflects the configured values | An incomplete configuration cannot produce the shown result | Supply the missing demonstrated choice | media/motion.mp4 45.1-59.5s |
| Confirmation | Activate the demonstrated commit, create, run, send, or save action | The product starts the operation | Committing with the task incomplete leaves a no-result state | Complete the required context and confirm again | media/motion.mp4 59.5-75.7s |
| Backtracking | Return from the current detail or configuration surface | The previous product context is restored | Leaving too early abandons the pending result | Re-enter the demonstrated task from the restored context | media/motion.mp4 45.1-59.5s |
| System feedback | Wait after the demonstrated committed action | The first meaningful result becomes visible | Stopping before feedback provides no completion evidence | Wait for the visible result state | media/motion.mp4 75.7-90.1s |
| Failure and recovery | Attempt to proceed without the demonstrated selection or context | The workflow remains incomplete rather than reaching the result | Required context is missing | Restore the demonstrated context and repeat the confirmed action | Contrast media/motion.mp4 28.8-45.1s with media/motion.mp4 75.7-90.1s |

## Motion behavior

Nothing the viewer does drives this clip; it is a 90.12s screen recording of the Xero web app that plays itself, with the pointer moving on its own — it climbs to the Purchases nav item by ~13.5s, opens the menu at 14.0s and hovers "Expenses" by 15.5s, then reaches the claim row's overflow control at 72.0s. The grey "Xero Expenses" title card cross-dissolves into the Foxglove Studios business overview between 10.0s and 11.0s, with both images visible in the same frame at 10.5s; from there the recording is continuous live UI with no further dissolves, so the timing class is continuous. Feedback is numeric and immediate: submitting the expense claim puts 243.92 under "Submitted" by 54s, the 33 mi mileage claim raises "To be paid" from 0.00 to 23.10 by 66s, and choosing "Approve" from the row overflow menu between 73.0s and 75.0s flips "Submitted" 243.92 to 0.00 and "To be paid" 23.10 to 267.02 by 75.5s. One reversal is observable: the Filter panel opened over the To pay list at ~83.0s is dismissed at ~85.5s without applying a filter, leaving the list identical before and after. No reduced-motion or non-animated equivalent is observable anywhere in the clip.

## Accessibility

Observed: the Purchases dropdown names every destination in text ("Purchases overview", "Bills", "Purchase orders", "Expenses", "Suppliers", "Purchases settings") with no icon-only entries (state-01); counts ride as numerals beside their labels rather than on motion — "To review 0", "To pay 2", the group header "To be paid 2" (state-03) and "20 items to reconcile", "6 Overdue invoices", "3 Overdue bills" (state-01); the active tab "Your own" is both blue and underlined while its siblings stay dark grey, so the current section is not signalled by colour alone (state-03, y 28-52); the claim-row title crop of state-03 (x 88-222, y 181-193) measures 10.26:1 between its brightest 2% of pixels (255,255,255) and darkest 2% (65,64,70), and the summary-band crop holding "To be paid 267.02" (x 392-470, y 110-142) measures 9.65:1 for (57,60,65) on (236,240,241), both by the WCAG relative-luminance formula over the retained JPEG; the teal ellipses ringing the upload target at ~30s, the Optional block at 45s and the sub-nav tabs at 78-81s are drawn on top of the recording rather than rendered by the product, so no native focus indicator appears in the clip; and every state stays legible with the animation stopped, because organisation name, claim descriptions, dates, the account code "420 - Entertainment" and all amounts are text. Unknown: screen-reader names, roles and live-region announcements were not exposed by the static-source recording; complete keyboard traversal and focus order were not established; and reduced-motion preference handling and a non-animated equivalent were not demonstrated. Accessibility has never been measured against the running product — every statement above comes from the retained recording and frames alone.

Structured evidence: [reference.json](reference.json).
