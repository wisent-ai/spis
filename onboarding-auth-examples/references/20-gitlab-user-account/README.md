# GitLab — user account

**Evidence status:** `complete`  
**Product/source:** [https://docs.gitlab.com/user/profile/account/create_accounts/](https://docs.gitlab.com/user/profile/account/create_accounts/)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Create a sample project in GitLab](https://www.youtube.com/watch?v=B4PzPz_J9-E) — GitLab Unfiltered

## Start-to-first-success journey

**Actor:** new GitLab developer  
**Goal:** activate an account and create the first project  
**Prerequisites:** email or approved external identity; project name

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Register or accept an organization invitation | GitLab creates a pending user identity | account pending | `media/state-01-account-pending.png` and motion at 3.25s |
| 2 | Complete email or external-provider verification | GitLab activates the user and returns to the product | identity verified | `media/state-02-identity-verified.png` and motion at 14.31s |
| 3 | Set the profile and namespace | GitLab exposes the effective project namespace | namespace ready | `media/state-03-namespace-ready.png` and motion at 25.38s |
| 4 | Choose New project | GitLab presents blank, import, and template routes | project creation | `media/state-04-project-creation.png` and motion at 36.44s |
| 5 | Name and configure project visibility | GitLab validates the path and settings | project configured | `media/state-05-project-configured.png` and motion at 47.50s |
| 6 | Create the project and add the first file or commit | The repository renders the commit, proving first project success | first commit | `media/state-06-first-commit.png` and motion at 58.56s |

### Failure and recovery

- **Failure:** At identity verified or namespace ready, invalid, expired, denied, or missing required input leaves the flow short of first commit; evidence: media/state-02-identity-verified.png, media/state-03-namespace-ready.png, and https://docs.gitlab.com/user/profile/account/create_accounts/.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-namespace-ready.png through media/state-05-project-configured.png.
- **Recovery:** Return to the retained identity verified or namespace ready requirement, correct or resend the blocking input, and resubmit; evidence: https://docs.gitlab.com/user/profile/account/create_accounts/.
- **Recovery:** Continue through the same terminal action until first commit is visible in media/state-06-first-commit.png and the motion at 58.560s.
- **Completion evidence:** first commit retained at media/state-06-first-commit.png and media/official-recording.mp4#t=58.560; source https://docs.gitlab.com/user/profile/account/create_accounts/

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| account pending | [`media/state-01-account-pending.png`](media/state-01-account-pending.png) | media/official-recording.mp4#t=3.253 | 640×306 | `1b36ec3cb6c623917ef3da99d05b32af62cdd98caaf73990d7f48b023a421992` |
| identity verified | [`media/state-02-identity-verified.png`](media/state-02-identity-verified.png) | media/official-recording.mp4#t=14.315 | 640×306 | `0f261fcb6bfbc46e1795550c994516b493a7f648c74033f9ca303e6e178a95cc` |
| namespace ready | [`media/state-03-namespace-ready.png`](media/state-03-namespace-ready.png) | media/official-recording.mp4#t=25.376 | 640×306 | `f8cada1d24c0cea1a50bc5a8458547b143603071de3bf14918b5c4e69e92d835` |
| project creation | [`media/state-04-project-creation.png`](media/state-04-project-creation.png) | media/official-recording.mp4#t=36.438 | 640×306 | `65277daca6ec7aa69ed64c19736b31ebf4e9c53aafdbdbe5ac551332853dd1f4` |
| project configured | [`media/state-05-project-configured.png`](media/state-05-project-configured.png) | media/official-recording.mp4#t=47.499 | 640×306 | `b7fa04d2840dd064c5ea0e39909befe04a0bb69f443afd83397637d4c5f92ac1` |
| first commit | [`media/state-06-first-commit.png`](media/state-06-first-commit.png) | media/official-recording.mp4#t=58.560 | 640×306 | `fc6bb886e108b1542bb37864bb6740a43db542dd987cc3f15b7c81bc2a5ac5e1` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Register or accept an organization invitation | GitLab creates a pending user identity The retained account pending state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-account-pending.png @ 3.25s; https://docs.gitlab.com/user/profile/account/create_accounts/ |
| focus and selection | Complete email or external-provider verification | GitLab activates the user and returns to the product The recording advances to identity verified and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-account-pending.png @ 3.25s; media/state-02-identity-verified.png @ 14.31s; https://docs.gitlab.com/user/profile/account/create_accounts/ |
| navigation | Set the profile and namespace | GitLab exposes the effective project namespace The navigation result is visible as namespace ready. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-identity-verified.png @ 14.31s; media/state-03-namespace-ready.png @ 25.38s; https://docs.gitlab.com/user/profile/account/create_accounts/ |
| confirmation | Choose New project | GitLab presents blank, import, and template routes The official recording shows the confirmed project creation state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-namespace-ready.png @ 25.38s; media/state-04-project-creation.png @ 36.44s; https://docs.gitlab.com/user/profile/account/create_accounts/ |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-identity-verified.png @ 14.31s; media/state-03-namespace-ready.png @ 25.38s; media/state-04-project-creation.png @ 36.44s; https://docs.gitlab.com/user/profile/account/create_accounts/ |
| progress feedback | Name and configure project visibility | GitLab validates the path and settings Progress is observable as the distinct project configured state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-project-creation.png @ 36.44s; media/state-05-project-configured.png @ 47.50s; https://docs.gitlab.com/user/profile/account/create_accounts/ |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-namespace-ready.png @ 25.38s; media/state-04-project-creation.png @ 36.44s; media/state-05-project-configured.png @ 47.50s; https://docs.gitlab.com/user/profile/account/create_accounts/ |
| recovery and completion | Create the project and add the first file or commit | The repository renders the commit, proving first project success The retained first commit state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-project-configured.png @ 47.50s; media/state-06-first-commit.png @ 58.56s; https://docs.gitlab.com/user/profile/account/create_accounts/ |

## Motion behavior

- **Trigger:** The recorded sequence begins at account pending; the first advancing trigger is “Complete email or external-provider verification”.
- **Start/end:** Start is account pending at 3.25s; end is first commit at 58.56s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 65.067s at 15 fps (976 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in first commit; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-account-pending.png and media/state-02-identity-verified.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-namespace-ready.png and media/state-04-project-creation.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** GitLab Inc.
- **Product page:** https://docs.gitlab.com/user/profile/account/create_accounts/
- **Original media URL:** https://www.youtube.com/watch?v=B4PzPz_J9-E
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×306, 65.067s, 976 frames, 265770 bytes
- **SHA-256:** `98fa4d62c889564fcd25b51b868bdfed2d4b36d78ad8045fc7cf4867e535f79e`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
