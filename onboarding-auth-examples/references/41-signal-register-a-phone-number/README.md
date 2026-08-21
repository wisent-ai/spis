# Signal — register a phone number

**Evidence status:** `complete`  
**Product/source:** [https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number](https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Start a Signal chat via username](https://signal.org/blog/videos/usernames-chat-via-username.mp4) — Signal Blog

## Start-to-first-success journey

**Actor:** new Signal user  
**Goal:** register the phone number and send the first private message  
**Prerequisites:** Signal installed on a phone; phone number able to receive SMS or a call

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Open Signal and begin registration | Signal presents phone-number entry | registration entry | `media/state-01-registration-entry.png` and motion at 0.90s |
| 2 | Enter the phone number and request verification | Signal sends an SMS or offers the voice route | verification pending | `media/state-02-verification-pending.png` and motion at 3.95s |
| 3 | Enter the received verification code | Signal accepts the number and advances to account setup | number verified | `media/state-03-number-verified.png` and motion at 6.99s |
| 4 | Set the Signal PIN and profile name | Signal establishes the account profile | profile ready | `media/state-04-profile-ready.png` and motion at 10.04s |
| 5 | Grant or decline contact and notification permissions | Signal records the choices and opens the conversation list | app ready | `media/state-05-app-ready.png` and motion at 13.09s |
| 6 | Start a chat by contact or username and send a message | The conversation shows the sent message, proving first messaging success | first message | `media/state-06-first-message.png` and motion at 16.14s |

### Failure and recovery

- **Failure:** At verification pending or number verified, invalid, expired, denied, or missing required input leaves the flow short of first message; evidence: media/state-02-verification-pending.png, media/state-03-number-verified.png, and https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-number-verified.png through media/state-05-app-ready.png.
- **Recovery:** Return to the retained verification pending or number verified requirement, correct or resend the blocking input, and resubmit; evidence: https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number.
- **Recovery:** Continue through the same terminal action until first message is visible in media/state-06-first-message.png and the motion at 16.140s.
- **Completion evidence:** first message retained at media/state-06-first-message.png and media/official-recording.mp4#t=16.140; source https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| registration entry | [`media/state-01-registration-entry.png`](media/state-01-registration-entry.png) | media/official-recording.mp4#t=0.897 | 960×964 | `8f4c9f288e0b08b6fb9118cd94598244283248db39bc7195729beed352a429a6` |
| verification pending | [`media/state-02-verification-pending.png`](media/state-02-verification-pending.png) | media/official-recording.mp4#t=3.945 | 960×964 | `322d6192f422f21f8e6f28171b90ae5a415f6bfc487d37a275f3cc0ebd3d013e` |
| number verified | [`media/state-03-number-verified.png`](media/state-03-number-verified.png) | media/official-recording.mp4#t=6.994 | 960×964 | `1b8f918d5ec8958be27f7eab2ddab860a37ade9ff9f01bf4710ac5ed8f9834dd` |
| profile ready | [`media/state-04-profile-ready.png`](media/state-04-profile-ready.png) | media/official-recording.mp4#t=10.042 | 960×964 | `1ee25df7b85a200a05615fa9350ce5efefc433bec4d7cdc95d1f9a9f03e12731` |
| app ready | [`media/state-05-app-ready.png`](media/state-05-app-ready.png) | media/official-recording.mp4#t=13.091 | 960×964 | `289c4d8a84b502e7fa5d6fcfd681fef13e53bd0eb7e91c9cbeb3299f15bf02f8` |
| first message | [`media/state-06-first-message.png`](media/state-06-first-message.png) | media/official-recording.mp4#t=16.140 | 960×964 | `75d08e329cb4502deda9786a417accb687dd42b49eba8a0d55242759bd014fe0` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Open Signal and begin registration | Signal presents phone-number entry The retained registration entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-registration-entry.png @ 0.90s; https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number |
| focus and selection | Enter the phone number and request verification | Signal sends an SMS or offers the voice route The recording advances to verification pending and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-registration-entry.png @ 0.90s; media/state-02-verification-pending.png @ 3.95s; https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number |
| navigation | Enter the received verification code | Signal accepts the number and advances to account setup The navigation result is visible as number verified. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-verification-pending.png @ 3.95s; media/state-03-number-verified.png @ 6.99s; https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number |
| confirmation | Set the Signal PIN and profile name | Signal establishes the account profile The official recording shows the confirmed profile ready state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-number-verified.png @ 6.99s; media/state-04-profile-ready.png @ 10.04s; https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-verification-pending.png @ 3.95s; media/state-03-number-verified.png @ 6.99s; media/state-04-profile-ready.png @ 10.04s; https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number |
| progress feedback | Grant or decline contact and notification permissions | Signal records the choices and opens the conversation list Progress is observable as the distinct app ready state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-profile-ready.png @ 10.04s; media/state-05-app-ready.png @ 13.09s; https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-number-verified.png @ 6.99s; media/state-04-profile-ready.png @ 10.04s; media/state-05-app-ready.png @ 13.09s; https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number |
| recovery and completion | Start a chat by contact or username and send a message | The conversation shows the sent message, proving first messaging success The retained first message state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-app-ready.png @ 13.09s; media/state-06-first-message.png @ 16.14s; https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number |

## Motion behavior

- **Trigger:** The recorded sequence begins at registration entry; the first advancing trigger is “Enter the phone number and request verification”.
- **Start/end:** Start is registration entry at 0.90s; end is first message at 16.14s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 17.933s at 15 fps (269 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first message; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-registration-entry.png and media/state-02-verification-pending.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-number-verified.png and media/state-04-profile-ready.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Signal Technology Foundation
- **Product page:** https://support.signal.org/hc/en-us/articles/360007318691-Register-a-phone-number
- **Original media URL:** https://signal.org/blog/videos/usernames-chat-via-username.mp4
- **Capture method:** direct official-product download; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 960×964, 17.933s, 269 frames, 148014 bytes
- **SHA-256:** `e95fc9cfb4188e8a162f3ed4dc4c1b7747ae29ab55803506fe0e9c5f7e3c2b65`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
