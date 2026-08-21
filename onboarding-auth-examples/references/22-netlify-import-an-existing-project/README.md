# Netlify — import an existing project

**Evidence status:** `complete`  
**Product/source:** [https://docs.netlify.com/welcome/add-new-site/](https://docs.netlify.com/welcome/add-new-site/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Netlify Tutorial –Deploying from Git](https://www.youtube.com/watch?v=4h8B080Mv4U) — Netlify

## Start-to-first-success journey

**Actor:** new Netlify site owner  
**Goal:** import a repository and publish the first site  
**Prerequisites:** Netlify account; Git-provider repository

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and choose Add new site | Netlify presents import and manual deployment routes | site entry | `media/state-01-site-entry.png` and motion at 4.50s |
| 2 | Choose Import an existing project and Git provider | Netlify starts provider authorization | provider authorization | `media/state-02-provider-authorization.png` and motion at 19.79s |
| 3 | Grant access and select a repository | Netlify opens build configuration for the repository | repository selected | `media/state-03-repository-selected.png` and motion at 35.07s |
| 4 | Review branch, build command, and publish directory | Netlify validates deployment settings | site configured | `media/state-04-site-configured.png` and motion at 50.36s |
| 5 | Choose Deploy site | Netlify shows build and deploy logs | deployment running | `media/state-05-deployment-running.png` and motion at 65.65s |
| 6 | Open the generated site URL | The published site loads, proving first deployment success | site live | `media/state-06-site-live.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At provider authorization or repository selected, invalid, expired, denied, or missing required input leaves the flow short of site live; evidence: media/state-02-provider-authorization.png, media/state-03-repository-selected.png, and https://docs.netlify.com/welcome/add-new-site/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-repository-selected.png through media/state-05-deployment-running.png.
- **Recovery:** Return to the retained provider authorization or repository selected requirement, correct or resend the blocking input, and resubmit; evidence: https://docs.netlify.com/welcome/add-new-site/.
- **Recovery:** Continue through the same terminal action until site live is visible in media/state-06-site-live.png and the motion at 80.940s.
- **Completion evidence:** site live retained at media/state-06-site-live.png and media/official-recording.mp4#t=80.940; source https://docs.netlify.com/welcome/add-new-site/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| site entry | [`media/state-01-site-entry.png`](media/state-01-site-entry.png) | media/official-recording.mp4#t=4.497 | 640×360 | `6e31a08cf8d45b149223ac0c85c8e9f2c3f2f68e256ebe2acd5536b53593b606` |
| provider authorization | [`media/state-02-provider-authorization.png`](media/state-02-provider-authorization.png) | media/official-recording.mp4#t=19.785 | 640×360 | `a9c2ae3d6c302a5c7696cfcc82cbe5eeb3c05b83b09b26bb8a601b56e5d156a0` |
| repository selected | [`media/state-03-repository-selected.png`](media/state-03-repository-selected.png) | media/official-recording.mp4#t=35.074 | 640×360 | `eab054fe3efd714f6aee1512bf8d91cc0167e38932ba7f37dd3382535de691d5` |
| site configured | [`media/state-04-site-configured.png`](media/state-04-site-configured.png) | media/official-recording.mp4#t=50.362 | 640×360 | `704c59973641b53ad408aabcbce28e91afbd4ac163fee24bb395f42d8096a9df` |
| deployment running | [`media/state-05-deployment-running.png`](media/state-05-deployment-running.png) | media/official-recording.mp4#t=65.651 | 640×360 | `edf4c4a79ae9f4d65b64b1eb099c829f49283f7203670b89266d3eb61e88b135` |
| site live | [`media/state-06-site-live.png`](media/state-06-site-live.png) | media/official-recording.mp4#t=80.940 | 640×360 | `4313783718229535f1de9a4233c561e3727de0a954d62e2b4f2752bb44bd9bb2` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and choose Add new site | Netlify presents import and manual deployment routes The retained site entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-site-entry.png @ 4.50s; https://docs.netlify.com/welcome/add-new-site/ |
| focus and selection | Choose Import an existing project and Git provider | Netlify starts provider authorization The recording advances to provider authorization and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-site-entry.png @ 4.50s; media/state-02-provider-authorization.png @ 19.79s; https://docs.netlify.com/welcome/add-new-site/ |
| navigation | Grant access and select a repository | Netlify opens build configuration for the repository The navigation result is visible as repository selected. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-provider-authorization.png @ 19.79s; media/state-03-repository-selected.png @ 35.07s; https://docs.netlify.com/welcome/add-new-site/ |
| confirmation | Review branch, build command, and publish directory | Netlify validates deployment settings The official recording shows the confirmed site configured state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-repository-selected.png @ 35.07s; media/state-04-site-configured.png @ 50.36s; https://docs.netlify.com/welcome/add-new-site/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-provider-authorization.png @ 19.79s; media/state-03-repository-selected.png @ 35.07s; media/state-04-site-configured.png @ 50.36s; https://docs.netlify.com/welcome/add-new-site/ |
| progress feedback | Choose Deploy site | Netlify shows build and deploy logs Progress is observable as the distinct deployment running state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-site-configured.png @ 50.36s; media/state-05-deployment-running.png @ 65.65s; https://docs.netlify.com/welcome/add-new-site/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-repository-selected.png @ 35.07s; media/state-04-site-configured.png @ 50.36s; media/state-05-deployment-running.png @ 65.65s; https://docs.netlify.com/welcome/add-new-site/ |
| recovery and completion | Open the generated site URL | The published site loads, proving first deployment success The retained site live state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-deployment-running.png @ 65.65s; media/state-06-site-live.png @ 80.94s; https://docs.netlify.com/welcome/add-new-site/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at site entry; the first advancing trigger is “Choose Import an existing project and Git provider”.
- **Start/end:** Start is site entry at 4.50s; end is site live at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in site live; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-site-entry.png and media/state-02-provider-authorization.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-repository-selected.png and media/state-04-site-configured.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Netlify, Inc.
- **Product page:** https://docs.netlify.com/welcome/add-new-site/
- **Original media URL:** https://www.youtube.com/watch?v=4h8B080Mv4U
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 205474 bytes
- **SHA-256:** `10e0eda9c36cf794fe6036db21f96c82160e03e3c6c98b7f1d40187f1a18aacc`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
