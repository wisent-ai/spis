# Full iOS product interaction reference

This synthesis is derived from the 50 complete per-product records in [`references.json`](references.json). It does not replace them. Each row below resolves offline to local playable motion, three direct state frames, an observed five-state first-success journey, eight interaction classes, failure/recovery, accessibility observations and unknowns, and hash-bound provenance.

## Evidence base

- **Records:** 50 of 50 complete
- **Motion assets:** 50 local H.264 MP4 files; every file has measured dimensions, duration, frame count, bytes, and SHA-256
- **State evidence:** 150 local PNG frames, three distinct source-tied frames per product
- **Journey evidence:** 250 ordered observed states, five per product
- **Interaction evidence:** 400 structured interaction entries, eight per product
- **Duration:** 2.0s minimum, 72.0s median, 120.0s maximum, 3559.9s total
- **Orientation:** 7 portrait, 42 landscape, 1 square recordings
- **Catalog distribution:** Communication: 6, Creativity: 6, Developer tools: 4, Finance: 6, Health: 6, Media: 5, Navigation: 4, Productivity: 8, Utilities: 5

The media is retained locally rather than represented by prose or marketing claims. Product-site assets and publisher-owned tutorial recordings were preferred. The Signal entry uses an official Signal product recording from the Signal Blog. Transcoding normalizes playback only; no intermediate frames were synthesized.

## Recurring patterns across the 50 records

### 1. Entry context precedes commitment

All 50 journeys establish an orientation state before the decisive action. Mobile products compress this into a list, feed, map, canvas, player, or account overview. The primary action then opens a focused sheet, detail screen, composer, or task surface. The persistent lesson is to preserve enough origin context that Back or Cancel has a meaningful destination.

### 2. Five-state first success is a robust minimum

Across productivity, finance, health, creativity, media, navigation, and developer tools, first success repeatedly resolves into: **entry → selection → configuration → confirmation → durable result**. The labels differ, but the state responsibilities do not. Even very short product clips expose multiple product states; longer tutorials make intermediate validation and recovery easier to inspect.

### 3. Confirmation strength follows consequence

Low-risk actions such as filtering, drawing, or choosing media tend to acknowledge immediately through direct manipulation. Transfers, account creation, permission-bearing setup, remote connections, and beta distribution introduce explicit review or confirmation. A single universal confirmation style would be inappropriate: consequence, reversibility, latency, and user trust determine how much ceremony is justified.

### 4. Feedback is layered

The references combine at least two of: control emphasis, surface motion, changed content, progress, persistent result placement, audio/haptic implication, or status copy. Motion communicates continuity, but the end state remains visually legible. This is important for reduced-motion equivalents even though the sources do not expose each product's Reduce Motion implementation.

### 5. Recovery preserves work and place

The observed recovery model is usually local: return to the incomplete surface, restore the missing selection or input, and repeat confirmation. Products with expensive setup avoid resetting the whole journey. Navigation and editing tools especially benefit from retaining map position, canvas content, media choice, draft text, or repository context.

### 6. Bottom sheets and contextual trays dominate compact workflows

Map, finance, media, and creation products frequently keep the underlying object visible while presenting a draggable or modal control surface. Other products prefer full-screen progression when identity, permissions, or several required fields demand focused attention. This is a genuine design disagreement rather than a winner/loser distinction.

## Disagreements visible in the records

- **Persistent tabs vs task focus:** communication, media, and health apps often preserve broad tab navigation; cameras, drawing tools, meetings, and active navigation suppress it during the core task.
- **Immediate commit vs explicit review:** editing and playback actions commonly commit continuously; money, publishing, installation, and connection flows defer commitment to a review step.
- **Inline errors vs withheld completion:** some flows keep the corrective control inline; others prevent transition until a prerequisite is satisfied. Recovery should match the consequence and preserve prior work.
- **Animation density:** short direct-manipulation clips use rapid state changes, while onboarding and tutorial flows use deliberate pacing. Timing cannot be copied without accounting for comprehension, latency, and motor accessibility.
- **Information density:** finance, maps, repositories, and fitness summaries expose dense data after success; meditation, capture, drawing, and media playback reduce chrome around the active task.

## Applicability boundaries

These references document the captured product behavior at the source and capture date; they are not claims about every account tier, region, permission state, device size, or future release. Authentication, real money, private data, live meetings, outdoor navigation, sensors, and paid content were not invented to fill evidence gaps. VoiceOver focus order, Dynamic Type at accessibility sizes, Switch Control, exact contrast, haptics, and app-level Reduce Motion behavior remain explicit unknowns unless the retained recording exposes them. Use the records to study interaction structure, not to copy brand styling or infer unobserved accessibility support.

## All 50 source records

| # | Product reference | Category | Local motion | Structured evidence |
|---:|---|---|---:|---|
| 1 | [Things 3](references/01-things-3/README.md) | Productivity | 70.9s / 1063 frames | [`record`](references/01-things-3/reference.json) |
| 2 | [Todoist](references/02-todoist/README.md) | Productivity | 120.0s / 1800 frames | [`record`](references/02-todoist/reference.json) |
| 3 | [Notion](references/03-notion/README.md) | Productivity | 12.6s / 189 frames | [`record`](references/03-notion/reference.json) |
| 4 | [Bear](references/04-bear/README.md) | Productivity | 20.7s / 310 frames | [`record`](references/04-bear/reference.json) |
| 5 | [Craft](references/05-craft/README.md) | Productivity | 120.0s / 1800 frames | [`record`](references/05-craft/reference.json) |
| 6 | [Fantastical](references/06-fantastical/README.md) | Productivity | 67.0s / 1005 frames | [`record`](references/06-fantastical/reference.json) |
| 7 | [Microsoft 365 Copilot](references/07-microsoft-365-copilot/README.md) | Productivity | 120.0s / 1800 frames | [`record`](references/07-microsoft-365-copilot/reference.json) |
| 8 | [Dropbox](references/08-dropbox/README.md) | Productivity | 59.3s / 889 frames | [`record`](references/08-dropbox/reference.json) |
| 9 | [Slack](references/09-slack/README.md) | Communication | 45.1s / 676 frames | [`record`](references/09-slack/reference.json) |
| 10 | [Discord](references/10-discord/README.md) | Communication | 7.4s / 111 frames | [`record`](references/10-discord/reference.json) |
| 11 | [Signal](references/11-signal/README.md) | Communication | 26.8s / 402 frames | [`record`](references/11-signal/reference.json) |
| 12 | [Telegram Messenger](references/12-telegram-messenger/README.md) | Communication | 2.0s / 30 frames | [`record`](references/12-telegram-messenger/reference.json) |
| 13 | [WhatsApp Messenger](references/13-whatsapp-messenger/README.md) | Communication | 120.0s / 1800 frames | [`record`](references/13-whatsapp-messenger/reference.json) |
| 14 | [Zoom Workplace](references/14-zoom-workplace/README.md) | Communication | 120.0s / 1800 frames | [`record`](references/14-zoom-workplace/reference.json) |
| 15 | [Revolut](references/15-revolut/README.md) | Finance | 20.0s / 300 frames | [`record`](references/15-revolut/reference.json) |
| 16 | [Wise](references/16-wise/README.md) | Finance | 13.5s / 202 frames | [`record`](references/16-wise/reference.json) |
| 17 | [PayPal](references/17-paypal/README.md) | Finance | 30.0s / 450 frames | [`record`](references/17-paypal/reference.json) |
| 18 | [Coinbase](references/18-coinbase/README.md) | Finance | 120.0s / 1800 frames | [`record`](references/18-coinbase/reference.json) |
| 19 | [YNAB](references/19-ynab/README.md) | Finance | 120.0s / 1800 frames | [`record`](references/19-ynab/reference.json) |
| 20 | [Splitwise](references/20-splitwise/README.md) | Finance | 48.1s / 721 frames | [`record`](references/20-splitwise/reference.json) |
| 21 | [Apple Health](references/21-apple-health/README.md) | Health | 120.0s / 1800 frames | [`record`](references/21-apple-health/reference.json) |
| 22 | [Strava](references/22-strava/README.md) | Health | 29.0s / 435 frames | [`record`](references/22-strava/reference.json) |
| 23 | [Headspace](references/23-headspace/README.md) | Health | 120.0s / 1800 frames | [`record`](references/23-headspace/reference.json) |
| 24 | [Calm](references/24-calm/README.md) | Health | 76.4s / 1146 frames | [`record`](references/24-calm/reference.json) |
| 25 | [MyFitnessPal](references/25-myfitnesspal/README.md) | Health | 120.0s / 1800 frames | [`record`](references/25-myfitnesspal/reference.json) |
| 26 | [Nike Run Club](references/26-nike-run-club/README.md) | Health | 114.8s / 1722 frames | [`record`](references/26-nike-run-club/reference.json) |
| 27 | [Procreate Pocket](references/27-procreate-pocket/README.md) | Creativity | 10.0s / 150 frames | [`record`](references/27-procreate-pocket/reference.json) |
| 28 | [Adobe Lightroom for mobile](references/28-adobe-lightroom-for-mobile/README.md) | Creativity | 120.0s / 1800 frames | [`record`](references/28-adobe-lightroom-for-mobile/reference.json) |
| 29 | [Canva](references/29-canva/README.md) | Creativity | 120.0s / 1800 frames | [`record`](references/29-canva/reference.json) |
| 30 | [CapCut](references/30-capcut/README.md) | Creativity | 120.0s / 1800 frames | [`record`](references/30-capcut/reference.json) |
| 31 | [VSCO](references/31-vsco/README.md) | Creativity | 120.0s / 1800 frames | [`record`](references/31-vsco/reference.json) |
| 32 | [Concepts](references/32-concepts/README.md) | Creativity | 120.0s / 1800 frames | [`record`](references/32-concepts/reference.json) |
| 33 | [1Password for iOS](references/33-1password-for-ios/README.md) | Utilities | 120.0s / 1800 frames | [`record`](references/33-1password-for-ios/reference.json) |
| 34 | [Bitwarden](references/34-bitwarden/README.md) | Utilities | 120.0s / 1800 frames | [`record`](references/34-bitwarden/reference.json) |
| 35 | [Halide](references/35-halide/README.md) | Utilities | 14.5s / 217 frames | [`record`](references/35-halide/reference.json) |
| 36 | [Flighty](references/36-flighty/README.md) | Utilities | 15.4s / 231 frames | [`record`](references/36-flighty/reference.json) |
| 37 | [CARROT Weather](references/37-carrot-weather/README.md) | Utilities | 31.3s / 469 frames | [`record`](references/37-carrot-weather/reference.json) |
| 38 | [Spotify](references/38-spotify/README.md) | Media | 30.1s / 451 frames | [`record`](references/38-spotify/reference.json) |
| 39 | [Netflix](references/39-netflix/README.md) | Media | 77.3s / 1159 frames | [`record`](references/39-netflix/reference.json) |
| 40 | [YouTube](references/40-youtube/README.md) | Media | 49.3s / 740 frames | [`record`](references/40-youtube/reference.json) |
| 41 | [Pocket Casts](references/41-pocket-casts/README.md) | Media | 32.1s / 481 frames | [`record`](references/41-pocket-casts/reference.json) |
| 42 | [Libby](references/42-libby/README.md) | Media | 41.9s / 628 frames | [`record`](references/42-libby/reference.json) |
| 43 | [Apple Maps](references/43-apple-maps/README.md) | Navigation | 113.9s / 1708 frames | [`record`](references/43-apple-maps/reference.json) |
| 44 | [Google Maps](references/44-google-maps/README.md) | Navigation | 5.4s / 81 frames | [`record`](references/44-google-maps/reference.json) |
| 45 | [Citymapper](references/45-citymapper/README.md) | Navigation | 107.9s / 1618 frames | [`record`](references/45-citymapper/reference.json) |
| 46 | [AllTrails](references/46-alltrails/README.md) | Navigation | 6.9s / 104 frames | [`record`](references/46-alltrails/reference.json) |
| 47 | [GitHub Mobile](references/47-github-mobile/README.md) | Developer tools | 73.1s / 1096 frames | [`record`](references/47-github-mobile/reference.json) |
| 48 | [Working Copy](references/48-working-copy/README.md) | Developer tools | 65.7s / 986 frames | [`record`](references/48-working-copy/reference.json) |
| 49 | [Blink Shell](references/49-blink-shell/README.md) | Developer tools | 81.9s / 1229 frames | [`record`](references/49-blink-shell/reference.json) |
| 50 | [TestFlight](references/50-testflight/README.md) | Developer tools | 120.0s / 1800 frames | [`record`](references/50-testflight/reference.json) |

## How to inspect offline

Open any numbered directory, play `media/motion.mp4`, inspect `media/state-01.png` through `state-03.png`, then trace the journey and interaction claims in `reference.json` or the local README. Hashes and media facts in the record bind every claim to the retained evidence.
