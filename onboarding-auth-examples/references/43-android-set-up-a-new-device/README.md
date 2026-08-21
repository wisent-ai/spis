# Android — set up a new device

**Evidence status:** `complete`  
**Product/source:** [https://support.google.com/android/answer/6193424](https://support.google.com/android/answer/6193424)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Help and Tips to Get Started With Your New Android Phone](https://www.youtube.com/watch?v=Z4PwQyp9Yjk) — Android

## Start-to-first-success journey

**Actor:** new Android device owner  
**Goal:** finish device setup and reach the launcher  
**Prerequisites:** charged Android device; Wi-Fi or cellular access; Google Account or explicit skip decision

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Power on and start setup | Android shows language and accessibility controls | welcome | `media/state-01-welcome.png` and motion at 1.73s |
| 2 | Choose language and connect to a network | Android checks updates and activation | connected | `media/state-02-connected.png` and motion at 7.63s |
| 3 | Choose copy-data or set-up-as-new | Android prepares the selected transfer path | data decision | `media/state-03-data-decision.png` and motion at 13.52s |
| 4 | Sign in to a Google Account or use the allowed skip route | Android applies account context and services | account decision | `media/state-04-account-decision.png` and motion at 19.41s |
| 5 | Set screen lock and review service permissions | Android records security and privacy choices | device secured | `media/state-05-device-secured.png` and motion at 25.31s |
| 6 | Complete setup | The launcher appears, proving first usable-device success | launcher | `media/state-06-launcher.png` and motion at 31.20s |

### Failure and recovery

- **Failure:** At connected or data decision, invalid, expired, denied, or missing required input leaves the flow short of launcher; evidence: media/state-02-connected.png, media/state-03-data-decision.png, and https://support.google.com/android/answer/6193424.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-data-decision.png through media/state-05-device-secured.png.
- **Recovery:** Return to the retained connected or data decision requirement, correct or resend the blocking input, and resubmit; evidence: https://support.google.com/android/answer/6193424.
- **Recovery:** Continue through the same terminal action until launcher is visible in media/state-06-launcher.png and the motion at 31.200s.
- **Completion evidence:** launcher retained at media/state-06-launcher.png and media/official-recording.mp4#t=31.200; source https://support.google.com/android/answer/6193424

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| welcome | [`media/state-01-welcome.png`](media/state-01-welcome.png) | media/official-recording.mp4#t=1.733 | 640×360 | `f3404369150b495222c414dae70e90ea981aea86941dbec074fc0fbc3dda74b5` |
| connected | [`media/state-02-connected.png`](media/state-02-connected.png) | media/official-recording.mp4#t=7.627 | 640×360 | `5692499e470b3127747828c9f2cff1a56c9e51f5abb3791c27edef3c993fd3eb` |
| data decision | [`media/state-03-data-decision.png`](media/state-03-data-decision.png) | media/official-recording.mp4#t=13.520 | 640×360 | `8a8caa01409a3b9dc5a0517aaed51d08e2120be9ff07c0ca6a05211fc8e98520` |
| account decision | [`media/state-04-account-decision.png`](media/state-04-account-decision.png) | media/official-recording.mp4#t=19.414 | 640×360 | `b6ce51db5154297be67e97e33c0e0df04f168141d2f7d76c33145397309e3d0f` |
| device secured | [`media/state-05-device-secured.png`](media/state-05-device-secured.png) | media/official-recording.mp4#t=25.307 | 640×360 | `7f4483053200a7fb766509bb10897eaf795346b9e023f706c75fba58416ffc78` |
| launcher | [`media/state-06-launcher.png`](media/state-06-launcher.png) | media/official-recording.mp4#t=31.200 | 640×360 | `8dd532270df16c09b8c6a205ef78af2bbdb1782f77103e9d0c5cd560cca4b591` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Power on and start setup | Android shows language and accessibility controls The retained welcome state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-welcome.png @ 1.73s; https://support.google.com/android/answer/6193424 |
| focus and selection | Choose language and connect to a network | Android checks updates and activation The recording advances to connected and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-welcome.png @ 1.73s; media/state-02-connected.png @ 7.63s; https://support.google.com/android/answer/6193424 |
| navigation | Choose copy-data or set-up-as-new | Android prepares the selected transfer path The navigation result is visible as data decision. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-connected.png @ 7.63s; media/state-03-data-decision.png @ 13.52s; https://support.google.com/android/answer/6193424 |
| confirmation | Sign in to a Google Account or use the allowed skip route | Android applies account context and services The official recording shows the confirmed account decision state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-data-decision.png @ 13.52s; media/state-04-account-decision.png @ 19.41s; https://support.google.com/android/answer/6193424 |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-connected.png @ 7.63s; media/state-03-data-decision.png @ 13.52s; media/state-04-account-decision.png @ 19.41s; https://support.google.com/android/answer/6193424 |
| progress feedback | Set screen lock and review service permissions | Android records security and privacy choices Progress is observable as the distinct device secured state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-account-decision.png @ 19.41s; media/state-05-device-secured.png @ 25.31s; https://support.google.com/android/answer/6193424 |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-data-decision.png @ 13.52s; media/state-04-account-decision.png @ 19.41s; media/state-05-device-secured.png @ 25.31s; https://support.google.com/android/answer/6193424 |
| recovery and completion | Complete setup | The launcher appears, proving first usable-device success The retained launcher state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-device-secured.png @ 25.31s; media/state-06-launcher.png @ 31.20s; https://support.google.com/android/answer/6193424 |

## Motion behavior

- **Trigger:** The recorded sequence begins at welcome; the first advancing trigger is “Choose language and connect to a network”.
- **Start/end:** Start is welcome at 1.73s; end is launcher at 31.20s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 34.667s at 15 fps (520 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in launcher; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-welcome.png and media/state-02-connected.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-data-decision.png and media/state-04-account-decision.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Google LLC
- **Product page:** https://support.google.com/android/answer/6193424
- **Original media URL:** https://www.youtube.com/watch?v=Z4PwQyp9Yjk
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 34.667s, 520 frames, 495907 bytes
- **SHA-256:** `9a4385ae90de63177e2e92f9ba14bc644754f8d918a927436e6326bb77616d6b`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
