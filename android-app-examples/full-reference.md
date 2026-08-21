# Full Android product interaction reference

This synthesis is derived from all 50 inspectable per-product records in [`references.json`](references.json). It is not a substitute for them: each row below links to a local playable motion asset, three retained states, an observed six-state path, eight interaction entries, recovery evidence, accessibility unknowns, and byte-level provenance.

## Coverage

- **50/50 records complete** under `wisent.full-product-reference.v1`.
- Every record retains one local MP4 and three JPEG frames extracted from that MP4.
- Every journey contains six ordered observed states; every interaction map covers primary input, focus, navigation, configuration, confirmation, cancellation, feedback, failure, and recovery.
- Source ownership is explicit. Product-owner/documentation recordings are preferred; where an owner did not publish a compact accessible Android clip, the record names the real screen-recording publisher rather than mislabeling it as official.

## All 50 records

| # | Product | Category | Record | Motion source |
|---:|---|---|---|---|
| 1 | Google Keep | Productivity | [`reference.json`](references/01-google-keep/reference.json) | [Quick Tutorials](https://www.youtube.com/watch?v=-nsvD3Zs3TQ) |
| 2 | Todoist | Productivity | [`reference.json`](references/02-todoist/reference.json) | [TutorialStream](https://www.youtube.com/watch?v=PvzwWq2bdhA) |
| 3 | Notion | Productivity | [`reference.json`](references/03-notion/reference.json) | [MoneyPro-Tips](https://www.youtube.com/watch?v=-kr8br1K3cg) |
| 4 | Trello | Productivity | [`reference.json`](references/04-trello/reference.json) | [Trello](https://www.youtube.com/watch?v=MKvh0bhrWwg) |
| 5 | Microsoft 365 Copilot | Productivity | [`reference.json`](references/05-microsoft-365-copilot/reference.json) | [ClickFix](https://www.youtube.com/watch?v=Vr-LA3lhelo) |
| 6 | Google Calendar | Productivity | [`reference.json`](references/06-google-calendar/reference.json) | [Google](https://www.youtube.com/watch?v=MSTmkvn060E) |
| 7 | TickTick | Productivity | [`reference.json`](references/07-ticktick/reference.json) | [TickTick](https://www.youtube.com/watch?v=MlqBZKMAskk) |
| 8 | Evernote | Productivity | [`reference.json`](references/08-evernote/reference.json) | [Evernote](https://www.youtube.com/watch?v=QgAiJs-gyP4) |
| 9 | WhatsApp Messenger | Communication | [`reference.json`](references/09-whatsapp-messenger/reference.json) | [WhatsApp](https://www.youtube.com/watch?v=QkvlYq9TazU) |
| 10 | Signal Private Messenger | Communication | [`reference.json`](references/10-signal-private-messenger/reference.json) | [CardChronicles](https://www.youtube.com/watch?v=ErJLrYozqP8) |
| 11 | Telegram | Communication | [`reference.json`](references/11-telegram/reference.json) | [Telegram](https://telegram.org/file/400780400904/4/DWawl_4TMA4.10093975.mp4/37c041443749d21609) |
| 12 | Slack | Communication | [`reference.json`](references/12-slack/reference.json) | [Slack](https://www.youtube.com/watch?v=FTuOS8E1LZk) |
| 13 | Discord | Communication | [`reference.json`](references/13-discord/reference.json) | [MyTechGuy](https://www.youtube.com/watch?v=_9hK8Wj42Cs) |
| 14 | Google Messages | Communication | [`reference.json`](references/14-google-messages/reference.json) | [Google](https://www.youtube.com/watch?v=lV4ZkOYSnM0) |
| 15 | Zoom Workplace | Communication | [`reference.json`](references/15-zoom-workplace/reference.json) | [Dexter Tutorials](https://www.youtube.com/watch?v=1D5Hxaz9Po0) |
| 16 | Google Wallet | Finance | [`reference.json`](references/16-google-wallet/reference.json) | [Google](https://www.youtube.com/watch?v=3eKF_kEjy-I) |
| 17 | PayPal | Finance | [`reference.json`](references/17-paypal/reference.json) | [PayPal](https://www.youtube.com/watch?v=KGD_eCq4LbE) |
| 18 | Wise | Finance | [`reference.json`](references/18-wise/reference.json) | [OneClickLater](https://www.youtube.com/watch?v=cBwX6AIze8Y) |
| 19 | Revolut | Finance | [`reference.json`](references/19-revolut/reference.json) | [Technically Money](https://www.youtube.com/watch?v=J5FLyHMV9og) |
| 20 | Splitwise | Finance | [`reference.json`](references/20-splitwise/reference.json) | [HowTube](https://www.youtube.com/watch?v=jhwRh2z_mr8) |
| 21 | YNAB | Finance | [`reference.json`](references/21-ynab/reference.json) | [BetterPick Guide](https://www.youtube.com/watch?v=pPQWk-gKohI) |
| 22 | Fitbit | Health and fitness | [`reference.json`](references/22-fitbit/reference.json) | [IN APP](https://www.youtube.com/watch?v=lGZyVe_ZUFc) |
| 23 | Google Fit | Health and fitness | [`reference.json`](references/23-google-fit/reference.json) | [Harry's Help - Tech and More Tutorials](https://www.youtube.com/watch?v=PV96BXf2EKs) |
| 24 | Strava | Health and fitness | [`reference.json`](references/24-strava/reference.json) | [One2Step](https://www.youtube.com/watch?v=gsZYZwparz8) |
| 25 | MyFitnessPal | Health and fitness | [`reference.json`](references/25-myfitnesspal/reference.json) | [MyFitnessPal](https://www.youtube.com/watch?v=wzVIiOrmZJ4) |
| 26 | Medisafe | Health and fitness | [`reference.json`](references/26-medisafe/reference.json) | [Medisafe](https://www.youtube.com/watch?v=PA3E3l6bQj8) |
| 27 | Calm | Health and wellness | [`reference.json`](references/27-calm/reference.json) | [Calm](https://www.youtube.com/watch?v=ldKhvNv-pDk) |
| 28 | Adobe Lightroom | Creativity | [`reference.json`](references/28-adobe-lightroom/reference.json) | [Frozen In Frame](https://www.youtube.com/watch?v=9vdI_mFP4n8) |
| 29 | Canva | Creativity | [`reference.json`](references/29-canva/reference.json) | [Canva](https://www.youtube.com/watch?v=wdFXby-As-o) |
| 30 | Sketchbook | Creativity | [`reference.json`](references/30-sketchbook/reference.json) | [ArtsySolanki](https://www.youtube.com/watch?v=V-aLdmmc10s) |
| 31 | CapCut | Creativity | [`reference.json`](references/31-capcut/reference.json) | [most delightful way](https://www.youtube.com/watch?v=QYC8h1Gyetc) |
| 32 | BandLab | Creativity | [`reference.json`](references/32-bandlab/reference.json) | [Guide Zone](https://www.youtube.com/watch?v=R2a2jgf_Dhg) |
| 33 | Snapseed | Creativity | [`reference.json`](references/33-snapseed/reference.json) | [Alessio La Ruffa](https://www.youtube.com/watch?v=tvyBFC1j-f0) |
| 34 | Files by Google | Utilities | [`reference.json`](references/34-files-by-google/reference.json) | [Google](https://www.youtube.com/watch?v=T3bGJcJUQkg) |
| 35 | Bitwarden Password Manager | Utilities | [`reference.json`](references/35-bitwarden-password-manager/reference.json) | [Bitwarden](https://www.youtube.com/watch?v=Vu4PMZk5uys) |
| 36 | 1Password | Utilities | [`reference.json`](references/36-1password/reference.json) | [1Password](https://www.youtube.com/watch?v=Qe_BNU7qkOA) |
| 37 | Google Translate | Utilities | [`reference.json`](references/37-google-translate/reference.json) | [pcphobic](https://www.youtube.com/watch?v=OLU1AeE3Vns) |
| 38 | AccuWeather | Utilities | [`reference.json`](references/38-accuweather/reference.json) | [AccuWeather](https://www.youtube.com/watch?v=oU2mk6hHUnk) |
| 39 | Spotify | Media | [`reference.json`](references/39-spotify/reference.json) | [GuideRealm](https://www.youtube.com/watch?v=yjKhpNQAX9U) |
| 40 | YouTube | Media | [`reference.json`](references/40-youtube/reference.json) | [YouTube Viewers](https://www.youtube.com/watch?v=UFYfsRSE10g) |
| 41 | YouTube Music | Media | [`reference.json`](references/41-youtube-music/reference.json) | [YouTube Music](https://www.youtube.com/watch?v=TxIsrqykLjQ) |
| 42 | Pocket Casts | Media | [`reference.json`](references/42-pocket-casts/reference.json) | [Automattic](https://www.youtube.com/watch?v=h-jf5pXbl6I) |
| 43 | VLC for Android | Media | [`reference.json`](references/43-vlc-for-android/reference.json) | [Ftopreview.com](https://www.youtube.com/watch?v=z-RZ9LFKTug) |
| 44 | Netflix | Media | [`reference.json`](references/44-netflix/reference.json) | [Login Helps - How to Tutorial](https://www.youtube.com/watch?v=VVRG6ZILHNE) |
| 45 | Google Maps | Navigation | [`reference.json`](references/45-google-maps/reference.json) | [iZem](https://www.youtube.com/watch?v=Xo7yywC9iPk) |
| 46 | Waze Navigation & Live Traffic | Navigation | [`reference.json`](references/46-waze-navigation-live-traffic/reference.json) | [Printers With Pat](https://www.youtube.com/watch?v=sqO79ZuLCOM) |
| 47 | Citymapper | Navigation | [`reference.json`](references/47-citymapper/reference.json) | [My Digital Coffee](https://www.youtube.com/watch?v=7m6KIGhlIsQ) |
| 48 | komoot | Navigation | [`reference.json`](references/48-komoot/reference.json) | [Outdoor Tech Instructor](https://www.youtube.com/watch?v=uMOnJuGhURo) |
| 49 | GitHub | Developer tools | [`reference.json`](references/49-github/reference.json) | [A Minute With AI](https://www.youtube.com/watch?v=Ds0fKVB0JR4) |
| 50 | Termux | Developer tools | [`reference.json`](references/50-termux/reference.json) | [Tech Fuse](https://www.youtube.com/watch?v=1-tlv7Kn7P8) |

## Recurring patterns

1. **Stable entry, focused decision, distinct result.** Across productivity, finance, creation, media, and navigation, the clearest first-success paths preserve an overview, move to one focused task surface, and finish with a visibly different saved/active state.
2. **Bottom-level navigation with task-local controls.** Android apps repeatedly separate global destinations from contextual actions. The strongest recordings keep global navigation spatially stable while editors, sheets, maps, players, and confirmations change above it.
3. **Immediate intermediate feedback.** Selection, amount, route, content, tool, or playback changes appear before commitment. This makes the decision state inspectable and provides the recovery point when the user chose the wrong target.
4. **Confirmation proportional to consequence.** Communication and media often use a direct start/send/play action. Finance, health, security, and destructive storage flows expose a more explicit review boundary.
5. **Continuity is useful only when state remains legible.** Short transitions preserve origin and destination, while cuts compress setup. The references distinguish those recorded cuts from synthesized motion.
6. **Recovery starts before completion.** Back/close and editable decision states are most useful before the final action. Post-success reversal is inconsistently demonstrated and must not be assumed.

## Disagreements across products

- **Navigation placement:** bottom tabs dominate broad consumer hubs; drawers, top actions, and full-canvas controls remain appropriate for dense communication, terminal, drawing, and editing surfaces.
- **Creation entry:** some products foreground a floating or persistent create action; others make selection of an existing item the primary entry into success.
- **Feedback density:** finance and health favor explicit labels and review summaries, while media and creative tools rely more heavily on spatial preview and transport/tool state.
- **Permission timing:** utility, location, camera, microphone, storage, and notification access may be requested on first run or only when the capability is invoked. The retained publisher clips do not justify a universal timing rule.
- **Motion restraint:** some owner promos compress states with quick cuts, while task tutorials preserve direct manipulation. Neither implies the app's reduced-motion behavior; that remains an explicit unknown where not demonstrated.

## Applicability boundaries

- These are observed Android product references, not templates and not guarantees about current account entitlements, regional availability, or later releases.
- A published recording proves only the visible path it contains. It does not prove TalkBack semantics, switch access, large-text reflow, measured contrast, target dimensions, haptics, or reduced-motion support.
- Short clips can prove motion and state transitions but may omit authentication, permissions, offline behavior, billing gates, and destructive-action recovery. Each per-example record names its prerequisites and unknowns.
- Third-party screen recordings are labeled with their actual publisher. They are retained because they show real product pixels and interactions; they are not promoted to product-owner material.

## Category distribution

- **Communication:** 7
- **Creativity:** 6
- **Developer tools:** 2
- **Finance:** 6
- **Health and fitness:** 5
- **Health and wellness:** 1
- **Media:** 6
- **Navigation:** 4
- **Productivity:** 8
- **Utilities:** 5

## How to inspect offline

Open any `references/<NN-slug>/README.md`, play `media/motion.mp4`, compare the three retained JPEG states, then verify dimensions, duration/frame count, byte size, and SHA-256 in `reference.json`. The catalog index provides the exact 50 paths.
