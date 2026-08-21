# Tailscale — quickstart

**Evidence status:** `complete`  
**Product/source:** [https://tailscale.com/kb/1017/install](https://tailscale.com/kb/1017/install)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to get started with Tailscale in under 10 minutes](https://www.youtube.com/watch?v=sPdvyR7bLqI) — Tailscale

## Start-to-first-success journey

**Actor:** new Tailscale network owner  
**Goal:** add the first device and verify private connectivity  
**Prerequisites:** supported device; supported identity provider

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Install and launch Tailscale | Tailscale exposes the sign-in action | client ready | `media/state-01-client-ready.png` and motion at 4.50s |
| 2 | Choose Sign in | Tailscale opens browser authentication | auth handoff | `media/state-02-auth-handoff.png` and motion at 19.79s |
| 3 | Select and complete the identity-provider flow | Tailscale confirms authorization for the tailnet | identity verified | `media/state-03-identity-verified.png` and motion at 35.07s |
| 4 | Return to the client | The client reports connected status and assigned address | device connected | `media/state-04-device-connected.png` and motion at 50.36s |
| 5 | Open the admin device list | The new device appears with identity and status | device enrolled | `media/state-05-device-enrolled.png` and motion at 65.65s |
| 6 | Reach another enrolled service or run the documented connectivity check | The private connection succeeds, proving first network success | first connection | `media/state-06-first-connection.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At auth handoff or identity verified, invalid, expired, denied, or missing required input leaves the flow short of first connection; evidence: media/state-02-auth-handoff.png, media/state-03-identity-verified.png, and https://tailscale.com/kb/1017/install.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-identity-verified.png through media/state-05-device-enrolled.png.
- **Recovery:** Return to the retained auth handoff or identity verified requirement, correct or resend the blocking input, and resubmit; evidence: https://tailscale.com/kb/1017/install.
- **Recovery:** Continue through the same terminal action until first connection is visible in media/state-06-first-connection.png and the motion at 80.940s.
- **Completion evidence:** first connection retained at media/state-06-first-connection.png and media/official-recording.mp4#t=80.940; source https://tailscale.com/kb/1017/install

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| client ready | [`media/state-01-client-ready.png`](media/state-01-client-ready.png) | media/official-recording.mp4#t=4.497 | 640×360 | `fe5f11a15b51a429dd75abd5e37fc574efd530e091b578dd4025e794968e310d` |
| auth handoff | [`media/state-02-auth-handoff.png`](media/state-02-auth-handoff.png) | media/official-recording.mp4#t=19.785 | 640×360 | `e068f7f6833b750bba619531dd2d451b675aaee3a09dd5e912d690510350e53d` |
| identity verified | [`media/state-03-identity-verified.png`](media/state-03-identity-verified.png) | media/official-recording.mp4#t=35.074 | 640×360 | `cf97f3bf98bfe8539e202bd56dcb6fa0f7702801ad70762a884f4c311c7bf2f6` |
| device connected | [`media/state-04-device-connected.png`](media/state-04-device-connected.png) | media/official-recording.mp4#t=50.362 | 640×360 | `b17d04b82d2d1f72a9c2b9a2a6dc2a39c7c290a7d7fdac19230fefb7ba5ffa45` |
| device enrolled | [`media/state-05-device-enrolled.png`](media/state-05-device-enrolled.png) | media/official-recording.mp4#t=65.651 | 640×360 | `72f38d0fade7d082925514afaca67174d706a65597e79f242ecb0c7341ece2e3` |
| first connection | [`media/state-06-first-connection.png`](media/state-06-first-connection.png) | media/official-recording.mp4#t=80.940 | 640×360 | `a807bbfb6ed40e4eb92ed53e0a8ab32ef311a048cbf77f2a003009b4a4322624` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Install and launch Tailscale | Tailscale exposes the sign-in action The retained client ready state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-client-ready.png @ 4.50s; https://tailscale.com/kb/1017/install |
| focus and selection | Choose Sign in | Tailscale opens browser authentication The recording advances to auth handoff and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-client-ready.png @ 4.50s; media/state-02-auth-handoff.png @ 19.79s; https://tailscale.com/kb/1017/install |
| navigation | Select and complete the identity-provider flow | Tailscale confirms authorization for the tailnet The navigation result is visible as identity verified. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-auth-handoff.png @ 19.79s; media/state-03-identity-verified.png @ 35.07s; https://tailscale.com/kb/1017/install |
| confirmation | Return to the client | The client reports connected status and assigned address The official recording shows the confirmed device connected state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-identity-verified.png @ 35.07s; media/state-04-device-connected.png @ 50.36s; https://tailscale.com/kb/1017/install |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-auth-handoff.png @ 19.79s; media/state-03-identity-verified.png @ 35.07s; media/state-04-device-connected.png @ 50.36s; https://tailscale.com/kb/1017/install |
| progress feedback | Open the admin device list | The new device appears with identity and status Progress is observable as the distinct device enrolled state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-device-connected.png @ 50.36s; media/state-05-device-enrolled.png @ 65.65s; https://tailscale.com/kb/1017/install |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-identity-verified.png @ 35.07s; media/state-04-device-connected.png @ 50.36s; media/state-05-device-enrolled.png @ 65.65s; https://tailscale.com/kb/1017/install |
| recovery and completion | Reach another enrolled service or run the documented connectivity check | The private connection succeeds, proving first network success The retained first connection state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-device-enrolled.png @ 65.65s; media/state-06-first-connection.png @ 80.94s; https://tailscale.com/kb/1017/install |

## Motion behavior

- **Trigger:** The recorded sequence begins at client ready; the first advancing trigger is “Choose Sign in”.
- **Start/end:** Start is client ready at 4.50s; end is first connection at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first connection; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-client-ready.png and media/state-02-auth-handoff.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-identity-verified.png and media/state-04-device-connected.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Tailscale Inc.
- **Product page:** https://tailscale.com/kb/1017/install
- **Original media URL:** https://www.youtube.com/watch?v=sPdvyR7bLqI
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 881604 bytes
- **SHA-256:** `372f2b6329f35134be4666a0afbed3fe2d1ee4b0570028419877512ed66e9224`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
