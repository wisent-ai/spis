# Full reference synthesis: app store listings

This synthesis is derived from the 50 complete per-example records indexed in [`references.json`](references.json). It is not a substitute for them. Each record retains a real Weles/Stado listing interaction, seven ordered listing states, an eight-step discovery-to-open journey with observed failure and recovery, interaction evidence, accessibility observations and unknowns, provenance, dimensions, frame counts, byte sizes, and SHA-256. Fourteen canonical listings additionally exposed official Apple App Store or Google Play preview video; those records retain the official motion and three direct decoded key states alongside the listing interaction.

## Corpus boundaries

- **50 listings:** 25 Apple App Store and 25 Google Play.
- **50 authentic listing recordings:** captured only through Weles on the Stado-pinned `charless-mac-mini`; no local browser was used.
- **14 official store previews:** 9 Apple `apptrailers.itunes.apple.com` HLS assets and 5 Google Play listing trailers (4 direct `play-games.googleusercontent.com` MP4 assets and 1 publisher YouTube trailer).
- **36 screenshot-led listings:** no official preview asset was present in the canonical page metadata at capture time; their authentic motion evidence is the real listing interaction.
- **State floor exceeded:** every record retains seven listing states; preview-bearing records retain ten total states.

## Recurring patterns

1. **Header-to-proof progression.** Every observed listing starts with identity and acquisition context, then uses vertical movement to reveal media and later trust/detail modules. This makes scroll position part of the explanation architecture.
2. **Screenshot-first is still the majority.** Thirty-six listings did not expose official preview motion at capture time. Ordered static campaigns therefore remain the dominant publisher-controlled evidence, while the real listing scroll supplies the authentic interaction layer.
3. **Motion is concentrated where transformation or content is hard to explain in one frame.** The official-preview subset includes creation, fitness, social, professional-networking, storage, finance, food rescue, editing, productivity, shopping, and travel surfaces.
4. **Recovery is spatial and reversible.** In all 50 Weles journeys, an intentional overshoot is repaired by reversing the same scroll axis. The header can be restored without destructive action, authentication, or payment.
5. **Stores separate persuasion from trust.** Media appears before description, ratings, privacy, data safety, version notes, or policy details. The user first understands the promise and then evaluates evidence and risk.
6. **Media activation differs by store surface and viewport.** The measured `Screenshot` attempt activated visible media in 25 recordings and returned `no-target-found` in 25. Explicit failure feedback prevented arbitrary clicks; successful selection remained part of the reversible listing path.

## Disagreements across the 50 references

- **Motion-led versus screenshot-led acquisition:** 14 publishers use store-native preview motion; 36 rely on still campaigns in the canonical listing.
- **Outcome-first versus interface-first media:** creative, travel, entertainment, and fitness listings often lead with outcomes or content; productivity, finance, communication, and utility listings more often show controls and workflows.
- **Single promise versus breadth:** focused utilities can explain one loop quickly, while super-apps and broad platforms distribute evidence across several media panels and deeper metadata.
- **Brand continuity versus neutral proof:** recognizable brands can carry identity through icon, color, and content; less self-evident products need explicit captions and visible interface structure.
- **Trust emphasis varies by risk:** finance, health, dating, location, and communication products require privacy, safety, or regulatory context that entertainment and lightweight discovery products can defer.

## Applicability boundaries

- Use preview motion when sequence, transformation, or direct manipulation is essential to the promise; do not add motion merely because the store supports it.
- Use screenshot-led campaigns when states are independently legible and users need comparison time. Preserve a nonanimated ordered equivalent even when preview video exists.
- Do not generalize either media-selection outcome into a store-wide rule. The 25 successful activations and 25 `no-target-found` results are viewport-specific observations from these exact Weles capture paths.
- Do not infer screen-reader, keyboard, caption, autoplay, or reduced-motion support from visual capture. Those remain explicit unknowns in every record.
- For sensitive or regulated categories, acquisition clarity is insufficient without privacy, safety, permissions, pricing, and policy evidence later in the listing.
- A listing interaction is not evidence of the installed app's onboarding. The official-preview subset supplies product-native motion states; the remaining records truthfully document the store discovery surface only.

## Complete record set

| # | Record | Status |
|---:|---|---|
| 1 | [Duolingo: Language Lessons (Apple App Store)](references/01-duolingo-language-lessons/reference.json) | `complete` |
| 2 | [Canva: Design, Art & AI Editor (Apple App Store)](references/02-canva-design-art-ai-editor/reference.json) | `complete` |
| 3 | [Notion: Notes, Tasks, AI (Apple App Store)](references/03-notion-notes-tasks-ai/reference.json) | `complete` |
| 4 | [Headspace: Meditation & Sleep (Apple App Store)](references/04-headspace-meditation-sleep/reference.json) | `complete` |
| 5 | [Calm (Apple App Store)](references/05-calm/reference.json) | `complete` |
| 6 | [Strava: Run, Bike, Hike (Apple App Store)](references/06-strava-run-bike-hike/reference.json) | `complete` |
| 7 | [Airbnb (Apple App Store)](references/07-airbnb/reference.json) | `complete` |
| 8 | [Uber (Apple App Store)](references/08-uber/reference.json) | `complete` |
| 9 | [Spotify: Music and Podcasts (Apple App Store)](references/09-spotify-music-and-podcasts/reference.json) | `complete` |
| 10 | [TikTok (Apple App Store)](references/10-tiktok/reference.json) | `complete` |
| 11 | [Pinterest (Apple App Store)](references/11-pinterest/reference.json) | `complete` |
| 12 | [LinkedIn: Network & Job Finder (Apple App Store)](references/12-linkedin-network-job-finder/reference.json) | `complete` |
| 13 | [Slack (Apple App Store)](references/13-slack/reference.json) | `complete` |
| 14 | [Dropbox: Cloud Photo Storage (Apple App Store)](references/14-dropbox-cloud-photo-storage/reference.json) | `complete` |
| 15 | [Revolut: Send, Spend and Save (Apple App Store)](references/15-revolut-send-spend-and-save/reference.json) | `complete` |
| 16 | [Wise: International Transfers (Apple App Store)](references/16-wise-international-transfers/reference.json) | `complete` |
| 17 | [Too Good To Go: End Food Waste (Apple App Store)](references/17-too-good-to-go-end-food-waste/reference.json) | `complete` |
| 18 | [Adobe Lightroom: Photo Editor (Apple App Store)](references/18-adobe-lightroom-photo-editor/reference.json) | `complete` |
| 19 | [Picsart AI Photo Editor, Video (Apple App Store)](references/19-picsart-ai-photo-editor-video/reference.json) | `complete` |
| 20 | [Nike Run Club: Running Coach (Apple App Store)](references/20-nike-run-club-running-coach/reference.json) | `complete` |
| 21 | [AllTrails: Hike, Bike & Run (Apple App Store)](references/21-alltrails-hike-bike-run/reference.json) | `complete` |
| 22 | [Flo Period & Pregnancy Tracker (Apple App Store)](references/22-flo-period-pregnancy-tracker/reference.json) | `complete` |
| 23 | [ChatGPT (Apple App Store)](references/23-chatgpt/reference.json) | `complete` |
| 24 | [YAZIO Calorie Counter & Diet (Apple App Store)](references/24-yazio-calorie-counter-diet/reference.json) | `complete` |
| 25 | [Blinkist: Big Ideas in 15 Min (Apple App Store)](references/25-blinkist-big-ideas-in-15-min/reference.json) | `complete` |
| 26 | [Google Maps (Google Play)](references/26-google-maps/reference.json) | `complete` |
| 27 | [WhatsApp Messenger (Google Play)](references/27-whatsapp-messenger/reference.json) | `complete` |
| 28 | [Instagram (Google Play)](references/28-instagram/reference.json) | `complete` |
| 29 | [Netflix (Google Play)](references/29-netflix/reference.json) | `complete` |
| 30 | [Amazon Shopping (Google Play)](references/30-amazon-shopping/reference.json) | `complete` |
| 31 | [Microsoft Teams (Google Play)](references/31-microsoft-teams/reference.json) | `complete` |
| 32 | [Todoist: Planner & Calendar (Google Play)](references/32-todoist-planner-calendar/reference.json) | `complete` |
| 33 | [TickTick: To Do List & Calendar (Google Play)](references/33-ticktick-to-do-list-calendar/reference.json) | `complete` |
| 34 | [Any.do: To-do List & Calendar (Google Play)](references/34-any-do-to-do-list-calendar/reference.json) | `complete` |
| 35 | [Shazam: Find Music & Concerts (Google Play)](references/35-shazam-find-music-concerts/reference.json) | `complete` |
| 36 | [SoundCloud: Play Music & Songs (Google Play)](references/36-soundcloud-play-music-songs/reference.json) | `complete` |
| 37 | [Firefox Fast & Private Browser (Google Play)](references/37-firefox-fast-private-browser/reference.json) | `complete` |
| 38 | [Brave Private Web Browser, VPN (Google Play)](references/38-brave-private-web-browser-vpn/reference.json) | `complete` |
| 39 | [Booking.com: Hotels and More (Google Play)](references/39-booking-com-hotels-and-more/reference.json) | `complete` |
| 40 | [Tripadvisor: Plan & Book Trips (Google Play)](references/40-tripadvisor-plan-book-trips/reference.json) | `complete` |
| 41 | [Yelp: Food, Delivery & Reviews (Google Play)](references/41-yelp-food-delivery-reviews/reference.json) | `complete` |
| 42 | [GetYourGuide: Travel & Tickets (Google Play)](references/42-getyourguide-travel-tickets/reference.json) | `complete` |
| 43 | [Omio: Europe & U.S. Travel (Google Play)](references/43-omio-europe-u-s-travel/reference.json) | `complete` |
| 44 | [Memrise: Speak a New Language (Google Play)](references/44-memrise-speak-a-new-language/reference.json) | `complete` |
| 45 | [Babbel: Language Learning (Google Play)](references/45-babbel-language-learning/reference.json) | `complete` |
| 46 | [Khan Academy (Google Play)](references/46-khan-academy/reference.json) | `complete` |
| 47 | [Medium (Google Play)](references/47-medium/reference.json) | `complete` |
| 48 | [Goodreads (Google Play)](references/48-goodreads/reference.json) | `complete` |
| 49 | [YouTube Music (Google Play)](references/49-youtube-music/reference.json) | `complete` |
| 50 | [Tinder Dating App: Chat & Date (Google Play)](references/50-tinder-dating-app-chat-date/reference.json) | `complete` |
