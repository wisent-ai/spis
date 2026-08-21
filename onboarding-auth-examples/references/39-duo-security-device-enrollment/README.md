# Duo Security — device enrollment

**Evidence status:** `complete`  
**Product/source:** [https://guide.duo.com/enrollment](https://guide.duo.com/enrollment)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Getting Started with Duo Security (with voiceover)](https://www.youtube.com/watch?v=HDU35vn0SS0) — Duo Security

## Start-to-first-success journey

**Actor:** employee enrolling in Duo  
**Goal:** activate an authenticator and pass the first Duo challenge  
**Prerequisites:** organization enrollment link; supported phone or security key

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Open the Duo enrollment link after primary sign-in | Duo shows the organization enrollment welcome state | enrollment entry | `media/state-01-enrollment-entry.png` and motion at 1.50s |
| 2 | Choose the device or authenticator type | Duo displays method-specific setup | method selected | `media/state-02-method-selected.png` and motion at 6.59s |
| 3 | Enter and verify the phone number when required | Duo confirms ownership or advances to activation | device identity | `media/state-03-device-identity.png` and motion at 11.67s |
| 4 | Install Duo Mobile or prepare the selected authenticator | Duo presents activation instructions | authenticator ready | `media/state-04-authenticator-ready.png` and motion at 16.76s |
| 5 | Scan the activation code or complete key registration | Duo marks the device activated | device enrolled | `media/state-05-device-enrolled.png` and motion at 21.85s |
| 6 | Approve the test challenge | Duo confirms successful authentication, proving enrollment success | first approval | `media/state-06-first-approval.png` and motion at 26.94s |

### Failure and recovery

- **Failure:** At method selected or device identity, invalid, expired, denied, or missing required input leaves the flow short of first approval; evidence: media/state-02-method-selected.png, media/state-03-device-identity.png, and https://guide.duo.com/enrollment.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-device-identity.png through media/state-05-device-enrolled.png.
- **Recovery:** Return to the retained method selected or device identity requirement, correct or resend the blocking input, and resubmit; evidence: https://guide.duo.com/enrollment.
- **Recovery:** Continue through the same terminal action until first approval is visible in media/state-06-first-approval.png and the motion at 26.940s.
- **Completion evidence:** first approval retained at media/state-06-first-approval.png and media/official-recording.mp4#t=26.940; source https://guide.duo.com/enrollment

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| enrollment entry | [`media/state-01-enrollment-entry.png`](media/state-01-enrollment-entry.png) | media/official-recording.mp4#t=1.497 | 640×360 | `5ed323937d138c975f1dd4a6da1a6d1d7fa63ddcffc05265043c28e37997ae8b` |
| method selected | [`media/state-02-method-selected.png`](media/state-02-method-selected.png) | media/official-recording.mp4#t=6.585 | 640×360 | `f2cf2fc2a15adfbfe0bb5de26065e57f8b6e1b3155f5a66329c75066ca4711d8` |
| device identity | [`media/state-03-device-identity.png`](media/state-03-device-identity.png) | media/official-recording.mp4#t=11.674 | 640×360 | `9a58a05badf2fb97e59297698499302639dba3197dc8d44db094cc464d88cf2d` |
| authenticator ready | [`media/state-04-authenticator-ready.png`](media/state-04-authenticator-ready.png) | media/official-recording.mp4#t=16.762 | 640×360 | `ae656ad48fbd43d9c8bcc604a67378bba50ec9bc32e25288fe620d66d179f08a` |
| device enrolled | [`media/state-05-device-enrolled.png`](media/state-05-device-enrolled.png) | media/official-recording.mp4#t=21.851 | 640×360 | `96e88a7c3ba5293558a15b0a47770616a67ffc6e917bc53aa06d585cbe34320c` |
| first approval | [`media/state-06-first-approval.png`](media/state-06-first-approval.png) | media/official-recording.mp4#t=26.940 | 640×360 | `b27106caf796b64409a020d934f0cb391deeb3053a47123e88942478b3858ff7` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Open the Duo enrollment link after primary sign-in | Duo shows the organization enrollment welcome state The retained enrollment entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-enrollment-entry.png @ 1.50s; https://guide.duo.com/enrollment |
| focus and selection | Choose the device or authenticator type | Duo displays method-specific setup The recording advances to method selected and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-enrollment-entry.png @ 1.50s; media/state-02-method-selected.png @ 6.59s; https://guide.duo.com/enrollment |
| navigation | Enter and verify the phone number when required | Duo confirms ownership or advances to activation The navigation result is visible as device identity. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-method-selected.png @ 6.59s; media/state-03-device-identity.png @ 11.67s; https://guide.duo.com/enrollment |
| confirmation | Install Duo Mobile or prepare the selected authenticator | Duo presents activation instructions The official recording shows the confirmed authenticator ready state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-device-identity.png @ 11.67s; media/state-04-authenticator-ready.png @ 16.76s; https://guide.duo.com/enrollment |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-method-selected.png @ 6.59s; media/state-03-device-identity.png @ 11.67s; media/state-04-authenticator-ready.png @ 16.76s; https://guide.duo.com/enrollment |
| progress feedback | Scan the activation code or complete key registration | Duo marks the device activated Progress is observable as the distinct device enrolled state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-authenticator-ready.png @ 16.76s; media/state-05-device-enrolled.png @ 21.85s; https://guide.duo.com/enrollment |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-device-identity.png @ 11.67s; media/state-04-authenticator-ready.png @ 16.76s; media/state-05-device-enrolled.png @ 21.85s; https://guide.duo.com/enrollment |
| recovery and completion | Approve the test challenge | Duo confirms successful authentication, proving enrollment success The retained first approval state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-device-enrolled.png @ 21.85s; media/state-06-first-approval.png @ 26.94s; https://guide.duo.com/enrollment |

## Motion behavior

- **Trigger:** The recorded sequence begins at enrollment entry; the first advancing trigger is “Choose the device or authenticator type”.
- **Start/end:** Start is enrollment entry at 1.50s; end is first approval at 26.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 29.933s at 15 fps (449 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first approval; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-enrollment-entry.png and media/state-02-method-selected.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-device-identity.png and media/state-04-authenticator-ready.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Cisco Systems, Inc. / Duo Security
- **Product page:** https://guide.duo.com/enrollment
- **Original media URL:** https://www.youtube.com/watch?v=HDU35vn0SS0
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 29.933s, 449 frames, 261066 bytes
- **SHA-256:** `9863c4d6f8d71153ae444edbff205ede93f23d66e8fc8dfb8123ba3196bd9727`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
