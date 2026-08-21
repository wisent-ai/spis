# Trello — create a Workspace

**Evidence status:** `complete`  
**Product/source:** [https://support.atlassian.com/trello/docs/creating-a-new-workspace/](https://support.atlassian.com/trello/docs/creating-a-new-workspace/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to create a new Trello Workspace | Trello Administration](https://www.youtube.com/watch?v=JGLkH_EveHI) — Atlassian

## Start-to-first-success journey

**Actor:** new Trello workspace owner  
**Goal:** create a Workspace and first board  
**Prerequisites:** Atlassian or Trello account; workspace name

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and open the Workspaces menu | Trello shows current workspaces and create action | workspace menu | `media/state-01-workspace-menu.png` and motion at 4.50s |
| 2 | Choose Create Workspace | Trello opens the lightweight workspace form | workspace form | `media/state-02-workspace-form.png` and motion at 19.79s |
| 3 | Enter the workspace name and optional description | Trello validates the required name | workspace details | `media/state-03-workspace-details.png` and motion at 35.07s |
| 4 | Confirm creation and choose the available plan route | Trello adds the Workspace to navigation | workspace active | `media/state-04-workspace-active.png` and motion at 50.36s |
| 5 | Invite members or continue without invitations | Trello records the membership decision | membership decision | `media/state-05-membership-decision.png` and motion at 65.65s |
| 6 | Create or open the first board | The board is listed under the Workspace, proving first usable result | first board | `media/state-06-first-board.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At workspace form or workspace details, invalid, expired, denied, or missing required input leaves the flow short of first board; evidence: media/state-02-workspace-form.png, media/state-03-workspace-details.png, and https://support.atlassian.com/trello/docs/creating-a-new-workspace/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-workspace-details.png through media/state-05-membership-decision.png.
- **Recovery:** Return to the retained workspace form or workspace details requirement, correct or resend the blocking input, and resubmit; evidence: https://support.atlassian.com/trello/docs/creating-a-new-workspace/.
- **Recovery:** Continue through the same terminal action until first board is visible in media/state-06-first-board.png and the motion at 80.940s.
- **Completion evidence:** first board retained at media/state-06-first-board.png and media/official-recording.mp4#t=80.940; source https://support.atlassian.com/trello/docs/creating-a-new-workspace/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| workspace menu | [`media/state-01-workspace-menu.png`](media/state-01-workspace-menu.png) | media/official-recording.mp4#t=4.497 | 640×360 | `4373b9171b802f4df829ca68233b143f5d5d1421a619f2388a7b31645ec401ad` |
| workspace form | [`media/state-02-workspace-form.png`](media/state-02-workspace-form.png) | media/official-recording.mp4#t=19.785 | 640×360 | `e12402d3df2d22a5b1ad52bc8341bec26b3739593f722e63a9e941014c92521d` |
| workspace details | [`media/state-03-workspace-details.png`](media/state-03-workspace-details.png) | media/official-recording.mp4#t=35.074 | 640×360 | `bdc9e2d96d0c5045e46df5f64b845757d1e333983a454cae02bb8b99fd67e587` |
| workspace active | [`media/state-04-workspace-active.png`](media/state-04-workspace-active.png) | media/official-recording.mp4#t=50.362 | 640×360 | `f22b959d61dc22894aa0a38b480d8ae57119e32015c278765d4900d663b79128` |
| membership decision | [`media/state-05-membership-decision.png`](media/state-05-membership-decision.png) | media/official-recording.mp4#t=65.651 | 640×360 | `89a9e5ae9dd7a07d09fadaa53ad033b0ba9eaa7d0432b781a3513293495a292b` |
| first board | [`media/state-06-first-board.png`](media/state-06-first-board.png) | media/official-recording.mp4#t=80.940 | 640×360 | `d6c86f0f99a9a5c743724f8587eae8970e31aba33c93066f255982d994666a36` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and open the Workspaces menu | Trello shows current workspaces and create action The retained workspace menu state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-workspace-menu.png @ 4.50s; https://support.atlassian.com/trello/docs/creating-a-new-workspace/ |
| focus and selection | Choose Create Workspace | Trello opens the lightweight workspace form The recording advances to workspace form and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-workspace-menu.png @ 4.50s; media/state-02-workspace-form.png @ 19.79s; https://support.atlassian.com/trello/docs/creating-a-new-workspace/ |
| navigation | Enter the workspace name and optional description | Trello validates the required name The navigation result is visible as workspace details. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-workspace-form.png @ 19.79s; media/state-03-workspace-details.png @ 35.07s; https://support.atlassian.com/trello/docs/creating-a-new-workspace/ |
| confirmation | Confirm creation and choose the available plan route | Trello adds the Workspace to navigation The official recording shows the confirmed workspace active state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-workspace-details.png @ 35.07s; media/state-04-workspace-active.png @ 50.36s; https://support.atlassian.com/trello/docs/creating-a-new-workspace/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-workspace-form.png @ 19.79s; media/state-03-workspace-details.png @ 35.07s; media/state-04-workspace-active.png @ 50.36s; https://support.atlassian.com/trello/docs/creating-a-new-workspace/ |
| progress feedback | Invite members or continue without invitations | Trello records the membership decision Progress is observable as the distinct membership decision state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-workspace-active.png @ 50.36s; media/state-05-membership-decision.png @ 65.65s; https://support.atlassian.com/trello/docs/creating-a-new-workspace/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-workspace-details.png @ 35.07s; media/state-04-workspace-active.png @ 50.36s; media/state-05-membership-decision.png @ 65.65s; https://support.atlassian.com/trello/docs/creating-a-new-workspace/ |
| recovery and completion | Create or open the first board | The board is listed under the Workspace, proving first usable result The retained first board state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-membership-decision.png @ 65.65s; media/state-06-first-board.png @ 80.94s; https://support.atlassian.com/trello/docs/creating-a-new-workspace/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at workspace menu; the first advancing trigger is “Choose Create Workspace”.
- **Start/end:** Start is workspace menu at 4.50s; end is first board at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first board; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-workspace-menu.png and media/state-02-workspace-form.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-workspace-details.png and media/state-04-workspace-active.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Atlassian Pty Ltd
- **Product page:** https://support.atlassian.com/trello/docs/creating-a-new-workspace/
- **Original media URL:** https://www.youtube.com/watch?v=JGLkH_EveHI
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 583557 bytes
- **SHA-256:** `f67ccf55c8c97dbacf84e21d31af7ea4c1b26f567459d101c54ee0d27b0f7672`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
