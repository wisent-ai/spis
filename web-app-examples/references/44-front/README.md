# Front — full product reference

**Evidence status:** complete  
**Product:** [https://app.frontapp.com/](https://app.frontapp.com/)  
**Upstream owner:** Front  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=U9sSaUr8Mr4](https://www.youtube.com/watch?v=U9sSaUr8Mr4); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 640×360
- Duration: 66.440s
- Frames: 2110
- Bytes: 1043209
- SHA-256: `f4431d05be2c02321372cf00f551ad8a026b4c4a58191a48cd370057ab7bf9f0`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| Entry surface | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 10.630s | 640×360 | `337d99534f8f6a2401825d9e798c0a217e02f7260d9da627591243d59795d82b` |
| Configured in-progress state | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 33.220s | 640×360 | `4890536aea5553cf102c10d91fed61d9906e7539f5c54f3bb5670b1afba7cbb2` |
| First meaningful result | [media/state-03-result.jpg](media/state-03-result.jpg) | 55.810s | 640×360 | `b30e92c983f72de350a33d0dcf4ea511b79333775c658d1bdce61f710d86e048` |

## Observed first-success journey

Actor: A first-time Front user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful Front result demonstrated in the official walkthrough

| # | User action | System response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the product surface used by the official tutorial | The product presents the initial application context | Entry surface | media/motion.mp4 0.0-10.6s |
| 2 | Invoke the tutorial's demonstrated primary control | The product opens the task-specific surface | Task opened | media/motion.mp4 10.6-21.3s |
| 3 | Choose the demonstrated item, destination, or workspace context | The selected target becomes active | Target selected | media/motion.mp4 21.3-33.2s |
| 4 | Provide or adjust the demonstrated configuration | The interface reflects the in-progress configuration | Configured in-progress state | media/motion.mp4 33.2-43.9s |
| 5 | Confirm, create, run, send, or save as demonstrated | The product executes the requested operation | Operation committed | media/motion.mp4 43.9-55.8s |
| 6 | Observe the operation feedback | The official recording shows the first meaningful result | First meaningful result | media/motion.mp4 55.8-66.4s |

### Failure and recovery

The no-result/incomplete condition before commit is observable by contrast with the final result interval. Recovery is to return to the stable task surface, restore the demonstrated selection or configuration, repeat the commit action, and wait for result feedback. Completion is evidenced at media/motion.mp4 55.8-66.4s.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Primary input | Pointer activation of the demonstrated primary control | The product advances from the entry surface into the task | No activation leaves the entry state unchanged | Return to the visible control and activate it | media/motion.mp4 10.6-21.3s |
| Focus and selection | Select the demonstrated item, field, or workspace target | The target becomes the active context | An unselected target does not advance the workflow | Restore the intended selection | media/motion.mp4 21.3-33.2s |
| Navigation | Use the demonstrated navigation control | The central product surface changes while application context remains | A wrong destination shows content unrelated to the goal | Navigate back to the demonstrated destination | media/motion.mp4 21.3-33.2s |
| Configuration | Enter or choose the demonstrated task parameters | The product reflects the configured values | An incomplete configuration cannot produce the shown result | Supply the missing demonstrated choice | media/motion.mp4 33.2-43.9s |
| Confirmation | Activate the demonstrated commit, create, run, send, or save action | The product starts the operation | Committing with the task incomplete leaves a no-result state | Complete the required context and confirm again | media/motion.mp4 43.9-55.8s |
| Backtracking | Return from the current detail or configuration surface | The previous product context is restored | Leaving too early abandons the pending result | Re-enter the demonstrated task from the restored context | media/motion.mp4 33.2-43.9s |
| System feedback | Wait after the demonstrated committed action | The first meaningful result becomes visible | Stopping before feedback provides no completion evidence | Wait for the visible result state | media/motion.mp4 55.8-66.4s |
| Failure and recovery | Attempt to proceed without the demonstrated selection or context | The workflow remains incomplete rather than reaching the result | Required context is missing | Restore the demonstrated context and repeat the confirmed action | Contrast media/motion.mp4 21.3-33.2s with media/motion.mp4 55.8-66.4s |

## Motion behavior

The official screen recording preserves continuous product motion from an entry/configuration surface to visible feedback and a meaningful result. Controls respond immediately; result timing is task-dependent. Backtracking before commit is represented by the preceding stable surface. Post-commit reversal, interruption semantics, and reduced-motion behavior are not established by this recording.

## Accessibility

Observed: readable visible labels, persistent spatial grouping, and visual changes for active context/selection/result. Unknown: screen-reader semantics and announcements, full keyboard focus order, and reduced-motion/nonanimated equivalents.

Structured evidence: [reference.json](reference.json).
