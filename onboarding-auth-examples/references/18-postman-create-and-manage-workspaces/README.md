# Postman — create and manage workspaces

**Evidence status:** `complete`  
**Product/source:** [https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/](https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to Create and Manage Workspaces in Postman](https://www.youtube.com/watch?v=I10RCvMbPi0) — Postman

## Start-to-first-success journey

**Actor:** Postman workspace creator  
**Goal:** create a workspace and open the first API request  
**Prerequisites:** Postman account; workspace visibility choice

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and open Workspaces | Postman lists current workspaces and creation controls | workspace list | `media/state-01-workspace-list.png` and motion at 3.63s |
| 2 | Choose Create Workspace | Postman opens workspace details | workspace form | `media/state-02-workspace-form.png` and motion at 15.99s |
| 3 | Enter name and optional summary | Postman validates the required identity | workspace details | `media/state-03-workspace-details.png` and motion at 28.34s |
| 4 | Choose visibility and team access | Postman explains the effective audience | visibility selected | `media/state-04-visibility-selected.png` and motion at 40.69s |
| 5 | Confirm creation | Postman opens the new empty workspace | workspace active | `media/state-05-workspace-active.png` and motion at 53.05s |
| 6 | Create or import the first request | The request opens within the workspace, proving first usable result | first request | `media/state-06-first-request.png` and motion at 65.40s |

### Failure and recovery

- **Failure:** At workspace form or workspace details, invalid, expired, denied, or missing required input leaves the flow short of first request; evidence: media/state-02-workspace-form.png, media/state-03-workspace-details.png, and https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-workspace-details.png through media/state-05-workspace-active.png.
- **Recovery:** Return to the retained workspace form or workspace details requirement, correct or resend the blocking input, and resubmit; evidence: https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/.
- **Recovery:** Continue through the same terminal action until first request is visible in media/state-06-first-request.png and the motion at 65.400s.
- **Completion evidence:** first request retained at media/state-06-first-request.png and media/official-recording.mp4#t=65.400; source https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| workspace list | [`media/state-01-workspace-list.png`](media/state-01-workspace-list.png) | media/official-recording.mp4#t=3.633 | 640×360 | `345f913d5e9def7cd12074df30a07515d69d9dbbb0a39baf5d126b7da9c96b3a` |
| workspace form | [`media/state-02-workspace-form.png`](media/state-02-workspace-form.png) | media/official-recording.mp4#t=15.987 | 640×360 | `34bcafa6c0354ac25151b994a22bdea892d1e420065409b2c274f1cd7aca49f5` |
| workspace details | [`media/state-03-workspace-details.png`](media/state-03-workspace-details.png) | media/official-recording.mp4#t=28.340 | 640×360 | `166d8079a74f8ab734131101f0b3b50aa98d2c060e89a1a11dbf10d057c526f0` |
| visibility selected | [`media/state-04-visibility-selected.png`](media/state-04-visibility-selected.png) | media/official-recording.mp4#t=40.694 | 640×360 | `62e3fe0f1529acd66a37dce3cb1d5210c195fbcb3f3669f11fc8a9ea83ef08ab` |
| workspace active | [`media/state-05-workspace-active.png`](media/state-05-workspace-active.png) | media/official-recording.mp4#t=53.047 | 640×360 | `3279e676ef0bd9b06135dc8b59d6bee3fb13f5671d4971350a4d122a4848ed7c` |
| first request | [`media/state-06-first-request.png`](media/state-06-first-request.png) | media/official-recording.mp4#t=65.400 | 640×360 | `7f00dfbefd8d6e2f1fe8a04fe1724412b6d23fcf6d94a09916afca07b10d6c1f` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and open Workspaces | Postman lists current workspaces and creation controls The retained workspace list state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-workspace-list.png @ 3.63s; https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/ |
| focus and selection | Choose Create Workspace | Postman opens workspace details The recording advances to workspace form and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-workspace-list.png @ 3.63s; media/state-02-workspace-form.png @ 15.99s; https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/ |
| navigation | Enter name and optional summary | Postman validates the required identity The navigation result is visible as workspace details. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-workspace-form.png @ 15.99s; media/state-03-workspace-details.png @ 28.34s; https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/ |
| confirmation | Choose visibility and team access | Postman explains the effective audience The official recording shows the confirmed visibility selected state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-workspace-details.png @ 28.34s; media/state-04-visibility-selected.png @ 40.69s; https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-workspace-form.png @ 15.99s; media/state-03-workspace-details.png @ 28.34s; media/state-04-visibility-selected.png @ 40.69s; https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/ |
| progress feedback | Confirm creation | Postman opens the new empty workspace Progress is observable as the distinct workspace active state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-visibility-selected.png @ 40.69s; media/state-05-workspace-active.png @ 53.05s; https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-workspace-details.png @ 28.34s; media/state-04-visibility-selected.png @ 40.69s; media/state-05-workspace-active.png @ 53.05s; https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/ |
| recovery and completion | Create or import the first request | The request opens within the workspace, proving first usable result The retained first request state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-workspace-active.png @ 53.05s; media/state-06-first-request.png @ 65.40s; https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at workspace list; the first advancing trigger is “Choose Create Workspace”.
- **Start/end:** Start is workspace list at 3.63s; end is first request at 65.40s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 72.667s at 15 fps (1090 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first request; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-workspace-list.png and media/state-02-workspace-form.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-workspace-details.png and media/state-04-visibility-selected.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Postman, Inc.
- **Product page:** https://learning.postman.com/docs/collaborating-in-postman/using-workspaces/creating-workspaces/
- **Original media URL:** https://www.youtube.com/watch?v=I10RCvMbPi0
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 72.667s, 1090 frames, 299195 bytes
- **SHA-256:** `24b6b09c995a7611a686b18c5478960c519a0ca7563b8d02cd7901c31dfd4a02`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
