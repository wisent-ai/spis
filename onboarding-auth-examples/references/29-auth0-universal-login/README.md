# Auth0 — Universal Login

**Evidence status:** `complete`  
**Product/source:** [https://auth0.com/docs/authenticate/login/auth0-universal-login](https://auth0.com/docs/authenticate/login/auth0-universal-login)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Create Organizations and Allow Sign Ups](https://www.youtube.com/watch?v=M9z8MeQicz4) — Okta Support Center

## Start-to-first-success journey

**Actor:** Auth0 tenant administrator  
**Goal:** enable Universal Login and complete a test sign-up  
**Prerequisites:** Auth0 tenant; registered application and callback URL

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Sign in and open application settings | Auth0 shows the selected application's identity configuration | application context | `media/state-01-application-context.png` and motion at 4.50s |
| 2 | Configure allowed callback and logout URLs | Auth0 validates and saves redirect boundaries | redirects configured | `media/state-02-redirects-configured.png` and motion at 19.79s |
| 3 | Open Universal Login or organization sign-up settings | Auth0 displays hosted login options | login configured | `media/state-03-login-configured.png` and motion at 35.07s |
| 4 | Launch Try or the application login route | Auth0 opens the hosted Universal Login prompt | login prompt | `media/state-04-login-prompt.png` and motion at 50.36s |
| 5 | Choose Sign up, enter identity data, and confirm | Auth0 creates the user or presents verification requirements | user created | `media/state-05-user-created.png` and motion at 65.65s |
| 6 | Complete login and return to the callback | The application receives the authenticated result, proving first-success login | callback success | `media/state-06-callback-success.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At redirects configured or login configured, invalid, expired, denied, or missing required input leaves the flow short of callback success; evidence: media/state-02-redirects-configured.png, media/state-03-login-configured.png, and https://auth0.com/docs/authenticate/login/auth0-universal-login.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-login-configured.png through media/state-05-user-created.png.
- **Recovery:** Return to the retained redirects configured or login configured requirement, correct or resend the blocking input, and resubmit; evidence: https://auth0.com/docs/authenticate/login/auth0-universal-login.
- **Recovery:** Continue through the same terminal action until callback success is visible in media/state-06-callback-success.png and the motion at 80.940s.
- **Completion evidence:** callback success retained at media/state-06-callback-success.png and media/official-recording.mp4#t=80.940; source https://auth0.com/docs/authenticate/login/auth0-universal-login

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| application context | [`media/state-01-application-context.png`](media/state-01-application-context.png) | media/official-recording.mp4#t=4.497 | 640×360 | `d2b1354c2d2500f8a31c561e5b321316701d910bae9710e00e2c1a9435944b19` |
| redirects configured | [`media/state-02-redirects-configured.png`](media/state-02-redirects-configured.png) | media/official-recording.mp4#t=19.785 | 640×360 | `b8a8c52e41cbd2b167760289b6c4916e28ccdc2242b2a7d004560417d4ee6ce6` |
| login configured | [`media/state-03-login-configured.png`](media/state-03-login-configured.png) | media/official-recording.mp4#t=35.074 | 640×360 | `460ceeb054d25a6671267e0f2a466d25b989b6c469896cde61005a9f22661a0e` |
| login prompt | [`media/state-04-login-prompt.png`](media/state-04-login-prompt.png) | media/official-recording.mp4#t=50.362 | 640×360 | `2adf308aadc22630501d804e7f41d46979e89a36d7899d70080a21bcabeb8bad` |
| user created | [`media/state-05-user-created.png`](media/state-05-user-created.png) | media/official-recording.mp4#t=65.651 | 640×360 | `b1545e587b41d3db4c85c8306077dafbf8299a4c88b850635dd0ca11b0d9aaeb` |
| callback success | [`media/state-06-callback-success.png`](media/state-06-callback-success.png) | media/official-recording.mp4#t=80.940 | 640×360 | `1c5d60b2dde8d5b98811b7bfa5699ce8e507a46a694aed5bf96f89b9478c59fa` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Sign in and open application settings | Auth0 shows the selected application's identity configuration The retained application context state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-application-context.png @ 4.50s; https://auth0.com/docs/authenticate/login/auth0-universal-login |
| focus and selection | Configure allowed callback and logout URLs | Auth0 validates and saves redirect boundaries The recording advances to redirects configured and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-application-context.png @ 4.50s; media/state-02-redirects-configured.png @ 19.79s; https://auth0.com/docs/authenticate/login/auth0-universal-login |
| navigation | Open Universal Login or organization sign-up settings | Auth0 displays hosted login options The navigation result is visible as login configured. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-redirects-configured.png @ 19.79s; media/state-03-login-configured.png @ 35.07s; https://auth0.com/docs/authenticate/login/auth0-universal-login |
| confirmation | Launch Try or the application login route | Auth0 opens the hosted Universal Login prompt The official recording shows the confirmed login prompt state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-login-configured.png @ 35.07s; media/state-04-login-prompt.png @ 50.36s; https://auth0.com/docs/authenticate/login/auth0-universal-login |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-redirects-configured.png @ 19.79s; media/state-03-login-configured.png @ 35.07s; media/state-04-login-prompt.png @ 50.36s; https://auth0.com/docs/authenticate/login/auth0-universal-login |
| progress feedback | Choose Sign up, enter identity data, and confirm | Auth0 creates the user or presents verification requirements Progress is observable as the distinct user created state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-login-prompt.png @ 50.36s; media/state-05-user-created.png @ 65.65s; https://auth0.com/docs/authenticate/login/auth0-universal-login |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-login-configured.png @ 35.07s; media/state-04-login-prompt.png @ 50.36s; media/state-05-user-created.png @ 65.65s; https://auth0.com/docs/authenticate/login/auth0-universal-login |
| recovery and completion | Complete login and return to the callback | The application receives the authenticated result, proving first-success login The retained callback success state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-user-created.png @ 65.65s; media/state-06-callback-success.png @ 80.94s; https://auth0.com/docs/authenticate/login/auth0-universal-login |

## Motion behavior

- **Trigger:** The recorded sequence begins at application context; the first advancing trigger is “Configure allowed callback and logout URLs”.
- **Start/end:** Start is application context at 4.50s; end is callback success at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in callback success; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-application-context.png and media/state-02-redirects-configured.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-login-configured.png and media/state-04-login-prompt.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Okta, Inc. / Auth0
- **Product page:** https://auth0.com/docs/authenticate/login/auth0-universal-login
- **Original media URL:** https://www.youtube.com/watch?v=M9z8MeQicz4
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 301642 bytes
- **SHA-256:** `2a2c6b4750ff5e0cf0f36906203611de70dd55ee1da305a45409f7a3ece6994d`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
