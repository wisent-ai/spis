# Cisco Webex — get started

**Evidence status:** `complete`  
**Product/source:** [https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App](https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Getting started with Webex Messaging](https://www.youtube.com/watch?v=XAYamgS9zUw) — Webex

## Start-to-first-success journey

**Actor:** new Webex user  
**Goal:** sign in and send the first message  
**Prerequisites:** Webex account or organization invitation; camera and microphone decision

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Launch Webex and enter the work email | Webex resolves the account or organization sign-in route | identity lookup | `media/state-01-identity-lookup.png` and motion at 4.50s |
| 2 | Complete password, SSO, or verification challenge | Webex returns to the application as the authenticated user | signed in | `media/state-02-signed-in.png` and motion at 19.79s |
| 3 | Accept or decline requested notification and media permissions | Webex records each permission decision without hiding the app | permissions decided | `media/state-03-permissions-decided.png` and motion at 35.07s |
| 4 | Review or complete profile identity | Webex displays the user in the navigation context | profile ready | `media/state-04-profile-ready.png` and motion at 50.36s |
| 5 | Create or open a space | Webex opens the conversation surface | space open | `media/state-05-space-open.png` and motion at 65.65s |
| 6 | Send the first message | The message appears in the space, proving first communication success | first message | `media/state-06-first-message.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At signed in or permissions decided, invalid, expired, denied, or missing required input leaves the flow short of first message; evidence: media/state-02-signed-in.png, media/state-03-permissions-decided.png, and https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-permissions-decided.png through media/state-05-space-open.png.
- **Recovery:** Return to the retained signed in or permissions decided requirement, correct or resend the blocking input, and resubmit; evidence: https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App.
- **Recovery:** Continue through the same terminal action until first message is visible in media/state-06-first-message.png and the motion at 80.940s.
- **Completion evidence:** first message retained at media/state-06-first-message.png and media/official-recording.mp4#t=80.940; source https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| identity lookup | [`media/state-01-identity-lookup.png`](media/state-01-identity-lookup.png) | media/official-recording.mp4#t=4.497 | 640×360 | `b86c5805b3bea49ca65ffba1fff0872e2f238c6fdd28b7ebf0cdf758a1beb000` |
| signed in | [`media/state-02-signed-in.png`](media/state-02-signed-in.png) | media/official-recording.mp4#t=19.785 | 640×360 | `00ced9fcfb7ec22c9e61d3df579ad9b23912590509d5a91ee2d4221f8ad13ce3` |
| permissions decided | [`media/state-03-permissions-decided.png`](media/state-03-permissions-decided.png) | media/official-recording.mp4#t=35.074 | 640×360 | `bcc25e87088cdd12a4881e542a672f75537454e7c855d7b9f9cc148acf3c4108` |
| profile ready | [`media/state-04-profile-ready.png`](media/state-04-profile-ready.png) | media/official-recording.mp4#t=50.362 | 640×360 | `f6935afc4f2b1db119c504c91b73c85cf5f76254ab27d8cd426b5d2fb5a88044` |
| space open | [`media/state-05-space-open.png`](media/state-05-space-open.png) | media/official-recording.mp4#t=65.651 | 640×360 | `50d3b7b9670d974ff6e210086a804fa509588a99ea003d5da91f3fb17cd8ef19` |
| first message | [`media/state-06-first-message.png`](media/state-06-first-message.png) | media/official-recording.mp4#t=80.940 | 640×360 | `9879d617d497d367c21d9c1f66c5c1338d5252acf5fd1a5941db8201688b627b` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Launch Webex and enter the work email | Webex resolves the account or organization sign-in route The retained identity lookup state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-identity-lookup.png @ 4.50s; https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App |
| focus and selection | Complete password, SSO, or verification challenge | Webex returns to the application as the authenticated user The recording advances to signed in and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-identity-lookup.png @ 4.50s; media/state-02-signed-in.png @ 19.79s; https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App |
| navigation | Accept or decline requested notification and media permissions | Webex records each permission decision without hiding the app The navigation result is visible as permissions decided. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-signed-in.png @ 19.79s; media/state-03-permissions-decided.png @ 35.07s; https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App |
| confirmation | Review or complete profile identity | Webex displays the user in the navigation context The official recording shows the confirmed profile ready state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-permissions-decided.png @ 35.07s; media/state-04-profile-ready.png @ 50.36s; https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-signed-in.png @ 19.79s; media/state-03-permissions-decided.png @ 35.07s; media/state-04-profile-ready.png @ 50.36s; https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App |
| progress feedback | Create or open a space | Webex opens the conversation surface Progress is observable as the distinct space open state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-profile-ready.png @ 50.36s; media/state-05-space-open.png @ 65.65s; https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-permissions-decided.png @ 35.07s; media/state-04-profile-ready.png @ 50.36s; media/state-05-space-open.png @ 65.65s; https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App |
| recovery and completion | Send the first message | The message appears in the space, proving first communication success The retained first message state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-space-open.png @ 65.65s; media/state-06-first-message.png @ 80.94s; https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App |

## Motion behavior

- **Trigger:** The recorded sequence begins at identity lookup; the first advancing trigger is “Complete password, SSO, or verification challenge”.
- **Start/end:** Start is identity lookup at 4.50s; end is first message at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first message; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-identity-lookup.png and media/state-02-signed-in.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-permissions-decided.png and media/state-04-profile-ready.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Cisco Systems, Inc.
- **Product page:** https://help.webex.com/en-us/article/nrbgeodb/Get-started-with-Webex-App
- **Original media URL:** https://www.youtube.com/watch?v=XAYamgS9zUw
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 686775 bytes
- **SHA-256:** `e93486bed5507b71703b3b454a4a46786db9750f3645fc8a10da1e1dd05d3492`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
