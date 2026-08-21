# Dashlane — get started

**Evidence status:** `complete`  
**Product/source:** [https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started](https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Get Started with Dashlane Business for Employees](https://www.youtube.com/watch?v=TpCbf3ocOwk) — Dashlane

## Start-to-first-success journey

**Actor:** new Dashlane member  
**Goal:** activate the account and complete the first autofill-ready credential  
**Prerequisites:** Dashlane invitation or account; supported browser extension

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Open the invitation or create the account | Dashlane starts identity and security setup | account entry | `media/state-01-account-entry.png` and motion at 4.50s |
| 2 | Complete email verification and account authentication | Dashlane activates the member identity | identity verified | `media/state-02-identity-verified.png` and motion at 19.79s |
| 3 | Set the supported account-security method | Dashlane confirms the protected vault | vault secured | `media/state-03-vault-secured.png` and motion at 35.07s |
| 4 | Install and authorize the browser extension | Dashlane shows the connected extension state | extension ready | `media/state-04-extension-ready.png` and motion at 50.36s |
| 5 | Import or add a login credential | Dashlane stores the item in the vault | credential added | `media/state-05-credential-added.png` and motion at 65.65s |
| 6 | Use or inspect the credential from the extension | The item is available for autofill, proving first setup success | autofill ready | `media/state-06-autofill-ready.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At identity verified or vault secured, invalid, expired, denied, or missing required input leaves the flow short of autofill ready; evidence: media/state-02-identity-verified.png, media/state-03-vault-secured.png, and https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-vault-secured.png through media/state-05-credential-added.png.
- **Recovery:** Return to the retained identity verified or vault secured requirement, correct or resend the blocking input, and resubmit; evidence: https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started.
- **Recovery:** Continue through the same terminal action until autofill ready is visible in media/state-06-autofill-ready.png and the motion at 80.940s.
- **Completion evidence:** autofill ready retained at media/state-06-autofill-ready.png and media/official-recording.mp4#t=80.940; source https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account entry | [`media/state-01-account-entry.png`](media/state-01-account-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `fc501af350f85587b475325a04c4569447547d80a50718d7a68f6a9975e6f34f` |
| identity verified | [`media/state-02-identity-verified.png`](media/state-02-identity-verified.png) | media/official-recording.mp4#t=19.785 | 640×360 | `e7dd323cd540cc1aff6f61ee21cb69b39632661b10cbd072c76a449ea3a5f2fa` |
| vault secured | [`media/state-03-vault-secured.png`](media/state-03-vault-secured.png) | media/official-recording.mp4#t=35.074 | 640×360 | `036dababaaf6edd14a05d92ebe2e71837211b9ac8232c16a9921d6516884f54f` |
| extension ready | [`media/state-04-extension-ready.png`](media/state-04-extension-ready.png) | media/official-recording.mp4#t=50.362 | 640×360 | `6f77dc0390e5bd3853c4729c1464510e3b073fe5ccfc29e0543c74c606cfb215` |
| credential added | [`media/state-05-credential-added.png`](media/state-05-credential-added.png) | media/official-recording.mp4#t=65.651 | 640×360 | `eb8af13ab11080c2f7e087018a7d6f4a7a8c79fa215b5c8d5a8d2dc88a457c11` |
| autofill ready | [`media/state-06-autofill-ready.png`](media/state-06-autofill-ready.png) | media/official-recording.mp4#t=80.940 | 640×360 | `3aedabe49d69e73decbb34d68c2992a7e238be00705dc256d9d718a925244ce6` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Open the invitation or create the account | Dashlane starts identity and security setup The retained account entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-entry.png @ 4.50s; https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started |
| focus and selection | Complete email verification and account authentication | Dashlane activates the member identity The recording advances to identity verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-entry.png @ 4.50s; media/state-02-identity-verified.png @ 19.79s; https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started |
| navigation | Set the supported account-security method | Dashlane confirms the protected vault The navigation result is visible as vault secured. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-verified.png @ 19.79s; media/state-03-vault-secured.png @ 35.07s; https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started |
| confirmation | Install and authorize the browser extension | Dashlane shows the connected extension state The official recording shows the confirmed extension ready state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-vault-secured.png @ 35.07s; media/state-04-extension-ready.png @ 50.36s; https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-verified.png @ 19.79s; media/state-03-vault-secured.png @ 35.07s; media/state-04-extension-ready.png @ 50.36s; https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started |
| progress feedback | Import or add a login credential | Dashlane stores the item in the vault Progress is observable as the distinct credential added state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-extension-ready.png @ 50.36s; media/state-05-credential-added.png @ 65.65s; https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-vault-secured.png @ 35.07s; media/state-04-extension-ready.png @ 50.36s; media/state-05-credential-added.png @ 65.65s; https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started |
| recovery and completion | Use or inspect the credential from the extension | The item is available for autofill, proving first setup success The retained autofill ready state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-credential-added.png @ 65.65s; media/state-06-autofill-ready.png @ 80.94s; https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started |

## Motion behavior

- **Trigger:** The recorded sequence begins at account entry; the first advancing trigger is “Complete email verification and account authentication”.
- **Start/end:** Start is account entry at 4.50s; end is autofill ready at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in autofill ready; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-entry.png and media/state-02-identity-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-vault-secured.png and media/state-04-extension-ready.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Dashlane, Inc.
- **Product page:** https://support.dashlane.com/hc/en-us/categories/360001537079-Get-started
- **Original media URL:** https://www.youtube.com/watch?v=TpCbf3ocOwk
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 1069975 bytes
- **SHA-256:** `7f2adfa541926da993d69f1c45d8a2ca127856fcc1259e5702c67666303216aa`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
