# 27. Pixelmator Pro — observed macOS product reference

**Evidence status:** partial — the remaining gaps are that accessibility has never been measured against the running product, and that the retained clip is six hard cuts between frozen screens so it shows neither an interruption/reversal nor a reduced-motion equivalent  
**Product:** [https://www.pixelmator.com/pro/](https://www.pixelmator.com/pro/)  
**Motion source:** [https://www.youtube.com/watch?v=pcUOuy7nENI](https://www.youtube.com/watch?v=pcUOuy7nENI)  
**Upstream owner / recording owner:** Pixelmator Team  
**Captured:** 2026-08-16T23:27:18Z

<video controls src="media/motion.mp4" width="960"></video>

The local clip is authentic product motion, not animation synthesized from stills. Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames. Offline identity: `ae0a4be28caaea614340cf171f6012767903a24f631f2b4814c4797b7033a3b3` (147163 bytes, 960×540, 18.000s, 270 frames).

## First-success journey

**Actor:** A first-time or returning Pixelmator Pro user  
**Goal:** Create a Pixelmator Pro document  
**Prerequisites:** Pixelmator Pro available on macOS or the official recording environment; The product-specific input, document, account, repository, server, or project shown by the recording when required

| Step | Observed user action | Product response | Evidence |
|---:|---|---|---|
| 1 | Open the welcome window | Pixelmator Pro advances to the entry state retained in state 1. | `media/state-01.png extracted from media/motion.mp4 at 1.44s` |
| 2 | Choose a creation or open action | Pixelmator Pro advances to the invocation state retained in state 2. | `media/state-02.png extracted from media/motion.mp4 at 5.04s` |
| 3 | Select a template or asset | Pixelmator Pro advances to the focused intermediate state retained in state 3. | `media/state-03.png extracted from media/motion.mp4 at 8.64s` |
| 4 | Confirm the new document | Pixelmator Pro advances to the committed transition retained in state 4. | `media/state-04.png extracted from media/motion.mp4 at 12.24s` |
| 5 | Observe the created document canvas | Pixelmator Pro advances to the first-success result retained in state 5. | `media/state-05.png extracted from media/motion.mp4 at 15.84s` |

**Failure route:** Stop at the third retained state before confirmation; the meaningful result is absent. Treat a mismatched selection or incomplete input as non-success rather than inferring an unseen error.  
**Recovery route:** Return to the second retained state and restore the intended focus or selection. Repeat the fourth-state confirmation and verify the fifth-state completion evidence.  
**Completion evidence:** `media/state-05.png extracted from media/motion.mp4 at 15.84s`

## Retained product states

All states are direct frame extractions from `media/motion.mp4`. All five are different screens: the mean absolute difference between consecutive state PNGs is 34.407/255, 16.216/255, 14.717/255 and 18.679/255.

| State | Name | Local file | Motion time | SHA-256 |
|---:|---|---|---:|---|
| 1 | “Welcome to Pixelmator Pro” window: Version 3.3.7 Mosaic, three actions, recent files with Zahara selected | `media/state-01.png` | 1.44s | `716f65925d55c180531f1ef70809e2a9621ca215a060bbaa071d2833ab4122f6` |
| 2 | Template browser on “All Templates”: “Last Used / 1366 × 768” selected, Preset “Web Most Common”, Cancel and Create | `media/state-02.png` | 5.04s | `b72db491ff1e22f9db5cbe2b47b697465d929c707de379b07ec433829e50c922` |
| 3 | Template browser on “Recents”: “Recent Blank Documents” — Web Most Common (selected), Cinema 2K, A4 Paper — and “Clear Recents” | `media/state-03.png` | 8.64s | `ccf20f67d94d0ecdada5006d046f819197672cc68d9382ee1ebf03a8d0fc4c55` |
| 4 | Template browser on “Collections”: tile grid (Book Club, Whitepine, Design Fair, Jane Smith …) with the Billboard inspector, 1600 × 1200, 14.5 MB | `media/state-04.png` | 12.24s | `0b565dd65b0b7f471ba96e50c1ccd9ccced8fc85104fb6d333ddcbb180298144` |
| 5 | Template browser on “Mockups”: Devices grid with MacBook Pro selected, inspector “MacBook Pro 14-in.”, 6000 × 4500, 1.6 MB | `media/state-05.png` | 15.84s | `eb52a54e5a4019f8d5eb0ac05b00135790161a58de0bd9e361200346685cebc3` |

## Interaction map

| Interaction | Trigger | Response / feedback | Failure | Recovery |
|---|---|---|---|---|
| Primary input | Open the welcome window | Pixelmator Pro exposes the entry surface visible in the retained motion. | The goal is not yet complete in the entry state. | Choose a creation or open action. |
| Focus and selection | Choose a creation or open action | A target control, item, document, or workspace becomes the active context. | An unintended target would leave the desired result unavailable. | Move focus to the intended visible target and continue. |
| Navigation | Select a template or asset | The recording advances into the product’s intermediate working state. | Stopping at this state produces no completion evidence. | Confirm the new document. |
| Confirmation | Confirm the new document | The selected operation crosses from preparation into a committed transition. | Without confirmation, the fifth-state result does not appear. | Repeat the intended selection and confirm it. |
| Completion feedback | Observe the created document canvas | The recording reaches the first meaningful result for “Create a Pixelmator Pro document”. | Absence of the fifth state means the journey is incomplete. | Replay the recorded sequence from the entry state. |
| Cancellation and backtracking | Leave the path before the committed transition. | The visible intermediate surface remains non-destructive until the confirmation boundary shown by the excerpt. | Exiting early does not produce completion evidence. | Re-enter through state 2 and continue to state 4. |
| Failure boundary | Use an incomplete, unselected, or unconfirmed intermediate state. | The meaningful result is absent; the recording still shows the preparation surface. | The observed boundary is non-completion rather than an invented error dialog. | Restore the intended selection and cross the recorded confirmation boundary. |
| Recovery | Resume from the last valid intermediate state. | The same visible action sequence advances to the retained result. | A replay that never reaches state 5 remains incomplete. | Replay state 4 and verify state 5. |

Cancellation and backtracking are bounded at the pre-confirmation states; no unseen error dialog, undo behavior, or recovery animation is claimed.

## Motion analysis

- **Trigger:** No trigger is visible. The arrow pointer sits over the “Framed Poster” thumbnail at 4s, over the “Ocean Of Life” sidebar row at 11s and over the “Mockups” sidebar row at 16s, but each new screen is already fully drawn on the frame after the cut, and no pressed control, click ripple, cursor-down state or pointer travel between the two positions is recorded.
- **Start state:** The “Welcome to Pixelmator Pro” window (Version 3.3.7 Mosaic) centred on the macOS Ventura desktop with the Dock visible, its recent-file list showing Zahara selected, and the menu bar clock reading “Thu Jun 29 9:41 AM”.
- **End state:** The new-image template browser on the “Mockups” category, MacBook Pro selected in the Devices grid and the inspector reading “MacBook Pro 14-in.”, 6000 × 4500, Display P3, 1.6 MB. The clip never reaches an open document canvas — no image window, no tools sidebar and no layers list appears in any of the 270 frames.
- **Continuity:** Not one continuous take. The per-frame change profile over all 269 frame pairs is quiet in 263 of them (below 0.2/255) and spikes in exactly six: 3.93s (34.46/255), 7.33s (16.14), 9.13s (18.88), 11.07s (14.49), 13.53s (14.17) and 15.73s (17.8). Those six single-frame spikes are hard cuts between seven otherwise frozen screens — Welcome, All Templates, Recents, What’s New, Collections, Social Media, Mockups — with a mean change of 0.438/255 across the clip. The desktop wallpaper, Dock and the 9:41 AM clock are identical in every frame.
- **Timing class:** instant
- **Interruption / reversal:** Not shown by the retained asset.
- **Feedback:** The only feedback the clip contains is the replaced screen itself. Within each of the seven segments nothing at all moves — no spinner, no progress bar, no thumbnail loading placeholder, no hover highlight following the pointer — and the selection marks (blue thumbnail outline, blue caption pill, lighter sidebar bar) are already final in the first frame after each cut.
- **Reduced-motion equivalent:** Not shown by the retained asset.

## Accessibility

Observed:

- Every row of the Templates sidebar pairs a small icon with a readable text label — “All Templates”, “Recents”, “What’s New”, “Collections”, “Social Media”, “Print”, “Video”, “Logo”, “Resume”, “Mockups” — and every thumbnail in the browsing pane carries a name plus its size caption (“Web Most Common / 1366 × 768”, “Cinema 2K / 2048 × 1080”, “A4 Paper / 210 × 297 mm”), so no target in these frames is icon-only.
- Selection is doubly encoded and readable from one still: in `media/state-02.png` and `media/state-03.png` the chosen thumbnail carries a blue rounded outline and its caption is redrawn as white text in a filled blue pill (pill fill sampled at `#1C6CDE` in a 60×11 px crop at x=302,y=181 of `media/state-03.png`), while the active sidebar row is drawn as a lighter filled bar — the “Mockups” row in `media/state-05.png` with the arrow pointer over it.
- Contrast measured from `media/state-01.png`: in a 120×20 px crop at x=350,y=150 the “Welcome to Pixelmator Pro” title glyphs are `#EBEBEB` on the `#212121` panel, a WCAG contrast ratio of 13.51:1; the “Version 3.3.7 Mosaic” line one row down, in a 100×10 px crop at x=375,y=186, is only `#8D8D8D` on `#1B1B1B`, 5.19:1, so the two text tiers in the same window differ by more than a factor of two.
- State is carried by printed text rather than by movement: each of the seven screens in the clip names itself in the window’s title position (“All Templates”, “Recents”, “Collections”, “Social Media”, “Mockups”), and the right-hand inspector spells the consequence of the selection out in words (“Size 6000 × 4500”, “Color Profile Display P3”, “File Size 1.6 MB” in `media/state-05.png`).
- No keyboard shortcut, key hint, tooltip or focus ring appears in any of the 270 frames; the menu bar (Pixelmator Pro, File, Edit, Insert, Image, Tools, Format, Arrange, View, Window, Help) is never opened, so no shortcut is printed anywhere in the excerpt.

Unknown:

- VoiceOver announcements and accessible names are not audible in the retained excerpt.
- Full Keyboard Access order is not proven unless explicitly visible in the recording.
- The product’s Reduce Motion behavior and contrast preferences are not demonstrated.
- Keyboard focus is not observable: no focus ring appears on any control in the 270 frames, and because the clip is six hard cuts between frozen screens there is no tab traversal to read an order from.
- The template-thumbnail captions and the inspector labels are rendered at 6–8 px in this 960×540 transcode, so their exact glyph colours cannot be sampled cleanly and only the larger welcome-window text was measurable for contrast.

## Provenance

- **Product page:** https://www.pixelmator.com/pro/
- **Original motion:** https://www.youtube.com/watch?v=pcUOuy7nENI
- **Capture method:** Downloaded from the product publisher’s official video channel with yt-dlp, then excerpted and transcoded to H.264 without synthesized frames.
- **Captured at:** 2026-08-16T23:27:18Z
- **Media metadata:** 960×540; 18.000s; 270 frames; 147163 bytes
- **SHA-256:** `ae0a4be28caaea614340cf171f6012767903a24f631f2b4814c4797b7033a3b3`
- **Ownership:** Pixelmator Team. Product and recording rights remain with their respective upstream owners.
