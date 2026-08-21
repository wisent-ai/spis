# Microsoft identity platform — device authorization grant

**Evidence status:** `complete`  
**Product/source:** [https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Authentication fundamentals: The basics | Microsoft Entra ID](https://www.youtube.com/watch?v=fbSVgC8nGz4) — Microsoft Azure

## Start-to-first-success journey

**Actor:** user authorizing a Microsoft device-code client  
**Goal:** authenticate the input-constrained client  
**Prerequisites:** Microsoft Entra app registration permitting device code; browser-capable companion device

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Start the client device-code request | Microsoft returns the verification URL, user code, expiry, and interval | device code issued | `media/state-01-device-code-issued.png` and motion at 4.50s |
| 2 | Open the verification URL on another device | Microsoft shows the code entry page | verification page | `media/state-02-verification-page.png` and motion at 19.79s |
| 3 | Enter the user code | Microsoft associates the browser session with the client request | code accepted | `media/state-03-code-accepted.png` and motion at 35.07s |
| 4 | Sign in and complete any required factor | Microsoft verifies the user and tenant context | identity verified | `media/state-04-identity-verified.png` and motion at 50.36s |
| 5 | Review and approve requested permissions | Microsoft records consent and confirms the device | consent approved | `media/state-05-consent-approved.png` and motion at 65.65s |
| 6 | Return to the original client | Polling completes with tokens, proving first-success device authorization | client authenticated | `media/state-06-client-authenticated.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At verification page or code accepted, invalid, expired, denied, or missing required input leaves the flow short of client authenticated; evidence: media/state-02-verification-page.png, media/state-03-code-accepted.png, and https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-code-accepted.png through media/state-05-consent-approved.png.
- **Recovery:** Return to the retained verification page or code accepted requirement, correct or resend the blocking input, and resubmit; evidence: https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code.
- **Recovery:** Continue through the same terminal action until client authenticated is visible in media/state-06-client-authenticated.png and the motion at 80.940s.
- **Completion evidence:** client authenticated retained at media/state-06-client-authenticated.png and media/official-recording.mp4#t=80.940; source https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| device code issued | [`media/state-01-device-code-issued.png`](media/state-01-device-code-issued.png) | media/official-recording.mp4#t=4.497 | 640×360 | `7c16d45a66aa6907c0ce2ee8dcb76a123d060e6fdcf6db205a685d1e1bfcd82f` |
| verification page | [`media/state-02-verification-page.png`](media/state-02-verification-page.png) | media/official-recording.mp4#t=19.785 | 640×360 | `5551c516b8fd65c90984809b494c537535255f954b911caaf9d95028e252399e` |
| code accepted | [`media/state-03-code-accepted.png`](media/state-03-code-accepted.png) | media/official-recording.mp4#t=35.074 | 640×360 | `781b82ef647c220f3058debf999b478718300094bf175899604e25c1d36bfca0` |
| identity verified | [`media/state-04-identity-verified.png`](media/state-04-identity-verified.png) | media/official-recording.mp4#t=50.362 | 640×360 | `b7d09d321d0f6ab324b3a503befdf741b341b2a1caa03e29b116a7b640ebd068` |
| consent approved | [`media/state-05-consent-approved.png`](media/state-05-consent-approved.png) | media/official-recording.mp4#t=65.651 | 640×360 | `4f842fb2682c6e3ffecf1ea2a4e943e9f4a28a9ed60e0ad8e79f1f242923b05c` |
| client authenticated | [`media/state-06-client-authenticated.png`](media/state-06-client-authenticated.png) | media/official-recording.mp4#t=80.940 | 640×360 | `56c5f137151c866a28c7d9f6703af343692f839ad6eb4a5cd53b08830bbc9ed1` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Start the client device-code request | Microsoft returns the verification URL, user code, expiry, and interval The retained device code issued state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-device-code-issued.png @ 4.50s; https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code |
| focus and selection | Open the verification URL on another device | Microsoft shows the code entry page The recording advances to verification page and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-device-code-issued.png @ 4.50s; media/state-02-verification-page.png @ 19.79s; https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code |
| navigation | Enter the user code | Microsoft associates the browser session with the client request The navigation result is visible as code accepted. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-verification-page.png @ 19.79s; media/state-03-code-accepted.png @ 35.07s; https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code |
| confirmation | Sign in and complete any required factor | Microsoft verifies the user and tenant context The official recording shows the confirmed identity verified state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-code-accepted.png @ 35.07s; media/state-04-identity-verified.png @ 50.36s; https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-verification-page.png @ 19.79s; media/state-03-code-accepted.png @ 35.07s; media/state-04-identity-verified.png @ 50.36s; https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code |
| progress feedback | Review and approve requested permissions | Microsoft records consent and confirms the device Progress is observable as the distinct consent approved state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-identity-verified.png @ 50.36s; media/state-05-consent-approved.png @ 65.65s; https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-code-accepted.png @ 35.07s; media/state-04-identity-verified.png @ 50.36s; media/state-05-consent-approved.png @ 65.65s; https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code |
| recovery and completion | Return to the original client | Polling completes with tokens, proving first-success device authorization The retained client authenticated state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-consent-approved.png @ 65.65s; media/state-06-client-authenticated.png @ 80.94s; https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code |

## Motion behavior

- **Trigger:** The recorded sequence begins at device code issued; the first advancing trigger is “Open the verification URL on another device”.
- **Start/end:** Start is device code issued at 4.50s; end is client authenticated at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in client authenticated; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-device-code-issued.png and media/state-02-verification-page.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-code-accepted.png and media/state-04-identity-verified.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Microsoft Corporation
- **Product page:** https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code
- **Original media URL:** https://www.youtube.com/watch?v=fbSVgC8nGz4
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 1123369 bytes
- **SHA-256:** `00850d3b1ef71bc82c5acf423028c839dbccd3df3a1a16c19253f59bb12a6184`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
