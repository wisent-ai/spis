# Notion — observed iOS product reference

**Evidence status:** partial — everything below was measured from the retained files; accessibility has never been audited against the running app, and the asset shows no interruption, reversal, or reduced-motion variant  
**Product:** [https://www.notion.so/product](https://www.notion.so/product)  
**Captured:** 2026-08-16  
**Upstream owner / publisher:** Notion Labs, Inc.

## Authentic motion evidence

Play [`media/motion.mp4`](media/motion.mp4). This is retained product motion, not animation synthesized from stills.

- Source: [NotionAI-AcmeMobile-1600x1600-compressed-650k.mp4](https://videos.ctfassets.net/spoqsaf9291f/6LFgNE3oUQ1rr4UBPl4J9g/9d995df957125abd579a7a2bc88e282d/NotionAI-AcmeMobile-1600x1600-compressed-650k.mp4)
- Capture method: official product-site media downloaded over HTTPS and transcoded to H.264 MP4; no frames synthesized
- Media: 720×720, 12.600 s, 189 frames, 118865 bytes
- SHA-256: `22bc49a1ac66975f4373bab7e4be9be2f95cf7b202d0d060723fd9ab80b93543`

## Key states tied to the motion

| State | Local evidence | SHA-256 |
|---|---|---|
| "How can I help you today?" home with the suggestions "Search for anything", "Write meeting agenda", "Analyze PDFs or images" and the composer holding the half-typed prompt "Turn this page into a hub of key company pages, our road\|" | [media/state-01.png](media/state-01.png) | `d55040ffb76abc1444c3b2f05ad32b91dd7604ca2259b065ab04315b08662518` |
| the submitted prompt "Turn this page into a hub of key company pages, our roadmap, and company priorities" as a grey bubble above "96 search results", "Updated Acme Inc." and the blue "Acme Inc., Summary of company priorities" card, with a "Creating" spinner row below it | [media/state-02.png](media/state-02.png) | `b6ab130022742d8d00a7adba893c393e1a4ec6be1ba6bd6f3d9aed83a29101bf` |
| the created "Acme Inc." page open on its "Roadmap" database, "Status" view selected beside "Timeline", "Priority" and "By team", four task rows from "Launch AI-assisted onboarding / Luca Beetz" to "Build enterprise GTM capabilities / Jordan Scales" | [media/state-03.png](media/state-03.png) | `b5bff7992bfc48a994e1053b7e48be7fd662a2a67e64dc47a01fc9bd775a7f7e` |

## First-success journey

**Actor:** A first-time or returning iPhone user with the minimum product account/device prerequisites  
**Goal:** Create a mobile page

**Prerequisites:** Compatible iPhone and current iOS version; Product installed or product-native flow available; Network/account/permission access only where the observed flow requests it.

1. **Open the workspace** — The product exposes its initial context and available next action. Evidence: `media/motion.mp4 at 00:00.6`
2. **choose a page or new-page action** — The selected destination or task surface replaces or overlays the prior state. Evidence: `media/motion.mp4 at 00:03.1`
3. **add page content with blocks** — The product reflects the entered choice or manipulation and exposes the next control. Evidence: `media/motion.mp4 at 00:05.7`
4. **finish the edit** — The product acknowledges the commit and transitions toward the result. Evidence: `media/motion.mp4 at 00:08.2`
5. **see the updated page** — The completed result remains visible as durable evidence of first success. Evidence: `media/motion.mp4 at 00:10.7`

**Completion evidence:** `media/motion.mp4 at 00:10.7 and media/state-03.png`

### Failure and recovery observed in the retained flow

- Failure route: Stop at the recorded pre-confirmation state instead of committing. The intended result is absent at this point; the first-success goal remains incomplete. Evidence: `media/motion.mp4 00:04.0–00:07.3`
- Recovery route: Continue from the retained incomplete state and finish the edit. The confirmation transition resumes from the same observed flow. Then see the updated page; The product shows the documented first-success result: see the updated page. Evidence: `media/motion.mp4 00:08.6–00:11.8`

## Interaction map

| Interaction | Trigger | Response | Feedback | Evidence |
|---|---|---|---|---|
| primary input | choose a page or new-page action | The primary task surface opens. | Selection highlight and surface transition make the response visible. | `media/motion.mp4 00:01.5–00:03.5` |
| focus / selection | add page content with blocks | The chosen field, row, item, or canvas becomes the active target. | The target changes emphasis or reveals contextual controls. | `media/motion.mp4 00:03.3–00:05.5` |
| navigation | Open the workspace | The app moves from entry context to the task destination. | Header, content, or navigation state changes. | `media/motion.mp4 00:00.5–00:03.0` |
| confirmation | finish the edit | The pending action commits and the result transition begins. | A changed state, progress response, or completed item acknowledges the commit. | `media/motion.mp4 00:07.3–00:09.6` |
| cancellation / backtracking | Withhold or reverse the visible commit while the recorded pre-confirmation state is on screen. | The task remains in its pre-result state; the success artifact is absent. | The unchanged/incomplete state distinguishes cancellation from completion. | `media/motion.mp4 00:04.8–00:07.3` |
| feedback | Complete a visible selection or commit. | The interface updates immediately or after a bounded progress transition. | Motion, highlight, changed content, or result placement communicates status. | `media/motion.mp4 00:07.8–00:11.1` |
| failure | Reach the recorded pre-confirmation state without issuing the final commit. | The intended result is not yet present, so the goal remains incomplete. | The pre-result surface remains visibly distinct from state-03. | `media/motion.mp4 00:04.0–00:06.9` |
| recovery | Continue from the incomplete state and finish the edit. | The product resumes the path and produces the visible result. | The final state replaces the incomplete state. | `media/motion.mp4 00:06.6–00:11.6` |

Cancellation, failure, and recovery details for every interaction are retained verbatim in [`reference.json`](reference.json).

## Motion behavior

- **Trigger:** The finished prompt "Turn this page into a hub of key company pages, our roadmap, and company priorities" is submitted from the Notion AI composer, whose only send affordance on screen is the blue circular up-arrow at its bottom right; the last frame that still shows the composer is `media/motion.mp4` 2.800s.
- **Start → end:** "How can I help you today?" home — hat avatar, the three suggestion rows, and the composer carrying the "@" and "Acme Inc." chips, the full prompt, "All sources" and the blue up-arrow (2.800s) → the created "Acme Inc." page scrolled to its "Roadmap" database, "Status" view pill selected, all four rows fully opaque from "Launch AI-assisted onboarding / Luca Beetz" to "Build enterprise GTM capabilities / Jordan Scales" (10.267s, unchanged through the final frame at 12.6s).
- **Continuity:** One continuous take with no editorial cuts, but two hard one-frame swaps inside it: the composer is replaced by the answer thread between 2.800s and 2.867s, and the thread is replaced by the finished page between 7.000s and 7.067s, with no intermediate frame at the source's 15 fps (189 frames over 12.600s). Everything between those swaps is continuous — the spinner rotates, and the tab bar, row labels and DRI avatars fade up from grey skeleton bars between 9.867s and 10.267s. Transcoding to H.264 changed encoding only; no frames were synthesized.
- **Timing:** multi-second (7.47 s measured between 2.800s and 10.267s)
- **Interruption / reversal:** Not shown by this asset. Nothing stops, cancels or reverses the generation, and no stop control is drawn beside "Searching" or "Creating".
- **Feedback:** Acknowledgment is staged and mostly textual: the prompt collapses into a grey chat bubble at 2.867s, "Searching" with a rotating spinner glyph appears at 2.900s, "96 search results" plus Notion, Slack and Drive source icons and a chevron replace it at 4.867s, "Updated Acme Inc." and the blue "Acme Inc., Summary of company priorities" card are in place by 5.000s, "Creating" appears at 5.200s, and the new page arrives at 7.067s as grey skeleton bars that resolve into real rows by 10.267s.
- **Reduced motion / nonanimated equivalent:** Not shown by this asset; nothing is claimed.

## Accessibility

Observed:
- In `media/state-01.png` each of the three starting suggestions pairs an icon with its own text label — a magnifier with "Search for anything", a lines-and-pen glyph with "Write meeting agenda", a PDF page with "Analyze PDFs or images" — and the composer footer labels its scope control "All sources" in words, so no offer depends on the glyph alone.
- The text caret is visible after "our road" inside the composer of `media/state-01.png`, so the focused field is identifiable from a single still even though no on-screen keyboard is drawn.
- Progress is written out rather than only spun: `media/motion.mp4` shows "Searching" from 2.900s, "96 search results" with Notion, Slack and Drive source icons from 4.867s, "Updated Acme Inc." and the summary card by 5.000s, and "Creating" from 5.200s, so a reader who never sees the spinner rotate still gets the stage in text.
- The result text is high contrast: the row label "Launch AI-assisted onboarding" in `media/state-03.png` measures 18.88:1, darkest pixel `#111111` against lightest `#ffffff` in the crop 99,508 312×22, and the blue card title "Acme Inc., Summary of company priorities" in `media/state-02.png` measures 7.20:1 (`#1c5b8b` against `#ffffff`, crop 88,344 490×26).
- The selected database view is not carried by fill alone: its pill measures `#f0f0f0` against the `#ffffff` page beside it, only 1.14:1 (crops 102,368 and 620,368, 10×12 each), but the "Status" label inside it is `#000000` while the unselected "Timeline" label is `#4c4c4c` at 8.59:1, and the star icon beside "Status" is solid where the other three view icons are outlines.

Unknown from this evidence:
- VoiceOver names, hints, rotor order, and focus return were not exposed by the source recording.
- Dynamic Type behavior and text truncation at accessibility sizes were not exposed.
- Reduce Motion behavior and a nonanimated equivalent were not exposed.
- Switch Control, keyboard navigation, contrast ratios, and haptic/audio-only feedback were not measured.
- The prompt is typed into the composer with no on-screen keyboard, input accessory, or key hint ever drawn in the frame, so the input mechanism and its focus affordances are not exposed.

## Provenance

The source URL, local path, capture method, dimensions, duration, frame count, byte size, SHA-256, capture date, and upstream ownership are recorded in [`reference.json`](reference.json). All three state images are frames of `media/motion.mp4`: state-01 at 1.5s (mean abs diff 1.9297/255), state-02 at 5.5s (1.668/255), and state-03 at 10s (1.3789/255), each found with the same 16×16 grayscale mean-absolute-difference search.
