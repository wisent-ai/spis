# Plaid Link — overview

**Evidence status:** `complete`  
**Product/source:** [https://plaid.com/docs/link/](https://plaid.com/docs/link/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Simplify Onboarding: Link](https://www.youtube.com/watch?v=t5ZYArXOo4w) — Plaid

## Start-to-first-success journey

**Actor:** customer linking a financial account  
**Goal:** connect an institution and return a successful Link result  
**Prerequisites:** application-provided Link session; supported financial institution; institution credentials or OAuth access

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Open Plaid Link and review the consent context | Link shows purpose, data access, and Continue | consent entry | `media/state-01-consent-entry.png` and motion at 4.50s |
| 2 | Accept and search or choose an institution | Link selects the institution-specific route | institution selected | `media/state-02-institution-selected.png` and motion at 19.79s |
| 3 | Authenticate through credentials or institution OAuth | Link reports authentication progress | institution authenticated | `media/state-03-institution-authenticated.png` and motion at 35.07s |
| 4 | Complete multifactor verification when requested | Link confirms the challenge | MFA complete | `media/state-04-mfa-complete.png` and motion at 50.36s |
| 5 | Choose the accounts to share and confirm | Link displays the selected scope | accounts selected | `media/state-05-accounts-selected.png` and motion at 65.65s |
| 6 | Finish Link | The application receives success and selected-account metadata, proving first connection | account linked | `media/state-06-account-linked.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At institution selected or institution authenticated, invalid, expired, denied, or missing required input leaves the flow short of account linked; evidence: media/state-02-institution-selected.png, media/state-03-institution-authenticated.png, and https://plaid.com/docs/link/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-institution-authenticated.png through media/state-05-accounts-selected.png.
- **Recovery:** Return to the retained institution selected or institution authenticated requirement, correct or resend the blocking input, and resubmit; evidence: https://plaid.com/docs/link/.
- **Recovery:** Continue through the same terminal action until account linked is visible in media/state-06-account-linked.png and the motion at 80.940s.
- **Completion evidence:** account linked retained at media/state-06-account-linked.png and media/official-recording.mp4#t=80.940; source https://plaid.com/docs/link/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| consent entry | [`media/state-01-consent-entry.png`](media/state-01-consent-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `edd7742deda083e05d8a19e581f0e263e4bc9d78dea49c6f9954e0e6781140ab` |
| institution selected | [`media/state-02-institution-selected.png`](media/state-02-institution-selected.png) | media/official-recording.mp4#t=19.785 | 640×360 | `1f23d56027985f5c8da7c344474a05a7b84214510ee15273efe4ddc7c6b80ac6` |
| institution authenticated | [`media/state-03-institution-authenticated.png`](media/state-03-institution-authenticated.png) | media/official-recording.mp4#t=35.074 | 640×360 | `069e73a507c2aedbda10174d6ebe202957dc34521a4be4fafba422696765e70c` |
| MFA complete | [`media/state-04-mfa-complete.png`](media/state-04-mfa-complete.png) | media/official-recording.mp4#t=50.362 | 640×360 | `26c0e4f53ec378ad53a87f8318eae326584173aaea7ec1d5df10784bd40f4250` |
| accounts selected | [`media/state-05-accounts-selected.png`](media/state-05-accounts-selected.png) | media/official-recording.mp4#t=65.651 | 640×360 | `f95f82c6a252da5928e3f01483b7c86fe9bca5ce4c9239ab700d97ad2e871a78` |
| account linked | [`media/state-06-account-linked.png`](media/state-06-account-linked.png) | media/official-recording.mp4#t=80.940 | 640×360 | `294ffc7ee7e9287ceb45db3c58c27911a4acf77eeab90ef25ff8b58e9c59fca5` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Open Plaid Link and review the consent context | Link shows purpose, data access, and Continue The retained consent entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-consent-entry.png @ 4.50s; https://plaid.com/docs/link/ |
| focus and selection | Accept and search or choose an institution | Link selects the institution-specific route The recording advances to institution selected and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-consent-entry.png @ 4.50s; media/state-02-institution-selected.png @ 19.79s; https://plaid.com/docs/link/ |
| navigation | Authenticate through credentials or institution OAuth | Link reports authentication progress The navigation result is visible as institution authenticated. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-institution-selected.png @ 19.79s; media/state-03-institution-authenticated.png @ 35.07s; https://plaid.com/docs/link/ |
| confirmation | Complete multifactor verification when requested | Link confirms the challenge The official recording shows the confirmed MFA complete state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-institution-authenticated.png @ 35.07s; media/state-04-mfa-complete.png @ 50.36s; https://plaid.com/docs/link/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-institution-selected.png @ 19.79s; media/state-03-institution-authenticated.png @ 35.07s; media/state-04-mfa-complete.png @ 50.36s; https://plaid.com/docs/link/ |
| progress feedback | Choose the accounts to share and confirm | Link displays the selected scope Progress is observable as the distinct accounts selected state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-mfa-complete.png @ 50.36s; media/state-05-accounts-selected.png @ 65.65s; https://plaid.com/docs/link/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-institution-authenticated.png @ 35.07s; media/state-04-mfa-complete.png @ 50.36s; media/state-05-accounts-selected.png @ 65.65s; https://plaid.com/docs/link/ |
| recovery and completion | Finish Link | The application receives success and selected-account metadata, proving first connection The retained account linked state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-accounts-selected.png @ 65.65s; media/state-06-account-linked.png @ 80.94s; https://plaid.com/docs/link/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at consent entry; the first advancing trigger is “Accept and search or choose an institution”.
- **Start/end:** Start is consent entry at 4.50s; end is account linked at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in account linked; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-consent-entry.png and media/state-02-institution-selected.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-institution-authenticated.png and media/state-04-mfa-complete.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Plaid Inc.
- **Product page:** https://plaid.com/docs/link/
- **Original media URL:** https://www.youtube.com/watch?v=t5ZYArXOo4w
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 639134 bytes
- **SHA-256:** `5768d32a6e187c51b0f2779f16833bf16af9809684ef1efce95b43b8cb1768be`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
