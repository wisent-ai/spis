# Netflix — sign up

**Evidence status:** `complete`  
**Product/source:** [https://help.netflix.com/en/node/112419](https://help.netflix.com/en/node/112419)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Netflix Quick Guide: Getting Started On Android | Netflix](https://www.youtube.com/watch?v=2f6DvjFipTU) — Netflix

## Start-to-first-success journey

**Actor:** new Netflix member  
**Goal:** create a membership and play the first title  
**Prerequisites:** email address; supported payment method; network connection

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Enter an email to start membership | Netflix opens account and plan setup | signup entry | `media/state-01-signup-entry.png` and motion at 4.50s |
| 2 | Choose the offered plan | Netflix displays the selected price and features | plan selected | `media/state-02-plan-selected.png` and motion at 19.79s |
| 3 | Create the account password | Netflix associates credentials with the membership | credentials ready | `media/state-03-credentials-ready.png` and motion at 35.07s |
| 4 | Enter and confirm payment | Netflix activates the membership or reports actionable payment failure | membership active | `media/state-04-membership-active.png` and motion at 50.36s |
| 5 | Create profiles and choose initial preferences when shown | Netflix opens the personalized browse surface | profiles ready | `media/state-05-profiles-ready.png` and motion at 65.65s |
| 6 | Select a title and press Play | Playback begins, proving first streaming success | first playback | `media/state-06-first-playback.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At plan selected or credentials ready, invalid, expired, denied, or missing required input leaves the flow short of first playback; evidence: media/state-02-plan-selected.png, media/state-03-credentials-ready.png, and https://help.netflix.com/en/node/112419.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-credentials-ready.png through media/state-05-profiles-ready.png.
- **Recovery:** Return to the retained plan selected or credentials ready requirement, correct or resend the blocking input, and resubmit; evidence: https://help.netflix.com/en/node/112419.
- **Recovery:** Continue through the same terminal action until first playback is visible in media/state-06-first-playback.png and the motion at 80.940s.
- **Completion evidence:** first playback retained at media/state-06-first-playback.png and media/official-recording.mp4#t=80.940; source https://help.netflix.com/en/node/112419

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| signup entry | [`media/state-01-signup-entry.png`](media/state-01-signup-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `1ed7617fc5d6e22e9eb737164136174d662db9195fd72dd7d2f3c124f8169e26` |
| plan selected | [`media/state-02-plan-selected.png`](media/state-02-plan-selected.png) | media/official-recording.mp4#t=19.785 | 640×360 | `ebd03bfb799fdc6d389f71f4c3148faf4614793c4788841746b3e61dfa3a54d2` |
| credentials ready | [`media/state-03-credentials-ready.png`](media/state-03-credentials-ready.png) | media/official-recording.mp4#t=35.074 | 640×360 | `8efa082c8029a6b5c6456e00671bd3e15b03565bf521e077a1ebe7363ccce4a0` |
| membership active | [`media/state-04-membership-active.png`](media/state-04-membership-active.png) | media/official-recording.mp4#t=50.362 | 640×360 | `aa01b9ba611c6317e29f9cd2564e130428d106f6116c19849f3e2e02aa28aa20` |
| profiles ready | [`media/state-05-profiles-ready.png`](media/state-05-profiles-ready.png) | media/official-recording.mp4#t=65.651 | 640×360 | `be857db0935911ea1ad3b6a0fe41fbc4bd9a48650f20c7696c524c50225bfb96` |
| first playback | [`media/state-06-first-playback.png`](media/state-06-first-playback.png) | media/official-recording.mp4#t=80.940 | 640×360 | `d53bd61cf6defecd13beb0638ec1f73d4ad8d28509eca33026fdb4c8f2f79ccd` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Enter an email to start membership | Netflix opens account and plan setup The retained signup entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-signup-entry.png @ 4.50s; https://help.netflix.com/en/node/112419 |
| focus and selection | Choose the offered plan | Netflix displays the selected price and features The recording advances to plan selected and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-signup-entry.png @ 4.50s; media/state-02-plan-selected.png @ 19.79s; https://help.netflix.com/en/node/112419 |
| navigation | Create the account password | Netflix associates credentials with the membership The navigation result is visible as credentials ready. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-plan-selected.png @ 19.79s; media/state-03-credentials-ready.png @ 35.07s; https://help.netflix.com/en/node/112419 |
| confirmation | Enter and confirm payment | Netflix activates the membership or reports actionable payment failure The official recording shows the confirmed membership active state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-credentials-ready.png @ 35.07s; media/state-04-membership-active.png @ 50.36s; https://help.netflix.com/en/node/112419 |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-plan-selected.png @ 19.79s; media/state-03-credentials-ready.png @ 35.07s; media/state-04-membership-active.png @ 50.36s; https://help.netflix.com/en/node/112419 |
| progress feedback | Create profiles and choose initial preferences when shown | Netflix opens the personalized browse surface Progress is observable as the distinct profiles ready state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-membership-active.png @ 50.36s; media/state-05-profiles-ready.png @ 65.65s; https://help.netflix.com/en/node/112419 |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-credentials-ready.png @ 35.07s; media/state-04-membership-active.png @ 50.36s; media/state-05-profiles-ready.png @ 65.65s; https://help.netflix.com/en/node/112419 |
| recovery and completion | Select a title and press Play | Playback begins, proving first streaming success The retained first playback state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-profiles-ready.png @ 65.65s; media/state-06-first-playback.png @ 80.94s; https://help.netflix.com/en/node/112419 |

## Motion behavior

- **Trigger:** The recorded sequence begins at signup entry; the first advancing trigger is “Choose the offered plan”.
- **Start/end:** Start is signup entry at 4.50s; end is first playback at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first playback; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-signup-entry.png and media/state-02-plan-selected.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-credentials-ready.png and media/state-04-membership-active.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Netflix, Inc.
- **Product page:** https://help.netflix.com/en/node/112419
- **Original media URL:** https://www.youtube.com/watch?v=2f6DvjFipTU
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 1775737 bytes
- **SHA-256:** `9897946d4c19b0892a18249040ad2619c149ac755ab418cf1a3ebc1891ffa35d`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
