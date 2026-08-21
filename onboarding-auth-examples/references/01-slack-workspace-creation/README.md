# Slack — workspace creation

**Evidence status:** `partial`  
**Product/source:** [https://slack.com/help/articles/206845317-Create-a-Slack-workspace](https://slack.com/help/articles/206845317-Create-a-Slack-workspace)  
**Motion asset:** [`media/official-recording.mp4`](media/official-recording.mp4) — provenance class `upstream-owner-media`  
**Official recording:** [How to use Slack | Your quick start guide | Slack 101](https://www.youtube.com/watch?v=FTuOS8E1LZk) — Slack

## Start-to-first-success journey

**Actor:** new workspace owner  
**Goal:** create a Slack workspace and reach a usable conversation  
**Prerequisites:** email address able to receive Slack verification; workspace name and team context

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Choose Create a new workspace | Slack opens the workspace-creation entry surface | workspace entry | `media/state-01-workspace-entry.png` and motion at 2.25s |
| 2 | Enter an email address and submit | Slack sends a verification challenge and shows the code state | email verification pending | `media/state-02-email-verification-pending.png` and motion at 9.90s |
| 3 | Enter the received verification code | Slack accepts the verified identity and advances to workspace details | identity verified | `media/state-03-identity-verified.png` and motion at 17.55s |
| 4 | Name the company or team and answer the work-context prompt | Slack creates the workspace context and advances through setup | workspace configured | `media/state-04-workspace-configured.png` and motion at 25.20s |
| 5 | Invite teammates or use the visible skip route | Slack records invitations or preserves the option for later | membership decision | `media/state-05-membership-decision.png` and motion at 32.85s |
| 6 | Open the first channel and send a message | The message appears in the new workspace, proving first collaborative success | first message sent | `media/state-06-first-message-sent.png` and motion at 40.50s |

### Failure and recovery

- **Failure:** At email verification pending or identity verified, invalid, expired, denied, or missing required input leaves the flow short of first message sent; evidence: media/state-02-email-verification-pending.png, media/state-03-identity-verified.png, and https://slack.com/help/articles/206845317-Create-a-Slack-workspace.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-identity-verified.png through media/state-05-membership-decision.png.
- **Recovery:** Return to the retained email verification pending or identity verified requirement, correct or resend the blocking input, and resubmit; evidence: https://slack.com/help/articles/206845317-Create-a-Slack-workspace.
- **Recovery:** Continue through the same terminal action until first message sent is visible in media/state-06-first-message-sent.png and the motion at 40.500s.
- **Completion evidence:** first message sent retained at media/state-06-first-message-sent.png and media/official-recording.mp4#t=40.500; source https://slack.com/help/articles/206845317-Create-a-Slack-workspace

## Retained product states

| Declared label | Observed in frame | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---|---:|---|
| workspace entry | Slack logo and wordmark title card on plum | [`media/state-01-workspace-entry.png`](media/state-01-workspace-entry.png) | frame of media/official-recording.mp4 at 2s (mean abs diff 1.9766/255) | 640×360 | `cc9bfe961c8bf177fef36d0c09ff20557fdd43ae9a3d1a0027e2adec357fa86c` |
| email verification pending | `# project-unicorn` channel open, placeholder message rows above the Today divider | [`media/state-02-email-verification-pending.png`](media/state-02-email-verification-pending.png) | frame of media/official-recording.mp4 at 10s (mean abs diff 4.8555/255) | 640×360 | `76e14e326ec23f5aa07d0d50b9d5faa0cbc24e048d0aadb9fa14cc58b3472b70` |
| identity verified | channel thread with Jagdeep and Arcadio messages under a View in Asana button | [`media/state-03-identity-verified.png`](media/state-03-identity-verified.png) | frame of media/official-recording.mp4 at 16s (mean abs diff 7.1484/255) | 640×360 | `0e767876665c9593a299b2140a93a79daab5c0c5d95f2e7cf71ee0b808d43203` |
| workspace configured | clapping-hands illustration with sparkles on light blue | [`media/state-04-workspace-configured.png`](media/state-04-workspace-configured.png) | frame of media/official-recording.mp4 at 24s (mean abs diff 2.3242/255) | 640×360 | `19994de4692b474ea9bd7c140005b6b8d8d23f092899b537969386b94bea542f` |
| membership decision | huddle window with a Screen Share tile, a participant video tile, and a Leave button | [`media/state-05-membership-decision.png`](media/state-05-membership-decision.png) | frame of media/official-recording.mp4 at 32.5s (mean abs diff 3.0781/255) | 640×360 | `fd3ce73dfa915cd4f6293126653133b75a645f5b693002fde8910e79fe4c8a18` |
| first message sent | brand colour-wedge outro wipe, no product surface visible | [`media/state-06-first-message-sent.png`](media/state-06-first-message-sent.png) | frame 608 of media/official-recording.mp4 at 40.5333s (mean abs diff 3.0000/255); found by rescanning every decoded frame at the asset's native 15 fps with ~/.stado/work/onboarding-auth-examples/frame-match.py, which the record's 2 fps sample grid had missed because the frame falls inside the outro animation | 640×360 | `c2c3ff6610f2205531f7c15285a9103fca5913a0094ee61207681b995b245688` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Choose Create a new workspace | Slack opens the workspace-creation entry surface The retained workspace entry state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-workspace-entry.png @ 2.25s; https://slack.com/help/articles/206845317-Create-a-Slack-workspace |
| focus and selection | Enter an email address and submit | Slack sends a verification challenge and shows the code state The recording advances to email verification pending and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-workspace-entry.png @ 2.25s; media/state-02-email-verification-pending.png @ 9.90s; https://slack.com/help/articles/206845317-Create-a-Slack-workspace |
| navigation | Enter the received verification code | Slack accepts the verified identity and advances to workspace details The navigation result is visible as identity verified. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-email-verification-pending.png @ 9.90s; media/state-03-identity-verified.png @ 17.55s; https://slack.com/help/articles/206845317-Create-a-Slack-workspace |
| confirmation | Name the company or team and answer the work-context prompt | Slack creates the workspace context and advances through setup The official recording shows the confirmed workspace configured state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-identity-verified.png @ 17.55s; media/state-04-workspace-configured.png @ 25.20s; https://slack.com/help/articles/206845317-Create-a-Slack-workspace |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-email-verification-pending.png @ 9.90s; media/state-03-identity-verified.png @ 17.55s; media/state-04-workspace-configured.png @ 25.20s; https://slack.com/help/articles/206845317-Create-a-Slack-workspace |
| progress feedback | Invite teammates or use the visible skip route | Slack records invitations or preserves the option for later Progress is observable as the distinct membership decision state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-workspace-configured.png @ 25.20s; media/state-05-membership-decision.png @ 32.85s; https://slack.com/help/articles/206845317-Create-a-Slack-workspace |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-identity-verified.png @ 17.55s; media/state-04-workspace-configured.png @ 25.20s; media/state-05-membership-decision.png @ 32.85s; https://slack.com/help/articles/206845317-Create-a-Slack-workspace |
| recovery and completion | Open the first channel and send a message | The message appears in the new workspace, proving first collaborative success The retained first message sent state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-membership-decision.png @ 32.85s; media/state-06-first-message-sent.png @ 40.50s; https://slack.com/help/articles/206845317-Create-a-Slack-workspace |

## Motion behavior

- **Trigger:** Playback of the retained 45-second reel. Inside it the only visible trigger is a drawn mouse pointer that lands on a control before the result appears: on `# project-unicorn` in the sidebar at 8.5-9.7s, on the composer's + button at 26.0s, and on the Done button of the open Record video clip panel at 28.0s.
- **Start state:** A plum title card holding only the Slack logo and wordmark, from the first frame at 0.0s through the frame retained at 2.25s.
- **End state:** A full-frame rotating burst of Slack brand colour wedges, from about 40.2s to the last frame at 45.0s, with no product surface left on screen.
- **Continuity:** Not continuous end to end. Six prepared scenes are joined partly by hard cuts and partly by animated transitions, and the animation inside a scene advances on consecutive frames: the clapping illustration at 23.5-24.3s and the outro wedges at 40.0-40.9s both change every frame, so the asset is genuine frame-by-frame motion rather than a slideshow. The two transitions that can be timed are the sidebar zoom into the open channel, 8.5s to 9.9s, and the outro wipe covering the product surface, 39.4s to 40.2s. No single product transition is shown from its start to its finish.
- **Timing class:** `one-to-three-seconds`
- **Interruption or reversal:** Not shown by the retained asset.
- **Feedback:** Feedback appears in the product surface itself: the selected channel row `# project-unicorn` is filled solid blue with a white label under the pointer at 9.0s, Arcadio's PDF post carries counted reaction chips at 22.0s, and the recorded clip posts back into the channel as a message with a `1:36` duration badge at 30.0s.
- **Reduced-motion equivalent:** Not shown by the retained asset.

## Accessibility

### Observations

- Every sidebar entry visible at 8.5-9.7s carries a text name beside its `#` glyph — `announcements`, `project-unicorn`, `team-design` — so channel identity never rests on an icon alone.
- Selection is carried by more than motion: at 9.0s the row for `# project-unicorn` is filled solid blue with a white label while its neighbours stay unfilled on the plum sidebar, and that fill is still readable in a paused frame.
- Contrast measured from media/state-02-email-verification-pending.png with ~/.stado/work/onboarding-auth-examples/contrast.py over the channel-header box at 95,84 size 150x18: the glyphs are #000000 on #ffffff paper, a ratio of 21.00:1.
- Attachments and clips are labelled in text next to their icons: `PDF` sits above the `Q1 Campaign` file card at 22.0s and `Video` above the clip thumbnail at 30.0s, whose length `1:36` is printed on the thumbnail rather than implied.
- No focus ring, focus outline, or keyboard hint appears in any of the 675 frames; every action in the reel is performed by a drawn mouse pointer, so the reel says nothing either way about the product's own focus indicator.

### Unknowns

- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The asset exposes no product-level reduced-motion preference, and no reduced-motion variant of it was retained beside it.

## Provenance and media integrity

- **Upstream owner:** Slack Technologies, LLC
- **Product page:** https://slack.com/help/articles/206845317-Create-a-Slack-workspace
- **Original media URL:** https://www.youtube.com/watch?v=FTuOS8E1LZk
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 45.000s, 675 frames, 499175 bytes
- **SHA-256:** `d4dc2bf2aacea7147902bb0d3947a7f01e0eeb892f1add35cff18d610ddaf3ef`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
