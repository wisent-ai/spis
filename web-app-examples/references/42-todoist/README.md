# Todoist — full product reference

**Evidence status:** partial  
**Product:** [https://app.todoist.com/](https://app.todoist.com/)  
**Upstream owner:** Todoist  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=wEf2Mh0f_Mg](https://www.youtube.com/watch?v=wEf2Mh0f_Mg); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 554×360
- Duration: 90.033s
- Frames: 2698
- Bytes: 2672365
- SHA-256: `c95c969aa16b25700d7a420b28c82ba3bcc033815b30db590a5dabb9a3df9913`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| Cream agenda card listing Capture and Clarify, one line each, before Complete is added to the list | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 14.405s | 554×360 | `ce70d2ece68ac949f7e15e1ea746d1b3977d1bad853ac30efb9ba2c2df33eab9` |
| Add-task dialog over the Inbox with "send quote by eod  tod a" typed into the task field and a green Today date chip already applied beside Deadline and Priority | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 45.017s | 554×360 | `37826a2d294a53659afb73c993a18cf2ae572f5060623fcec6d88fdbdd93a8d4` |
| Mint-green section card reading Clarify alone, cut in between two Todoist Inbox scenes | [media/state-03-result.jpg](media/state-03-result.jpg) | 75.628s | 554×360 | `0e985430a9d2a83af92ed73fd34e3bed9e406ab2dcf2361d88f2f67a0ffe3be2` |

Each frame was located in the motion asset by a 16×16 grayscale mean-absolute-difference search: 14.5s (diff 3.3125/255), 45s (diff 2.4258/255), 75.5s (diff 3.2734/255).

## Observed first-success journey

Actor: A first-time Todoist user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful Todoist result demonstrated in the official walkthrough

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

Nothing the viewer does drives this clip; it is an official screen recording narrated by a presenter who stays in a round webcam bubble at the lower right, and input is drawn into it. A pointer hovers the sidebar's Add task control until an "Add task" tooltip appears at ~30s, then text types itself into the task field one character at a time — "send quote by eod today" over 42.0-45.4s and "call mom every other day at 9am" over 49.7-52.8s — and the date chip fills itself in while typing continues ("Today" by 44s, "Sunday 9 AM" by 52s). Activating the red Add task button both inserts the row into the Inbox list and raises a "Task added to Inbox" toast with an Open link, at 46.0s and again at 54.2s; the toast is still on screen at 60.6s. Scenes are joined by hard cuts: webcam to the cream agenda card at ~11.8s, back to webcam at ~16.8s, Inbox to the mint Clarify card at ~75.1s and back at ~77.3s. Timing class: continuous. The clip never pauses, cancels, dismisses or undoes anything — the dialog's Cancel button and the toast's × are both visible but never activated — so no interruption or reversal is observable in it, and it shows no static or non-animated equivalent of the animated surface.

## Accessibility

Observed: every sidebar destination carries a text label beside its icon (Add task, Search, Inbox, Today, Upcoming, Filters & Labels, More) and the current destination is marked by both a pink row highlight and pink label text (motion asset at 88s); each task's due date is written out as words and numerals — "Today 5 PM", "Yesterday 9-11 AM", "Today", "Sunday 9 AM" — with red used redundantly alongside the word "Yesterday" for the overdue item; the add-task dialog names all its controls in text (Today, Deadline, Priority, Inbox, Cancel, Add task) so the parsed date reads as the word "Today" rather than only as a calendar glyph (state-02, 45.017s); the agenda-card crop of state-01 (x 195-360, y 95-200) measures 17.03:1 between its brightest and darkest 2% of pixels — (255,255,230) against (33,26,7) — and the dialog-title crop of state-02 (x 170-300, y 78-95) measures 8.11:1 for (81,79,82) on white, both by the WCAG relative-luminance formula; committed state stays legible with the animation stopped, the toast text and every task title and date being text. Unknown: screen-reader names, roles and live-region announcements were not exposed by the static-source recording; complete keyboard traversal and focus order were not established; reduced-motion preference handling and a non-animated equivalent were not demonstrated. Accessibility has never been measured against the running product — every statement above is read off the retained frames of the official recording.

Structured evidence: [reference.json](reference.json).
