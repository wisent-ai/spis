# AWS IAM Identity Center — device authorization

**Evidence status:** `complete`  
**Product/source:** [https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html](https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [AWS IAM Identity Center (Successor to AWS SSO) Overview Demo](https://www.youtube.com/watch?v=4yJp5-jGGNk) — Amazon Web Services

## Start-to-first-success journey

**Actor:** AWS IAM Identity Center user  
**Goal:** authorize a device client for the assigned AWS access  
**Prerequisites:** IAM Identity Center start URL and assignment; browser-capable companion device

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Start device authorization from the AWS client | AWS returns the verification URI, user code, and expiry | device code issued | `media/state-01-device-code-issued.png` and motion at 4.50s |
| 2 | Open the verification URI | IAM Identity Center shows the code or sign-in state | verification page | `media/state-02-verification-page.png` and motion at 19.79s |
| 3 | Enter or confirm the user code | AWS links the browser session to the client | code accepted | `media/state-03-code-accepted.png` and motion at 35.07s |
| 4 | Authenticate with the configured identity source | AWS verifies the user and assigned organization | identity verified | `media/state-04-identity-verified.png` and motion at 50.36s |
| 5 | Approve client access | AWS confirms authorization | authorization approved | `media/state-05-authorization-approved.png` and motion at 65.65s |
| 6 | Return to the original client | The client receives tokens and lists assigned access, proving success | client authorized | `media/state-06-client-authorized.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At verification page or code accepted, invalid, expired, denied, or missing required input leaves the flow short of client authorized; evidence: media/state-02-verification-page.png, media/state-03-code-accepted.png, and https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-code-accepted.png through media/state-05-authorization-approved.png.
- **Recovery:** Return to the retained verification page or code accepted requirement, correct or resend the blocking input, and resubmit; evidence: https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html.
- **Recovery:** Continue through the same terminal action until client authorized is visible in media/state-06-client-authorized.png and the motion at 80.940s.
- **Completion evidence:** client authorized retained at media/state-06-client-authorized.png and media/official-recording.mp4#t=80.940; source https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| device code issued | [`media/state-01-device-code-issued.png`](media/state-01-device-code-issued.png) | media/official-recording.mp4#t=4.497 | 640×360 | `19dc2b261f0971992a40cb5794152910b1e9d3a117dbaf6283c732771cebb164` |
| verification page | [`media/state-02-verification-page.png`](media/state-02-verification-page.png) | media/official-recording.mp4#t=19.785 | 640×360 | `533b5b79eb961e179227297c45b5113b8d095f69da894914f2dd5844edab7a98` |
| code accepted | [`media/state-03-code-accepted.png`](media/state-03-code-accepted.png) | media/official-recording.mp4#t=35.074 | 640×360 | `b2fe73f34b29745ed075404ebbd9e5463512e6cc8f802ea5c42c9e7ef2ccb7e2` |
| identity verified | [`media/state-04-identity-verified.png`](media/state-04-identity-verified.png) | media/official-recording.mp4#t=50.362 | 640×360 | `159438ea636269d8042f230db6f7ac956e373e9d84edf89cffff7815708589f5` |
| authorization approved | [`media/state-05-authorization-approved.png`](media/state-05-authorization-approved.png) | media/official-recording.mp4#t=65.651 | 640×360 | `8ba23e67770d944ec5abdbaf47f53ccc137520fbd3bc8974f65787cdfca40300` |
| client authorized | [`media/state-06-client-authorized.png`](media/state-06-client-authorized.png) | media/official-recording.mp4#t=80.940 | 640×360 | `ba08d62967cac7c1e6cf07912654e042dc5949e88ded8ca0d138173fb1fdcd27` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Start device authorization from the AWS client | AWS returns the verification URI, user code, and expiry The retained device code issued state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-device-code-issued.png @ 4.50s; https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html |
| focus and selection | Open the verification URI | IAM Identity Center shows the code or sign-in state The recording advances to verification page and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-device-code-issued.png @ 4.50s; media/state-02-verification-page.png @ 19.79s; https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html |
| navigation | Enter or confirm the user code | AWS links the browser session to the client The navigation result is visible as code accepted. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-verification-page.png @ 19.79s; media/state-03-code-accepted.png @ 35.07s; https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html |
| confirmation | Authenticate with the configured identity source | AWS verifies the user and assigned organization The official recording shows the confirmed identity verified state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-code-accepted.png @ 35.07s; media/state-04-identity-verified.png @ 50.36s; https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-verification-page.png @ 19.79s; media/state-03-code-accepted.png @ 35.07s; media/state-04-identity-verified.png @ 50.36s; https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html |
| progress feedback | Approve client access | AWS confirms authorization Progress is observable as the distinct authorization approved state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-identity-verified.png @ 50.36s; media/state-05-authorization-approved.png @ 65.65s; https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-code-accepted.png @ 35.07s; media/state-04-identity-verified.png @ 50.36s; media/state-05-authorization-approved.png @ 65.65s; https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html |
| recovery and completion | Return to the original client | The client receives tokens and lists assigned access, proving success The retained client authorized state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-authorization-approved.png @ 65.65s; media/state-06-client-authorized.png @ 80.94s; https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html |

## Motion behavior

- **Trigger:** The recorded sequence begins at device code issued; the first advancing trigger is “Open the verification URI”.
- **Start/end:** Start is device code issued at 4.50s; end is client authorized at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in client authorized; pending or error feedback is not promoted to success.
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

- **Upstream owner:** Amazon Web Services, Inc.
- **Product page:** https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/API_StartDeviceAuthorization.html
- **Original media URL:** https://www.youtube.com/watch?v=4yJp5-jGGNk
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 431104 bytes
- **SHA-256:** `477b795d4343e8f69142f5a855ed3c94fbcb6467f04f7262d2bf91568d6e05df`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
