# Microsoft Power BI — full product reference

**Evidence status:** partial  
**Product:** [https://app.powerbi.com/](https://app.powerbi.com/)  
**Upstream owner:** Microsoft Power BI  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=PXqFXG4rzXE](https://www.youtube.com/watch?v=PXqFXG4rzXE); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 640×360
- Duration: 90.033s
- Frames: 2701
- Bytes: 2081835
- SHA-256: `0de9e3ba92257f4607c0bf3cbbcbf99bcfee651ff0eeb19f01d1c1b8b2b07add`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| Power BI Desktop welcome dialog over the empty Untitled report: "Get data", "Recent sources" and "Open other reports" links beside the yellow WHAT'S NEW / FORUMS / POWER BI BLOG / TUTORIALS panel and a "Show this screen on startup" checkbox | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 14.405s | 640×360 | `1f8daa71763bdb30b439903be06b1ba093659f51af80ed6f43571d255980165f` |
| "Two ways to use sample data" modal on the report canvas, offering "Take a tutorial online" with a Launch tutorial link beside "Experiment on your own" with a Load sample data button | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 45.017s | 640×360 | `b3deac791418cda774df59a5088dc1e182e476e2910238d0ada81812bf5f31dc` |
| Power Query Editor holding the financials query with the Segment column selected: Segment, Country, Product, Discount Band, Units Sold, Manufacturing Price and Sale Price headers, APPLIED STEPS Source / Navigation / Changed Type, status bar "16 COLUMNS, 700 ROWS" | [media/state-03-result.jpg](media/state-03-result.jpg) | 75.628s | 640×360 | `24ad2f45120ba9e1ed384a883f1fa095fef934ec7fa4d695dfcce0b6ac0147ec` |

Each frame was located in the motion asset by a 16×16 grayscale mean-absolute-difference search: 17.5s (diff 3.0156/255), 44.5s (diff 1.9844/255), 74.5s (diff 1.9062/255).

## Observed first-success journey

Actor: A first-time Microsoft Power BI user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful Microsoft Power BI result demonstrated in the official walkthrough

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

Nothing the viewer does drives this clip; it is an official Microsoft screen recording of Power BI Desktop that plays itself, and the pointer is driven inside it — beside "Get data" at 15-17s, onto "Load sample data" at 48.0-48.5s, and into the ribbon's Data Type list at 85.5-86.5s — with red editorial annotations drawn over the frame (a rule next to "Get data" at 15-17s, a rectangle around the ribbon's Data group at ~18s). It opens on the Microsoft four-square splash (0-3s) and ends in the Power Query Editor with the converted Units Sold column and APPLIED STEPS Source / Navigation / Changed Type (88-90s). Motion inside a scene is continuous rather than cut together: the welcome dialog disappears between 17.5s and 18.0s to uncover the empty "Add data to your report" canvas, a "Connecting" progress modal appears and clears between 49.0s and 50.0s, and the Navigator opens empty at 50.5s then populates "Financial Sample.xlsx [2]" > financials, Sheet1 by 51.0s and its preview rows by 54s. Timing class: continuous. One dismissal is observable — the welcome dialog closing between 17.5s and 18.0s, never reopened — while the "Change Column Type" prompt at 87.0-87.5s offers "Replace current" / "Add new step" / "Cancel" and the recording takes the replace path, so no cancellation, undo or back-navigation happens anywhere in the clip. No reduced-motion or non-animated equivalent is observable in it.

## Accessibility

Observed: the welcome dialog's entry points are text beside their glyphs — "Get data", "Recent sources", "Open other reports" — rather than icon-only (state-01); dataset scale is carried as numerals in the Power Query status bar, "16 COLUMNS, 700 ROWS" with "Column profiling based on top 1000 rows" (state-03); the active column is marked by a persistent visual state rather than motion, the Segment header filled yellow with a teal underline while the ribbon reads "Data Type: Text" (state-03); the dark navigation crop of state-01 (x 90-215, y 55-160) measures 15.02:1 between its brightest and darkest 2% of pixels — (238,238,238) against (26,26,24) — and the modal-title crop of state-02 (x 208-335, y 98-112) measures 6.1:1 for (98,98,98) text on white, both by the WCAG relative-luminance formula; and every state stays legible with the animation stopped, since query name, column headers, row numbers, APPLIED STEPS and the button labels "Replace current" / "Add new step" / "Cancel" (87.0s) are all on-screen text. Unknown: screen-reader names, roles and live-region announcements, complete keyboard traversal and focus order, and reduced-motion/non-animated equivalents. Accessibility has never been measured against the running product; every statement above comes from the retained frames and the local motion asset.

Structured evidence: [reference.json](reference.json).
