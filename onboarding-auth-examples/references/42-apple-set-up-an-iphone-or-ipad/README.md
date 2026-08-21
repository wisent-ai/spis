# Apple — set up an iPhone or iPad

**Evidence status:** `complete`  
**Product/source:** [https://support.apple.com/en-us/105132](https://support.apple.com/en-us/105132)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to transfer your data and set up your new iPhone | Apple Support](https://www.youtube.com/watch?v=gGftJwMa84M) — Apple Support

## Start-to-first-success journey

**Actor:** new iPhone or iPad owner  
**Goal:** complete Setup Assistant and reach the usable Home Screen  
**Prerequisites:** charged iPhone or iPad; Wi-Fi or cellular access; Apple Account or explicit later decision

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Power on and use the Hello screen | Setup Assistant presents language and accessibility entry points | welcome | `media/state-01-welcome.png` and motion at 4.50s |
| 2 | Choose language and region | The device applies locale and advances | locale set | `media/state-02-locale-set.png` and motion at 19.79s |
| 3 | Connect to Wi-Fi or cellular service | The device activates and checks setup eligibility | connected | `media/state-03-connected.png` and motion at 35.07s |
| 4 | Choose Quick Start, restore, transfer, or set up without another device | Setup Assistant processes the selected data route | data decision | `media/state-04-data-decision.png` and motion at 50.36s |
| 5 | Authenticate an Apple Account and configure device security or defer allowed options | The device applies identity and security settings | identity secured | `media/state-05-identity-secured.png` and motion at 65.65s |
| 6 | Finish remaining service choices | The Home Screen appears, proving first device success | Home Screen | `media/state-06-home-screen.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At locale set or connected, invalid, expired, denied, or missing required input leaves the flow short of Home Screen; evidence: media/state-02-locale-set.png, media/state-03-connected.png, and https://support.apple.com/en-us/105132.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-connected.png through media/state-05-identity-secured.png.
- **Recovery:** Return to the retained locale set or connected requirement, correct or resend the blocking input, and resubmit; evidence: https://support.apple.com/en-us/105132.
- **Recovery:** Continue through the same terminal action until Home Screen is visible in media/state-06-home-screen.png and the motion at 80.940s.
- **Completion evidence:** Home Screen retained at media/state-06-home-screen.png and media/official-recording.mp4#t=80.940; source https://support.apple.com/en-us/105132

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| welcome | [`media/state-01-welcome.png`](media/state-01-welcome.png) | media/official-recording.mp4#t=4.497 | 640×360 | `4a0e9e33315bb150bd90e462761ff05e6f9e89168c0b192b89233a285c2180f1` |
| locale set | [`media/state-02-locale-set.png`](media/state-02-locale-set.png) | media/official-recording.mp4#t=19.785 | 640×360 | `17f58e090c2352bda2b0de4667894b06e68f28f7ffb54e4116543da5a781b177` |
| connected | [`media/state-03-connected.png`](media/state-03-connected.png) | media/official-recording.mp4#t=35.074 | 640×360 | `d953c14e3fd89c118104666ee146043b2586d56c9780f56c809fa94d7a327da7` |
| data decision | [`media/state-04-data-decision.png`](media/state-04-data-decision.png) | media/official-recording.mp4#t=50.362 | 640×360 | `28160a6c55d0495b45a899852641c2637913667c2d57d3a8460babc056a6c4e7` |
| identity secured | [`media/state-05-identity-secured.png`](media/state-05-identity-secured.png) | media/official-recording.mp4#t=65.651 | 640×360 | `eb077d2b37faa4ee2136bc2990d60496d6c2e1724173307a294fa66100c4a606` |
| Home Screen | [`media/state-06-home-screen.png`](media/state-06-home-screen.png) | media/official-recording.mp4#t=80.940 | 640×360 | `45dcbe27ca07b65c181e5e7e95c239dbc88c745db795541db1bdee5292629085` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Power on and use the Hello screen | Setup Assistant presents language and accessibility entry points The retained welcome state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-welcome.png @ 4.50s; https://support.apple.com/en-us/105132 |
| focus and selection | Choose language and region | The device applies locale and advances The recording advances to locale set and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-welcome.png @ 4.50s; media/state-02-locale-set.png @ 19.79s; https://support.apple.com/en-us/105132 |
| navigation | Connect to Wi-Fi or cellular service | The device activates and checks setup eligibility The navigation result is visible as connected. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-locale-set.png @ 19.79s; media/state-03-connected.png @ 35.07s; https://support.apple.com/en-us/105132 |
| confirmation | Choose Quick Start, restore, transfer, or set up without another device | Setup Assistant processes the selected data route The official recording shows the confirmed data decision state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-connected.png @ 35.07s; media/state-04-data-decision.png @ 50.36s; https://support.apple.com/en-us/105132 |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-locale-set.png @ 19.79s; media/state-03-connected.png @ 35.07s; media/state-04-data-decision.png @ 50.36s; https://support.apple.com/en-us/105132 |
| progress feedback | Authenticate an Apple Account and configure device security or defer allowed options | The device applies identity and security settings Progress is observable as the distinct identity secured state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-data-decision.png @ 50.36s; media/state-05-identity-secured.png @ 65.65s; https://support.apple.com/en-us/105132 |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-connected.png @ 35.07s; media/state-04-data-decision.png @ 50.36s; media/state-05-identity-secured.png @ 65.65s; https://support.apple.com/en-us/105132 |
| recovery and completion | Finish remaining service choices | The Home Screen appears, proving first device success The retained Home Screen state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-identity-secured.png @ 65.65s; media/state-06-home-screen.png @ 80.94s; https://support.apple.com/en-us/105132 |

## Motion behavior

- **Trigger:** The recorded sequence begins at welcome; the first advancing trigger is “Choose language and region”.
- **Start/end:** Start is welcome at 4.50s; end is Home Screen at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in Home Screen; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-welcome.png and media/state-02-locale-set.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-connected.png and media/state-04-data-decision.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Apple Inc.
- **Product page:** https://support.apple.com/en-us/105132
- **Original media URL:** https://www.youtube.com/watch?v=gGftJwMa84M
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 463282 bytes
- **SHA-256:** `094da96508ceb500cb57544cf5e4b7abc0e95f0d36b56a820b634057d19e0eb4`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
