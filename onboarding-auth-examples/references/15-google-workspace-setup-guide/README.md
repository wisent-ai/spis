# Google Workspace — setup guide

**Evidence status:** `complete`  
**Product/source:** [https://support.google.com/a/answer/6365252](https://support.google.com/a/answer/6365252)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Sign up for Google Workspace (beginner's guide)](https://www.youtube.com/watch?v=QL5oIWBrhP0) — Google Workspace

## Start-to-first-success journey

**Actor:** Google Workspace administrator  
**Goal:** provision a domain and send the first managed email  
**Prerequisites:** business domain; DNS administrator access; admin identity

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Start Workspace sign-up and identify the organization | Google creates the administrator setup context | organization entry | `media/state-01-organization-entry.png` and motion at 4.50s |
| 2 | Provide or acquire the business domain | Google records the domain and presents verification instructions | domain selected | `media/state-02-domain-selected.png` and motion at 19.79s |
| 3 | Create the administrator identity | Google establishes the tenant admin account | admin created | `media/state-03-admin-created.png` and motion at 35.07s |
| 4 | Add the required DNS verification record | Google verifies domain control and unlocks managed services | domain verified | `media/state-04-domain-verified.png` and motion at 50.36s |
| 5 | Create a user and activate Gmail routing | Google lists the licensed user and service status | user provisioned | `media/state-05-user-provisioned.png` and motion at 65.65s |
| 6 | Send the first managed-domain email | The message is accepted by Gmail, proving first tenant success | first email | `media/state-06-first-email.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At domain selected or admin created, invalid, expired, denied, or missing required input leaves the flow short of first email; evidence: media/state-02-domain-selected.png, media/state-03-admin-created.png, and https://support.google.com/a/answer/6365252.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-admin-created.png through media/state-05-user-provisioned.png.
- **Recovery:** Return to the retained domain selected or admin created requirement, correct or resend the blocking input, and resubmit; evidence: https://support.google.com/a/answer/6365252.
- **Recovery:** Continue through the same terminal action until first email is visible in media/state-06-first-email.png and the motion at 80.940s.
- **Completion evidence:** first email retained at media/state-06-first-email.png and media/official-recording.mp4#t=80.940; source https://support.google.com/a/answer/6365252

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| organization entry | [`media/state-01-organization-entry.png`](media/state-01-organization-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `5dccc3bba8ee85f89737d12cb2671687e632f15b31b979e7c4a3947753143989` |
| domain selected | [`media/state-02-domain-selected.png`](media/state-02-domain-selected.png) | media/official-recording.mp4#t=19.785 | 640×360 | `625c09da9f9cc5c7698845b2c5c68aaa7320f026112902266faacd5ced8fc184` |
| admin created | [`media/state-03-admin-created.png`](media/state-03-admin-created.png) | media/official-recording.mp4#t=35.074 | 640×360 | `a3c840b791e3b6c817339e7da6ac17d928cd1faf047a4131ae5207a7d8c5df19` |
| domain verified | [`media/state-04-domain-verified.png`](media/state-04-domain-verified.png) | media/official-recording.mp4#t=50.362 | 640×360 | `e93661c9551cb7a1462330869425a6f6ccf6ab34a746c6c88622d7ba5aee9ee8` |
| user provisioned | [`media/state-05-user-provisioned.png`](media/state-05-user-provisioned.png) | media/official-recording.mp4#t=65.651 | 640×360 | `72f8ce8437b15e6b8839a7aa96496d09a493134c1713264371852a08adefd9ba` |
| first email | [`media/state-06-first-email.png`](media/state-06-first-email.png) | media/official-recording.mp4#t=80.940 | 640×360 | `e966b4b724e53e15693e9784b8c951afafb6ef2c83a73e672e620bbd4bab25f9` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Start Workspace sign-up and identify the organization | Google creates the administrator setup context The retained organization entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-organization-entry.png @ 4.50s; https://support.google.com/a/answer/6365252 |
| focus and selection | Provide or acquire the business domain | Google records the domain and presents verification instructions The recording advances to domain selected and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-organization-entry.png @ 4.50s; media/state-02-domain-selected.png @ 19.79s; https://support.google.com/a/answer/6365252 |
| navigation | Create the administrator identity | Google establishes the tenant admin account The navigation result is visible as admin created. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-domain-selected.png @ 19.79s; media/state-03-admin-created.png @ 35.07s; https://support.google.com/a/answer/6365252 |
| confirmation | Add the required DNS verification record | Google verifies domain control and unlocks managed services The official recording shows the confirmed domain verified state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-admin-created.png @ 35.07s; media/state-04-domain-verified.png @ 50.36s; https://support.google.com/a/answer/6365252 |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-domain-selected.png @ 19.79s; media/state-03-admin-created.png @ 35.07s; media/state-04-domain-verified.png @ 50.36s; https://support.google.com/a/answer/6365252 |
| progress feedback | Create a user and activate Gmail routing | Google lists the licensed user and service status Progress is observable as the distinct user provisioned state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-domain-verified.png @ 50.36s; media/state-05-user-provisioned.png @ 65.65s; https://support.google.com/a/answer/6365252 |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-admin-created.png @ 35.07s; media/state-04-domain-verified.png @ 50.36s; media/state-05-user-provisioned.png @ 65.65s; https://support.google.com/a/answer/6365252 |
| recovery and completion | Send the first managed-domain email | The message is accepted by Gmail, proving first tenant success The retained first email state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-user-provisioned.png @ 65.65s; media/state-06-first-email.png @ 80.94s; https://support.google.com/a/answer/6365252 |

## Motion behavior

- **Trigger:** The recorded sequence begins at organization entry; the first advancing trigger is “Provide or acquire the business domain”.
- **Start/end:** Start is organization entry at 4.50s; end is first email at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first email; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-organization-entry.png and media/state-02-domain-selected.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-admin-created.png and media/state-04-domain-verified.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Google LLC
- **Product page:** https://support.google.com/a/answer/6365252
- **Original media URL:** https://www.youtube.com/watch?v=QL5oIWBrhP0
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 281905 bytes
- **SHA-256:** `a0a36b7db646820730f712c962d6eda155fa4e39deb2054d33f7e2d071c48b38`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
