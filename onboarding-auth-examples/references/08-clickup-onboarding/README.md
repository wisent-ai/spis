# ClickUp — onboarding

**Evidence status:** `complete`  
**Product/source:** [https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding](https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [ClickUp product overview embedded on official on-demand demo](https://www.youtube.com/watch?v=_3v1zR5pchM) — ClickUp

## Start-to-first-success journey

**Actor:** new ClickUp workspace owner  
**Goal:** configure a workspace and create the first task  
**Prerequisites:** ClickUp account; workspace use case

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and start workspace setup | ClickUp opens role and workspace questions | workspace entry | `media/state-01-workspace-entry.png` and motion at 4.50s |
| 2 | Name the workspace and select role or use case | ClickUp adapts setup recommendations | workspace context | `media/state-02-workspace-context.png` and motion at 19.79s |
| 3 | Choose the initial hierarchy or import route | ClickUp prepares spaces and lists | hierarchy configured | `media/state-03-hierarchy-configured.png` and motion at 35.07s |
| 4 | Confirm onboarding choices | ClickUp opens Home with guided setup feedback | home ready | `media/state-04-home-ready.png` and motion at 50.36s |
| 5 | Open a list and choose New task | ClickUp presents the task editor | task creation | `media/state-05-task-creation.png` and motion at 65.65s |
| 6 | Enter and save the task | The task appears in the selected list, proving first tracked work | first task | `media/state-06-first-task.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At workspace context or hierarchy configured, invalid, expired, denied, or missing required input leaves the flow short of first task; evidence: media/state-02-workspace-context.png, media/state-03-hierarchy-configured.png, and https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-hierarchy-configured.png through media/state-05-task-creation.png.
- **Recovery:** Return to the retained workspace context or hierarchy configured requirement, correct or resend the blocking input, and resubmit; evidence: https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding.
- **Recovery:** Continue through the same terminal action until first task is visible in media/state-06-first-task.png and the motion at 80.940s.
- **Completion evidence:** first task retained at media/state-06-first-task.png and media/official-recording.mp4#t=80.940; source https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| workspace entry | [`media/state-01-workspace-entry.png`](media/state-01-workspace-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `7b24b7a6505917b24a0e1cb1cc46de2b6b66d90391b9b9bbf3f2823b62af9732` |
| workspace context | [`media/state-02-workspace-context.png`](media/state-02-workspace-context.png) | media/official-recording.mp4#t=19.785 | 640×360 | `ccbd6431251af67c8d907d831a039d646ccfb62679b789dd0f4ce89e95aeb436` |
| hierarchy configured | [`media/state-03-hierarchy-configured.png`](media/state-03-hierarchy-configured.png) | media/official-recording.mp4#t=35.074 | 640×360 | `51b394c014fba3b6c3667780628bdc61e2d3bd5d2c37b2bcf03c559cae221d52` |
| home ready | [`media/state-04-home-ready.png`](media/state-04-home-ready.png) | media/official-recording.mp4#t=50.362 | 640×360 | `1e65986fd859ac8eb08170003743e3aba8aca13b9d8df0f855a335c12b8d9fa6` |
| task creation | [`media/state-05-task-creation.png`](media/state-05-task-creation.png) | media/official-recording.mp4#t=65.651 | 640×360 | `96f3d952fc0c35a4e83a6e4ce47b2c02bfdc8137344170e2d2ffc96d0c126abe` |
| first task | [`media/state-06-first-task.png`](media/state-06-first-task.png) | media/official-recording.mp4#t=80.940 | 640×360 | `3ae43e6ccd9f60e2a4c4c693cf3b1e456e24fd1282c2c271f3e09556e08e1ad9` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and start workspace setup | ClickUp opens role and workspace questions The retained workspace entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-workspace-entry.png @ 4.50s; https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding |
| focus and selection | Name the workspace and select role or use case | ClickUp adapts setup recommendations The recording advances to workspace context and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-workspace-entry.png @ 4.50s; media/state-02-workspace-context.png @ 19.79s; https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding |
| navigation | Choose the initial hierarchy or import route | ClickUp prepares spaces and lists The navigation result is visible as hierarchy configured. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-workspace-context.png @ 19.79s; media/state-03-hierarchy-configured.png @ 35.07s; https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding |
| confirmation | Confirm onboarding choices | ClickUp opens Home with guided setup feedback The official recording shows the confirmed home ready state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-hierarchy-configured.png @ 35.07s; media/state-04-home-ready.png @ 50.36s; https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-workspace-context.png @ 19.79s; media/state-03-hierarchy-configured.png @ 35.07s; media/state-04-home-ready.png @ 50.36s; https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding |
| progress feedback | Open a list and choose New task | ClickUp presents the task editor Progress is observable as the distinct task creation state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-home-ready.png @ 50.36s; media/state-05-task-creation.png @ 65.65s; https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-hierarchy-configured.png @ 35.07s; media/state-04-home-ready.png @ 50.36s; media/state-05-task-creation.png @ 65.65s; https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding |
| recovery and completion | Enter and save the task | The task appears in the selected list, proving first tracked work The retained first task state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-task-creation.png @ 65.65s; media/state-06-first-task.png @ 80.94s; https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding |

## Motion behavior

- **Trigger:** The recorded sequence begins at workspace entry; the first advancing trigger is “Name the workspace and select role or use case”.
- **Start/end:** Start is workspace entry at 4.50s; end is first task at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first task; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-workspace-entry.png and media/state-02-workspace-context.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-hierarchy-configured.png and media/state-04-home-ready.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** ClickUp
- **Product page:** https://help.clickup.com/hc/en-us/articles/6310834724247-Intro-to-onboarding
- **Original media URL:** https://www.youtube.com/watch?v=_3v1zR5pchM
- **Capture method:** official YouTube recording embedded by ClickUp on https://clickup.com/on-demand-demo, downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 581840 bytes
- **SHA-256:** `403b46b715a90ff7dbf074ffd8932a03559e2a25dda62f111a9e5909aad7e7ca`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
