# Asana — account setup

**Evidence status:** `complete`  
**Product/source:** [https://asana.com/guide/get-started/begin/quick-start](https://asana.com/guide/get-started/begin/quick-start)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to get started with Asana | Beginner overview 2024](https://www.youtube.com/watch?v=hcY-2Xux2oI) — Asana

## Start-to-first-success journey

**Actor:** new Asana member  
**Goal:** finish account setup and create the first tracked work item  
**Prerequisites:** email address; team or work context

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Start account creation with email or an available identity provider | Asana opens identity verification and profile setup | account entry | `media/state-01-account-entry.png` and motion at 4.50s |
| 2 | Complete the email or provider verification | Asana marks the identity accepted and advances | identity verified | `media/state-02-identity-verified.png` and motion at 19.79s |
| 3 | Enter profile and organization context | Asana tailors the setup questions and team suggestions | profile context | `media/state-03-profile-context.png` and motion at 35.07s |
| 4 | Choose the primary work use and team structure | Asana prepares a starter workspace | workspace configured | `media/state-04-workspace-configured.png` and motion at 50.36s |
| 5 | Create or select a starter project | Asana opens the project surface with an empty work list | project ready | `media/state-05-project-ready.png` and motion at 65.65s |
| 6 | Add the first task | The task persists in the project, proving first-success work tracking | first task | `media/state-06-first-task.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At identity verified or profile context, invalid, expired, denied, or missing required input leaves the flow short of first task; evidence: media/state-02-identity-verified.png, media/state-03-profile-context.png, and https://asana.com/guide/get-started/begin/quick-start.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-profile-context.png through media/state-05-project-ready.png.
- **Recovery:** Return to the retained identity verified or profile context requirement, correct or resend the blocking input, and resubmit; evidence: https://asana.com/guide/get-started/begin/quick-start.
- **Recovery:** Continue through the same terminal action until first task is visible in media/state-06-first-task.png and the motion at 80.940s.
- **Completion evidence:** first task retained at media/state-06-first-task.png and media/official-recording.mp4#t=80.940; source https://asana.com/guide/get-started/begin/quick-start

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account entry | [`media/state-01-account-entry.png`](media/state-01-account-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `ca835b40093418d04854dbd38d549ff1b6ae53957928d4751023d73eb991aebb` |
| identity verified | [`media/state-02-identity-verified.png`](media/state-02-identity-verified.png) | media/official-recording.mp4#t=19.785 | 640×360 | `454fcd39d53e77924a7861331a0b41a7d1566bc6954279103dc2fecc7b1570d9` |
| profile context | [`media/state-03-profile-context.png`](media/state-03-profile-context.png) | media/official-recording.mp4#t=35.074 | 640×360 | `fe63f48a2dab6060a4f7e9801728854e58a7a5e20844a621cc5c77486406adac` |
| workspace configured | [`media/state-04-workspace-configured.png`](media/state-04-workspace-configured.png) | media/official-recording.mp4#t=50.362 | 640×360 | `3ddf4629a276abac21bbbc85ee5b44196b4472194e10a9e42156b7dc7da53d5c` |
| project ready | [`media/state-05-project-ready.png`](media/state-05-project-ready.png) | media/official-recording.mp4#t=65.651 | 640×360 | `8a3eb2954cc5969e34f624054619e41f6cbdf6d0366fa885adfcdfada0d3bda0` |
| first task | [`media/state-06-first-task.png`](media/state-06-first-task.png) | media/official-recording.mp4#t=80.940 | 640×360 | `69d8df2b03c7644539b58eb05e3adede20ac8a3932904839baa0d763eef3f24b` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Start account creation with email or an available identity provider | Asana opens identity verification and profile setup The retained account entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-entry.png @ 4.50s; https://asana.com/guide/get-started/begin/quick-start |
| focus and selection | Complete the email or provider verification | Asana marks the identity accepted and advances The recording advances to identity verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-entry.png @ 4.50s; media/state-02-identity-verified.png @ 19.79s; https://asana.com/guide/get-started/begin/quick-start |
| navigation | Enter profile and organization context | Asana tailors the setup questions and team suggestions The navigation result is visible as profile context. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-verified.png @ 19.79s; media/state-03-profile-context.png @ 35.07s; https://asana.com/guide/get-started/begin/quick-start |
| confirmation | Choose the primary work use and team structure | Asana prepares a starter workspace The official recording shows the confirmed workspace configured state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-profile-context.png @ 35.07s; media/state-04-workspace-configured.png @ 50.36s; https://asana.com/guide/get-started/begin/quick-start |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-verified.png @ 19.79s; media/state-03-profile-context.png @ 35.07s; media/state-04-workspace-configured.png @ 50.36s; https://asana.com/guide/get-started/begin/quick-start |
| progress feedback | Create or select a starter project | Asana opens the project surface with an empty work list Progress is observable as the distinct project ready state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-workspace-configured.png @ 50.36s; media/state-05-project-ready.png @ 65.65s; https://asana.com/guide/get-started/begin/quick-start |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-profile-context.png @ 35.07s; media/state-04-workspace-configured.png @ 50.36s; media/state-05-project-ready.png @ 65.65s; https://asana.com/guide/get-started/begin/quick-start |
| recovery and completion | Add the first task | The task persists in the project, proving first-success work tracking The retained first task state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-project-ready.png @ 65.65s; media/state-06-first-task.png @ 80.94s; https://asana.com/guide/get-started/begin/quick-start |

## Motion behavior

- **Trigger:** The recorded sequence begins at account entry; the first advancing trigger is “Complete the email or provider verification”.
- **Start/end:** Start is account entry at 4.50s; end is first task at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first task; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-entry.png and media/state-02-identity-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-profile-context.png and media/state-04-workspace-configured.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Asana, Inc.
- **Product page:** https://asana.com/guide/get-started/begin/quick-start
- **Original media URL:** https://www.youtube.com/watch?v=hcY-2Xux2oI
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 627568 bytes
- **SHA-256:** `603b6343084794b88ff3b7a91bf26bf711b340a208712ab9cf85995479fa7096`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
