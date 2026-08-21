# PayPal — create an account

**Evidence status:** `complete`  
**Product/source:** [https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315](https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to set up a PayPal Account](https://www.youtube.com/watch?v=pzubfbvi5Ns) — PayPal Canada

## Start-to-first-success journey

**Actor:** new PayPal customer  
**Goal:** verify an account and complete the first money action  
**Prerequisites:** email and phone; personal or business account decision; funding method when required

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Choose Sign up and personal or business account | PayPal opens the appropriate identity form | account type | `media/state-01-account-type.png` and motion at 4.50s |
| 2 | Enter phone or email and complete the verification challenge | PayPal accepts the verified contact channel | contact verified | `media/state-02-contact-verified.png` and motion at 19.79s |
| 3 | Create credentials and enter required identity details | PayPal creates the account profile | account created | `media/state-03-account-created.png` and motion at 35.07s |
| 4 | Link or confirm a card or bank when required | PayPal records the funding method | funding ready | `media/state-04-funding-ready.png` and motion at 50.36s |
| 5 | Review the account summary and security prompts | PayPal exposes send, receive, and payment controls | account ready | `media/state-05-account-ready.png` and motion at 65.65s |
| 6 | Complete a supported payment or transfer action | PayPal records the transaction state, proving first money-movement success | first transaction | `media/state-06-first-transaction.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At contact verified or account created, invalid, expired, denied, or missing required input leaves the flow short of first transaction; evidence: media/state-02-contact-verified.png, media/state-03-account-created.png, and https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-account-created.png through media/state-05-account-ready.png.
- **Recovery:** Return to the retained contact verified or account created requirement, correct or resend the blocking input, and resubmit; evidence: https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315.
- **Recovery:** Continue through the same terminal action until first transaction is visible in media/state-06-first-transaction.png and the motion at 80.940s.
- **Completion evidence:** first transaction retained at media/state-06-first-transaction.png and media/official-recording.mp4#t=80.940; source https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account type | [`media/state-01-account-type.png`](media/state-01-account-type.png) | media/official-recording.mp4#t=4.497 | 640×360 | `3dffc2335294e752190af7ca22424ed2b1f5f25159c17338108ee030a5fbb3ea` |
| contact verified | [`media/state-02-contact-verified.png`](media/state-02-contact-verified.png) | media/official-recording.mp4#t=19.785 | 640×360 | `2d049fd25d54cce36bb9f8b38de1f60160815ffdaae2599b6f4bacbda0099264` |
| account created | [`media/state-03-account-created.png`](media/state-03-account-created.png) | media/official-recording.mp4#t=35.074 | 640×360 | `d32a79507e089755888be46321bcdf5d9b44ba8ffaec728a8faf8aebdf40bfbc` |
| funding ready | [`media/state-04-funding-ready.png`](media/state-04-funding-ready.png) | media/official-recording.mp4#t=50.362 | 640×360 | `020f552560b0ec931e1042fd0cb3513dd047c92a1eea69b9a58a98c83a3d8930` |
| account ready | [`media/state-05-account-ready.png`](media/state-05-account-ready.png) | media/official-recording.mp4#t=65.651 | 640×360 | `d83515912a7f6f7349ca3cc970c4c60b42cc619bca3a99821ad11f96c040da19` |
| first transaction | [`media/state-06-first-transaction.png`](media/state-06-first-transaction.png) | media/official-recording.mp4#t=80.940 | 640×360 | `52b3079cfebdecdc8e8682d6f500a91103a2d2feb03392c4ca4c036e61bd34b3` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Choose Sign up and personal or business account | PayPal opens the appropriate identity form The retained account type state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-type.png @ 4.50s; https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315 |
| focus and selection | Enter phone or email and complete the verification challenge | PayPal accepts the verified contact channel The recording advances to contact verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-type.png @ 4.50s; media/state-02-contact-verified.png @ 19.79s; https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315 |
| navigation | Create credentials and enter required identity details | PayPal creates the account profile The navigation result is visible as account created. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-contact-verified.png @ 19.79s; media/state-03-account-created.png @ 35.07s; https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315 |
| confirmation | Link or confirm a card or bank when required | PayPal records the funding method The official recording shows the confirmed funding ready state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-account-created.png @ 35.07s; media/state-04-funding-ready.png @ 50.36s; https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315 |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-contact-verified.png @ 19.79s; media/state-03-account-created.png @ 35.07s; media/state-04-funding-ready.png @ 50.36s; https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315 |
| progress feedback | Review the account summary and security prompts | PayPal exposes send, receive, and payment controls Progress is observable as the distinct account ready state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-funding-ready.png @ 50.36s; media/state-05-account-ready.png @ 65.65s; https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315 |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-account-created.png @ 35.07s; media/state-04-funding-ready.png @ 50.36s; media/state-05-account-ready.png @ 65.65s; https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315 |
| recovery and completion | Complete a supported payment or transfer action | PayPal records the transaction state, proving first money-movement success The retained first transaction state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-account-ready.png @ 65.65s; media/state-06-first-transaction.png @ 80.94s; https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315 |

## Motion behavior

- **Trigger:** The recorded sequence begins at account type; the first advancing trigger is “Enter phone or email and complete the verification challenge”.
- **Start/end:** Start is account type at 4.50s; end is first transaction at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first transaction; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-type.png and media/state-02-contact-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-account-created.png and media/state-04-funding-ready.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** PayPal, Inc.
- **Product page:** https://www.paypal.com/us/cshelp/article/how-do-i-open-a-paypal-account-help315
- **Original media URL:** https://www.youtube.com/watch?v=pzubfbvi5Ns
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 781293 bytes
- **SHA-256:** `b09905026f3603aaa9c5a02606999b67b988351cf94f436faa71b8e81c5f3b18`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
