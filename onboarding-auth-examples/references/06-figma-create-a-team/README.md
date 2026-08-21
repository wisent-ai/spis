# Figma — create a team

**Evidence status:** `complete`  
**Product/source:** [https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team](https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Figma tutorial: Setup account, teams, projects, and files [1 of 8]](https://www.youtube.com/watch?v=hrHL2VLMl7g) — Figma

## Start-to-first-success journey

**Actor:** new Figma team owner  
**Goal:** create a team and first collaborative file  
**Prerequisites:** Figma account; team name

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and open the file browser | Figma shows account, drafts, and team creation controls | file browser | `media/state-01-file-browser.png` and motion at 4.50s |
| 2 | Choose Create team | Figma opens team naming and setup | team creation | `media/state-02-team-creation.png` and motion at 19.79s |
| 3 | Enter the team name | Figma validates the name and advances to membership or plan choices | team named | `media/state-03-team-named.png` and motion at 35.07s |
| 4 | Invite members or continue with the visible skip route | Figma records the membership decision | membership decision | `media/state-04-membership-decision.png` and motion at 50.36s |
| 5 | Confirm the plan or starter option | Figma creates the team space in the file browser | team active | `media/state-05-team-active.png` and motion at 65.65s |
| 6 | Create the first team file | The editor opens under the new team, proving first collaborative workspace success | first file | `media/state-06-first-file.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At team creation or team named, invalid, expired, denied, or missing required input leaves the flow short of first file; evidence: media/state-02-team-creation.png, media/state-03-team-named.png, and https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-team-named.png through media/state-05-team-active.png.
- **Recovery:** Return to the retained team creation or team named requirement, correct or resend the blocking input, and resubmit; evidence: https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team.
- **Recovery:** Continue through the same terminal action until first file is visible in media/state-06-first-file.png and the motion at 80.940s.
- **Completion evidence:** first file retained at media/state-06-first-file.png and media/official-recording.mp4#t=80.940; source https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| file browser | [`media/state-01-file-browser.png`](media/state-01-file-browser.png) | media/official-recording.mp4#t=4.497 | 640×360 | `1168f0a32bf69fdc7f6792d9d30315779f356ae208380399fb03aedce7b31619` |
| team creation | [`media/state-02-team-creation.png`](media/state-02-team-creation.png) | media/official-recording.mp4#t=19.785 | 640×360 | `b76e897e601537e9792b34e3656f080e13337daaa892c9abe6fbef1dc77f6970` |
| team named | [`media/state-03-team-named.png`](media/state-03-team-named.png) | media/official-recording.mp4#t=35.074 | 640×360 | `39e2a96018fe1d04b00bce93d7db716356d434e4b00922b422a186367ab7bf6d` |
| membership decision | [`media/state-04-membership-decision.png`](media/state-04-membership-decision.png) | media/official-recording.mp4#t=50.362 | 640×360 | `616f92760a4cac5598903ad038841c8e37f4de9e5039f5d36e3535ac8a720ddf` |
| team active | [`media/state-05-team-active.png`](media/state-05-team-active.png) | media/official-recording.mp4#t=65.651 | 640×360 | `f53adaf4219583528d615672ab769de20517c3a2e1ff3936193e542781bd750b` |
| first file | [`media/state-06-first-file.png`](media/state-06-first-file.png) | media/official-recording.mp4#t=80.940 | 640×360 | `6e85137d01721ea91a00e35d8b70d2fa959975da85080628437ccf19c01da2a5` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and open the file browser | Figma shows account, drafts, and team creation controls The retained file browser state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-file-browser.png @ 4.50s; https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team |
| focus and selection | Choose Create team | Figma opens team naming and setup The recording advances to team creation and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-file-browser.png @ 4.50s; media/state-02-team-creation.png @ 19.79s; https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team |
| navigation | Enter the team name | Figma validates the name and advances to membership or plan choices The navigation result is visible as team named. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-team-creation.png @ 19.79s; media/state-03-team-named.png @ 35.07s; https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team |
| confirmation | Invite members or continue with the visible skip route | Figma records the membership decision The official recording shows the confirmed membership decision state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-team-named.png @ 35.07s; media/state-04-membership-decision.png @ 50.36s; https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-team-creation.png @ 19.79s; media/state-03-team-named.png @ 35.07s; media/state-04-membership-decision.png @ 50.36s; https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team |
| progress feedback | Confirm the plan or starter option | Figma creates the team space in the file browser Progress is observable as the distinct team active state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-membership-decision.png @ 50.36s; media/state-05-team-active.png @ 65.65s; https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-team-named.png @ 35.07s; media/state-04-membership-decision.png @ 50.36s; media/state-05-team-active.png @ 65.65s; https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team |
| recovery and completion | Create the first team file | The editor opens under the new team, proving first collaborative workspace success The retained first file state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-team-active.png @ 65.65s; media/state-06-first-file.png @ 80.94s; https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team |

## Motion behavior

- **Trigger:** The recorded sequence begins at file browser; the first advancing trigger is “Choose Create team”.
- **Start/end:** Start is file browser at 4.50s; end is first file at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first file; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-file-browser.png and media/state-02-team-creation.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-team-named.png and media/state-04-membership-decision.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Figma, Inc.
- **Product page:** https://help.figma.com/hc/en-us/articles/360038006494-Create-a-team
- **Original media URL:** https://www.youtube.com/watch?v=hrHL2VLMl7g
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 362738 bytes
- **SHA-256:** `78a4744ec128525387c0670d8589608999b9b690781d81352b4122a7adcab066`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
