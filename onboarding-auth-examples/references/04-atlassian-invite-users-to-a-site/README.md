# Atlassian — invite users to a site

**Evidence status:** `complete`  
**Product/source:** [https://support.atlassian.com/user-management/docs/invite-a-user/](https://support.atlassian.com/user-management/docs/invite-a-user/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Jira Tutorial for Beginners | Atlassian Answered](https://www.youtube.com/watch?v=emidrJeUTaM) — Atlassian

## Start-to-first-success journey

**Actor:** Atlassian site administrator  
**Goal:** invite a person and assign product access  
**Prerequisites:** site-admin permission; invitee email address

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Sign in and open user management | Atlassian shows managed users and invitation controls | user administration | `media/state-01-user-administration.png` and motion at 4.50s |
| 2 | Choose Invite users | Atlassian opens the invitation form | invite form | `media/state-02-invite-form.png` and motion at 19.79s |
| 3 | Enter one or more email addresses | Atlassian resolves invite recipients and enables access choices | recipient selection | `media/state-03-recipient-selection.png` and motion at 35.07s |
| 4 | Select the products or roles the invitees may access | Atlassian displays the effective access assignment | access assignment | `media/state-04-access-assignment.png` and motion at 50.36s |
| 5 | Review and send the invitation | Atlassian confirms submission and records pending invitations | invitation pending | `media/state-05-invitation-pending.png` and motion at 65.65s |
| 6 | Open the pending-user record | The invite and assigned access are visible, proving administrative success | invite recorded | `media/state-06-invite-recorded.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At invite form or recipient selection, invalid, expired, denied, or missing required input leaves the flow short of invite recorded; evidence: media/state-02-invite-form.png, media/state-03-recipient-selection.png, and https://support.atlassian.com/user-management/docs/invite-a-user/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-recipient-selection.png through media/state-05-invitation-pending.png.
- **Recovery:** Return to the retained invite form or recipient selection requirement, correct or resend the blocking input, and resubmit; evidence: https://support.atlassian.com/user-management/docs/invite-a-user/.
- **Recovery:** Continue through the same terminal action until invite recorded is visible in media/state-06-invite-recorded.png and the motion at 80.940s.
- **Completion evidence:** invite recorded retained at media/state-06-invite-recorded.png and media/official-recording.mp4#t=80.940; source https://support.atlassian.com/user-management/docs/invite-a-user/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| user administration | [`media/state-01-user-administration.png`](media/state-01-user-administration.png) | media/official-recording.mp4#t=4.497 | 640×360 | `73b0314022f49f3ff8feb5ccdc8097477a52523de8f0898032eff75e6716b85a` |
| invite form | [`media/state-02-invite-form.png`](media/state-02-invite-form.png) | media/official-recording.mp4#t=19.785 | 640×360 | `981e3f4f63d44030003b4c8da639d3d74bd7a1d4cfdbf6a698eec1826783b4e6` |
| recipient selection | [`media/state-03-recipient-selection.png`](media/state-03-recipient-selection.png) | media/official-recording.mp4#t=35.074 | 640×360 | `37b95705fb99f15982210d30cc7a4567fff0ee486481a135faf9ccbd8f45a08a` |
| access assignment | [`media/state-04-access-assignment.png`](media/state-04-access-assignment.png) | media/official-recording.mp4#t=50.362 | 640×360 | `9643e425b855009c700c2174882d1832f22e0102096177f426df97b2c6290007` |
| invitation pending | [`media/state-05-invitation-pending.png`](media/state-05-invitation-pending.png) | media/official-recording.mp4#t=65.651 | 640×360 | `6fa0a36d5ce22d118ff14599e8f062ae23a7826dcd29b404c4fdc0c1ded9a508` |
| invite recorded | [`media/state-06-invite-recorded.png`](media/state-06-invite-recorded.png) | media/official-recording.mp4#t=80.940 | 640×360 | `331f19658fb6eba4b280a6d0ecbef48f8cd7b90feb142a8d4bad7bf93cc602a3` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Sign in and open user management | Atlassian shows managed users and invitation controls The retained user administration state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-user-administration.png @ 4.50s; https://support.atlassian.com/user-management/docs/invite-a-user/ |
| focus and selection | Choose Invite users | Atlassian opens the invitation form The recording advances to invite form and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-user-administration.png @ 4.50s; media/state-02-invite-form.png @ 19.79s; https://support.atlassian.com/user-management/docs/invite-a-user/ |
| navigation | Enter one or more email addresses | Atlassian resolves invite recipients and enables access choices The navigation result is visible as recipient selection. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-invite-form.png @ 19.79s; media/state-03-recipient-selection.png @ 35.07s; https://support.atlassian.com/user-management/docs/invite-a-user/ |
| confirmation | Select the products or roles the invitees may access | Atlassian displays the effective access assignment The official recording shows the confirmed access assignment state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-recipient-selection.png @ 35.07s; media/state-04-access-assignment.png @ 50.36s; https://support.atlassian.com/user-management/docs/invite-a-user/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-invite-form.png @ 19.79s; media/state-03-recipient-selection.png @ 35.07s; media/state-04-access-assignment.png @ 50.36s; https://support.atlassian.com/user-management/docs/invite-a-user/ |
| progress feedback | Review and send the invitation | Atlassian confirms submission and records pending invitations Progress is observable as the distinct invitation pending state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-access-assignment.png @ 50.36s; media/state-05-invitation-pending.png @ 65.65s; https://support.atlassian.com/user-management/docs/invite-a-user/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-recipient-selection.png @ 35.07s; media/state-04-access-assignment.png @ 50.36s; media/state-05-invitation-pending.png @ 65.65s; https://support.atlassian.com/user-management/docs/invite-a-user/ |
| recovery and completion | Open the pending-user record | The invite and assigned access are visible, proving administrative success The retained invite recorded state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-invitation-pending.png @ 65.65s; media/state-06-invite-recorded.png @ 80.94s; https://support.atlassian.com/user-management/docs/invite-a-user/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at user administration; the first advancing trigger is “Choose Invite users”.
- **Start/end:** Start is user administration at 4.50s; end is invite recorded at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in invite recorded; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-user-administration.png and media/state-02-invite-form.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-recipient-selection.png and media/state-04-access-assignment.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Atlassian Pty Ltd
- **Product page:** https://support.atlassian.com/user-management/docs/invite-a-user/
- **Original media URL:** https://www.youtube.com/watch?v=emidrJeUTaM
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 925193 bytes
- **SHA-256:** `74289536524fabc3a10cdef1ddd47f3a403e2d6b7d7f19b0e7d3c1d49a750738`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
