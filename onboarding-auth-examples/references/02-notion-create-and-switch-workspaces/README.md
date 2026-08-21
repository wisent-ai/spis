# Notion — create and switch workspaces

**Evidence status:** `complete`  
**Product/source:** [https://www.notion.com/help/create-delete-and-switch-workspaces](https://www.notion.com/help/create-delete-and-switch-workspaces)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Workspace & sidebar](https://www.youtube.com/watch?v=lSmgY5OsZmU) — Notion

## Start-to-first-success journey

**Actor:** new Notion workspace owner  
**Goal:** create and enter a new workspace  
**Prerequisites:** Notion account or sign-in method; workspace name

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and open the workspace switcher | Notion shows the current account and workspace list | account context | `media/state-01-account-context.png` and motion at 4.50s |
| 2 | Choose the create-or-join workspace action | Notion opens workspace setup | workspace creation | `media/state-02-workspace-creation.png` and motion at 19.79s |
| 3 | Enter the workspace name and intended use | Notion updates the workspace preview and available setup choices | workspace details | `media/state-03-workspace-details.png` and motion at 35.07s |
| 4 | Confirm workspace creation | Notion creates the workspace and selects it in the sidebar | workspace active | `media/state-04-workspace-active.png` and motion at 50.36s |
| 5 | Invite collaborators or skip the optional invitation | Notion records invitations or continues without blocking | membership decision | `media/state-05-membership-decision.png` and motion at 65.65s |
| 6 | Create or open the first page | The page appears inside the new workspace, proving first usable content | first page | `media/state-06-first-page.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At workspace creation or workspace details, invalid, expired, denied, or missing required input leaves the flow short of first page; evidence: media/state-02-workspace-creation.png, media/state-03-workspace-details.png, and https://www.notion.com/help/create-delete-and-switch-workspaces.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-workspace-details.png through media/state-05-membership-decision.png.
- **Recovery:** Return to the retained workspace creation or workspace details requirement, correct or resend the blocking input, and resubmit; evidence: https://www.notion.com/help/create-delete-and-switch-workspaces.
- **Recovery:** Continue through the same terminal action until first page is visible in media/state-06-first-page.png and the motion at 80.940s.
- **Completion evidence:** first page retained at media/state-06-first-page.png and media/official-recording.mp4#t=80.940; source https://www.notion.com/help/create-delete-and-switch-workspaces

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account context | [`media/state-01-account-context.png`](media/state-01-account-context.png) | media/official-recording.mp4#t=4.497 | 640×360 | `961f17ce15f2e2b8c6428f016f873b7129bd374a2097503813de0744d93ef8b1` |
| workspace creation | [`media/state-02-workspace-creation.png`](media/state-02-workspace-creation.png) | media/official-recording.mp4#t=19.785 | 640×360 | `af7fd7f3badc3b7670f0b5e203009266179755683d178c10cca6da8dd5b92043` |
| workspace details | [`media/state-03-workspace-details.png`](media/state-03-workspace-details.png) | media/official-recording.mp4#t=35.074 | 640×360 | `34f054bf30dbbab72ee5e8d1f21712f95e8b6d73ce506eaf46f5e2a400f7689a` |
| workspace active | [`media/state-04-workspace-active.png`](media/state-04-workspace-active.png) | media/official-recording.mp4#t=50.362 | 640×360 | `d55dde326fc0278888bb74bef4fde77ac3b6da0c2463954799c831f594107b4c` |
| membership decision | [`media/state-05-membership-decision.png`](media/state-05-membership-decision.png) | media/official-recording.mp4#t=65.651 | 640×360 | `e384afc025a33e78b2b369614d74f31c3a99ec6f7d3de9d9ac277e0e852b9985` |
| first page | [`media/state-06-first-page.png`](media/state-06-first-page.png) | media/official-recording.mp4#t=80.940 | 640×360 | `47f5d8b30b6ad087768c1692b38ba4fd37b72657155fe3cd12c2c7d900634652` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and open the workspace switcher | Notion shows the current account and workspace list The retained account context state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-context.png @ 4.50s; https://www.notion.com/help/create-delete-and-switch-workspaces |
| focus and selection | Choose the create-or-join workspace action | Notion opens workspace setup The recording advances to workspace creation and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-context.png @ 4.50s; media/state-02-workspace-creation.png @ 19.79s; https://www.notion.com/help/create-delete-and-switch-workspaces |
| navigation | Enter the workspace name and intended use | Notion updates the workspace preview and available setup choices The navigation result is visible as workspace details. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-workspace-creation.png @ 19.79s; media/state-03-workspace-details.png @ 35.07s; https://www.notion.com/help/create-delete-and-switch-workspaces |
| confirmation | Confirm workspace creation | Notion creates the workspace and selects it in the sidebar The official recording shows the confirmed workspace active state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-workspace-details.png @ 35.07s; media/state-04-workspace-active.png @ 50.36s; https://www.notion.com/help/create-delete-and-switch-workspaces |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-workspace-creation.png @ 19.79s; media/state-03-workspace-details.png @ 35.07s; media/state-04-workspace-active.png @ 50.36s; https://www.notion.com/help/create-delete-and-switch-workspaces |
| progress feedback | Invite collaborators or skip the optional invitation | Notion records invitations or continues without blocking Progress is observable as the distinct membership decision state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-workspace-active.png @ 50.36s; media/state-05-membership-decision.png @ 65.65s; https://www.notion.com/help/create-delete-and-switch-workspaces |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-workspace-details.png @ 35.07s; media/state-04-workspace-active.png @ 50.36s; media/state-05-membership-decision.png @ 65.65s; https://www.notion.com/help/create-delete-and-switch-workspaces |
| recovery and completion | Create or open the first page | The page appears inside the new workspace, proving first usable content The retained first page state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-membership-decision.png @ 65.65s; media/state-06-first-page.png @ 80.94s; https://www.notion.com/help/create-delete-and-switch-workspaces |

## Motion behavior

- **Trigger:** The recorded sequence begins at account context; the first advancing trigger is “Choose the create-or-join workspace action”.
- **Start/end:** Start is account context at 4.50s; end is first page at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first page; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-context.png and media/state-02-workspace-creation.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-workspace-details.png and media/state-04-workspace-active.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Notion Labs, Inc.
- **Product page:** https://www.notion.com/help/create-delete-and-switch-workspaces
- **Original media URL:** https://www.youtube.com/watch?v=lSmgY5OsZmU
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 550412 bytes
- **SHA-256:** `8f7ba67b361032dfab069fde897b7cff2a50fe89b99fd0a993bf67b529f9e563`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
