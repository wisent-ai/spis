# Square — account setup guide

**Evidence status:** `complete`  
**Product/source:** [https://squareup.com/help/us/en/article/5123-square-get-started-guide](https://squareup.com/help/us/en/article/5123-square-get-started-guide)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Getting Started with Square House Accounts](https://www.youtube.com/watch?v=Gbf0ljareRk) — Square

## Start-to-first-success journey

**Actor:** new Square seller  
**Goal:** complete operational setup and take the first payment  
**Prerequisites:** Square account; business identity; bank details or payment hardware

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Create or sign in to the Square account | Square opens the seller setup checklist | account entry | `media/state-01-account-entry.png` and motion at 4.50s |
| 2 | Enter and verify business and representative details | Square activates the seller identity | identity verified | `media/state-02-identity-verified.png` and motion at 19.79s |
| 3 | Link the payout bank account | Square confirms the payout destination | bank linked | `media/state-03-bank-linked.png` and motion at 35.07s |
| 4 | Create the first item and price | Square persists the catalog item | catalog ready | `media/state-04-catalog-ready.png` and motion at 50.36s |
| 5 | Connect or configure the payment device or Tap to Pay | Square reports the reader ready | payments ready | `media/state-05-payments-ready.png` and motion at 65.65s |
| 6 | Run the first payment or documented test | Square records the transaction, proving first selling success | first payment | `media/state-06-first-payment.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At identity verified or bank linked, invalid, expired, denied, or missing required input leaves the flow short of first payment; evidence: media/state-02-identity-verified.png, media/state-03-bank-linked.png, and https://squareup.com/help/us/en/article/5123-square-get-started-guide.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-bank-linked.png through media/state-05-payments-ready.png.
- **Recovery:** Return to the retained identity verified or bank linked requirement, correct or resend the blocking input, and resubmit; evidence: https://squareup.com/help/us/en/article/5123-square-get-started-guide.
- **Recovery:** Continue through the same terminal action until first payment is visible in media/state-06-first-payment.png and the motion at 80.940s.
- **Completion evidence:** first payment retained at media/state-06-first-payment.png and media/official-recording.mp4#t=80.940; source https://squareup.com/help/us/en/article/5123-square-get-started-guide

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account entry | [`media/state-01-account-entry.png`](media/state-01-account-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `4726395c70e7ab6b8e4626af5ccd05a6b74b2cb2675d3bedb90816432f89634d` |
| identity verified | [`media/state-02-identity-verified.png`](media/state-02-identity-verified.png) | media/official-recording.mp4#t=19.785 | 640×360 | `a3b1b1f139cc5ea80de1ddc16ac54a1569dae7d1bec89dd813d7095e13e75ccd` |
| bank linked | [`media/state-03-bank-linked.png`](media/state-03-bank-linked.png) | media/official-recording.mp4#t=35.074 | 640×360 | `d52f86106fea9a3a394396d2f1bb929782a3df3ca2eaa0fe0884c908793b7a42` |
| catalog ready | [`media/state-04-catalog-ready.png`](media/state-04-catalog-ready.png) | media/official-recording.mp4#t=50.362 | 640×360 | `01c4fb9eb1c23b2f17de114fe13081a10d62fc764362f6fffda475b53e06dc7b` |
| payments ready | [`media/state-05-payments-ready.png`](media/state-05-payments-ready.png) | media/official-recording.mp4#t=65.651 | 640×360 | `74e1d9a225d93f8ac44d6788ec39b9ab145b3967a092d6be5a4270fc998314b3` |
| first payment | [`media/state-06-first-payment.png`](media/state-06-first-payment.png) | media/official-recording.mp4#t=80.940 | 640×360 | `18a798274c9a325a14544f087a9419aa5fe3f5b9f3ad47b0bef82edab7e57bd2` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Create or sign in to the Square account | Square opens the seller setup checklist The retained account entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-entry.png @ 4.50s; https://squareup.com/help/us/en/article/5123-square-get-started-guide |
| focus and selection | Enter and verify business and representative details | Square activates the seller identity The recording advances to identity verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-entry.png @ 4.50s; media/state-02-identity-verified.png @ 19.79s; https://squareup.com/help/us/en/article/5123-square-get-started-guide |
| navigation | Link the payout bank account | Square confirms the payout destination The navigation result is visible as bank linked. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-verified.png @ 19.79s; media/state-03-bank-linked.png @ 35.07s; https://squareup.com/help/us/en/article/5123-square-get-started-guide |
| confirmation | Create the first item and price | Square persists the catalog item The official recording shows the confirmed catalog ready state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-bank-linked.png @ 35.07s; media/state-04-catalog-ready.png @ 50.36s; https://squareup.com/help/us/en/article/5123-square-get-started-guide |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-verified.png @ 19.79s; media/state-03-bank-linked.png @ 35.07s; media/state-04-catalog-ready.png @ 50.36s; https://squareup.com/help/us/en/article/5123-square-get-started-guide |
| progress feedback | Connect or configure the payment device or Tap to Pay | Square reports the reader ready Progress is observable as the distinct payments ready state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-catalog-ready.png @ 50.36s; media/state-05-payments-ready.png @ 65.65s; https://squareup.com/help/us/en/article/5123-square-get-started-guide |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-bank-linked.png @ 35.07s; media/state-04-catalog-ready.png @ 50.36s; media/state-05-payments-ready.png @ 65.65s; https://squareup.com/help/us/en/article/5123-square-get-started-guide |
| recovery and completion | Run the first payment or documented test | Square records the transaction, proving first selling success The retained first payment state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-payments-ready.png @ 65.65s; media/state-06-first-payment.png @ 80.94s; https://squareup.com/help/us/en/article/5123-square-get-started-guide |

## Motion behavior

- **Trigger:** The recorded sequence begins at account entry; the first advancing trigger is “Enter and verify business and representative details”.
- **Start/end:** Start is account entry at 4.50s; end is first payment at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first payment; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-entry.png and media/state-02-identity-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-bank-linked.png and media/state-04-catalog-ready.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Block, Inc. / Square
- **Product page:** https://squareup.com/help/us/en/article/5123-square-get-started-guide
- **Original media URL:** https://www.youtube.com/watch?v=Gbf0ljareRk
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 538123 bytes
- **SHA-256:** `796fea8c483569a53cea88e525a3f69bcadbb99ba3db755b8a71d5145a52ef9e`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
