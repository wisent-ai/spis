# Airtable — onboarding

**Evidence status:** `complete`  
**Product/source:** [https://support.airtable.com/docs/getting-started-with-airtable](https://support.airtable.com/docs/getting-started-with-airtable)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to Use Airtable & Getting Started Tutorial](https://www.youtube.com/watch?v=pRUB4nnUp9o) — Airtable

## Start-to-first-success journey

**Actor:** new Airtable builder  
**Goal:** create a base and persist the first record  
**Prerequisites:** Airtable account; workflow or dataset idea

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and enter Airtable Home | Airtable shows workspaces, bases, and creation controls | home | `media/state-01-home.png` and motion at 4.50s |
| 2 | Choose to create a base | Airtable presents template, import, and blank-base routes | base creation | `media/state-02-base-creation.png` and motion at 19.79s |
| 3 | Select a template, import source, or blank base | Airtable creates an initial table structure | table ready | `media/state-03-table-ready.png` and motion at 35.07s |
| 4 | Name the base and table | Airtable persists the new workspace object | base named | `media/state-04-base-named.png` and motion at 50.36s |
| 5 | Enter or import the first row | Airtable validates cell values and saves the record | record entered | `media/state-05-record-entered.png` and motion at 65.65s |
| 6 | Return to the base overview | The populated base remains available, proving first workflow success | base persisted | `media/state-06-base-persisted.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At base creation or table ready, invalid, expired, denied, or missing required input leaves the flow short of base persisted; evidence: media/state-02-base-creation.png, media/state-03-table-ready.png, and https://support.airtable.com/docs/getting-started-with-airtable.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-table-ready.png through media/state-05-record-entered.png.
- **Recovery:** Return to the retained base creation or table ready requirement, correct or resend the blocking input, and resubmit; evidence: https://support.airtable.com/docs/getting-started-with-airtable.
- **Recovery:** Continue through the same terminal action until base persisted is visible in media/state-06-base-persisted.png and the motion at 80.940s.
- **Completion evidence:** base persisted retained at media/state-06-base-persisted.png and media/official-recording.mp4#t=80.940; source https://support.airtable.com/docs/getting-started-with-airtable

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| home | [`media/state-01-home.png`](media/state-01-home.png) | media/official-recording.mp4#t=4.497 | 640×360 | `84fae5122146ed75767648d71a2d90762f0c3265d1a13ed68190ce95e670a8aa` |
| base creation | [`media/state-02-base-creation.png`](media/state-02-base-creation.png) | media/official-recording.mp4#t=19.785 | 640×360 | `d7b72c92350fe910b49e06565e58a445a5ab54d802d5d233f53116f8bfd78d8c` |
| table ready | [`media/state-03-table-ready.png`](media/state-03-table-ready.png) | media/official-recording.mp4#t=35.074 | 640×360 | `a20206708365592c8b6c2eb348c168110e8c88e7fb91948bdb0f5ef6016be51b` |
| base named | [`media/state-04-base-named.png`](media/state-04-base-named.png) | media/official-recording.mp4#t=50.362 | 640×360 | `8d5d98f67aeacaa35e962e016fce92df1fca3b050686b8664a8bd9ba4ad66ece` |
| record entered | [`media/state-05-record-entered.png`](media/state-05-record-entered.png) | media/official-recording.mp4#t=65.651 | 640×360 | `b5fc536ee6268d01e78983f43abbaad82f0bacf715d6fa24ed811defbe397a91` |
| base persisted | [`media/state-06-base-persisted.png`](media/state-06-base-persisted.png) | media/official-recording.mp4#t=80.940 | 640×360 | `c6c0afcb512954ff7419c52dcdf4788c39595fb0ff9e6a5f86e1028d7c8c33d0` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and enter Airtable Home | Airtable shows workspaces, bases, and creation controls The retained home state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-home.png @ 4.50s; https://support.airtable.com/docs/getting-started-with-airtable |
| focus and selection | Choose to create a base | Airtable presents template, import, and blank-base routes The recording advances to base creation and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-home.png @ 4.50s; media/state-02-base-creation.png @ 19.79s; https://support.airtable.com/docs/getting-started-with-airtable |
| navigation | Select a template, import source, or blank base | Airtable creates an initial table structure The navigation result is visible as table ready. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-base-creation.png @ 19.79s; media/state-03-table-ready.png @ 35.07s; https://support.airtable.com/docs/getting-started-with-airtable |
| confirmation | Name the base and table | Airtable persists the new workspace object The official recording shows the confirmed base named state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-table-ready.png @ 35.07s; media/state-04-base-named.png @ 50.36s; https://support.airtable.com/docs/getting-started-with-airtable |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-base-creation.png @ 19.79s; media/state-03-table-ready.png @ 35.07s; media/state-04-base-named.png @ 50.36s; https://support.airtable.com/docs/getting-started-with-airtable |
| progress feedback | Enter or import the first row | Airtable validates cell values and saves the record Progress is observable as the distinct record entered state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-base-named.png @ 50.36s; media/state-05-record-entered.png @ 65.65s; https://support.airtable.com/docs/getting-started-with-airtable |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-table-ready.png @ 35.07s; media/state-04-base-named.png @ 50.36s; media/state-05-record-entered.png @ 65.65s; https://support.airtable.com/docs/getting-started-with-airtable |
| recovery and completion | Return to the base overview | The populated base remains available, proving first workflow success The retained base persisted state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-record-entered.png @ 65.65s; media/state-06-base-persisted.png @ 80.94s; https://support.airtable.com/docs/getting-started-with-airtable |

## Motion behavior

- **Trigger:** The recorded sequence begins at home; the first advancing trigger is “Choose to create a base”.
- **Start/end:** Start is home at 4.50s; end is base persisted at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in base persisted; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-home.png and media/state-02-base-creation.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-table-ready.png and media/state-04-base-named.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Airtable
- **Product page:** https://support.airtable.com/docs/getting-started-with-airtable
- **Original media URL:** https://www.youtube.com/watch?v=pRUB4nnUp9o
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 494883 bytes
- **SHA-256:** `93af97cf4478dcaa100a38d9e0cb02967e533d3df7d2f97dc194d38d66d733b0`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
