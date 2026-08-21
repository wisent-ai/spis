# Yubico — set up a security key

**Evidence status:** `complete`  
**Product/source:** [https://www.yubico.com/setup/](https://www.yubico.com/setup/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Instructional Setup Series: YubiKey Security Key Series](https://www.youtube.com/watch?v=V6mxPS5O-sY) — Yubico

## Start-to-first-success journey

**Actor:** security-key owner  
**Goal:** enroll and verify a YubiKey with an online account  
**Prerequisites:** supported YubiKey; account supporting security keys; backup authentication method

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Choose the YubiKey model and target service | Yubico routes to compatible enrollment guidance | setup route | `media/state-01-setup-route.png` and motion at 4.50s |
| 2 | Sign in and open the service's security settings | The service lists authentication methods | security settings | `media/state-02-security-settings.png` and motion at 19.79s |
| 3 | Choose Add security key | The browser presents the WebAuthn registration prompt | registration prompt | `media/state-03-registration-prompt.png` and motion at 35.07s |
| 4 | Insert the key and touch it when requested | The authenticator completes the physical-presence check | key detected | `media/state-04-key-detected.png` and motion at 50.36s |
| 5 | Name the key and confirm enrollment | The service lists the new key | key enrolled | `media/state-05-key-enrolled.png` and motion at 65.65s |
| 6 | Sign out and authenticate with the key | The service accepts the key, proving first-success hardware authentication | key verified | `media/state-06-key-verified.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At security settings or registration prompt, invalid, expired, denied, or missing required input leaves the flow short of key verified; evidence: media/state-02-security-settings.png, media/state-03-registration-prompt.png, and https://www.yubico.com/setup/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-registration-prompt.png through media/state-05-key-enrolled.png.
- **Recovery:** Return to the retained security settings or registration prompt requirement, correct or resend the blocking input, and resubmit; evidence: https://www.yubico.com/setup/.
- **Recovery:** Continue through the same terminal action until key verified is visible in media/state-06-key-verified.png and the motion at 80.940s.
- **Completion evidence:** key verified retained at media/state-06-key-verified.png and media/official-recording.mp4#t=80.940; source https://www.yubico.com/setup/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| setup route | [`media/state-01-setup-route.png`](media/state-01-setup-route.png) | media/official-recording.mp4#t=4.497 | 640×360 | `45c5444f0f8732f822b1b001f302601bd0b3491cd08113a008e842e439307f10` |
| security settings | [`media/state-02-security-settings.png`](media/state-02-security-settings.png) | media/official-recording.mp4#t=19.785 | 640×360 | `aa1aa621e99a22618f9fee1625f44111c265051ef2a984a72575ee3406b1c4cb` |
| registration prompt | [`media/state-03-registration-prompt.png`](media/state-03-registration-prompt.png) | media/official-recording.mp4#t=35.074 | 640×360 | `7761468e85b131decf79fd534175cf391fa69bb01730266929637f250aa34591` |
| key detected | [`media/state-04-key-detected.png`](media/state-04-key-detected.png) | media/official-recording.mp4#t=50.362 | 640×360 | `2739342b929ae97e86c7a120e1b7652b36e341d3c835acc76d201baba2e2c566` |
| key enrolled | [`media/state-05-key-enrolled.png`](media/state-05-key-enrolled.png) | media/official-recording.mp4#t=65.651 | 640×360 | `9f4ec1f897ae89bfa43d1a0f4246e47f7e8de63e6ca091abd522f36073c5b113` |
| key verified | [`media/state-06-key-verified.png`](media/state-06-key-verified.png) | media/official-recording.mp4#t=80.940 | 640×360 | `18933cab1007af5baa75be1e83d498dd59dc63462d3e933df3404f75aca5e3fe` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Choose the YubiKey model and target service | Yubico routes to compatible enrollment guidance The retained setup route state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-setup-route.png @ 4.50s; https://www.yubico.com/setup/ |
| focus and selection | Sign in and open the service's security settings | The service lists authentication methods The recording advances to security settings and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-setup-route.png @ 4.50s; media/state-02-security-settings.png @ 19.79s; https://www.yubico.com/setup/ |
| navigation | Choose Add security key | The browser presents the WebAuthn registration prompt The navigation result is visible as registration prompt. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-security-settings.png @ 19.79s; media/state-03-registration-prompt.png @ 35.07s; https://www.yubico.com/setup/ |
| confirmation | Insert the key and touch it when requested | The authenticator completes the physical-presence check The official recording shows the confirmed key detected state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-registration-prompt.png @ 35.07s; media/state-04-key-detected.png @ 50.36s; https://www.yubico.com/setup/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-security-settings.png @ 19.79s; media/state-03-registration-prompt.png @ 35.07s; media/state-04-key-detected.png @ 50.36s; https://www.yubico.com/setup/ |
| progress feedback | Name the key and confirm enrollment | The service lists the new key Progress is observable as the distinct key enrolled state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-key-detected.png @ 50.36s; media/state-05-key-enrolled.png @ 65.65s; https://www.yubico.com/setup/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-registration-prompt.png @ 35.07s; media/state-04-key-detected.png @ 50.36s; media/state-05-key-enrolled.png @ 65.65s; https://www.yubico.com/setup/ |
| recovery and completion | Sign out and authenticate with the key | The service accepts the key, proving first-success hardware authentication The retained key verified state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-key-enrolled.png @ 65.65s; media/state-06-key-verified.png @ 80.94s; https://www.yubico.com/setup/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at setup route; the first advancing trigger is “Sign in and open the service's security settings”.
- **Start/end:** Start is setup route at 4.50s; end is key verified at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in key verified; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-setup-route.png and media/state-02-security-settings.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-registration-prompt.png and media/state-04-key-detected.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Yubico AB
- **Product page:** https://www.yubico.com/setup/
- **Original media URL:** https://www.youtube.com/watch?v=V6mxPS5O-sY
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 413913 bytes
- **SHA-256:** `c1deeb69c2d6beb4965f081930fd39e531c0ee63b6a0c4d93c63620c59d6c76a`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
