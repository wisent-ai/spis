# Miro — getting started

**Evidence status:** `complete`  
**Product/source:** [https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics](https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [Miro 101: Create Your First Board (2026)](https://www.youtube.com/watch?v=sxqdN2eD59Q) — Miro

## Start-to-first-success journey

**Actor:** new Miro board owner  
**Goal:** create a board and place the first object  
**Prerequisites:** Miro account; team or workspace context

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and enter the Miro dashboard | Miro shows team spaces and board creation controls | dashboard | `media/state-01-dashboard.png` and motion at 4.50s |
| 2 | Choose to create a new board | Miro opens the template or blank-board choice | board creation | `media/state-02-board-creation.png` and motion at 19.79s |
| 3 | Select a blank board or starter template | Miro loads the editable canvas | canvas ready | `media/state-03-canvas-ready.png` and motion at 35.07s |
| 4 | Name the board | Miro persists the board identity in the workspace | board named | `media/state-04-board-named.png` and motion at 50.36s |
| 5 | Add a note, shape, or text object | Miro places the object on the canvas with editing feedback | object added | `media/state-05-object-added.png` and motion at 65.65s |
| 6 | Leave the board or share it from the saved state | The board remains in the dashboard, proving first persisted result | board saved | `media/state-06-board-saved.png` and motion at 80.94s |

### Failure and recovery

- **Failure:** At board creation or canvas ready, invalid, expired, denied, or missing required input leaves the flow short of board saved; evidence: media/state-02-board-creation.png, media/state-03-canvas-ready.png, and https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-canvas-ready.png through media/state-05-object-added.png.
- **Recovery:** Return to the retained board creation or canvas ready requirement, correct or resend the blocking input, and resubmit; evidence: https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics.
- **Recovery:** Continue through the same terminal action until board saved is visible in media/state-06-board-saved.png and the motion at 80.940s.
- **Completion evidence:** board saved retained at media/state-06-board-saved.png and media/official-recording.mp4#t=80.940; source https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| dashboard | [`media/state-01-dashboard.png`](media/state-01-dashboard.png) | media/official-recording.mp4#t=4.497 | 640×360 | `070b40f82490f2d725d1864cfa9fdc545163f549db03af6e75acc6e331a81e4c` |
| board creation | [`media/state-02-board-creation.png`](media/state-02-board-creation.png) | media/official-recording.mp4#t=19.785 | 640×360 | `d1133aa30bc86765ca7c9e7a7a1abe1ffa11a00565b1b03c1f4a01c5b4458cff` |
| canvas ready | [`media/state-03-canvas-ready.png`](media/state-03-canvas-ready.png) | media/official-recording.mp4#t=35.074 | 640×360 | `53db6cc681046ab03619a7fc739f1eea1583c53e8a654f61f86997481ad06eaf` |
| board named | [`media/state-04-board-named.png`](media/state-04-board-named.png) | media/official-recording.mp4#t=50.362 | 640×360 | `24dac9683946657cd097c3dbf177e525156cbb34d4f7b4a429a59e0a57e5ad17` |
| object added | [`media/state-05-object-added.png`](media/state-05-object-added.png) | media/official-recording.mp4#t=65.651 | 640×360 | `f7beb51f82e568086ef848edbfce08a68cbc77ccb3b0525581c52e7640827c1b` |
| board saved | [`media/state-06-board-saved.png`](media/state-06-board-saved.png) | media/official-recording.mp4#t=80.940 | 640×360 | `55ee619e8a0d7421f02e8b849ea6c9ae1c46b22c5faf9ee6f4e324f769ab368e` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and enter the Miro dashboard | Miro shows team spaces and board creation controls The retained dashboard state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-dashboard.png @ 4.50s; https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics |
| focus and selection | Choose to create a new board | Miro opens the template or blank-board choice The recording advances to board creation and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-dashboard.png @ 4.50s; media/state-02-board-creation.png @ 19.79s; https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics |
| navigation | Select a blank board or starter template | Miro loads the editable canvas The navigation result is visible as canvas ready. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-board-creation.png @ 19.79s; media/state-03-canvas-ready.png @ 35.07s; https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics |
| confirmation | Name the board | Miro persists the board identity in the workspace The official recording shows the confirmed board named state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-canvas-ready.png @ 35.07s; media/state-04-board-named.png @ 50.36s; https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-board-creation.png @ 19.79s; media/state-03-canvas-ready.png @ 35.07s; media/state-04-board-named.png @ 50.36s; https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics |
| progress feedback | Add a note, shape, or text object | Miro places the object on the canvas with editing feedback Progress is observable as the distinct object added state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-board-named.png @ 50.36s; media/state-05-object-added.png @ 65.65s; https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-canvas-ready.png @ 35.07s; media/state-04-board-named.png @ 50.36s; media/state-05-object-added.png @ 65.65s; https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics |
| recovery and completion | Leave the board or share it from the saved state | The board remains in the dashboard, proving first persisted result The retained board saved state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-object-added.png @ 65.65s; media/state-06-board-saved.png @ 80.94s; https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics |

## Motion behavior

- **Trigger:** The recorded sequence begins at dashboard; the first advancing trigger is “Choose to create a new board”.
- **Start/end:** Start is dashboard at 4.50s; end is board saved at 80.94s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 89.933s at 15 fps (1349 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in board saved; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-dashboard.png and media/state-02-board-creation.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-canvas-ready.png and media/state-04-board-named.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Miro
- **Product page:** https://help.miro.com/hc/en-us/articles/360017730233-Miro-basics
- **Original media URL:** https://www.youtube.com/watch?v=sxqdN2eD59Q
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 89.933s, 1349 frames, 827270 bytes
- **SHA-256:** `f7d9125be9d645d5d25767e41aadeef2fdfc0b25639634a8276a29998e59eb52`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
