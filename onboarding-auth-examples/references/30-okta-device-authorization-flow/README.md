# Okta — device authorization flow

**Evidence status:** `complete`  
**Product/source:** [https://developer.okta.com/docs/guides/device-authorization-grant/main/](https://developer.okta.com/docs/guides/device-authorization-grant/main/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [OAuth Happy Hour - Pushed Authorization Requests, Device Flow, Okta+Auth0 Developer Day](https://www.youtube.com/watch?v=irkrhuLiPbc) — OktaDev

## Start-to-first-success journey

**Actor:** user authorizing an input-constrained device  
**Goal:** approve the device and let its token request complete  
**Prerequisites:** Okta authorization server with device grant enabled; separate browser-capable device

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Start the client on the input-constrained device | Okta returns a verification URI, user code, expiry, and polling interval | device code issued | `media/state-01-device-code-issued.png` and motion at 4.50s |
| 2 | Open the verification URI on a second device | Okta presents the device-authorization entry surface | verification page | `media/state-02-verification-page.png` and motion at 19.80s |
| 3 | Enter the displayed user code | Okta resolves the pending client request | code accepted | `media/state-03-code-accepted.png` and motion at 35.10s |
| 4 | Authenticate and review requested access | Okta displays the identity and consent decision | consent | `media/state-04-consent.png` and motion at 50.40s |
| 5 | Approve the request | Okta marks the device authorization approved | authorization approved | `media/state-05-authorization-approved.png` and motion at 65.70s |
| 6 | Return to the original device | Its polling request receives tokens and continues, proving first-success authorization | device authenticated | `media/state-06-device-authenticated.png` and motion at 81.00s |

### Failure and recovery

- **Failure:** At verification page or code accepted, invalid, expired, denied, or missing required input leaves the flow short of device authenticated; evidence: media/state-02-verification-page.png, media/state-03-code-accepted.png, and https://developer.okta.com/docs/guides/device-authorization-grant/main/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-code-accepted.png through media/state-05-authorization-approved.png.
- **Recovery:** Return to the retained verification page or code accepted requirement, correct or resend the blocking input, and resubmit; evidence: https://developer.okta.com/docs/guides/device-authorization-grant/main/.
- **Recovery:** Continue through the same terminal action until device authenticated is visible in media/state-06-device-authenticated.png and the motion at 81.000s.
- **Completion evidence:** device authenticated retained at media/state-06-device-authenticated.png and media/official-recording.mp4#t=81.000; source https://developer.okta.com/docs/guides/device-authorization-grant/main/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| device code issued | [`media/state-01-device-code-issued.png`](media/state-01-device-code-issued.png) | media/official-recording.mp4#t=4.500 | 640×360 | `ec8ab8abd337cd91a2ca86fb09384a15976c89b97f68bd38b83b0e447a847363` |
| verification page | [`media/state-02-verification-page.png`](media/state-02-verification-page.png) | media/official-recording.mp4#t=19.800 | 640×360 | `c11ff7f2bc0191566a2a31d26e816e34a7f2a6b89ccf7da5bab0493a9bf444eb` |
| code accepted | [`media/state-03-code-accepted.png`](media/state-03-code-accepted.png) | media/official-recording.mp4#t=35.100 | 640×360 | `92ffcfd4c1b9f1b120dc2701289b78b997c1eac94971bfd27b43eda218d0eea8` |
| consent | [`media/state-04-consent.png`](media/state-04-consent.png) | media/official-recording.mp4#t=50.400 | 640×360 | `7bbaee9cb0ca62bcfdcf592b4f6ccd3d137101b8410d80369fef831b35169f3c` |
| authorization approved | [`media/state-05-authorization-approved.png`](media/state-05-authorization-approved.png) | media/official-recording.mp4#t=65.700 | 640×360 | `1054f0801e17e9cbc091ba23e866db05054a3ce455e4bebbf4a1f4f3b957370b` |
| device authenticated | [`media/state-06-device-authenticated.png`](media/state-06-device-authenticated.png) | media/official-recording.mp4#t=81.000 | 640×360 | `ad2fc14b87d78503ae4972988a8727160981f6c3f546f92f051b63415fb73781` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Start the client on the input-constrained device | Okta returns a verification URI, user code, expiry, and polling interval The retained device code issued state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-device-code-issued.png @ 4.50s; https://developer.okta.com/docs/guides/device-authorization-grant/main/ |
| focus and selection | Open the verification URI on a second device | Okta presents the device-authorization entry surface The recording advances to verification page and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-device-code-issued.png @ 4.50s; media/state-02-verification-page.png @ 19.80s; https://developer.okta.com/docs/guides/device-authorization-grant/main/ |
| navigation | Enter the displayed user code | Okta resolves the pending client request The navigation result is visible as code accepted. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-verification-page.png @ 19.80s; media/state-03-code-accepted.png @ 35.10s; https://developer.okta.com/docs/guides/device-authorization-grant/main/ |
| confirmation | Authenticate and review requested access | Okta displays the identity and consent decision The official recording shows the confirmed consent state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-code-accepted.png @ 35.10s; media/state-04-consent.png @ 50.40s; https://developer.okta.com/docs/guides/device-authorization-grant/main/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-verification-page.png @ 19.80s; media/state-03-code-accepted.png @ 35.10s; media/state-04-consent.png @ 50.40s; https://developer.okta.com/docs/guides/device-authorization-grant/main/ |
| progress feedback | Approve the request | Okta marks the device authorization approved Progress is observable as the distinct authorization approved state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-consent.png @ 50.40s; media/state-05-authorization-approved.png @ 65.70s; https://developer.okta.com/docs/guides/device-authorization-grant/main/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-code-accepted.png @ 35.10s; media/state-04-consent.png @ 50.40s; media/state-05-authorization-approved.png @ 65.70s; https://developer.okta.com/docs/guides/device-authorization-grant/main/ |
| recovery and completion | Return to the original device | Its polling request receives tokens and continues, proving first-success authorization The retained device authenticated state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-authorization-approved.png @ 65.70s; media/state-06-device-authenticated.png @ 81.00s; https://developer.okta.com/docs/guides/device-authorization-grant/main/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at device code issued; the first advancing trigger is “Open the verification URI on a second device”.
- **Start/end:** Start is device code issued at 4.50s; end is device authenticated at 81.00s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 90.000s at 15 fps (1350 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in device authenticated; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-device-code-issued.png and media/state-02-verification-page.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-code-accepted.png and media/state-04-consent.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Okta, Inc.
- **Product page:** https://developer.okta.com/docs/guides/device-authorization-grant/main/
- **Original media URL:** https://www.youtube.com/watch?v=irkrhuLiPbc
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 90.000s, 1350 frames, 1495061 bytes
- **SHA-256:** `92a4e062e727419bec480946898f38ff516390ff71f964a7d6faa5b3107f970d`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
