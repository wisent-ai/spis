# Sentry — guided onboarding

**Evidence status:** `complete`  
**Product/source:** [https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/](https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Sentry in Six Minutes](https://www.youtube.com/watch?v=4djseRVSan8) — Sentry

## Start-to-first-success journey

**Actor:** new Sentry project owner  
**Goal:** install an SDK and capture the first error  
**Prerequisites:** Sentry account and organization; supported application project

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and choose Create project | Sentry opens platform selection | project entry | `media/state-01-project-entry.png` and motion at 4.50s |
| 2 | Select the application framework and team | Sentry generates framework-specific onboarding | onboarding generated | `media/state-02-onboarding-generated.png` and motion at 19.79s |
| 3 | Install the shown SDK package | The application gains the required dependency | SDK installed | `media/state-03-sdk-installed.png` and motion at 35.07s |
| 4 | Apply the generated initialization snippet and DSN | Sentry displays the verification step | SDK configured | `media/state-04-sdk-configured.png` and motion at 50.36s |
| 5 | Trigger the provided test error | The application sends an event and Sentry polls for it | event pending | `media/state-05-event-pending.png` and motion at 65.65s |
| 6 | Open the received issue | Sentry renders the event details, proving first monitoring success | first issue | `media/state-06-first-issue.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At onboarding generated or SDK installed, invalid, expired, denied, or missing required input leaves the flow short of first issue; evidence: media/state-02-onboarding-generated.png, media/state-03-sdk-installed.png, and https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-sdk-installed.png through media/state-05-event-pending.png.
- **Recovery:** Return to the retained onboarding generated or SDK installed requirement, correct or resend the blocking input, and resubmit; evidence: https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/.
- **Recovery:** Continue through the same terminal action until first issue is visible in media/state-06-first-issue.png and the motion at 80.940s.
- **Completion evidence:** first issue retained at media/state-06-first-issue.png and media/official-recording.mp4#t=80.940; source https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| project entry | [`media/state-01-project-entry.png`](media/state-01-project-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `56f22d00a54389684d7ad0dffc926b2f08e6999127ab57f2bb3632e5b174d995` |
| onboarding generated | [`media/state-02-onboarding-generated.png`](media/state-02-onboarding-generated.png) | media/official-recording.mp4#t=19.785 | 640×360 | `ffc876e5c1450e9a2434398ef5d91a3f1e15973da61cebd90ab1f8fecef45f7d` |
| SDK installed | [`media/state-03-sdk-installed.png`](media/state-03-sdk-installed.png) | media/official-recording.mp4#t=35.074 | 640×360 | `ecb1c7067008ddc89cc98a4537322cdd623aa1bd75cfaaca59f37694f4fbf6f8` |
| SDK configured | [`media/state-04-sdk-configured.png`](media/state-04-sdk-configured.png) | media/official-recording.mp4#t=50.362 | 640×360 | `fba67a0e2d321bec7fb961d3ca5638d81eaa371856d133a01fd38d1b56cdc07e` |
| event pending | [`media/state-05-event-pending.png`](media/state-05-event-pending.png) | media/official-recording.mp4#t=65.651 | 640×360 | `2ba9bed51363a8dbcdde442e0e8116ae3fe068be1c43f82e7cee8135e6fb190f` |
| first issue | [`media/state-06-first-issue.png`](media/state-06-first-issue.png) | media/official-recording.mp4#t=80.940 | 640×360 | `b4ddf703d2e7b3030efc92b1256451904aab6ed3ca6146fb9d5eb4815afe542f` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and choose Create project | Sentry opens platform selection The retained project entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-project-entry.png @ 4.50s; https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/ |
| focus and selection | Select the application framework and team | Sentry generates framework-specific onboarding The recording advances to onboarding generated and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-project-entry.png @ 4.50s; media/state-02-onboarding-generated.png @ 19.79s; https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/ |
| navigation | Install the shown SDK package | The application gains the required dependency The navigation result is visible as SDK installed. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-onboarding-generated.png @ 19.79s; media/state-03-sdk-installed.png @ 35.07s; https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/ |
| confirmation | Apply the generated initialization snippet and DSN | Sentry displays the verification step The official recording shows the confirmed SDK configured state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-sdk-installed.png @ 35.07s; media/state-04-sdk-configured.png @ 50.36s; https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-onboarding-generated.png @ 19.79s; media/state-03-sdk-installed.png @ 35.07s; media/state-04-sdk-configured.png @ 50.36s; https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/ |
| progress feedback | Trigger the provided test error | The application sends an event and Sentry polls for it Progress is observable as the distinct event pending state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-sdk-configured.png @ 50.36s; media/state-05-event-pending.png @ 65.65s; https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-sdk-installed.png @ 35.07s; media/state-04-sdk-configured.png @ 50.36s; media/state-05-event-pending.png @ 65.65s; https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/ |
| recovery and completion | Open the received issue | Sentry renders the event details, proving first monitoring success The retained first issue state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-event-pending.png @ 65.65s; media/state-06-first-issue.png @ 80.94s; https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at project entry; the first advancing trigger is “Select the application framework and team”.
- **Start/end:** Start is project entry at 4.50s; end is first issue at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first issue; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-project-entry.png and media/state-02-onboarding-generated.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-sdk-installed.png and media/state-04-sdk-configured.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Functional Software, Inc. (Sentry)
- **Product page:** https://docs.sentry.io/product/sentry-basics/integrate-frontend/onboarding-guides/
- **Original media URL:** https://www.youtube.com/watch?v=4djseRVSan8
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 568350 bytes
- **SHA-256:** `9e824023d4345a2e36f8a2eb5b3de19db2c3650708bbbac86d9e08615d56d55b`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
