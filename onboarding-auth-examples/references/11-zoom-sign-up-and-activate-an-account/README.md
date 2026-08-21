# Zoom — sign up and activate an account

**Evidence status:** `complete`  
**Product/source:** [https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655](https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to Set Up Your Zoom Phone Account](https://www.youtube.com/watch?v=tPyVeQvgtZY) — Zoom

## Start-to-first-success journey

**Actor:** new Zoom account owner  
**Goal:** activate an account and reach the meeting controls  
**Prerequisites:** email address able to receive activation; supported browser

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Choose Sign up and provide the required identity details | Zoom shows the account-creation and eligibility state | signup entry | `media/state-01-signup-entry.png` and motion at 4.50s |
| 2 | Submit the email address | Zoom sends an activation message and shows the pending state | activation pending | `media/state-02-activation-pending.png` and motion at 19.79s |
| 3 | Open the activation link | Zoom verifies the link and opens credential setup | activation verified | `media/state-03-activation-verified.png` and motion at 35.07s |
| 4 | Set account credentials and profile details | Zoom creates the account and opens the web portal | account active | `media/state-04-account-active.png` and motion at 50.36s |
| 5 | Review portal settings or skip optional profile work | Zoom preserves settings and exposes meeting controls | portal ready | `media/state-05-portal-ready.png` and motion at 65.65s |
| 6 | Start or schedule the first meeting | Zoom creates the meeting record, proving first communication success | first meeting | `media/state-06-first-meeting.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At activation pending or activation verified, invalid, expired, denied, or missing required input leaves the flow short of first meeting; evidence: media/state-02-activation-pending.png, media/state-03-activation-verified.png, and https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-activation-verified.png through media/state-05-portal-ready.png.
- **Recovery:** Return to the retained activation pending or activation verified requirement, correct or resend the blocking input, and resubmit; evidence: https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655.
- **Recovery:** Continue through the same terminal action until first meeting is visible in media/state-06-first-meeting.png and the motion at 80.940s.
- **Completion evidence:** first meeting retained at media/state-06-first-meeting.png and media/official-recording.mp4#t=80.940; source https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| signup entry | [`media/state-01-signup-entry.png`](media/state-01-signup-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `80fa01a2e3389e6988014b1705ed06ca1456db18d4df7c183dc37897bfec756e` |
| activation pending | [`media/state-02-activation-pending.png`](media/state-02-activation-pending.png) | media/official-recording.mp4#t=19.785 | 640×360 | `b5e244b27fec9effc044d73aeeb8a2326e52c4f2896ff27a54557a503b0c0b8f` |
| activation verified | [`media/state-03-activation-verified.png`](media/state-03-activation-verified.png) | media/official-recording.mp4#t=35.074 | 640×360 | `4694a3ea80f45610583ca8951abb3143314ae23ef06b7578f1468b6db65779bc` |
| account active | [`media/state-04-account-active.png`](media/state-04-account-active.png) | media/official-recording.mp4#t=50.362 | 640×360 | `8455e3d138b6cb03c52fdf55823e66b2ceb3aa2edd1581873271e4074029d990` |
| portal ready | [`media/state-05-portal-ready.png`](media/state-05-portal-ready.png) | media/official-recording.mp4#t=65.651 | 640×360 | `61b3809fc9bc2fb9eb0c6fb88ea63bc7b84b9d17b4c6a77285ac10fc1d399fd9` |
| first meeting | [`media/state-06-first-meeting.png`](media/state-06-first-meeting.png) | media/official-recording.mp4#t=80.940 | 640×360 | `240275b49a219112a2c4efb1c929b8d48d40d26b3b3831f6ea23d167d65ec10a` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Choose Sign up and provide the required identity details | Zoom shows the account-creation and eligibility state The retained signup entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-signup-entry.png @ 4.50s; https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655 |
| focus and selection | Submit the email address | Zoom sends an activation message and shows the pending state The recording advances to activation pending and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-signup-entry.png @ 4.50s; media/state-02-activation-pending.png @ 19.79s; https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655 |
| navigation | Open the activation link | Zoom verifies the link and opens credential setup The navigation result is visible as activation verified. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-activation-pending.png @ 19.79s; media/state-03-activation-verified.png @ 35.07s; https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655 |
| confirmation | Set account credentials and profile details | Zoom creates the account and opens the web portal The official recording shows the confirmed account active state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-activation-verified.png @ 35.07s; media/state-04-account-active.png @ 50.36s; https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655 |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-activation-pending.png @ 19.79s; media/state-03-activation-verified.png @ 35.07s; media/state-04-account-active.png @ 50.36s; https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655 |
| progress feedback | Review portal settings or skip optional profile work | Zoom preserves settings and exposes meeting controls Progress is observable as the distinct portal ready state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-account-active.png @ 50.36s; media/state-05-portal-ready.png @ 65.65s; https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655 |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-activation-verified.png @ 35.07s; media/state-04-account-active.png @ 50.36s; media/state-05-portal-ready.png @ 65.65s; https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655 |
| recovery and completion | Start or schedule the first meeting | Zoom creates the meeting record, proving first communication success The retained first meeting state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-portal-ready.png @ 65.65s; media/state-06-first-meeting.png @ 80.94s; https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655 |

## Motion behavior

- **Trigger:** The recorded sequence begins at signup entry; the first advancing trigger is “Submit the email address”.
- **Start/end:** Start is signup entry at 4.50s; end is first meeting at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first meeting; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-signup-entry.png and media/state-02-activation-pending.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-activation-verified.png and media/state-04-account-active.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Zoom Video Communications, Inc.
- **Product page:** https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0063655
- **Original media URL:** https://www.youtube.com/watch?v=tPyVeQvgtZY
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 380693 bytes
- **SHA-256:** `97e78ed08d8dc3c746995b56e6790afe3e3df0d6e4594618e0d886918632640b`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
