# Discord — getting started

**Evidence status:** `complete`  
**Product/source:** [https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started](https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How Discord Works in 148,000 Miliseconds or Less](https://www.youtube.com/watch?v=TJ13BA3-NR4) — Discord

## Start-to-first-success journey

**Actor:** new Discord member  
**Goal:** register and participate in the first server  
**Prerequisites:** email address; server invitation or server concept

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Choose Register and enter account details | Discord validates the account form | registration | `media/state-01-registration.png` and motion at 4.50s |
| 2 | Complete the email verification challenge | Discord marks the account verified and returns to the app | identity verified | `media/state-02-identity-verified.png` and motion at 19.79s |
| 3 | Choose to join a server or create one | Discord opens invitation resolution or server templates | server route | `media/state-03-server-route.png` and motion at 35.07s |
| 4 | Accept the invitation or name the new server | Discord adds the server to navigation | server active | `media/state-04-server-active.png` and motion at 50.36s |
| 5 | Set optional profile or server identity | Discord updates visible identity without blocking chat | identity configured | `media/state-05-identity-configured.png` and motion at 65.65s |
| 6 | Send the first channel message | The message appears in the server, proving first participation | first message | `media/state-06-first-message.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At identity verified or server route, invalid, expired, denied, or missing required input leaves the flow short of first message; evidence: media/state-02-identity-verified.png, media/state-03-server-route.png, and https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-server-route.png through media/state-05-identity-configured.png.
- **Recovery:** Return to the retained identity verified or server route requirement, correct or resend the blocking input, and resubmit; evidence: https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started.
- **Recovery:** Continue through the same terminal action until first message is visible in media/state-06-first-message.png and the motion at 80.940s.
- **Completion evidence:** first message retained at media/state-06-first-message.png and media/official-recording.mp4#t=80.940; source https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| registration | [`media/state-01-registration.png`](media/state-01-registration.png) | media/official-recording.mp4#t=4.497 | 640×360 | `7813f32a785848cb05f6a5d6915b8e0b4739fdaaaac5f0b165b88c2e20af527e` |
| identity verified | [`media/state-02-identity-verified.png`](media/state-02-identity-verified.png) | media/official-recording.mp4#t=19.785 | 640×360 | `09f0868be59828ecb38c5266166f7b255b2e139f74ccb9adc94ad6ab9217aa00` |
| server route | [`media/state-03-server-route.png`](media/state-03-server-route.png) | media/official-recording.mp4#t=35.074 | 640×360 | `09dadeea180fb85cd8ba25f350f34131b8b53f56ab91c5cae92a0d00d097ab5c` |
| server active | [`media/state-04-server-active.png`](media/state-04-server-active.png) | media/official-recording.mp4#t=50.362 | 640×360 | `45b736ac26612ff7623f310f7eddb23a1f0c1728bb5816135d6208852aff7ead` |
| identity configured | [`media/state-05-identity-configured.png`](media/state-05-identity-configured.png) | media/official-recording.mp4#t=65.651 | 640×360 | `3363fc658f75a13ca81bcf9a6f2b4ae5d3b4f90de05a04e1f3b38b1378b9ecd6` |
| first message | [`media/state-06-first-message.png`](media/state-06-first-message.png) | media/official-recording.mp4#t=80.940 | 640×360 | `3f54ced8ca07a894c35a5421a9654451aa6b3ec08b0aa02ad3733f2ad98cd025` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Choose Register and enter account details | Discord validates the account form The retained registration state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-registration.png @ 4.50s; https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started |
| focus and selection | Complete the email verification challenge | Discord marks the account verified and returns to the app The recording advances to identity verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-registration.png @ 4.50s; media/state-02-identity-verified.png @ 19.79s; https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started |
| navigation | Choose to join a server or create one | Discord opens invitation resolution or server templates The navigation result is visible as server route. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-verified.png @ 19.79s; media/state-03-server-route.png @ 35.07s; https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started |
| confirmation | Accept the invitation or name the new server | Discord adds the server to navigation The official recording shows the confirmed server active state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-server-route.png @ 35.07s; media/state-04-server-active.png @ 50.36s; https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-verified.png @ 19.79s; media/state-03-server-route.png @ 35.07s; media/state-04-server-active.png @ 50.36s; https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started |
| progress feedback | Set optional profile or server identity | Discord updates visible identity without blocking chat Progress is observable as the distinct identity configured state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-server-active.png @ 50.36s; media/state-05-identity-configured.png @ 65.65s; https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-server-route.png @ 35.07s; media/state-04-server-active.png @ 50.36s; media/state-05-identity-configured.png @ 65.65s; https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started |
| recovery and completion | Send the first channel message | The message appears in the server, proving first participation The retained first message state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-identity-configured.png @ 65.65s; media/state-06-first-message.png @ 80.94s; https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started |

## Motion behavior

- **Trigger:** The recorded sequence begins at registration; the first advancing trigger is “Complete the email verification challenge”.
- **Start/end:** Start is registration at 4.50s; end is first message at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first message; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-registration.png and media/state-02-identity-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-server-route.png and media/state-04-server-active.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Discord Inc.
- **Product page:** https://support.discord.com/hc/en-us/articles/360033931551-Getting-Started
- **Original media URL:** https://www.youtube.com/watch?v=TJ13BA3-NR4
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 767988 bytes
- **SHA-256:** `768c0c8dc4db85c348b4896bde7086d83a2c3a735bee390164373a75891a35d9`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
