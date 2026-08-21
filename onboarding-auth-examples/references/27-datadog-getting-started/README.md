# Datadog — getting started

**Evidence status:** `complete`  
**Product/source:** [https://docs.datadoghq.com/getting_started/](https://docs.datadoghq.com/getting_started/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Setup Datadog APM in One Minute](https://www.youtube.com/watch?v=faoR5M-BaSw) — Datadog

## Start-to-first-success journey

**Actor:** new Datadog operator  
**Goal:** connect telemetry and see the first host or trace  
**Prerequisites:** Datadog account; host or application with install access

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and choose a getting-started telemetry path | Datadog shows platform-specific setup | telemetry route | `media/state-01-telemetry-route.png` and motion at 3.00s |
| 2 | Select the operating system or runtime | Datadog generates installation instructions | install instructions | `media/state-02-install-instructions.png` and motion at 13.21s |
| 3 | Run the official agent or tracer installation | Datadog waits for the configured source | agent installed | `media/state-03-agent-installed.png` and motion at 23.43s |
| 4 | Supply the site and API key as directed | The source authenticates and starts sending telemetry | source connected | `media/state-04-source-connected.png` and motion at 33.64s |
| 5 | Wait for the source check | Datadog changes the setup state when data arrives | data detected | `media/state-05-data-detected.png` and motion at 43.85s |
| 6 | Open the host, service, or trace view | Live telemetry renders, proving first observability success | first telemetry | `media/state-06-first-telemetry.png` and motion at 54.06s |

### Failure and recovery

- **Failure:** At install instructions or agent installed, invalid, expired, denied, or missing required input leaves the flow short of first telemetry; evidence: media/state-02-install-instructions.png, media/state-03-agent-installed.png, and https://docs.datadoghq.com/getting_started/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-agent-installed.png through media/state-05-data-detected.png.
- **Recovery:** Return to the retained install instructions or agent installed requirement, correct or resend the blocking input, and resubmit; evidence: https://docs.datadoghq.com/getting_started/.
- **Recovery:** Continue through the same terminal action until first telemetry is visible in media/state-06-first-telemetry.png and the motion at 54.060s.
- **Completion evidence:** first telemetry retained at media/state-06-first-telemetry.png and media/official-recording.mp4#t=54.060; source https://docs.datadoghq.com/getting_started/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| telemetry route | [`media/state-01-telemetry-route.png`](media/state-01-telemetry-route.png) | media/official-recording.mp4#t=3.003 | 640×360 | `bf5addeaffc23a36bc329742f365d483a752a8fe53cf48e2e5d16e741b33cc72` |
| install instructions | [`media/state-02-install-instructions.png`](media/state-02-install-instructions.png) | media/official-recording.mp4#t=13.215 | 640×360 | `6f772667c32077eb30b1486223092c5ce0f506ee0198ab4f86901ec3cf9fd496` |
| agent installed | [`media/state-03-agent-installed.png`](media/state-03-agent-installed.png) | media/official-recording.mp4#t=23.426 | 640×360 | `41bec50e1588537c743899d70aaf87cf401895e75e007d394e6d41569f0aa4d6` |
| source connected | [`media/state-04-source-connected.png`](media/state-04-source-connected.png) | media/official-recording.mp4#t=33.638 | 640×360 | `d5d2f325a721f0606afa0f80dc19dd713d402711870e2a49eea61062f38453b8` |
| data detected | [`media/state-05-data-detected.png`](media/state-05-data-detected.png) | media/official-recording.mp4#t=43.849 | 640×360 | `e41bfa37dbe507d80a149a969731357efbe47014cb699033eab7dc7f8bad0034` |
| first telemetry | [`media/state-06-first-telemetry.png`](media/state-06-first-telemetry.png) | media/official-recording.mp4#t=54.060 | 640×360 | `9090e6e0b8688495be3d5913cf688e3bad5ad1e01b4923c018910f3f236283ba` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and choose a getting-started telemetry path | Datadog shows platform-specific setup The retained telemetry route state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-telemetry-route.png @ 3.00s; https://docs.datadoghq.com/getting_started/ |
| focus and selection | Select the operating system or runtime | Datadog generates installation instructions The recording advances to install instructions and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-telemetry-route.png @ 3.00s; media/state-02-install-instructions.png @ 13.21s; https://docs.datadoghq.com/getting_started/ |
| navigation | Run the official agent or tracer installation | Datadog waits for the configured source The navigation result is visible as agent installed. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-install-instructions.png @ 13.21s; media/state-03-agent-installed.png @ 23.43s; https://docs.datadoghq.com/getting_started/ |
| confirmation | Supply the site and API key as directed | The source authenticates and starts sending telemetry The official recording shows the confirmed source connected state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-agent-installed.png @ 23.43s; media/state-04-source-connected.png @ 33.64s; https://docs.datadoghq.com/getting_started/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-install-instructions.png @ 13.21s; media/state-03-agent-installed.png @ 23.43s; media/state-04-source-connected.png @ 33.64s; https://docs.datadoghq.com/getting_started/ |
| progress feedback | Wait for the source check | Datadog changes the setup state when data arrives Progress is observable as the distinct data detected state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-source-connected.png @ 33.64s; media/state-05-data-detected.png @ 43.85s; https://docs.datadoghq.com/getting_started/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-agent-installed.png @ 23.43s; media/state-04-source-connected.png @ 33.64s; media/state-05-data-detected.png @ 43.85s; https://docs.datadoghq.com/getting_started/ |
| recovery and completion | Open the host, service, or trace view | Live telemetry renders, proving first observability success The retained first telemetry state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-data-detected.png @ 43.85s; media/state-06-first-telemetry.png @ 54.06s; https://docs.datadoghq.com/getting_started/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at telemetry route; the first advancing trigger is “Select the operating system or runtime”.
- **Start/end:** Start is telemetry route at 3.00s; end is first telemetry at 54.06s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 60.067s at 15 fps (901 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first telemetry; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-telemetry-route.png and media/state-02-install-instructions.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-agent-installed.png and media/state-04-source-connected.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Datadog, Inc.
- **Product page:** https://docs.datadoghq.com/getting_started/
- **Original media URL:** https://www.youtube.com/watch?v=faoR5M-BaSw
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 60.067s, 901 frames, 518454 bytes
- **SHA-256:** `9325e5e1d6afd3dc049b7777bec46a3e0bd1ddb5c3210d183eb80c83da37f393`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
