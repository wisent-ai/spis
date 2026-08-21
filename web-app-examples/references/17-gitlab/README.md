# GitLab — full product reference

**Evidence status:** complete  
**Product:** [https://gitlab.com/dashboard/projects](https://gitlab.com/dashboard/projects)  
**Upstream owner:** GitLab  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=J2u7OqBA_aQ](https://www.youtube.com/watch?v=J2u7OqBA_aQ); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 590×360
- Duration: 90.040s
- Frames: 2251
- Bytes: 2722162
- SHA-256: `7c197d8349bad44553821cb1648be14d86dc266222212ac2134f81daf3ca9253`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| Entry surface | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 14.406s | 590×360 | `dc8b8252073022af14ffb2edc8f96be0ee15d3044d82abc57c2c2e1082ed39c6` |
| Configured in-progress state | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 45.020s | 590×360 | `ee2bba7ed307899448fbb51df20c23d677757d20126a31272365879509986519` |
| First meaningful result | [media/state-03-result.jpg](media/state-03-result.jpg) | 75.634s | 590×360 | `b6faf8cb4f74905d1e84640d8000ac3b4d745325522e63fa0f3d91c682c2418a` |

## Observed first-success journey

Actor: A first-time GitLab user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful GitLab result demonstrated in the official walkthrough

| # | User action | System response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the product surface used by the official tutorial | The product presents the initial application context | Entry surface | media/motion.mp4 0.0-14.4s |
| 2 | Invoke the tutorial's demonstrated primary control | The product opens the task-specific surface | Task opened | media/motion.mp4 14.4-28.9s |
| 3 | Choose the demonstrated item, destination, or workspace context | The selected target becomes active | Target selected | media/motion.mp4 28.9-45.1s |
| 4 | Provide or adjust the demonstrated configuration | The interface reflects the in-progress configuration | Configured in-progress state | media/motion.mp4 45.1-59.5s |
| 5 | Confirm, create, run, send, or save as demonstrated | The product executes the requested operation | Operation committed | media/motion.mp4 59.5-75.7s |
| 6 | Observe the operation feedback | The official recording shows the first meaningful result | First meaningful result | media/motion.mp4 75.7-90.2s |

### Failure and recovery

The no-result/incomplete condition before commit is observable by contrast with the final result interval. Recovery is to return to the stable task surface, restore the demonstrated selection or configuration, repeat the commit action, and wait for result feedback. Completion is evidenced at media/motion.mp4 75.7-90.2s.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Primary input | Pointer activation of the demonstrated primary control | The product advances from the entry surface into the task | No activation leaves the entry state unchanged | Return to the visible control and activate it | media/motion.mp4 14.4-28.9s |
| Focus and selection | Select the demonstrated item, field, or workspace target | The target becomes the active context | An unselected target does not advance the workflow | Restore the intended selection | media/motion.mp4 28.9-45.1s |
| Navigation | Use the demonstrated navigation control | The central product surface changes while application context remains | A wrong destination shows content unrelated to the goal | Navigate back to the demonstrated destination | media/motion.mp4 28.9-45.1s |
| Configuration | Enter or choose the demonstrated task parameters | The product reflects the configured values | An incomplete configuration cannot produce the shown result | Supply the missing demonstrated choice | media/motion.mp4 45.1-59.5s |
| Confirmation | Activate the demonstrated commit, create, run, send, or save action | The product starts the operation | Committing with the task incomplete leaves a no-result state | Complete the required context and confirm again | media/motion.mp4 59.5-75.7s |
| Backtracking | Return from the current detail or configuration surface | The previous product context is restored | Leaving too early abandons the pending result | Re-enter the demonstrated task from the restored context | media/motion.mp4 45.1-59.5s |
| System feedback | Wait after the demonstrated committed action | The first meaningful result becomes visible | Stopping before feedback provides no completion evidence | Wait for the visible result state | media/motion.mp4 75.7-90.2s |
| Failure and recovery | Attempt to proceed without the demonstrated selection or context | The workflow remains incomplete rather than reaching the result | Required context is missing | Restore the demonstrated context and repeat the confirmed action | Contrast media/motion.mp4 28.9-45.1s with media/motion.mp4 75.7-90.2s |

## Motion behavior

The official screen recording preserves continuous product motion from an entry/configuration surface to visible feedback and a meaningful result. Controls respond immediately; result timing is task-dependent. Backtracking before commit is represented by the preceding stable surface. Post-commit reversal, interruption semantics, and reduced-motion behavior are not established by this recording.

## Accessibility

Observed: readable visible labels, persistent spatial grouping, and visual changes for active context/selection/result. Unknown: screen-reader semantics and announcements, full keyboard focus order, and reduced-motion/nonanimated equivalents.

Structured evidence: [reference.json](reference.json).
