# 1Password — get started

**Evidence status:** `complete`  
**Product/source:** [https://support.1password.com/explore/get-started/](https://support.1password.com/explore/get-started/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to get started with 1Password](https://www.youtube.com/watch?v=seMl5imFNCQ) — 1Password

## Start-to-first-success journey

**Actor:** new 1Password account owner  
**Goal:** secure the account and save the first item  
**Prerequisites:** email access; strong account-password decision; safe place for recovery material

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Create the 1Password account | 1Password opens email and account setup | account entry | `media/state-01-account-entry.png` and motion at 4.50s |
| 2 | Verify the email or invitation | 1Password accepts the account identity | identity verified | `media/state-02-identity-verified.png` and motion at 19.79s |
| 3 | Set the account password and retain the Secret Key or Emergency Kit | 1Password confirms the recovery-sensitive setup | account secured | `media/state-03-account-secured.png` and motion at 35.07s |
| 4 | Install or open an app and sign in | 1Password unlocks the first vault | vault open | `media/state-04-vault-open.png` and motion at 50.36s |
| 5 | Choose New Item and enter a credential | 1Password validates and encrypts the item | item ready | `media/state-05-item-ready.png` and motion at 65.65s |
| 6 | Save and reopen the item | The credential persists in the vault, proving first password-management success | first item | `media/state-06-first-item.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At identity verified or account secured, invalid, expired, denied, or missing required input leaves the flow short of first item; evidence: media/state-02-identity-verified.png, media/state-03-account-secured.png, and https://support.1password.com/explore/get-started/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-account-secured.png through media/state-05-item-ready.png.
- **Recovery:** Return to the retained identity verified or account secured requirement, correct or resend the blocking input, and resubmit; evidence: https://support.1password.com/explore/get-started/.
- **Recovery:** Continue through the same terminal action until first item is visible in media/state-06-first-item.png and the motion at 80.940s.
- **Completion evidence:** first item retained at media/state-06-first-item.png and media/official-recording.mp4#t=80.940; source https://support.1password.com/explore/get-started/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account entry | [`media/state-01-account-entry.png`](media/state-01-account-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `c8b4f08b2ec8849e4b5e0c91c56436eb45936dcdd3cf7bae5a612225e70f3350` |
| identity verified | [`media/state-02-identity-verified.png`](media/state-02-identity-verified.png) | media/official-recording.mp4#t=19.785 | 640×360 | `30b0af04af0d433fb373e79c99e2bfa295db1df676c162c74c2a25d4ab660ed9` |
| account secured | [`media/state-03-account-secured.png`](media/state-03-account-secured.png) | media/official-recording.mp4#t=35.074 | 640×360 | `58e8d482224ecc4c86ef3b0447afa7d8dec69f34fcd759a98cc0712ce331a677` |
| vault open | [`media/state-04-vault-open.png`](media/state-04-vault-open.png) | media/official-recording.mp4#t=50.362 | 640×360 | `9734f0d35b29532ae640d3871f515949c484395bbb311835e49936680924fe72` |
| item ready | [`media/state-05-item-ready.png`](media/state-05-item-ready.png) | media/official-recording.mp4#t=65.651 | 640×360 | `bbbf07f4d7e451699b19bd3b10304b787a44ce6319fb6631865c4eaae56f6509` |
| first item | [`media/state-06-first-item.png`](media/state-06-first-item.png) | media/official-recording.mp4#t=80.940 | 640×360 | `e5020102679422af00e46042cde71a9b9873788db7e57c10d41a3f7e6f715e37` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Create the 1Password account | 1Password opens email and account setup The retained account entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-entry.png @ 4.50s; https://support.1password.com/explore/get-started/ |
| focus and selection | Verify the email or invitation | 1Password accepts the account identity The recording advances to identity verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-entry.png @ 4.50s; media/state-02-identity-verified.png @ 19.79s; https://support.1password.com/explore/get-started/ |
| navigation | Set the account password and retain the Secret Key or Emergency Kit | 1Password confirms the recovery-sensitive setup The navigation result is visible as account secured. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-verified.png @ 19.79s; media/state-03-account-secured.png @ 35.07s; https://support.1password.com/explore/get-started/ |
| confirmation | Install or open an app and sign in | 1Password unlocks the first vault The official recording shows the confirmed vault open state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-account-secured.png @ 35.07s; media/state-04-vault-open.png @ 50.36s; https://support.1password.com/explore/get-started/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-verified.png @ 19.79s; media/state-03-account-secured.png @ 35.07s; media/state-04-vault-open.png @ 50.36s; https://support.1password.com/explore/get-started/ |
| progress feedback | Choose New Item and enter a credential | 1Password validates and encrypts the item Progress is observable as the distinct item ready state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-vault-open.png @ 50.36s; media/state-05-item-ready.png @ 65.65s; https://support.1password.com/explore/get-started/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-account-secured.png @ 35.07s; media/state-04-vault-open.png @ 50.36s; media/state-05-item-ready.png @ 65.65s; https://support.1password.com/explore/get-started/ |
| recovery and completion | Save and reopen the item | The credential persists in the vault, proving first password-management success The retained first item state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-item-ready.png @ 65.65s; media/state-06-first-item.png @ 80.94s; https://support.1password.com/explore/get-started/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at account entry; the first advancing trigger is “Verify the email or invitation”.
- **Start/end:** Start is account entry at 4.50s; end is first item at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first item; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-entry.png and media/state-02-identity-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-account-secured.png and media/state-04-vault-open.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** AgileBits Inc.
- **Product page:** https://support.1password.com/explore/get-started/
- **Original media URL:** https://www.youtube.com/watch?v=seMl5imFNCQ
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 326708 bytes
- **SHA-256:** `5c1a7434c52e77eff4bee424bc42bff94a143f28894c9c245ccf13e628360f45`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
