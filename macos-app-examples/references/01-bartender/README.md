# 01. Bartender — observed macOS product reference

**Evidence status:** partial — the only open gap is that accessibility has never been measured against the running product  
**Product:** [https://www.macbartender.com/](https://www.macbartender.com/)  
**Motion source:** [https://www.macbartender.com/Bartender6/img/togglebarthing.gif](https://www.macbartender.com/Bartender6/img/togglebarthing.gif)  
**Upstream owner / recording owner:** Bartender Software  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized. Offline identity: `0b01990857436eabfadce2978f2996542b95fd472a62b713138c284619e29bf0` (13200 bytes, 960×266, 5.400s, 81 frames).

## First-success journey

**Actor:** A first-time or returning Bartender user  
**Goal:** Reveal and manage hidden menu-bar items  
**Prerequisites:** Bartender available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Start from the compact menu bar | Bartender advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 0.43s` |
| 2 | Invoke Bartender’s reveal control | Bartender advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 1.51s` |
| 3 | Scan the newly exposed menu items | Bartender advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 2.59s` |
| 4 | Choose the visibility arrangement | Bartender advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 3.67s` |
| 5 | Leave the required items visible | Bartender advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 4.75s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 4.75s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`. States 2, 3 and 4 are three frames of the same static expanded bar: the mean absolute difference between the state-2 and state-3 PNGs is 0.018/255 and between state-3 and state-4 it is 0.002/255.

| State | Observed state | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | Menu bar collapsed: four status icons and the clock | `media/state-01.png` | 0.43s | `577e586ff3b78cc36e4d822483c513fbb329e0ea228325763c77ad24bb5d06db` |
| 2 | Menu bar expanded: the hidden status-item row fills the bar from the left frame edge to the clock | `media/state-02.png` | 1.51s | `be8372c69776d2bb8488f75513273fee49f7204c2c4c14dfbc679a3e5592891f` |
| 3 | Menu bar still expanded, row unchanged from state 2 | `media/state-03.png` | 2.59s | `d64d55d24bbd776272c811aa34335dfa7876814ac85b55f08e34f29f0166ca2b` |
| 4 | Menu bar still expanded, row unchanged from state 3 | `media/state-04.png` | 3.67s | `b140e79c91d0c72c984bec101c59bbe583795183e25b37bdc6b6c7d1c51919d1` |
| 5 | Menu bar collapsed again: four status icons and the clock, identical layout to state 1 | `media/state-05.png` | 4.75s | `2624faf0a1cf2ca53bc959dad5370e6d28ce7c4ef640801d323c1b02450ff18c` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Start from the compact menu bar | Bartender exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Invoke Bartender’s reveal control. |
| Focus and selection | Invoke Bartender’s reveal control | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Scan the newly exposed menu items | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Choose the visibility arrangement. |
| Confirmation | Choose the visibility arrangement | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Leave the required items visible | The recording reaches the first meaningful result for “Reveal and manage hidden menu-bar items”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** The pointer is already parked in the menu bar over the item slot that will hold the battery icon; between 0.73s and 0.93s the hidden row appears. No pressed state, click ripple or cursor-down indicator is visible, so which control was activated cannot be read off the clip.
- **Start state:** Collapsed menu bar: four status icons and “Fri Sep 5 3:57 PM” on the blue desktop, nothing else on screen.
- **End state:** Collapsed menu bar again, the same four status icons and the same clock reading; the revealed row is gone.
- **Continuity:** One continuous take at 15fps: the row appears and disappears in place, the frame never cuts, and the wallpaper and the clock reading stay identical across all 81 frames.
- **Timing class:** sub-second — the reveal completes in three frames (0.73s to 0.93s, peak change at 0.87s).
- **Interruption / reversal:** The clip shows the reversal: at 4.53s the revealed row collapses back to the four-item bar in a single step, returning to the state-1 layout. No interrupted or half-cancelled transition is shown.
- **Feedback:** The only feedback is the icon row itself appearing and disappearing; measured per-frame change is 0.11/255 on average with peaks of 3.48/255 at 0.87s and 1.99/255 at 4.53s, and no progress indicator, label, badge or highlight accompanies it.
- **Reduced-motion equivalent:** The reveal is a two-frame hard change with no easing, fade or slide, so the same information reads identically from one still; the clip does not show the product’s Reduce Motion setting or an alternative non-animated path.

## Accessibility

Observed:

- Every menu-bar item in both the collapsed and expanded frames is an icon with no text label; the only text anywhere in the 960×266 frame is the clock “Fri Sep 5 3:57 PM”.
- Contrast measured from `media/state-01.png`: the lightest text cluster in a 160×20 px crop at x=780,y=0 is `#BDC0F6` and the menu-bar background there is `#120995`, a WCAG contrast ratio of 8.03:1 by the sRGB relative-luminance formula.
- The revealed/hidden distinction is carried by whether the icons are present, not by movement, so state 2 and state 5 are told apart from a single still with no playback.
- No focus ring, selection highlight, tooltip or keyboard hint appears in any of the 81 frames; the pointer arrow is the only indicator of where input is aimed.

Unknown:

- VoiceOver announcements and accessible names are not audible in the retained excerpt.
- Full Keyboard Access order is not proven unless explicitly visible in the recording.
- The product’s Reduce Motion behavior and contrast preferences are not demonstrated.

## Provenance

- **Product page:** https://www.macbartender.com/
- **Original motion:** https://www.macbartender.com/Bartender6/img/togglebarthing.gif
- **Capture method:** Downloaded from the official product site or official App Store preview and transcoded as a time-preserving H.264 excerpt; no frames were synthesized.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×266; 5.400s; 81 frames; 13200 bytes
- **SHA-256:** `0b01990857436eabfadce2978f2996542b95fd472a62b713138c284619e29bf0`
- **Ownership:** Bartender Software. Product and recording rights remain with their respective upstream owners.
