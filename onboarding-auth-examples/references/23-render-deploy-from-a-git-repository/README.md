# Render — deploy from a Git repository

**Evidence status:** `complete`  
**Product/source:** [https://render.com/docs/deploys](https://render.com/docs/deploys)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Deploy your AI code in minutes with Render](https://www.youtube.com/watch?v=xDu7I4lXvrw) — Render

## Start-to-first-success journey

**Actor:** new Render service owner  
**Goal:** connect a repository and make the first service live  
**Prerequisites:** Render account; Git repository; build and start commands

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and choose New service | Render shows supported service types | service entry | `media/state-01-service-entry.png` and motion at 4.50s |
| 2 | Connect the Git provider | Render requests and receives repository authorization | provider connected | `media/state-02-provider-connected.png` and motion at 19.79s |
| 3 | Select the repository | Render opens service configuration | repository selected | `media/state-03-repository-selected.png` and motion at 35.07s |
| 4 | Set runtime, branch, build, start, and environment values | Render validates required configuration | service configured | `media/state-04-service-configured.png` and motion at 50.36s |
| 5 | Create the service | Render clones, builds, and streams deployment logs | deployment running | `media/state-05-deployment-running.png` and motion at 65.65s |
| 6 | Open the assigned service URL | The application responds, proving first deployment success | service live | `media/state-06-service-live.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At provider connected or repository selected, invalid, expired, denied, or missing required input leaves the flow short of service live; evidence: media/state-02-provider-connected.png, media/state-03-repository-selected.png, and https://render.com/docs/deploys.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-repository-selected.png through media/state-05-deployment-running.png.
- **Recovery:** Return to the retained provider connected or repository selected requirement, correct or resend the blocking input, and resubmit; evidence: https://render.com/docs/deploys.
- **Recovery:** Continue through the same terminal action until service live is visible in media/state-06-service-live.png and the motion at 80.940s.
- **Completion evidence:** service live retained at media/state-06-service-live.png and media/official-recording.mp4#t=80.940; source https://render.com/docs/deploys

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| service entry | [`media/state-01-service-entry.png`](media/state-01-service-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `6a1adf707f14e008fb1597607c9c192325c0685caea85cb4292dadd5beb93f12` |
| provider connected | [`media/state-02-provider-connected.png`](media/state-02-provider-connected.png) | media/official-recording.mp4#t=19.785 | 640×360 | `402368e9eadc9ac6281d04f7eefd312b54c591cc3651a241c26deab92faf9f2b` |
| repository selected | [`media/state-03-repository-selected.png`](media/state-03-repository-selected.png) | media/official-recording.mp4#t=35.074 | 640×360 | `a0c46d3fb43411c1443b6b5e72d84b441bff36b7a8411b6cb2f55f9665b08866` |
| service configured | [`media/state-04-service-configured.png`](media/state-04-service-configured.png) | media/official-recording.mp4#t=50.362 | 640×360 | `1b65d9fbddd05ae061399bed9373106de32b90051a35d305b5cb9b572ba3be91` |
| deployment running | [`media/state-05-deployment-running.png`](media/state-05-deployment-running.png) | media/official-recording.mp4#t=65.651 | 640×360 | `16834b1620fcfba26a29481b0fac52685a8530b885106b9940dc48bae1070857` |
| service live | [`media/state-06-service-live.png`](media/state-06-service-live.png) | media/official-recording.mp4#t=80.940 | 640×360 | `eba7222c1abb67e094043bd3f1de6d7581087ab44a474b88830c9a1bfdfff6ac` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and choose New service | Render shows supported service types The retained service entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-service-entry.png @ 4.50s; https://render.com/docs/deploys |
| focus and selection | Connect the Git provider | Render requests and receives repository authorization The recording advances to provider connected and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-service-entry.png @ 4.50s; media/state-02-provider-connected.png @ 19.79s; https://render.com/docs/deploys |
| navigation | Select the repository | Render opens service configuration The navigation result is visible as repository selected. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-provider-connected.png @ 19.79s; media/state-03-repository-selected.png @ 35.07s; https://render.com/docs/deploys |
| confirmation | Set runtime, branch, build, start, and environment values | Render validates required configuration The official recording shows the confirmed service configured state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-repository-selected.png @ 35.07s; media/state-04-service-configured.png @ 50.36s; https://render.com/docs/deploys |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-provider-connected.png @ 19.79s; media/state-03-repository-selected.png @ 35.07s; media/state-04-service-configured.png @ 50.36s; https://render.com/docs/deploys |
| progress feedback | Create the service | Render clones, builds, and streams deployment logs Progress is observable as the distinct deployment running state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-service-configured.png @ 50.36s; media/state-05-deployment-running.png @ 65.65s; https://render.com/docs/deploys |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-repository-selected.png @ 35.07s; media/state-04-service-configured.png @ 50.36s; media/state-05-deployment-running.png @ 65.65s; https://render.com/docs/deploys |
| recovery and completion | Open the assigned service URL | The application responds, proving first deployment success The retained service live state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-deployment-running.png @ 65.65s; media/state-06-service-live.png @ 80.94s; https://render.com/docs/deploys |

## Motion behavior

- **Trigger:** The recorded sequence begins at service entry; the first advancing trigger is “Connect the Git provider”.
- **Start/end:** Start is service entry at 4.50s; end is service live at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in service live; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-service-entry.png and media/state-02-provider-connected.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-repository-selected.png and media/state-04-service-configured.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Render Services, Inc.
- **Product page:** https://render.com/docs/deploys
- **Original media URL:** https://www.youtube.com/watch?v=xDu7I4lXvrw
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 311424 bytes
- **SHA-256:** `20e56d9660b4dd19f15f05299e4a304921eccf37edc6f360b19d84cc786fe264`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
