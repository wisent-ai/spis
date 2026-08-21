# Microsoft 365 — setup wizard

**Evidence status:** `complete`  
**Product/source:** [https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide](https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to set up Microsoft 365 Business Premium](https://www.youtube.com/watch?v=um6I7IosARc) — Microsoft 365

## Start-to-first-success journey

**Actor:** Microsoft 365 tenant administrator  
**Goal:** complete tenant setup and provision the first user  
**Prerequisites:** Microsoft 365 subscription; domain and DNS access

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Sign in as tenant administrator and open setup | Microsoft 365 presents the guided setup checklist | setup entry | `media/state-01-setup-entry.png` and motion at 4.50s |
| 2 | Add the organization domain | The admin center provides domain verification records | domain pending | `media/state-02-domain-pending.png` and motion at 19.79s |
| 3 | Publish the requested DNS record | Microsoft verifies ownership and advances service configuration | domain verified | `media/state-03-domain-verified.png` and motion at 35.07s |
| 4 | Create users and assign licenses | The admin center shows active licensed identities | users provisioned | `media/state-04-users-provisioned.png` and motion at 50.36s |
| 5 | Apply recommended app, mail, and security settings | Microsoft marks setup tasks complete | services configured | `media/state-05-services-configured.png` and motion at 65.65s |
| 6 | Sign in as the provisioned user and open a service | The service loads under the managed identity, proving first tenant success | first user success | `media/state-06-first-user-success.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At domain pending or domain verified, invalid, expired, denied, or missing required input leaves the flow short of first user success; evidence: media/state-02-domain-pending.png, media/state-03-domain-verified.png, and https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-domain-verified.png through media/state-05-services-configured.png.
- **Recovery:** Return to the retained domain pending or domain verified requirement, correct or resend the blocking input, and resubmit; evidence: https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide.
- **Recovery:** Continue through the same terminal action until first user success is visible in media/state-06-first-user-success.png and the motion at 80.940s.
- **Completion evidence:** first user success retained at media/state-06-first-user-success.png and media/official-recording.mp4#t=80.940; source https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| setup entry | [`media/state-01-setup-entry.png`](media/state-01-setup-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `16493ae91c19b251f33af85e874b02a195d11d2e6770fd5d5d25abaed6bcf084` |
| domain pending | [`media/state-02-domain-pending.png`](media/state-02-domain-pending.png) | media/official-recording.mp4#t=19.785 | 640×360 | `35762545aece86b63e30b9d9cb27ac112da9ebd1c9898c12e0eac85126c4987b` |
| domain verified | [`media/state-03-domain-verified.png`](media/state-03-domain-verified.png) | media/official-recording.mp4#t=35.074 | 640×360 | `fe58b258773ec4d6cd88c9c1503a87ecc2f635876c4586cee7a43a120b8de85c` |
| users provisioned | [`media/state-04-users-provisioned.png`](media/state-04-users-provisioned.png) | media/official-recording.mp4#t=50.362 | 640×360 | `8b0aa7388740c2957ce000c71b60a7751a5eb60b8d2fdfa863780925de536ad4` |
| services configured | [`media/state-05-services-configured.png`](media/state-05-services-configured.png) | media/official-recording.mp4#t=65.651 | 640×360 | `18bf7a84bc14a6fd0de663d751f662cae45095f779873db18abf6d5f7d7b5521` |
| first user success | [`media/state-06-first-user-success.png`](media/state-06-first-user-success.png) | media/official-recording.mp4#t=80.940 | 640×360 | `ed6e05172fac97a7431946423b6db45ee5735a910f5c5c0f155443f136a1bdf5` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Sign in as tenant administrator and open setup | Microsoft 365 presents the guided setup checklist The retained setup entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-setup-entry.png @ 4.50s; https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide |
| focus and selection | Add the organization domain | The admin center provides domain verification records The recording advances to domain pending and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-setup-entry.png @ 4.50s; media/state-02-domain-pending.png @ 19.79s; https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide |
| navigation | Publish the requested DNS record | Microsoft verifies ownership and advances service configuration The navigation result is visible as domain verified. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-domain-pending.png @ 19.79s; media/state-03-domain-verified.png @ 35.07s; https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide |
| confirmation | Create users and assign licenses | The admin center shows active licensed identities The official recording shows the confirmed users provisioned state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-domain-verified.png @ 35.07s; media/state-04-users-provisioned.png @ 50.36s; https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-domain-pending.png @ 19.79s; media/state-03-domain-verified.png @ 35.07s; media/state-04-users-provisioned.png @ 50.36s; https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide |
| progress feedback | Apply recommended app, mail, and security settings | Microsoft marks setup tasks complete Progress is observable as the distinct services configured state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-users-provisioned.png @ 50.36s; media/state-05-services-configured.png @ 65.65s; https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-domain-verified.png @ 35.07s; media/state-04-users-provisioned.png @ 50.36s; media/state-05-services-configured.png @ 65.65s; https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide |
| recovery and completion | Sign in as the provisioned user and open a service | The service loads under the managed identity, proving first tenant success The retained first user success state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-services-configured.png @ 65.65s; media/state-06-first-user-success.png @ 80.94s; https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide |

## Motion behavior

- **Trigger:** The recorded sequence begins at setup entry; the first advancing trigger is “Add the organization domain”.
- **Start/end:** Start is setup entry at 4.50s; end is first user success at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first user success; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-setup-entry.png and media/state-02-domain-pending.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-domain-verified.png and media/state-04-users-provisioned.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Microsoft Corporation
- **Product page:** https://learn.microsoft.com/en-us/microsoft-365/admin/setup/setup-business-basic?view=o365-worldwide
- **Original media URL:** https://www.youtube.com/watch?v=um6I7IosARc
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 331697 bytes
- **SHA-256:** `1b18aa89303c62b32ea0e10455d03bef12b5e45b24d79d7a699e20b070f68ba9`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
