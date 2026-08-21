# Proton — create an account

**Evidence status:** `complete`  
**Product/source:** [https://proton.me/support/create-a-free-email-account-address](https://proton.me/support/create-a-free-email-account-address)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Proton Mail Basics & Key Features](https://www.youtube.com/watch?v=K2vzs6Q39Zc) — Proton Guides & Updates (@ProtonAG)

## Start-to-first-success journey

**Actor:** new Proton Mail user  
**Goal:** create a private email account and send the first message  
**Prerequisites:** available Proton username; password and recovery decision

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Choose Create account and a plan | Proton opens username and password setup | account entry | `media/state-01-account-entry.png` and motion at 4.50s |
| 2 | Choose the address and set the password | Proton validates account credentials | credentials ready | `media/state-02-credentials-ready.png` and motion at 19.79s |
| 3 | Add or decline recovery information with the consequence visible | Proton records the recovery decision | recovery decision | `media/state-03-recovery-decision.png` and motion at 35.07s |
| 4 | Complete the anti-abuse verification | Proton creates the encrypted mailbox | account created | `media/state-04-account-created.png` and motion at 50.36s |
| 5 | Enter the mailbox and compose a message | Proton opens the composer with the new address | message composed | `media/state-05-message-composed.png` and motion at 65.65s |
| 6 | Send the message | The message appears in Sent, proving first email success | first email | `media/state-06-first-email.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At credentials ready or recovery decision, invalid, expired, denied, or missing required input leaves the flow short of first email; evidence: media/state-02-credentials-ready.png, media/state-03-recovery-decision.png, and https://proton.me/support/create-a-free-email-account-address.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-recovery-decision.png through media/state-05-message-composed.png.
- **Recovery:** Return to the retained credentials ready or recovery decision requirement, correct or resend the blocking input, and resubmit; evidence: https://proton.me/support/create-a-free-email-account-address.
- **Recovery:** Continue through the same terminal action until first email is visible in media/state-06-first-email.png and the motion at 80.940s.
- **Completion evidence:** first email retained at media/state-06-first-email.png and media/official-recording.mp4#t=80.940; source https://proton.me/support/create-a-free-email-account-address

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account entry | [`media/state-01-account-entry.png`](media/state-01-account-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `f1299bbcbf2ca29d0af4f736c5022ffdbd983d2ac4f517b7e1c100f5e9ab11f1` |
| credentials ready | [`media/state-02-credentials-ready.png`](media/state-02-credentials-ready.png) | media/official-recording.mp4#t=19.785 | 640×360 | `558c14df1308d42055ad1ac91adabadde68c8a472df34f80bd512bf4209e1054` |
| recovery decision | [`media/state-03-recovery-decision.png`](media/state-03-recovery-decision.png) | media/official-recording.mp4#t=35.074 | 640×360 | `4bbe901cfd798df56a359af1376651675e71520c99effa9aa1ac083572037b2c` |
| account created | [`media/state-04-account-created.png`](media/state-04-account-created.png) | media/official-recording.mp4#t=50.362 | 640×360 | `3381edf8c9d8ea9596498fa52934f26199c49e1c2f6eaa29bba36265b420b547` |
| message composed | [`media/state-05-message-composed.png`](media/state-05-message-composed.png) | media/official-recording.mp4#t=65.651 | 640×360 | `32aca0f2db1a2d3a7476c95f25cea0a04d2ebfccd31e6a049aff2a8c4a5026ef` |
| first email | [`media/state-06-first-email.png`](media/state-06-first-email.png) | media/official-recording.mp4#t=80.940 | 640×360 | `80d37ab78312fe1aa33aa838b4acfab656eadf8cd9355194769652e412bf3920` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Choose Create account and a plan | Proton opens username and password setup The retained account entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-entry.png @ 4.50s; https://proton.me/support/create-a-free-email-account-address |
| focus and selection | Choose the address and set the password | Proton validates account credentials The recording advances to credentials ready and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-entry.png @ 4.50s; media/state-02-credentials-ready.png @ 19.79s; https://proton.me/support/create-a-free-email-account-address |
| navigation | Add or decline recovery information with the consequence visible | Proton records the recovery decision The navigation result is visible as recovery decision. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-credentials-ready.png @ 19.79s; media/state-03-recovery-decision.png @ 35.07s; https://proton.me/support/create-a-free-email-account-address |
| confirmation | Complete the anti-abuse verification | Proton creates the encrypted mailbox The official recording shows the confirmed account created state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-recovery-decision.png @ 35.07s; media/state-04-account-created.png @ 50.36s; https://proton.me/support/create-a-free-email-account-address |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-credentials-ready.png @ 19.79s; media/state-03-recovery-decision.png @ 35.07s; media/state-04-account-created.png @ 50.36s; https://proton.me/support/create-a-free-email-account-address |
| progress feedback | Enter the mailbox and compose a message | Proton opens the composer with the new address Progress is observable as the distinct message composed state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-account-created.png @ 50.36s; media/state-05-message-composed.png @ 65.65s; https://proton.me/support/create-a-free-email-account-address |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-recovery-decision.png @ 35.07s; media/state-04-account-created.png @ 50.36s; media/state-05-message-composed.png @ 65.65s; https://proton.me/support/create-a-free-email-account-address |
| recovery and completion | Send the message | The message appears in Sent, proving first email success The retained first email state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-message-composed.png @ 65.65s; media/state-06-first-email.png @ 80.94s; https://proton.me/support/create-a-free-email-account-address |

## Motion behavior

- **Trigger:** The recorded sequence begins at account entry; the first advancing trigger is “Choose the address and set the password”.
- **Start/end:** Start is account entry at 4.50s; end is first email at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first email; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-entry.png and media/state-02-credentials-ready.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-recovery-decision.png and media/state-04-account-created.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Proton AG
- **Product page:** https://proton.me/support/create-a-free-email-account-address
- **Original media URL:** https://www.youtube.com/watch?v=K2vzs6Q39Zc
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 2202780 bytes
- **SHA-256:** `a7daaaec00a64739be8deeb24150de06aaa18d9786972aaa15eb612bd54aad1c`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
