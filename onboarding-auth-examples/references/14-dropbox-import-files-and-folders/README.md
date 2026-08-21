# Dropbox — import files and folders

**Evidence status:** `complete`  
**Product/source:** [https://help.dropbox.com/create-upload/add-files](https://help.dropbox.com/create-upload/add-files)  
**Authentic motion:** [`media/official-recording.mp4`](media/official-recording.mp4)  
**Official recording:** [How to upload and download in Dropbox | Dropbox Tutorials | Dropbox](https://www.youtube.com/watch?v=mRd0tRVBvCw) — Dropbox

## Start-to-first-success journey

**Actor:** new Dropbox user  
**Goal:** upload a file and see it in cloud storage  
**Prerequisites:** Dropbox account; local file or folder

| # | User action | Product response | Retained state | Evidence |
|---:|---|---|---|---|
| 1 | Authenticate and open All files | Dropbox shows the file list and upload controls | file home | `media/state-01-file-home.png` and motion at 3.05s |
| 2 | Choose Upload files, Upload folder, or drag-and-drop | Dropbox opens the picker or accepts the dropped item | upload selection | `media/state-02-upload-selection.png` and motion at 13.43s |
| 3 | Select the file or folder | Dropbox queues the selected content | upload queued | `media/state-03-upload-queued.png` and motion at 23.82s |
| 4 | Confirm the destination when prompted | Dropbox starts transfer and shows progress feedback | uploading | `media/state-04-uploading.png` and motion at 34.20s |
| 5 | Wait for transfer completion | Dropbox removes progress state and adds the item to the list | upload complete | `media/state-05-upload-complete.png` and motion at 44.58s |
| 6 | Open the uploaded item | The cloud copy renders or downloads, proving first storage success | file available | `media/state-06-file-available.png` and motion at 54.96s |

### Failure and recovery

- **Failure:** At upload selection or upload queued, invalid, expired, denied, or missing required input leaves the flow short of file available; evidence: media/state-02-upload-selection.png, media/state-03-upload-queued.png, and https://help.dropbox.com/create-upload/add-files.
- **Failure:** Back, Cancel, Decline, or an explicit optional Skip interrupts setup without substituting a success state; evidence boundary: media/state-03-upload-queued.png through media/state-05-upload-complete.png.
- **Recovery:** Return to the retained upload selection or upload queued requirement, correct or resend the blocking input, and resubmit; evidence: https://help.dropbox.com/create-upload/add-files.
- **Recovery:** Continue through the same terminal action until file available is visible in media/state-06-file-available.png and the motion at 54.960s.
- **Completion evidence:** file available retained at media/state-06-file-available.png and media/official-recording.mp4#t=54.960; source https://help.dropbox.com/create-upload/add-files

## Retained product states

| State | Local frame | Relationship to motion | Dimensions | SHA-256 |
|---|---|---|---:|---|
| file home | [`media/state-01-file-home.png`](media/state-01-file-home.png) | media/official-recording.mp4#t=3.053 | 640×360 | `9f2c6369679666dd3fb9812c1278a45290e5cf447591dcf82f47240ef4feae71` |
| upload selection | [`media/state-02-upload-selection.png`](media/state-02-upload-selection.png) | media/official-recording.mp4#t=13.435 | 640×360 | `e5ec245e44fe21be8fe3a71df6a18cdfb9f4fbe5bae47588fd588cb0441f2791` |
| upload queued | [`media/state-03-upload-queued.png`](media/state-03-upload-queued.png) | media/official-recording.mp4#t=23.816 | 640×360 | `da1e7a2941d94b0842c8878a287873aad92c635b4b8e8fafd9fa99ee4486232e` |
| uploading | [`media/state-04-uploading.png`](media/state-04-uploading.png) | media/official-recording.mp4#t=34.198 | 640×360 | `ecd64dd8680fb4fdd9e191bbc7c272d80c5eb581d17d9d8c47e2cb66404ec312` |
| upload complete | [`media/state-05-upload-complete.png`](media/state-05-upload-complete.png) | media/official-recording.mp4#t=44.579 | 640×360 | `70e38377fd4d24c66d5a5bb1c6cb6685d181fc7aa815d1ae3371389d3312f810` |
| file available | [`media/state-06-file-available.png`](media/state-06-file-available.png) | media/official-recording.mp4#t=54.960 | 640×360 | `6c680f5f29f80de66edfe0592a379f52b62b6becd29260313cfe6430eb54f571` |

## Interaction map

| Interaction | Trigger | Response and feedback | Cancellation / failure / recovery | Evidence |
|---|---|---|---|---|
| primary input | Authenticate and open All files | Dropbox shows the file list and upload controls The retained file home state makes the available next action visible. | **Cancel:** Closing or leaving before submission produces no completion claim. **Failure:** Unavailable identity, permission, or prerequisite keeps the user at entry. **Recovery:** Restore the prerequisite and repeat the same entry action. | media/state-01-file-home.png @ 3.05s; https://help.dropbox.com/create-upload/add-files |
| focus and selection | Choose Upload files, Upload folder, or drag-and-drop | Dropbox opens the picker or accepts the dropped item The recording advances to upload selection and retains the selected context. | **Cancel:** Back returns to the preceding entry context when the product exposes it. **Failure:** An invalid, expired, or unavailable selection does not advance to the next recorded state. **Recovery:** Correct, resend, or reselect and submit again. | media/state-01-file-home.png @ 3.05s; media/state-02-upload-selection.png @ 13.43s; https://help.dropbox.com/create-upload/add-files |
| navigation | Select the file or folder | Dropbox queues the selected content The navigation result is visible as upload queued. | **Cancel:** The prior state remains the safe return target; optional work may be skipped only when shown. **Failure:** A denied authorization or missing required value leaves the destination unavailable. **Recovery:** Return to the preserved requirement and satisfy it before continuing. | media/state-02-upload-selection.png @ 13.43s; media/state-03-upload-queued.png @ 23.82s; https://help.dropbox.com/create-upload/add-files |
| confirmation | Confirm the destination when prompted | Dropbox starts transfer and shows progress feedback The official recording shows the confirmed uploading state. | **Cancel:** A cancel or back action before confirmation does not create the terminal result. **Failure:** Rejected confirmation or validation keeps the flow incomplete with the current requirement visible. **Recovery:** Correct the blocking value or repeat authorization, then confirm again. | media/state-03-upload-queued.png @ 23.82s; media/state-04-uploading.png @ 34.20s; https://help.dropbox.com/create-upload/add-files |
| cancellation and backtracking | Use Back, Cancel, Decline, or the product's explicit optional Skip route before the terminal action. | The flow returns to the preceding requirement or leaves optional work pending rather than fabricating success. The current-step surface and absence of terminal evidence make the interruption visible. | **Cancel:** This interaction is the observed cancellation boundary. **Failure:** Closing an unsaved or verification-pending step can leave onboarding incomplete. **Recovery:** Re-enter from the documented entry point and resume the last retained requirement. | media/state-02-upload-selection.png @ 13.43s; media/state-03-upload-queued.png @ 23.82s; media/state-04-uploading.png @ 34.20s; https://help.dropbox.com/create-upload/add-files |
| progress feedback | Wait for transfer completion | Dropbox removes progress state and adds the item to the list Progress is observable as the distinct upload complete state. | **Cancel:** Leaving during feedback does not count as completion. **Failure:** No success is claimed while the product still reports pending, building, verifying, uploading, or incomplete work. **Recovery:** Wait for durable feedback or use the product's retry route. | media/state-04-uploading.png @ 34.20s; media/state-05-upload-complete.png @ 44.58s; https://help.dropbox.com/create-upload/add-files |
| failure | Submit missing, invalid, expired, denied, or otherwise unacceptable required input at the current requirement. | The product remains short of the terminal state and exposes the requirement or failure feedback. The current state is retained; values already accepted by the product are not described as completed again. | **Cancel:** The user can leave without a false success state. **Failure:** The first-success outcome is unavailable until the blocking requirement is resolved. **Recovery:** Correct the value, resend the challenge, reconnect the provider, or retry the operation as appropriate. | media/state-03-upload-queued.png @ 23.82s; media/state-04-uploading.png @ 34.20s; media/state-05-upload-complete.png @ 44.58s; https://help.dropbox.com/create-upload/add-files |
| recovery and completion | Open the uploaded item | The cloud copy renders or downloads, proving first storage success The retained file available state is the completion evidence. | **Cancel:** After durable completion, leaving does not erase the recorded result; before it, leaving remains incomplete. **Failure:** If the terminal action fails, the user must remain on the actionable prior state rather than a false success page. **Recovery:** Return to the last durable state, retry the terminal action, and require the same completion evidence. | media/state-05-upload-complete.png @ 44.58s; media/state-06-file-available.png @ 54.96s; https://help.dropbox.com/create-upload/add-files |

## Motion behavior

- **Trigger:** The recorded sequence begins at file home; the first advancing trigger is “Choose Upload files, Upload folder, or drag-and-drop”.
- **Start/end:** Start is file home at 3.05s; end is file available at 54.96s.
- **Continuity:** The retained official tutorial contains real product interaction and may use editorial cuts. Cuts establish ordered product states but are not treated as evidence of uninterrupted transition duration.
- **Timing class:** The local evidence is 61.067s at 15 fps (916 frames); setup feedback ranges from immediate control-state changes to task-duration progress shown by the product.
- **Interruption/reversal:** Back, cancel, decline, and optional-skip behavior is mapped separately. An editorial cut is never interpreted as a reversible animation; after interruption, recovery must return to the last durable requirement.
- **Feedback:** Progress is evidenced by the six retained states, culminating in file available; pending or error feedback is not promoted to success.
- **Reduced motion/nonanimated equivalent:** The source does not expose a reduced-motion toggle. The six directly extracted PNG states provide the nonanimated equivalent while preserving order and labels.

## Accessibility

### Observations
- Visual observation from media/state-01-file-home.png and media/state-02-upload-selection.png: the current task is presented with persistent on-screen text rather than motion alone.
- Visual observation from media/state-03-upload-queued.png and media/state-04-uploading.png: primary choices remain spatially associated with their current setup context.
- The six local frames provide a nonanimated way to inspect the sequence without playing motion.

### Unknowns
- Screen-reader names, roles, state announcements, and error associations cannot be established from the retained official recording.
- Keyboard-only focus order and focus restoration after each transition were not exposed by the recording.
- Text scaling, zoom reflow, voice input, one-time-code autofill, and password-manager behavior remain unverified.
- The recording does not expose a product-level reduced-motion preference; the extracted state sequence is the retained nonanimated equivalent.

## Provenance and media integrity

- **Upstream owner:** Dropbox, Inc.
- **Product page:** https://help.dropbox.com/create-upload/add-files
- **Original media URL:** https://www.youtube.com/watch?v=mRd0tRVBvCw
- **Capture method:** official YouTube video downloaded through opt-in Cobalt instance https://dog.kittycat.boo; visual-only transcode to H.264 at 15 fps, at most 90 seconds, no synthesized frames
- **Captured at:** 2026-08-16T23:25:10Z
- **Media:** mp4, 640×360, 61.067s, 916 frames, 490228 bytes
- **SHA-256:** `337f9cb2fb5639744283ba05cc64772fefff95c2058f7aa1e60f91e1bc74d14d`

The local MP4 is a time-bounded, size-reduced transcode of the upstream owner’s real product recording. It was not interpolated, rebuilt from stills, or generated. The local PNG states were extracted directly from that MP4 at the recorded timestamps. Tutorial cuts are treated as state evidence, not as proof of unedited transition duration.
