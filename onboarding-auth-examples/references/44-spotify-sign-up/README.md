# Spotify — sign up

**Evidence status:** `complete`  
**Product/source:** [https://support.spotify.com/article/how-to-sign-up/](https://support.spotify.com/article/how-to-sign-up/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [What is Spotify](https://www.youtube.com/watch?v=j8L5ySUufdE) — SpotifyCares

## Start-to-first-success journey

**Actor:** new Spotify listener  
**Goal:** create an account and play the first track  
**Prerequisites:** email or supported identity provider; network connection

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Choose Sign up and an identity method | Spotify opens provider authorization or account fields | signup route | `media/state-01-signup-route.png` and motion at 2.69s |
| 2 | Complete provider authorization or enter email and password | Spotify validates the identity | credentials ready | `media/state-02-credentials-ready.png` and motion at 11.84s |
| 3 | Enter required profile details and accept terms | Spotify creates the account | account created | `media/state-03-account-created.png` and motion at 20.98s |
| 4 | Complete email verification or sign in when required | Spotify opens the authenticated Home surface | identity active | `media/state-04-identity-active.png` and motion at 30.13s |
| 5 | Search for or select music | Spotify opens the track, album, or playlist context | content selected | `media/state-05-content-selected.png` and motion at 39.27s |
| 6 | Press Play | Playback begins and controls update, proving first listening success | first playback | `media/state-06-first-playback.png` and motion at 48.42s |

### Failure and recovery

- **Failure:** At credentials ready or account created, invalid, expired, denied, or missing required input leaves the flow short of first playback; evidence: media/state-02-credentials-ready.png, media/state-03-account-created.png, and https://support.spotify.com/article/how-to-sign-up/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-account-created.png through media/state-05-content-selected.png.
- **Recovery:** Return to the retained credentials ready or account created requirement, correct or resend the blocking input, and resubmit; evidence: https://support.spotify.com/article/how-to-sign-up/.
- **Recovery:** Continue through the same terminal action until first playback is visible in media/state-06-first-playback.png and the motion at 48.420s.
- **Completion evidence:** first playback retained at media/state-06-first-playback.png and media/official-recording.mp4#t=48.420; source https://support.spotify.com/article/how-to-sign-up/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| signup route | [`media/state-01-signup-route.png`](media/state-01-signup-route.png) | media/official-recording.mp4#t=2.690 | 640×360 | `d315a7701fdc020ccb819f4c923997c0f096ad2cfb22825061484dfdb8acd2a4` |
| credentials ready | [`media/state-02-credentials-ready.png`](media/state-02-credentials-ready.png) | media/official-recording.mp4#t=11.836 | 640×360 | `c019ea4315aa068208b04dd083bef7a8c3c435bdd40a99f4d8cb7509467c44ee` |
| account created | [`media/state-03-account-created.png`](media/state-03-account-created.png) | media/official-recording.mp4#t=20.982 | 640×360 | `1974318bbf1e27853bd9a99c54c27af54fdf8181c0894e1256fb03a9764b5176` |
| identity active | [`media/state-04-identity-active.png`](media/state-04-identity-active.png) | media/official-recording.mp4#t=30.128 | 640×360 | `7fed4966b52bb34445b09bcb25e9b63374c2c2222531fe35e06154673dd4e975` |
| content selected | [`media/state-05-content-selected.png`](media/state-05-content-selected.png) | media/official-recording.mp4#t=39.274 | 640×360 | `8bc2df59c44bcd42dd5479e7fc8970c5ba69386061db37c20a40b15870717f7b` |
| first playback | [`media/state-06-first-playback.png`](media/state-06-first-playback.png) | media/official-recording.mp4#t=48.420 | 640×360 | `2e72c14936b9a356445e54528895a239edddf6f21aa0f948fbaf88a9b3aa2b6d` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Choose Sign up and an identity method | Spotify opens provider authorization or account fields The retained signup route state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-signup-route.png @ 2.69s; https://support.spotify.com/article/how-to-sign-up/ |
| focus and selection | Complete provider authorization or enter email and password | Spotify validates the identity The recording advances to credentials ready and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-signup-route.png @ 2.69s; media/state-02-credentials-ready.png @ 11.84s; https://support.spotify.com/article/how-to-sign-up/ |
| navigation | Enter required profile details and accept terms | Spotify creates the account The navigation result is visible as account created. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-credentials-ready.png @ 11.84s; media/state-03-account-created.png @ 20.98s; https://support.spotify.com/article/how-to-sign-up/ |
| confirmation | Complete email verification or sign in when required | Spotify opens the authenticated Home surface The official recording shows the confirmed identity active state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-account-created.png @ 20.98s; media/state-04-identity-active.png @ 30.13s; https://support.spotify.com/article/how-to-sign-up/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-credentials-ready.png @ 11.84s; media/state-03-account-created.png @ 20.98s; media/state-04-identity-active.png @ 30.13s; https://support.spotify.com/article/how-to-sign-up/ |
| progress feedback | Search for or select music | Spotify opens the track, album, or playlist context Progress is observable as the distinct content selected state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-identity-active.png @ 30.13s; media/state-05-content-selected.png @ 39.27s; https://support.spotify.com/article/how-to-sign-up/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-account-created.png @ 20.98s; media/state-04-identity-active.png @ 30.13s; media/state-05-content-selected.png @ 39.27s; https://support.spotify.com/article/how-to-sign-up/ |
| recovery and completion | Press Play | Playback begins and controls update, proving first listening success The retained first playback state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-content-selected.png @ 39.27s; media/state-06-first-playback.png @ 48.42s; https://support.spotify.com/article/how-to-sign-up/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at signup route; the first advancing trigger is “Complete provider authorization or enter email and password”.
- **Start/end:** Start is signup route at 2.69s; end is first playback at 48.42s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 53.800s at 15 fps (807 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first playback; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-signup-route.png and media/state-02-credentials-ready.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-account-created.png and media/state-04-identity-active.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Spotify AB
- **Product page:** https://support.spotify.com/article/how-to-sign-up/
- **Original media URL:** https://www.youtube.com/watch?v=j8L5ySUufdE
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 53.800s, 807 frames, 513296 bytes
- **SHA-256:** `a3c10cbc7ca24bc15c74a3f787df8077984f681bda87e8ce88981b129e406eb0`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
