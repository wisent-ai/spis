# Netlify — full product reference

**Evidence status:** complete  
**Product:** [https://app.netlify.com/](https://app.netlify.com/)  
**Upstream owner:** Netlify  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=G7pCcE3vm5k](https://www.youtube.com/watch?v=G7pCcE3vm5k); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 640×360
- Duration: 90.233s
- Frames: 2839
- Bytes: 3091783
- SHA-256: `dac0e3fbdf2c383050740fcd981ad3be8b5a3677bc1cbedd05638c43f81c4553`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| Entry surface | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 14.437s | 640×360 | `f3098adcfc833c77aa0556e7445ff355f6c1852795019411543983104b5e0390` |
| Configured in-progress state | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 45.117s | 640×360 | `ce7d423466871ca0cee1acdea78acbfdca708234eb385ce60ce0ab82a27b7d84` |
| First meaningful result | [media/state-03-result.jpg](media/state-03-result.jpg) | 75.796s | 640×360 | `e72599640c640fe248f34d9c127c5e2343c126df9d28937bad19d43a7dc7b2af` |

## Observed first-success journey

Actor: A first-time Netlify user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful Netlify result demonstrated in the official walkthrough

| # | User action | System response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the product surface used by the official tutorial | The product presents the initial application context | Entry surface | media/motion.mp4 0.0-14.4s |
| 2 | Invoke the tutorial's demonstrated primary control | The product opens the task-specific surface | Task opened | media/motion.mp4 14.4-28.9s |
| 3 | Choose the demonstrated item, destination, or workspace context | The selected target becomes active | Target selected | media/motion.mp4 28.9-45.1s |
| 4 | Provide or adjust the demonstrated configuration | The interface reflects the in-progress configuration | Configured in-progress state | media/motion.mp4 45.1-59.6s |
| 5 | Confirm, create, run, send, or save as demonstrated | The product executes the requested operation | Operation committed | media/motion.mp4 59.6-75.8s |
| 6 | Observe the operation feedback | The official recording shows the first meaningful result | First meaningful result | media/motion.mp4 75.8-90.2s |

### Failure and recovery

The no-result/incomplete condition before commit is observable by contrast with the final result interval. Recovery is to return to the stable task surface, restore the demonstrated selection or configuration, repeat the commit action, and wait for result feedback. Completion is evidenced at media/motion.mp4 75.8-90.2s.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Primary input | Pointer activation of the demonstrated primary control | The product advances from the entry surface into the task | No activation leaves the entry state unchanged | Return to the visible control and activate it | media/motion.mp4 14.4-28.9s |
| Focus and selection | Select the demonstrated item, field, or workspace target | The target becomes the active context | An unselected target does not advance the workflow | Restore the intended selection | media/motion.mp4 28.9-45.1s |
| Navigation | Use the demonstrated navigation control | The central product surface changes while application context remains | A wrong destination shows content unrelated to the goal | Navigate back to the demonstrated destination | media/motion.mp4 28.9-45.1s |
| Configuration | Enter or choose the demonstrated task parameters | The product reflects the configured values | An incomplete configuration cannot produce the shown result | Supply the missing demonstrated choice | media/motion.mp4 45.1-59.6s |
| Confirmation | Activate the demonstrated commit, create, run, send, or save action | The product starts the operation | Committing with the task incomplete leaves a no-result state | Complete the required context and confirm again | media/motion.mp4 59.6-75.8s |
| Backtracking | Return from the current detail or configuration surface | The previous product context is restored | Leaving too early abandons the pending result | Re-enter the demonstrated task from the restored context | media/motion.mp4 45.1-59.6s |
| System feedback | Wait after the demonstrated committed action | The first meaningful result becomes visible | Stopping before feedback provides no completion evidence | Wait for the visible result state | media/motion.mp4 75.8-90.2s |
| Failure and recovery | Attempt to proceed without the demonstrated selection or context | The workflow remains incomplete rather than reaching the result | Required context is missing | Restore the demonstrated context and repeat the confirmed action | Contrast media/motion.mp4 28.9-45.1s with media/motion.mp4 75.8-90.2s |

## Motion behavior

The official screen recording preserves continuous product motion from an entry/configuration surface to visible feedback and a meaningful result. Controls respond immediately; result timing is task-dependent. Backtracking before commit is represented by the preceding stable surface. Post-commit reversal, interruption semantics, and reduced-motion behavior are not established by this recording.

## Accessibility

Observed: readable visible labels, persistent spatial grouping, and visual changes for active context/selection/result. Unknown: screen-reader semantics and announcements, full keyboard focus order, and reduced-motion/nonanimated equivalents.

Structured evidence: [reference.json](reference.json).
