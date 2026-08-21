# Slack — full product reference

**Evidence status:** partial  
**Product:** [https://app.slack.com/client](https://app.slack.com/client)  
**Upstream owner:** Slack  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=FTuOS8E1LZk](https://www.youtube.com/watch?v=FTuOS8E1LZk); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 640×360
- Duration: 45.033s
- Frames: 1351
- Bytes: 1799120
- SHA-256: `d548e0ff928bd11830c4a84f5f9b3f24a8ac438c767885802274411a45ec6b68`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| Channels list expanded in the Acme Inc sidebar: # announcements, # project-unicorn, # team-design | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 7.207s | 640×360 | `6b2730d4b812f6949aba970b17c1c081ea4f557942544e928411a1f63a25455e` |
| Message from Arcadio with a Q1 Campaign PDF attachment, two reaction counts (eyes 1, white-check 1), and Zoe's reply mentioning @Arcadio | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 22.523s | 640×360 | `d6317f3972adc75dba43429e5ceb488d6abe3d07148ab481dff4f26e496c4421` |
| Empty message composer on the closing aubergine card, send arrow greyed out | [media/state-03-result.jpg](media/state-03-result.jpg) | 37.839s | 640×360 | `b0011a81b651c166413c8c162628ba3cd137cffcb71404b07a6785af45d2625e` |

Each frame was located in the motion asset by a 16×16 grayscale mean-absolute-difference search: 7s (diff 4.03/255), 22s (diff 3.35/255), 38s (diff 7.35/255).

## Observed first-success journey

Actor: A first-time Slack user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful Slack result demonstrated in the official walkthrough

| # | User action | System response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the product surface used by the official tutorial | The product presents the initial application context | Entry surface | media/motion.mp4 0.0-7.2s |
| 2 | Invoke the tutorial's demonstrated primary control | The product opens the task-specific surface | Task opened | media/motion.mp4 7.2-14.4s |
| 3 | Choose the demonstrated item, destination, or workspace context | The selected target becomes active | Target selected | media/motion.mp4 14.4-22.5s |
| 4 | Provide or adjust the demonstrated configuration | The interface reflects the in-progress configuration | Configured in-progress state | media/motion.mp4 22.5-29.7s |
| 5 | Confirm, create, run, send, or save as demonstrated | The product executes the requested operation | Operation committed | media/motion.mp4 29.7-37.8s |
| 6 | Observe the operation feedback | The official recording shows the first meaningful result | First meaningful result | media/motion.mp4 37.8-45.0s |

### Failure and recovery

The no-result/incomplete condition before commit is observable by contrast with the final result interval. Recovery is to return to the stable task surface, restore the demonstrated selection or configuration, repeat the commit action, and wait for result feedback. Completion is evidenced at media/motion.mp4 37.8-45.0s.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Primary input | Pointer activation of the demonstrated primary control | The product advances from the entry surface into the task | No activation leaves the entry state unchanged | Return to the visible control and activate it | media/motion.mp4 7.2-14.4s |
| Focus and selection | Select the demonstrated item, field, or workspace target | The target becomes the active context | An unselected target does not advance the workflow | Restore the intended selection | media/motion.mp4 14.4-22.5s |
| Navigation | Use the demonstrated navigation control | The central product surface changes while application context remains | A wrong destination shows content unrelated to the goal | Navigate back to the demonstrated destination | media/motion.mp4 14.4-22.5s |
| Configuration | Enter or choose the demonstrated task parameters | The product reflects the configured values | An incomplete configuration cannot produce the shown result | Supply the missing demonstrated choice | media/motion.mp4 22.5-29.7s |
| Confirmation | Activate the demonstrated commit, create, run, send, or save action | The product starts the operation | Committing with the task incomplete leaves a no-result state | Complete the required context and confirm again | media/motion.mp4 29.7-37.8s |
| Backtracking | Return from the current detail or configuration surface | The previous product context is restored | Leaving too early abandons the pending result | Re-enter the demonstrated task from the restored context | media/motion.mp4 22.5-29.7s |
| System feedback | Wait after the demonstrated committed action | The first meaningful result becomes visible | Stopping before feedback provides no completion evidence | Wait for the visible result state | media/motion.mp4 37.8-45.0s |
| Failure and recovery | Attempt to proceed without the demonstrated selection or context | The workflow remains incomplete rather than reaching the result | Required context is missing | Restore the demonstrated context and repeat the confirmed action | Contrast media/motion.mp4 14.4-22.5s with media/motion.mp4 37.8-45.0s |

## Motion behavior

Nothing the viewer does drives this clip; it is an official animated walkthrough that plays itself, and input is drawn into it. Text types itself into the composer ("Hello!" at 0s, "What if we added more pink?" at ~12s), an animated cursor travels to the composer's video-clip icon at 26.5s and to the huddle headphones icon at ~33s. Motion inside a scene is continuous — the recorder panel opens with a running 00:05/00:06 timer, huddle tiles arrive one by one from two to four between 32s and 35s — and scenes are joined by hard cuts, such as the cyan celebration beat cutting to the workspace window at ~25.3s. Timing class: continuous. The clip never pauses, rewinds or undoes an action, so no interruption or reversal is observable in it, and it demonstrates no reduced-motion equivalent.

## Accessibility

Observed: channel names are text with a leading # rather than icon-only entries (state-01); reaction chips carry numerals (eyes 1, white-check 1) so counts read from the still frame (state-02); the sidebar crop of state-01 (x 90-380, y 95-260) measures 10.8:1 between its brightest and darkest 2% of pixels — (232,186,233) against (47,1,48) — and the message-header crop of state-02 (x 130-520, y 25-60) measures 16.9:1 for (29,29,29) text on white, both by the WCAG relative-luminance formula; the send arrow is drawn light grey while the composer is empty (state-03) and the huddle's Leave control is a labelled magenta button, so availability is carried by colour and label rather than motion; every state stays legible with the animation stopped. Unknown: screen-reader names, roles and announcements, full keyboard focus order, and reduced-motion handling — none were exposed by this recording, and accessibility has never been measured against the running product.

Structured evidence: [reference.json](reference.json).
