# Bitwarden — get started

**Evidence status:** `complete`  
**Product/source:** [https://bitwarden.com/help/getting-started-webvault/](https://bitwarden.com/help/getting-started-webvault/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Bitwarden Secrets Manager 101: How to create a service account](https://www.youtube.com/watch?v=WGWR9-6CdwA) — Bitwarden

## Start-to-first-success journey

**Actor:** new Bitwarden vault owner  
**Goal:** create a vault and save the first login  
**Prerequisites:** email address; master-password decision

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Choose Create account and enter email | Bitwarden opens account security setup | account entry | `media/state-01-account-entry.png` and motion at 3.40s |
| 2 | Set and confirm the master password | Bitwarden creates the encrypted account context | vault secured | `media/state-02-vault-secured.png` and motion at 14.95s |
| 3 | Complete email verification when prompted | Bitwarden marks the email verified | identity verified | `media/state-03-identity-verified.png` and motion at 26.49s |
| 4 | Sign in to the Web Vault | Bitwarden opens the empty vault | vault open | `media/state-04-vault-open.png` and motion at 38.04s |
| 5 | Choose New item and enter login details | Bitwarden validates the item fields | item ready | `media/state-05-item-ready.png` and motion at 49.59s |
| 6 | Save the login | The item appears in the vault, proving first-success credential storage | first item | `media/state-06-first-item.png` and motion at 61.14s |

### Failure and recovery

- **Failure:** At vault secured or identity verified, invalid, expired, denied, or missing required input leaves the flow short of first item; evidence: media/state-02-vault-secured.png, media/state-03-identity-verified.png, and https://bitwarden.com/help/getting-started-webvault/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-identity-verified.png through media/state-05-item-ready.png.
- **Recovery:** Return to the retained vault secured or identity verified requirement, correct or resend the blocking input, and resubmit; evidence: https://bitwarden.com/help/getting-started-webvault/.
- **Recovery:** Continue through the same terminal action until first item is visible in media/state-06-first-item.png and the motion at 61.140s.
- **Completion evidence:** first item retained at media/state-06-first-item.png and media/official-recording.mp4#t=61.140; source https://bitwarden.com/help/getting-started-webvault/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account entry | [`media/state-01-account-entry.png`](media/state-01-account-entry.png) | media/official-recording.mp4#t=3.397 | 640×360 | `1a384adc467f2c706e230b0edfcc1d16efe0942b155e465de0f82c2c67e35a00` |
| vault secured | [`media/state-02-vault-secured.png`](media/state-02-vault-secured.png) | media/official-recording.mp4#t=14.945 | 640×360 | `91a04e6f2fbdf954d9341b9f58e9e509b8a1f4a85c89f8ce3b0041ad2ec28a99` |
| identity verified | [`media/state-03-identity-verified.png`](media/state-03-identity-verified.png) | media/official-recording.mp4#t=26.494 | 640×360 | `cad81801e3814357d1d22a11afe2d1fee849761755520af551f5489722ce3b19` |
| vault open | [`media/state-04-vault-open.png`](media/state-04-vault-open.png) | media/official-recording.mp4#t=38.042 | 640×360 | `bde8faa65e9e832c172a9876a2f7f341c8aa345c0f19c57e77b07ddf805c512a` |
| item ready | [`media/state-05-item-ready.png`](media/state-05-item-ready.png) | media/official-recording.mp4#t=49.591 | 640×360 | `b02298c22b46c94c54ce09cf0b6e300739bd2b227bdf22fd162b4de168ec974d` |
| first item | [`media/state-06-first-item.png`](media/state-06-first-item.png) | media/official-recording.mp4#t=61.140 | 640×360 | `76291acb7a5327e6c0acde8234c38cd21e52646a79755c5fcfe1fc91a7c9f53f` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Choose Create account and enter email | Bitwarden opens account security setup The retained account entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-entry.png @ 3.40s; https://bitwarden.com/help/getting-started-webvault/ |
| focus and selection | Set and confirm the master password | Bitwarden creates the encrypted account context The recording advances to vault secured and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-entry.png @ 3.40s; media/state-02-vault-secured.png @ 14.95s; https://bitwarden.com/help/getting-started-webvault/ |
| navigation | Complete email verification when prompted | Bitwarden marks the email verified The navigation result is visible as identity verified. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-vault-secured.png @ 14.95s; media/state-03-identity-verified.png @ 26.49s; https://bitwarden.com/help/getting-started-webvault/ |
| confirmation | Sign in to the Web Vault | Bitwarden opens the empty vault The official recording shows the confirmed vault open state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-identity-verified.png @ 26.49s; media/state-04-vault-open.png @ 38.04s; https://bitwarden.com/help/getting-started-webvault/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-vault-secured.png @ 14.95s; media/state-03-identity-verified.png @ 26.49s; media/state-04-vault-open.png @ 38.04s; https://bitwarden.com/help/getting-started-webvault/ |
| progress feedback | Choose New item and enter login details | Bitwarden validates the item fields Progress is observable as the distinct item ready state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-vault-open.png @ 38.04s; media/state-05-item-ready.png @ 49.59s; https://bitwarden.com/help/getting-started-webvault/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-identity-verified.png @ 26.49s; media/state-04-vault-open.png @ 38.04s; media/state-05-item-ready.png @ 49.59s; https://bitwarden.com/help/getting-started-webvault/ |
| recovery and completion | Save the login | The item appears in the vault, proving first-success credential storage The retained first item state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-item-ready.png @ 49.59s; media/state-06-first-item.png @ 61.14s; https://bitwarden.com/help/getting-started-webvault/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at account entry; the first advancing trigger is “Set and confirm the master password”.
- **Start/end:** Start is account entry at 3.40s; end is first item at 61.14s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 67.933s at 15 fps (1019 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first item; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-entry.png and media/state-02-vault-secured.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-identity-verified.png and media/state-04-vault-open.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Bitwarden Inc.
- **Product page:** https://bitwarden.com/help/getting-started-webvault/
- **Original media URL:** https://www.youtube.com/watch?v=WGWR9-6CdwA
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 67.933s, 1019 frames, 286049 bytes
- **SHA-256:** `e9a35c7007c0fb519ab154ed0e4c62797c2c29b55331672ad98f614bb7498af2`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
