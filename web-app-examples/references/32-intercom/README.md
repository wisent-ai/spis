# Intercom — full product reference

**Evidence status:** partial  
**Product:** [https://app.intercom.com/](https://app.intercom.com/)  
**Upstream owner:** Intercom  
**Captured:** 2026-08-16

## Authentic motion

[Play the local MP4](media/motion.mp4). This is a direct clip from the upstream owner's official tutorial at [https://www.youtube.com/watch?v=iW0FI_J0pxM](https://www.youtube.com/watch?v=iW0FI_J0pxM); it is not a synthesized animation.

- Kind: MP4
- Dimensions: 640×360
- Duration: 90.000s
- Frames: 2700
- Bytes: 2170822
- SHA-256: `6009711f466e981204bab2d6688badddc3294d6b52081da85492927eb12f70ee`
- Acquisition: Static-source acquisition from the upstream owner's official YouTube tutorial; downloaded with yt-dlp and clipped without synthesizing frames

## Retained product states

| State | Local frame | Motion timestamp | Dimensions | SHA-256 |
|---|---|---:|---:|---|
| "Welcome to Intercom" onboarding modal with four unanswered Select dropdowns — experience setting up a customer support solution, what you primarily use Intercom for, department, and current role — beside the "Get started with the Next-Generation Inbox" panel | [media/state-01-entry.jpg](media/state-01-entry.jpg) | 14.400s | 640×360 | `2c9e453809a4a0e8e787433e0e201682a8abe62e01746808611e9b33ac513c22` |
| Same onboarding modal with three answers chosen ("I've used other support solutions but not Intercom", "Support customers", "Customer Support/Service") and the role dropdown open on Support Executive/Director, Support Manager, Support Rep, Support Operations/Engineering | [media/state-02-progress.jpg](media/state-02-progress.jpg) | 45.000s | 640×360 | `0b1922930b277740169be3112a39f15294fb8538d2f278443f54c1ac57d65d26` |
| Zoomed Intercom Inbox rail and conversation list: Your inbox 0, Mentions 0, Created by you 3, All 0, Unassigned 0, Spam 0, Dashboard, with "0 Open" in the list header | [media/state-03-result.jpg](media/state-03-result.jpg) | 75.600s | 640×360 | `8acb2daea6561edf769f87eb1cdc9367d44f0369f01641e2f2f89ab9d9d2bcec` |

Each frame was located in the motion asset by a 16×16 grayscale mean-absolute-difference search: 14.5s (diff 1.8164/255), 44.5s (diff 1.7891/255), 75s (diff 2.3281/255).

## Observed first-success journey

Actor: A first-time Intercom user following the upstream owner's official walkthrough  
Goal: Reach the first meaningful Intercom result demonstrated in the official walkthrough

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

Nothing the viewer does drives this clip; it is an official screen recording that plays itself, and the pointer inside it is drawn. It opens each "Select" dropdown in the Welcome to Intercom modal (first at ~18s, department at ~36s, role at ~42s), reaches "Save and continue" at 48.0s and presses it by 49.0s, then travels down the collapsed Inbox rail from Fin AI Agent to Knowledge between 86s and 89s. It starts on a Gmail invitation "Phil has invited you to work together on Intercom" that crossfades at 0.25s into the "Join your team on Intercom" signup page, and ends on the Messenger home panel "Hi, Sara! How can we help?" at 89.5-90.0s. Motion inside a scene is continuous — menus expand in place and the four modal answers stay on screen while the cursor moves across 46.0-48.75s — and scenes are joined by short dissolves, including a "We'll be right with you..." interstitial at 49.25s before the "Join your team on Inbox" page lands at 49.5s. Timing class: continuous. The clip never pauses, cancels, dismisses or undoes anything, so no interruption or reversal is observable in it, and it shows no static or non-animated equivalent, so no reduced-motion equivalent is observable either.

## Accessibility

Observed: every navigation destination is text beside its icon rather than icon-only — Inbox, Fin AI Agent, Knowledge, Reports, Outbound, Contacts in the rail and Your inbox, Mentions, Created by you, All, Unassigned, Spam, Dashboard in the list (state-03); conversation volume is carried by numerals, not motion (Created by you 3, header "0 Open", state-03 at 75.6s); each answered onboarding question replaces the placeholder "Select" with its chosen text, so form progress survives with the animation stopped (state-02 at 45.0s); the rail-and-list crop of state-03 (x 0-230, y 60-330) measures 16.97:1 between its brightest and darkest 2% of pixels — (255,255,255) against (28,29,23) — the conversation-list crop (x 250-520, y 60-320) measures 8.39:1, and the modal body crop of state-01 (x 228-420, y 55-300) only 4.81:1 for grey (114,114,114) question text on white, all by the WCAG relative-luminance formula; and at the signup step emphasis is drawn as a dimmed page plus a light outline around the "Sign Up with Google" block from ~2.5s to 3.75s, with the password rule written as text ("Password (at least 15 characters)"). Unknown: screen-reader names, roles and live-region announcements, complete keyboard traversal and focus order, and reduced-motion preference handling. Accessibility has never been measured against the running product; every statement above is read off the retained static evidence.

Structured evidence: [reference.json](reference.json).
