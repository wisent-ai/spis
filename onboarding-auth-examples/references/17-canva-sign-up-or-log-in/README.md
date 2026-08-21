# Canva — sign up or log in

**Evidence status:** `complete`  
**Product/source:** [https://www.canva.com/help/sign-up-log-in/](https://www.canva.com/help/sign-up-log-in/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Canva for Beginners: Opening Canva (1/10)](https://www.youtube.com/watch?v=V9LtRF6EbyY) — Canva

## Start-to-first-success journey

**Actor:** new Canva creator  
**Goal:** create an account and save the first design  
**Prerequisites:** email or supported identity provider; design goal

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Choose a sign-up method | Canva opens provider authorization or email entry | signup route | `media/state-01-signup-route.png` and motion at 4.50s |
| 2 | Complete provider or email verification | Canva creates the authenticated account | identity verified | `media/state-02-identity-verified.png` and motion at 19.79s |
| 3 | Answer role or intended-use prompts or skip optional setup | Canva tailors the home surface | personalization decided | `media/state-03-personalization-decided.png` and motion at 35.07s |
| 4 | Choose a template or blank design | Canva opens the editor | editor ready | `media/state-04-editor-ready.png` and motion at 50.36s |
| 5 | Add or change visible content | Canva updates the canvas and autosave state | design edited | `media/state-05-design-edited.png` and motion at 65.65s |
| 6 | Return to Home or share the design | The saved design remains in the account, proving first creation success | first design | `media/state-06-first-design.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At identity verified or personalization decided, invalid, expired, denied, or missing required input leaves the flow short of first design; evidence: media/state-02-identity-verified.png, media/state-03-personalization-decided.png, and https://www.canva.com/help/sign-up-log-in/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-personalization-decided.png through media/state-05-design-edited.png.
- **Recovery:** Return to the retained identity verified or personalization decided requirement, correct or resend the blocking input, and resubmit; evidence: https://www.canva.com/help/sign-up-log-in/.
- **Recovery:** Continue through the same terminal action until first design is visible in media/state-06-first-design.png and the motion at 80.940s.
- **Completion evidence:** first design retained at media/state-06-first-design.png and media/official-recording.mp4#t=80.940; source https://www.canva.com/help/sign-up-log-in/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| signup route | [`media/state-01-signup-route.png`](media/state-01-signup-route.png) | media/official-recording.mp4#t=4.497 | 640×360 | `bea32ba23b1e00573a78bc036bd47f3697deaee485f0f9968fc46bb08698cf9b` |
| identity verified | [`media/state-02-identity-verified.png`](media/state-02-identity-verified.png) | media/official-recording.mp4#t=19.785 | 640×360 | `90e24d4beda5f5fc40265b5cc5833194e861e9e9e7f32a4447a2753f2aaebb16` |
| personalization decided | [`media/state-03-personalization-decided.png`](media/state-03-personalization-decided.png) | media/official-recording.mp4#t=35.074 | 640×360 | `117b8260ebf666b9ed16f61d0ef32a8055b0188538feaeaca33926edc455eca2` |
| editor ready | [`media/state-04-editor-ready.png`](media/state-04-editor-ready.png) | media/official-recording.mp4#t=50.362 | 640×360 | `1670ba0f33928ac8951efcf2550c075f44f187bd91e5e8abf52f9cab7a2686b1` |
| design edited | [`media/state-05-design-edited.png`](media/state-05-design-edited.png) | media/official-recording.mp4#t=65.651 | 640×360 | `ae14477e155f1eb63c0a1fb3fecb983c6546d2427400def07abbeb76962863af` |
| first design | [`media/state-06-first-design.png`](media/state-06-first-design.png) | media/official-recording.mp4#t=80.940 | 640×360 | `ed1f9bca2269c380a4052694744a985b9ddf26a4859ed23dbce8e316e60a4989` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Choose a sign-up method | Canva opens provider authorization or email entry The retained signup route state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-signup-route.png @ 4.50s; https://www.canva.com/help/sign-up-log-in/ |
| focus and selection | Complete provider or email verification | Canva creates the authenticated account The recording advances to identity verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-signup-route.png @ 4.50s; media/state-02-identity-verified.png @ 19.79s; https://www.canva.com/help/sign-up-log-in/ |
| navigation | Answer role or intended-use prompts or skip optional setup | Canva tailors the home surface The navigation result is visible as personalization decided. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-verified.png @ 19.79s; media/state-03-personalization-decided.png @ 35.07s; https://www.canva.com/help/sign-up-log-in/ |
| confirmation | Choose a template or blank design | Canva opens the editor The official recording shows the confirmed editor ready state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-personalization-decided.png @ 35.07s; media/state-04-editor-ready.png @ 50.36s; https://www.canva.com/help/sign-up-log-in/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-verified.png @ 19.79s; media/state-03-personalization-decided.png @ 35.07s; media/state-04-editor-ready.png @ 50.36s; https://www.canva.com/help/sign-up-log-in/ |
| progress feedback | Add or change visible content | Canva updates the canvas and autosave state Progress is observable as the distinct design edited state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-editor-ready.png @ 50.36s; media/state-05-design-edited.png @ 65.65s; https://www.canva.com/help/sign-up-log-in/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-personalization-decided.png @ 35.07s; media/state-04-editor-ready.png @ 50.36s; media/state-05-design-edited.png @ 65.65s; https://www.canva.com/help/sign-up-log-in/ |
| recovery and completion | Return to Home or share the design | The saved design remains in the account, proving first creation success The retained first design state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-design-edited.png @ 65.65s; media/state-06-first-design.png @ 80.94s; https://www.canva.com/help/sign-up-log-in/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at signup route; the first advancing trigger is “Complete provider or email verification”.
- **Start/end:** Start is signup route at 4.50s; end is first design at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first design; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-signup-route.png and media/state-02-identity-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-personalization-decided.png and media/state-04-editor-ready.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Canva Pty Ltd
- **Product page:** https://www.canva.com/help/sign-up-log-in/
- **Original media URL:** https://www.youtube.com/watch?v=V9LtRF6EbyY
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 969837 bytes
- **SHA-256:** `5e02223fa62a7061d8a3eacd971f982a41509817c9b68746af5c24adaf64655d`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
