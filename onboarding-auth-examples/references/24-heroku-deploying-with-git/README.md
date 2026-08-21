# Heroku — deploying with Git

**Evidence status:** `complete`  
**Product/source:** [https://devcenter.heroku.com/articles/git](https://devcenter.heroku.com/articles/git)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Go on Heroku: Continuous Deployment with GitHub](https://www.youtube.com/watch?v=sffWuu7XBN4) — Heroku

## Start-to-first-success journey

**Actor:** new Heroku developer  
**Goal:** deploy a Git repository and open the application  
**Prerequisites:** Heroku account; Heroku CLI; Git repository

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate with the Heroku CLI | Heroku completes browser or terminal authentication | CLI authenticated | `media/state-01-cli-authenticated.png` and motion at 2.53s |
| 2 | Create or select an app | Heroku returns the app name and Git remote | app created | `media/state-02-app-created.png` and motion at 11.12s |
| 3 | Confirm the Heroku remote | Git shows the deployment destination | remote ready | `media/state-03-remote-ready.png` and motion at 19.71s |
| 4 | Push the target branch | Heroku receives source and starts the build | build running | `media/state-04-build-running.png` and motion at 28.30s |
| 5 | Read build and release feedback | Heroku reports a successful release or actionable failure | release complete | `media/state-05-release-complete.png` and motion at 36.89s |
| 6 | Open the app URL | The deployed application responds, proving first delivery success | app live | `media/state-06-app-live.png` and motion at 45.48s |

### Failure and recovery

- **Failure:** At app created or remote ready, invalid, expired, denied, or missing required input leaves the flow short of app live; evidence: media/state-02-app-created.png, media/state-03-remote-ready.png, and https://devcenter.heroku.com/articles/git.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-remote-ready.png through media/state-05-release-complete.png.
- **Recovery:** Return to the retained app created or remote ready requirement, correct or resend the blocking input, and resubmit; evidence: https://devcenter.heroku.com/articles/git.
- **Recovery:** Continue through the same terminal action until app live is visible in media/state-06-app-live.png and the motion at 45.480s.
- **Completion evidence:** app live retained at media/state-06-app-live.png and media/official-recording.mp4#t=45.480; source https://devcenter.heroku.com/articles/git

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| CLI authenticated | [`media/state-01-cli-authenticated.png`](media/state-01-cli-authenticated.png) | media/official-recording.mp4#t=2.527 | 640×360 | `a99da9719718eb5e2f622406449f5612668874ea9280b8c9706d265830b3bbfc` |
| app created | [`media/state-02-app-created.png`](media/state-02-app-created.png) | media/official-recording.mp4#t=11.117 | 640×360 | `88ae7d6eb10f79c6f5b14ba750815037eeb1673028ea6878a2d01240b9f15235` |
| remote ready | [`media/state-03-remote-ready.png`](media/state-03-remote-ready.png) | media/official-recording.mp4#t=19.708 | 640×360 | `d4607b52675a48f34a8b5314c1d0bc112332c55563d9481271b552b43cb7556c` |
| build running | [`media/state-04-build-running.png`](media/state-04-build-running.png) | media/official-recording.mp4#t=28.298 | 640×360 | `d9e04c7810067bec6746bb26570ac4aeadb9d017f65e20ea16db199073d2f9ac` |
| release complete | [`media/state-05-release-complete.png`](media/state-05-release-complete.png) | media/official-recording.mp4#t=36.889 | 640×360 | `0c389f76682a5e03d4527dfbdded1766853e582ea63c00f92249ebf94224b829` |
| app live | [`media/state-06-app-live.png`](media/state-06-app-live.png) | media/official-recording.mp4#t=45.480 | 640×360 | `656da7e5f4d70974c6c9e8e833f4eb3c14b83ffc62f82391060df329a8cfde9c` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate with the Heroku CLI | Heroku completes browser or terminal authentication The retained CLI authenticated state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-cli-authenticated.png @ 2.53s; https://devcenter.heroku.com/articles/git |
| focus and selection | Create or select an app | Heroku returns the app name and Git remote The recording advances to app created and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-cli-authenticated.png @ 2.53s; media/state-02-app-created.png @ 11.12s; https://devcenter.heroku.com/articles/git |
| navigation | Confirm the Heroku remote | Git shows the deployment destination The navigation result is visible as remote ready. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-app-created.png @ 11.12s; media/state-03-remote-ready.png @ 19.71s; https://devcenter.heroku.com/articles/git |
| confirmation | Push the target branch | Heroku receives source and starts the build The official recording shows the confirmed build running state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-remote-ready.png @ 19.71s; media/state-04-build-running.png @ 28.30s; https://devcenter.heroku.com/articles/git |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-app-created.png @ 11.12s; media/state-03-remote-ready.png @ 19.71s; media/state-04-build-running.png @ 28.30s; https://devcenter.heroku.com/articles/git |
| progress feedback | Read build and release feedback | Heroku reports a successful release or actionable failure Progress is observable as the distinct release complete state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-build-running.png @ 28.30s; media/state-05-release-complete.png @ 36.89s; https://devcenter.heroku.com/articles/git |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-remote-ready.png @ 19.71s; media/state-04-build-running.png @ 28.30s; media/state-05-release-complete.png @ 36.89s; https://devcenter.heroku.com/articles/git |
| recovery and completion | Open the app URL | The deployed application responds, proving first delivery success The retained app live state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-release-complete.png @ 36.89s; media/state-06-app-live.png @ 45.48s; https://devcenter.heroku.com/articles/git |

## Motion behavior

- **Trigger:** The recorded sequence begins at CLI authenticated; the first advancing trigger is “Create or select an app”.
- **Start/end:** Start is CLI authenticated at 2.53s; end is app live at 45.48s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 50.533s at 15 fps (758 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in app live; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-cli-authenticated.png and media/state-02-app-created.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-remote-ready.png and media/state-04-build-running.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Salesforce, Inc.
- **Product page:** https://devcenter.heroku.com/articles/git
- **Original media URL:** https://www.youtube.com/watch?v=sffWuu7XBN4
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 50.533s, 758 frames, 428140 bytes
- **SHA-256:** `8adc5f1cae5738b37b032113592806ba7ebe30c3c025f8606a47925c13df5235`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
