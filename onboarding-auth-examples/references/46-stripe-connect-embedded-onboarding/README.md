# Stripe Connect — embedded onboarding

**Evidence status:** `complete`  
**Product/source:** [https://docs.stripe.com/connect/embedded-onboarding](https://docs.stripe.com/connect/embedded-onboarding)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Using embedded components to streamline Connect onboarding](https://www.youtube.com/watch?v=OPgGdjXUX54) — Stripe Developers

## Start-to-first-success journey

**Actor:** connected-account representative  
**Goal:** satisfy current requirements and submit Stripe Connect onboarding  
**Prerequisites:** platform-provided onboarding session; business, representative, and payout information

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Open the embedded onboarding component | Stripe shows the connected account and current requirement sections | onboarding entry | `media/state-01-onboarding-entry.png` and motion at 4.50s |
| 2 | Select business country, type, and structure | Stripe determines the applicable requirement set | business classification | `media/state-02-business-classification.png` and motion at 19.79s |
| 3 | Enter business and representative identity | Stripe validates and saves each durable section | identity collected | `media/state-03-identity-collected.png` and motion at 35.07s |
| 4 | Provide payout or bank details | Stripe records the external account and masks sensitive values | payout configured | `media/state-04-payout-configured.png` and motion at 50.36s |
| 5 | Review outstanding requirements and correct flagged sections | Stripe updates completion status as requirements clear | requirements review | `media/state-05-requirements-review.png` and motion at 65.65s |
| 6 | Submit the completed onboarding | Stripe returns a completed or pending-verification state, proving submission success | onboarding submitted | `media/state-06-onboarding-submitted.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At business classification or identity collected, invalid, expired, denied, or missing required input leaves the flow short of onboarding submitted; evidence: media/state-02-business-classification.png, media/state-03-identity-collected.png, and https://docs.stripe.com/connect/embedded-onboarding.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-identity-collected.png through media/state-05-requirements-review.png.
- **Recovery:** Return to the retained business classification or identity collected requirement, correct or resend the blocking input, and resubmit; evidence: https://docs.stripe.com/connect/embedded-onboarding.
- **Recovery:** Continue through the same terminal action until onboarding submitted is visible in media/state-06-onboarding-submitted.png and the motion at 80.940s.
- **Completion evidence:** onboarding submitted retained at media/state-06-onboarding-submitted.png and media/official-recording.mp4#t=80.940; source https://docs.stripe.com/connect/embedded-onboarding

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| onboarding entry | [`media/state-01-onboarding-entry.png`](media/state-01-onboarding-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `ee571992176be158ff6229991fc62cff6e865f5a7443adf96de01e9694898eef` |
| business classification | [`media/state-02-business-classification.png`](media/state-02-business-classification.png) | media/official-recording.mp4#t=19.785 | 640×360 | `e58adf65d956a3e112b641e7e20072dd3a91c365ed58704cb021b0bed49dbe7e` |
| identity collected | [`media/state-03-identity-collected.png`](media/state-03-identity-collected.png) | media/official-recording.mp4#t=35.074 | 640×360 | `4c6361b9febd2696bf0848936166d74b07b4a2e8ff29ba61d08a8c127b021dcb` |
| payout configured | [`media/state-04-payout-configured.png`](media/state-04-payout-configured.png) | media/official-recording.mp4#t=50.362 | 640×360 | `bd2dccb069c3b7c0c0d5c10de2fed788af94bc92a5e2265a55ef2bdc7c6aadbb` |
| requirements review | [`media/state-05-requirements-review.png`](media/state-05-requirements-review.png) | media/official-recording.mp4#t=65.651 | 640×360 | `c67482b37f939435fabf5bd2ee4334da93c480d36eb37bc1676fe805a738c5a0` |
| onboarding submitted | [`media/state-06-onboarding-submitted.png`](media/state-06-onboarding-submitted.png) | media/official-recording.mp4#t=80.940 | 640×360 | `05d06550c9b4fdfdbbd4dd226f7f1f311735234e22d5c5d26ecc4aeeb1ce3ce2` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Open the embedded onboarding component | Stripe shows the connected account and current requirement sections The retained onboarding entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-onboarding-entry.png @ 4.50s; https://docs.stripe.com/connect/embedded-onboarding |
| focus and selection | Select business country, type, and structure | Stripe determines the applicable requirement set The recording advances to business classification and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-onboarding-entry.png @ 4.50s; media/state-02-business-classification.png @ 19.79s; https://docs.stripe.com/connect/embedded-onboarding |
| navigation | Enter business and representative identity | Stripe validates and saves each durable section The navigation result is visible as identity collected. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-business-classification.png @ 19.79s; media/state-03-identity-collected.png @ 35.07s; https://docs.stripe.com/connect/embedded-onboarding |
| confirmation | Provide payout or bank details | Stripe records the external account and masks sensitive values The official recording shows the confirmed payout configured state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-identity-collected.png @ 35.07s; media/state-04-payout-configured.png @ 50.36s; https://docs.stripe.com/connect/embedded-onboarding |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-business-classification.png @ 19.79s; media/state-03-identity-collected.png @ 35.07s; media/state-04-payout-configured.png @ 50.36s; https://docs.stripe.com/connect/embedded-onboarding |
| progress feedback | Review outstanding requirements and correct flagged sections | Stripe updates completion status as requirements clear Progress is observable as the distinct requirements review state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-payout-configured.png @ 50.36s; media/state-05-requirements-review.png @ 65.65s; https://docs.stripe.com/connect/embedded-onboarding |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-identity-collected.png @ 35.07s; media/state-04-payout-configured.png @ 50.36s; media/state-05-requirements-review.png @ 65.65s; https://docs.stripe.com/connect/embedded-onboarding |
| recovery and completion | Submit the completed onboarding | Stripe returns a completed or pending-verification state, proving submission success The retained onboarding submitted state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-requirements-review.png @ 65.65s; media/state-06-onboarding-submitted.png @ 80.94s; https://docs.stripe.com/connect/embedded-onboarding |

## Motion behavior

- **Trigger:** The recorded sequence begins at onboarding entry; the first advancing trigger is “Select business country, type, and structure”.
- **Start/end:** Start is onboarding entry at 4.50s; end is onboarding submitted at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in onboarding submitted; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-onboarding-entry.png and media/state-02-business-classification.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-identity-collected.png and media/state-04-payout-configured.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Stripe, Inc.
- **Product page:** https://docs.stripe.com/connect/embedded-onboarding
- **Original media URL:** https://www.youtube.com/watch?v=OPgGdjXUX54
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 326173 bytes
- **SHA-256:** `169f1f4f914a5ba0f8bcb731ccfc6c2b6ccd6e692319ffc95a0bcb6c3fba1320`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
