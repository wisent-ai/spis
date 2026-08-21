# Vercel — import a Git repository

**Evidence status:** `complete`  
**Product/source:** [https://vercel.com/docs/getting-started-with-vercel/import](https://vercel.com/docs/getting-started-with-vercel/import)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Deploying Next.js to Vercel](https://www.youtube.com/watch?v=AiiGjB2AxqA) — Vercel

## Start-to-first-success journey

**Actor:** new Vercel project owner  
**Goal:** import a repository and reach a live deployment  
**Prerequisites:** Vercel account; Git-provider repository and authorization

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Sign in with the Git provider | Vercel establishes account and provider context | provider signed in | `media/state-01-provider-signed-in.png` and motion at 4.50s |
| 2 | Authorize repository access | Vercel returns with visible repositories | repository access | `media/state-02-repository-access.png` and motion at 19.79s |
| 3 | Choose Import beside a repository | Vercel opens project configuration | import selected | `media/state-03-import-selected.png` and motion at 35.07s |
| 4 | Review framework, root directory, build, and environment settings | Vercel validates deployment configuration | project configured | `media/state-04-project-configured.png` and motion at 50.36s |
| 5 | Choose Deploy | Vercel streams build and deployment progress | deployment running | `media/state-05-deployment-running.png` and motion at 65.65s |
| 6 | Open the generated production URL | The deployed application loads, proving first-success delivery | deployment live | `media/state-06-deployment-live.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At repository access or import selected, invalid, expired, denied, or missing required input leaves the flow short of deployment live; evidence: media/state-02-repository-access.png, media/state-03-import-selected.png, and https://vercel.com/docs/getting-started-with-vercel/import.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-import-selected.png through media/state-05-deployment-running.png.
- **Recovery:** Return to the retained repository access or import selected requirement, correct or resend the blocking input, and resubmit; evidence: https://vercel.com/docs/getting-started-with-vercel/import.
- **Recovery:** Continue through the same terminal action until deployment live is visible in media/state-06-deployment-live.png and the motion at 80.940s.
- **Completion evidence:** deployment live retained at media/state-06-deployment-live.png and media/official-recording.mp4#t=80.940; source https://vercel.com/docs/getting-started-with-vercel/import

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| provider signed in | [`media/state-01-provider-signed-in.png`](media/state-01-provider-signed-in.png) | media/official-recording.mp4#t=4.497 | 640×360 | `b22040695d77faa7ac045528ad5e0603c48b083c8ee7c354d5b84770b58c29f0` |
| repository access | [`media/state-02-repository-access.png`](media/state-02-repository-access.png) | media/official-recording.mp4#t=19.785 | 640×360 | `c30e98aea40aaaafc645728e9b837ec3100c9a79e8a4fddb065342f183a0e4d0` |
| import selected | [`media/state-03-import-selected.png`](media/state-03-import-selected.png) | media/official-recording.mp4#t=35.074 | 640×360 | `298ff18e596d8d01f1b0e8e1981edd3c6772930436c3373baa439bb667c00d23` |
| project configured | [`media/state-04-project-configured.png`](media/state-04-project-configured.png) | media/official-recording.mp4#t=50.362 | 640×360 | `797bed96d87799e2d022e85e4c6f7b7ae018447cc84c5b71eabbb0c77c5c62fa` |
| deployment running | [`media/state-05-deployment-running.png`](media/state-05-deployment-running.png) | media/official-recording.mp4#t=65.651 | 640×360 | `0f46e4fc97d9c838571d49366ad7356d6bb7b8d02c8794ae5c9c273f2f95ac57` |
| deployment live | [`media/state-06-deployment-live.png`](media/state-06-deployment-live.png) | media/official-recording.mp4#t=80.940 | 640×360 | `c198c33b64edb614980adfac0196d6945146fcd1e8d28c1fd6a6892aff2fe16b` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Sign in with the Git provider | Vercel establishes account and provider context The retained provider signed in state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-provider-signed-in.png @ 4.50s; https://vercel.com/docs/getting-started-with-vercel/import |
| focus and selection | Authorize repository access | Vercel returns with visible repositories The recording advances to repository access and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-provider-signed-in.png @ 4.50s; media/state-02-repository-access.png @ 19.79s; https://vercel.com/docs/getting-started-with-vercel/import |
| navigation | Choose Import beside a repository | Vercel opens project configuration The navigation result is visible as import selected. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-repository-access.png @ 19.79s; media/state-03-import-selected.png @ 35.07s; https://vercel.com/docs/getting-started-with-vercel/import |
| confirmation | Review framework, root directory, build, and environment settings | Vercel validates deployment configuration The official recording shows the confirmed project configured state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-import-selected.png @ 35.07s; media/state-04-project-configured.png @ 50.36s; https://vercel.com/docs/getting-started-with-vercel/import |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-repository-access.png @ 19.79s; media/state-03-import-selected.png @ 35.07s; media/state-04-project-configured.png @ 50.36s; https://vercel.com/docs/getting-started-with-vercel/import |
| progress feedback | Choose Deploy | Vercel streams build and deployment progress Progress is observable as the distinct deployment running state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-project-configured.png @ 50.36s; media/state-05-deployment-running.png @ 65.65s; https://vercel.com/docs/getting-started-with-vercel/import |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-import-selected.png @ 35.07s; media/state-04-project-configured.png @ 50.36s; media/state-05-deployment-running.png @ 65.65s; https://vercel.com/docs/getting-started-with-vercel/import |
| recovery and completion | Open the generated production URL | The deployed application loads, proving first-success delivery The retained deployment live state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-deployment-running.png @ 65.65s; media/state-06-deployment-live.png @ 80.94s; https://vercel.com/docs/getting-started-with-vercel/import |

## Motion behavior

- **Trigger:** The recorded sequence begins at provider signed in; the first advancing trigger is “Authorize repository access”.
- **Start/end:** Start is provider signed in at 4.50s; end is deployment live at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in deployment live; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-provider-signed-in.png and media/state-02-repository-access.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-import-selected.png and media/state-04-project-configured.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Vercel Inc.
- **Product page:** https://vercel.com/docs/getting-started-with-vercel/import
- **Original media URL:** https://www.youtube.com/watch?v=AiiGjB2AxqA
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 360444 bytes
- **SHA-256:** `730e81e3409b3af16c1a0d233ad8570398b8f452943a99f9a89f320f8812bc97`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
