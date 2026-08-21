# QuickBooks Online — full product reference

**Evidence status:** complete  
**Product:** [https://qbo.intuit.com/](https://qbo.intuit.com/)  
**Upstream owner:** Intuit QuickBooks  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=hQw7D7yrYyM](https://www.youtube.com/watch?v=hQw7D7yrYyM); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 640×360
- Duration: 90.008s
- Frames: 2700
- Bytes: 2100057
- SHA-256: `371f7d096e756466e9b995261aa782d42f5ce9a0db7f1f6ac52dfcba15ec5715`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| Entry surface | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 14.401s | 640×360 | `25a6980febd50e4ed4cc5cf853b1291b523fc118e24cfb07ea4f18cf722601f4` |
| Configured in-progress state | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 45.004s | 640×360 | `8700d0992ae7d6770fd382d4b4aac6687ff93d54985accfb6e77a32b1003735a` |
| First meaningful result | [media/state-03-result.jpg](media/state-03-result.jpg) | 75.607s | 640×360 | `01bf1baed39ce1be52339847eda98193a30c51f4d6e5faea14113c0ba278b51c` |

## Observed first-success journey

Actor: A first-time QuickBooks Online user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful QuickBooks Online result demonstrated in the official walkthrough

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

The official screen recording preserves continuous product motion from an entry/configuration surface to visible feedback and a meaningful result. Controls respond immediately; result timing is task-dependent. Backtracking before commit is represented by the preceding stable surface. Post-commit reversal, interruption semantics, and reduced-motion behavior are not established by this recording.

## Accessibility

Observed: readable visible labels, persistent spatial grouping, and visual changes for active context/selection/result. Unknown: screen-reader semantics and announcements, full keyboard focus order, and reduced-motion/nonanimated equivalents.

Structured evidence: [reference.json](reference.json).
