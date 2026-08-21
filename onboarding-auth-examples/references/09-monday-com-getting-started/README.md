# monday.com — getting started

**Evidence status:** `complete`  
**Product/source:** [https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com](https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to get started | monday.com tutorials](https://www.youtube.com/watch?v=UMZeynogqqc) — monday.com

## Start-to-first-success journey

**Actor:** new monday.com board owner  
**Goal:** create a board and first item  
**Prerequisites:** monday.com account; board purpose

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and enter the workspace | monday.com shows board and template entry points | workspace | `media/state-01-workspace.png` and motion at 4.50s |
| 2 | Choose New board | monday.com opens template and board configuration | board creation | `media/state-02-board-creation.png` and motion at 19.79s |
| 3 | Select a template or blank board and name it | monday.com creates the board structure | board ready | `media/state-03-board-ready.png` and motion at 35.07s |
| 4 | Add the first item | monday.com persists the item row | item created | `media/state-04-item-created.png` and motion at 50.36s |
| 5 | Set an owner, status, or date | monday.com updates the item and gives cell-level feedback | item configured | `media/state-05-item-configured.png` and motion at 65.65s |
| 6 | Invite a teammate or use the board solo | The configured board remains available, proving first workflow success | board active | `media/state-06-board-active.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At board creation or board ready, invalid, expired, denied, or missing required input leaves the flow short of board active; evidence: media/state-02-board-creation.png, media/state-03-board-ready.png, and https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-board-ready.png through media/state-05-item-configured.png.
- **Recovery:** Return to the retained board creation or board ready requirement, correct or resend the blocking input, and resubmit; evidence: https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com.
- **Recovery:** Continue through the same terminal action until board active is visible in media/state-06-board-active.png and the motion at 80.940s.
- **Completion evidence:** board active retained at media/state-06-board-active.png and media/official-recording.mp4#t=80.940; source https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| workspace | [`media/state-01-workspace.png`](media/state-01-workspace.png) | media/official-recording.mp4#t=4.497 | 640×360 | `7e9164d0d99c43d25474b94be448e6c0ce586679dade5ee283c81b72dbd84351` |
| board creation | [`media/state-02-board-creation.png`](media/state-02-board-creation.png) | media/official-recording.mp4#t=19.785 | 640×360 | `349fa686070f9f435a27724a6f066c66e06a3c12885f4f5a60e27d6c64246ca0` |
| board ready | [`media/state-03-board-ready.png`](media/state-03-board-ready.png) | media/official-recording.mp4#t=35.074 | 640×360 | `0f944a6ab77e736c459702fe1780bd989a4e79391e258cda7526e44479ad0769` |
| item created | [`media/state-04-item-created.png`](media/state-04-item-created.png) | media/official-recording.mp4#t=50.362 | 640×360 | `30c69264e1821f1965fef38f261739d3c60dd8db1a8d111ddac0d9a5947c83d8` |
| item configured | [`media/state-05-item-configured.png`](media/state-05-item-configured.png) | media/official-recording.mp4#t=65.651 | 640×360 | `5004de54b8d17990f82692cb7edf836e022d3dfd0412b4956b38a11ff94e4584` |
| board active | [`media/state-06-board-active.png`](media/state-06-board-active.png) | media/official-recording.mp4#t=80.940 | 640×360 | `3b7eda30f5414af2d4fc820c9b628754f3ea68f47b6897ad993a0d4470eabd79` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and enter the workspace | monday.com shows board and template entry points The retained workspace state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-workspace.png @ 4.50s; https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com |
| focus and selection | Choose New board | monday.com opens template and board configuration The recording advances to board creation and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-workspace.png @ 4.50s; media/state-02-board-creation.png @ 19.79s; https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com |
| navigation | Select a template or blank board and name it | monday.com creates the board structure The navigation result is visible as board ready. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-board-creation.png @ 19.79s; media/state-03-board-ready.png @ 35.07s; https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com |
| confirmation | Add the first item | monday.com persists the item row The official recording shows the confirmed item created state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-board-ready.png @ 35.07s; media/state-04-item-created.png @ 50.36s; https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-board-creation.png @ 19.79s; media/state-03-board-ready.png @ 35.07s; media/state-04-item-created.png @ 50.36s; https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com |
| progress feedback | Set an owner, status, or date | monday.com updates the item and gives cell-level feedback Progress is observable as the distinct item configured state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-item-created.png @ 50.36s; media/state-05-item-configured.png @ 65.65s; https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-board-ready.png @ 35.07s; media/state-04-item-created.png @ 50.36s; media/state-05-item-configured.png @ 65.65s; https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com |
| recovery and completion | Invite a teammate or use the board solo | The configured board remains available, proving first workflow success The retained board active state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-item-configured.png @ 65.65s; media/state-06-board-active.png @ 80.94s; https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com |

## Motion behavior

- **Trigger:** The recorded sequence begins at workspace; the first advancing trigger is “Choose New board”.
- **Start/end:** Start is workspace at 4.50s; end is board active at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in board active; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-workspace.png and media/state-02-board-creation.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-board-ready.png and media/state-04-item-created.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** monday.com Ltd.
- **Product page:** https://support.monday.com/hc/en-us/articles/360002197259-How-to-get-started-with-monday-com
- **Original media URL:** https://www.youtube.com/watch?v=UMZeynogqqc
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 993423 bytes
- **SHA-256:** `2e73d9a149cfadbcb75c43c8003da0d2718829a1db4b2e21364b4745b761409f`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
