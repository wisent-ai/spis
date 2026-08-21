# Docker Desktop — sign in

**Evidence status:** `complete`  
**Product/source:** [https://docs.docker.com/desktop/setup/sign-in/](https://docs.docker.com/desktop/setup/sign-in/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Get Docker Desktop | Docker Concepts](https://www.youtube.com/watch?v=DWkJzYJFov0) — Docker

## Start-to-first-success journey

**Actor:** new Docker Desktop user  
**Goal:** authenticate Desktop and run the first container  
**Prerequisites:** installed Docker Desktop; Docker account

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Launch Docker Desktop and choose Sign in | Desktop opens the browser-mediated authentication route | sign-in handoff | `media/state-01-sign-in-handoff.png` and motion at 4.50s |
| 2 | Enter Docker account credentials | Docker validates identity and presents authorization | identity verified | `media/state-02-identity-verified.png` and motion at 19.79s |
| 3 | Approve the Desktop authorization | The browser confirms and returns control to Docker Desktop | authorization complete | `media/state-03-authorization-complete.png` and motion at 35.07s |
| 4 | Choose the effective organization if prompted | Desktop displays the signed-in account context | Desktop authenticated | `media/state-04-desktop-authenticated.png` and motion at 50.36s |
| 5 | Pull or select a starter image | Docker reports download and image availability | image ready | `media/state-05-image-ready.png` and motion at 65.65s |
| 6 | Run the image and open its output | The container reaches running state, proving first-success execution | first container | `media/state-06-first-container.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At identity verified or authorization complete, invalid, expired, denied, or missing required input leaves the flow short of first container; evidence: media/state-02-identity-verified.png, media/state-03-authorization-complete.png, and https://docs.docker.com/desktop/setup/sign-in/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-authorization-complete.png through media/state-05-image-ready.png.
- **Recovery:** Return to the retained identity verified or authorization complete requirement, correct or resend the blocking input, and resubmit; evidence: https://docs.docker.com/desktop/setup/sign-in/.
- **Recovery:** Continue through the same terminal action until first container is visible in media/state-06-first-container.png and the motion at 80.940s.
- **Completion evidence:** first container retained at media/state-06-first-container.png and media/official-recording.mp4#t=80.940; source https://docs.docker.com/desktop/setup/sign-in/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| sign-in handoff | [`media/state-01-sign-in-handoff.png`](media/state-01-sign-in-handoff.png) | media/official-recording.mp4#t=4.497 | 640×360 | `c4f201ff1b15611a810ef99920a83aad16556a7b6532f22f063245e75949a962` |
| identity verified | [`media/state-02-identity-verified.png`](media/state-02-identity-verified.png) | media/official-recording.mp4#t=19.785 | 640×360 | `ef5736245a7c5dedc8ca9f4ed103264111b7c3c45dde1d2b1e5743e447b549b0` |
| authorization complete | [`media/state-03-authorization-complete.png`](media/state-03-authorization-complete.png) | media/official-recording.mp4#t=35.074 | 640×360 | `3deac5facc13935ca6feca25bba98c977d25104470ebc49208c02a1da44097fc` |
| Desktop authenticated | [`media/state-04-desktop-authenticated.png`](media/state-04-desktop-authenticated.png) | media/official-recording.mp4#t=50.362 | 640×360 | `812f532b920b221982af402abb004e62d00201471362d6ad2ef538a0ff207c71` |
| image ready | [`media/state-05-image-ready.png`](media/state-05-image-ready.png) | media/official-recording.mp4#t=65.651 | 640×360 | `fbcbb2bffffc7543046e2bcd757f93a9502a8a8c53387bdac47d745a8fc541f0` |
| first container | [`media/state-06-first-container.png`](media/state-06-first-container.png) | media/official-recording.mp4#t=80.940 | 640×360 | `fe7bb1dcb9d645d2146d29a7ebd69587387500821e3eb1b0172e0efd0a6d8db1` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Launch Docker Desktop and choose Sign in | Desktop opens the browser-mediated authentication route The retained sign-in handoff state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-sign-in-handoff.png @ 4.50s; https://docs.docker.com/desktop/setup/sign-in/ |
| focus and selection | Enter Docker account credentials | Docker validates identity and presents authorization The recording advances to identity verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-sign-in-handoff.png @ 4.50s; media/state-02-identity-verified.png @ 19.79s; https://docs.docker.com/desktop/setup/sign-in/ |
| navigation | Approve the Desktop authorization | The browser confirms and returns control to Docker Desktop The navigation result is visible as authorization complete. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-verified.png @ 19.79s; media/state-03-authorization-complete.png @ 35.07s; https://docs.docker.com/desktop/setup/sign-in/ |
| confirmation | Choose the effective organization if prompted | Desktop displays the signed-in account context The official recording shows the confirmed Desktop authenticated state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-authorization-complete.png @ 35.07s; media/state-04-desktop-authenticated.png @ 50.36s; https://docs.docker.com/desktop/setup/sign-in/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-verified.png @ 19.79s; media/state-03-authorization-complete.png @ 35.07s; media/state-04-desktop-authenticated.png @ 50.36s; https://docs.docker.com/desktop/setup/sign-in/ |
| progress feedback | Pull or select a starter image | Docker reports download and image availability Progress is observable as the distinct image ready state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-desktop-authenticated.png @ 50.36s; media/state-05-image-ready.png @ 65.65s; https://docs.docker.com/desktop/setup/sign-in/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-authorization-complete.png @ 35.07s; media/state-04-desktop-authenticated.png @ 50.36s; media/state-05-image-ready.png @ 65.65s; https://docs.docker.com/desktop/setup/sign-in/ |
| recovery and completion | Run the image and open its output | The container reaches running state, proving first-success execution The retained first container state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-image-ready.png @ 65.65s; media/state-06-first-container.png @ 80.94s; https://docs.docker.com/desktop/setup/sign-in/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at sign-in handoff; the first advancing trigger is “Enter Docker account credentials”.
- **Start/end:** Start is sign-in handoff at 4.50s; end is first container at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first container; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-sign-in-handoff.png and media/state-02-identity-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-authorization-complete.png and media/state-04-desktop-authenticated.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Docker, Inc.
- **Product page:** https://docs.docker.com/desktop/setup/sign-in/
- **Original media URL:** https://www.youtube.com/watch?v=DWkJzYJFov0
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 641202 bytes
- **SHA-256:** `fb8cea25582fa5de9c0beb0565e876bdc2a8a9b8f0206d5f2c11b3256ac88ce3`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
