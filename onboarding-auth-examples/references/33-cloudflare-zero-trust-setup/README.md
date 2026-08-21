# Cloudflare Zero Trust — setup

**Evidence status:** `complete`  
**Product/source:** [https://developers.cloudflare.com/cloudflare-one/setup/](https://developers.cloudflare.com/cloudflare-one/setup/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Cloudflare Zero Trust Network Access Demo](https://www.youtube.com/watch?v=Lxp-LYbKwiY) — Cloudflare

## Start-to-first-success journey

**Actor:** Cloudflare Zero Trust administrator  
**Goal:** create protected access and confirm the first authorized connection  
**Prerequisites:** Cloudflare account and domain or application; identity-provider access

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Open Zero Trust and create the team context | Cloudflare records the organization or team name | team created | `media/state-01-team-created.png` and motion at 4.50s |
| 2 | Choose plan and configure the identity provider | Cloudflare displays the enabled authentication method | identity connected | `media/state-02-identity-connected.png` and motion at 19.79s |
| 3 | Add an application or network target | Cloudflare opens access policy configuration | target added | `media/state-03-target-added.png` and motion at 35.07s |
| 4 | Define an allow policy and save it | Cloudflare shows the effective policy | policy active | `media/state-04-policy-active.png` and motion at 50.36s |
| 5 | Enroll a device or open the protected application | Cloudflare presents authentication and device checks | access challenge | `media/state-05-access-challenge.png` and motion at 65.65s |
| 6 | Authenticate as an allowed user | The protected resource opens, proving first Zero Trust success | access granted | `media/state-06-access-granted.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At identity connected or target added, invalid, expired, denied, or missing required input leaves the flow short of access granted; evidence: media/state-02-identity-connected.png, media/state-03-target-added.png, and https://developers.cloudflare.com/cloudflare-one/setup/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-target-added.png through media/state-05-access-challenge.png.
- **Recovery:** Return to the retained identity connected or target added requirement, correct or resend the blocking input, and resubmit; evidence: https://developers.cloudflare.com/cloudflare-one/setup/.
- **Recovery:** Continue through the same terminal action until access granted is visible in media/state-06-access-granted.png and the motion at 80.940s.
- **Completion evidence:** access granted retained at media/state-06-access-granted.png and media/official-recording.mp4#t=80.940; source https://developers.cloudflare.com/cloudflare-one/setup/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| team created | [`media/state-01-team-created.png`](media/state-01-team-created.png) | media/official-recording.mp4#t=4.497 | 640×352 | `2e4e59612dde1c9bf7bceaae9873c45a8f1a605685805a111fd5ea18d23657ab` |
| identity connected | [`media/state-02-identity-connected.png`](media/state-02-identity-connected.png) | media/official-recording.mp4#t=19.785 | 640×352 | `53db14cc44cacfe086d795a720ab440911378442535bf9c21e753d779d61bcce` |
| target added | [`media/state-03-target-added.png`](media/state-03-target-added.png) | media/official-recording.mp4#t=35.074 | 640×352 | `4923cdc2a0eed815e4dbd3c9ed20aa4bd9a56bea1979a1ffc04281741aedb5be` |
| policy active | [`media/state-04-policy-active.png`](media/state-04-policy-active.png) | media/official-recording.mp4#t=50.362 | 640×352 | `26809f0a7b3d3e0e1c70a36a431782603ad0168cb8f0bf100027676794dddd59` |
| access challenge | [`media/state-05-access-challenge.png`](media/state-05-access-challenge.png) | media/official-recording.mp4#t=65.651 | 640×352 | `807efda3c4dfc3e3f88f947d9db7ea3ee301c5f7cf5d874f8f8c68e3c46e4e43` |
| access granted | [`media/state-06-access-granted.png`](media/state-06-access-granted.png) | media/official-recording.mp4#t=80.940 | 640×352 | `097913cd3ec8b5b7033374f0db49ca7d96425703783b93dfe79a4da5aeedaaae` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Open Zero Trust and create the team context | Cloudflare records the organization or team name The retained team created state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-team-created.png @ 4.50s; https://developers.cloudflare.com/cloudflare-one/setup/ |
| focus and selection | Choose plan and configure the identity provider | Cloudflare displays the enabled authentication method The recording advances to identity connected and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-team-created.png @ 4.50s; media/state-02-identity-connected.png @ 19.79s; https://developers.cloudflare.com/cloudflare-one/setup/ |
| navigation | Add an application or network target | Cloudflare opens access policy configuration The navigation result is visible as target added. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-connected.png @ 19.79s; media/state-03-target-added.png @ 35.07s; https://developers.cloudflare.com/cloudflare-one/setup/ |
| confirmation | Define an allow policy and save it | Cloudflare shows the effective policy The official recording shows the confirmed policy active state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-target-added.png @ 35.07s; media/state-04-policy-active.png @ 50.36s; https://developers.cloudflare.com/cloudflare-one/setup/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-connected.png @ 19.79s; media/state-03-target-added.png @ 35.07s; media/state-04-policy-active.png @ 50.36s; https://developers.cloudflare.com/cloudflare-one/setup/ |
| progress feedback | Enroll a device or open the protected application | Cloudflare presents authentication and device checks Progress is observable as the distinct access challenge state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-policy-active.png @ 50.36s; media/state-05-access-challenge.png @ 65.65s; https://developers.cloudflare.com/cloudflare-one/setup/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-target-added.png @ 35.07s; media/state-04-policy-active.png @ 50.36s; media/state-05-access-challenge.png @ 65.65s; https://developers.cloudflare.com/cloudflare-one/setup/ |
| recovery and completion | Authenticate as an allowed user | The protected resource opens, proving first Zero Trust success The retained access granted state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-access-challenge.png @ 65.65s; media/state-06-access-granted.png @ 80.94s; https://developers.cloudflare.com/cloudflare-one/setup/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at team created; the first advancing trigger is “Choose plan and configure the identity provider”.
- **Start/end:** Start is team created at 4.50s; end is access granted at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in access granted; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-team-created.png and media/state-02-identity-connected.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-target-added.png and media/state-04-policy-active.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Cloudflare, Inc.
- **Product page:** https://developers.cloudflare.com/cloudflare-one/setup/
- **Original media URL:** https://www.youtube.com/watch?v=Lxp-LYbKwiY
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×352, 89.933s, 1349 frames, 250717 bytes
- **SHA-256:** `e3f78de7c5da1cc2cbda80d29378760205e519ccfc6c1b5413ab5e7030285c8a`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
