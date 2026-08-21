# Twilio — account setup

**Evidence status:** `complete`  
**Product/source:** [https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account](https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Getting started with Twilio Verify](https://www.youtube.com/watch?v=UBjMm_nb45U) — Twilio

## Start-to-first-success journey

**Actor:** new Twilio developer  
**Goal:** verify identity and complete the first verification check  
**Prerequisites:** Twilio account; phone able to receive verification; test application or console access

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Create or sign in to a Twilio account | Twilio opens account and trial setup | account entry | `media/state-01-account-entry.png` and motion at 4.50s |
| 2 | Verify email and phone ownership | Twilio marks identity channels verified | identity verified | `media/state-02-identity-verified.png` and motion at 19.79s |
| 3 | Create or select the project | Twilio exposes account credentials and console navigation | project ready | `media/state-03-project-ready.png` and motion at 35.07s |
| 4 | Create a Verify service or use the guided setup | Twilio returns service configuration | verification service | `media/state-04-verification-service.png` and motion at 50.36s |
| 5 | Send a one-time code to the test recipient | Twilio reports delivery state | code sent | `media/state-05-code-sent.png` and motion at 65.65s |
| 6 | Submit the received code for checking | Twilio returns approved status, proving first verification success | verification approved | `media/state-06-verification-approved.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At identity verified or project ready, invalid, expired, denied, or missing required input leaves the flow short of verification approved; evidence: media/state-02-identity-verified.png, media/state-03-project-ready.png, and https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-project-ready.png through media/state-05-code-sent.png.
- **Recovery:** Return to the retained identity verified or project ready requirement, correct or resend the blocking input, and resubmit; evidence: https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account.
- **Recovery:** Continue through the same terminal action until verification approved is visible in media/state-06-verification-approved.png and the motion at 80.940s.
- **Completion evidence:** verification approved retained at media/state-06-verification-approved.png and media/official-recording.mp4#t=80.940; source https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account entry | [`media/state-01-account-entry.png`](media/state-01-account-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `a076dfd9bc43b58712a7a77403f13eaf152724020c2c4da9d4b4b282ada64cbc` |
| identity verified | [`media/state-02-identity-verified.png`](media/state-02-identity-verified.png) | media/official-recording.mp4#t=19.785 | 640×360 | `5b3ff3e25e4912ef8c244c4acccc423efdf92fd552555e17088ae024a91397f9` |
| project ready | [`media/state-03-project-ready.png`](media/state-03-project-ready.png) | media/official-recording.mp4#t=35.074 | 640×360 | `a38f180c7b635f36853033563cb99178deb14a86fd5556749b1a94e1cff35225` |
| verification service | [`media/state-04-verification-service.png`](media/state-04-verification-service.png) | media/official-recording.mp4#t=50.362 | 640×360 | `e895772a3c31f7c5b671618f2329ec898c240eb5f3c4455f4db0e38a06cf52f4` |
| code sent | [`media/state-05-code-sent.png`](media/state-05-code-sent.png) | media/official-recording.mp4#t=65.651 | 640×360 | `26f8030b650d7672746a2dd163c3271a895ab4eafa3cec89ebbc62303b324c0e` |
| verification approved | [`media/state-06-verification-approved.png`](media/state-06-verification-approved.png) | media/official-recording.mp4#t=80.940 | 640×360 | `3ce37dc50389974a0ddb64aefb8285e641e6867d1513d9ccc948b826a29644a1` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Create or sign in to a Twilio account | Twilio opens account and trial setup The retained account entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-entry.png @ 4.50s; https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account |
| focus and selection | Verify email and phone ownership | Twilio marks identity channels verified The recording advances to identity verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-entry.png @ 4.50s; media/state-02-identity-verified.png @ 19.79s; https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account |
| navigation | Create or select the project | Twilio exposes account credentials and console navigation The navigation result is visible as project ready. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-verified.png @ 19.79s; media/state-03-project-ready.png @ 35.07s; https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account |
| confirmation | Create a Verify service or use the guided setup | Twilio returns service configuration The official recording shows the confirmed verification service state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-project-ready.png @ 35.07s; media/state-04-verification-service.png @ 50.36s; https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-verified.png @ 19.79s; media/state-03-project-ready.png @ 35.07s; media/state-04-verification-service.png @ 50.36s; https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account |
| progress feedback | Send a one-time code to the test recipient | Twilio reports delivery state Progress is observable as the distinct code sent state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-verification-service.png @ 50.36s; media/state-05-code-sent.png @ 65.65s; https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-project-ready.png @ 35.07s; media/state-04-verification-service.png @ 50.36s; media/state-05-code-sent.png @ 65.65s; https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account |
| recovery and completion | Submit the received code for checking | Twilio returns approved status, proving first verification success The retained verification approved state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-code-sent.png @ 65.65s; media/state-06-verification-approved.png @ 80.94s; https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account |

## Motion behavior

- **Trigger:** The recorded sequence begins at account entry; the first advancing trigger is “Verify email and phone ownership”.
- **Start/end:** Start is account entry at 4.50s; end is verification approved at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in verification approved; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-entry.png and media/state-02-identity-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-project-ready.png and media/state-04-verification-service.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Twilio Inc.
- **Product page:** https://www.twilio.com/docs/usage/tutorials/how-to-use-your-free-trial-account
- **Original media URL:** https://www.youtube.com/watch?v=UBjMm_nb45U
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 507047 bytes
- **SHA-256:** `f43f41be9a886c6f82aa519d692cc79de7804aadba3ef07965378564257b07c8`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
