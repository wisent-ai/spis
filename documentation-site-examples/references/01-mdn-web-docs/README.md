# 01. MDN Web Docs — observed search/learn-to-first-answer reference

**Evidence status:** partial — 2 named evidence gaps in [`reference.json`](reference.json), measured 2026-08-19  
**Official product:** [https://developer.mozilla.org/en-US/docs/Web](https://developer.mozilla.org/en-US/docs/Web)  
**Playable motion:** [motion.webm](media/motion.webm)  
**Captured:** 2026-08-16T23:06:22.683Z

This is a real live-product recording from a Weles browser on the Stado-selected dedicated `charless-mac-mini` host, not an animation synthesized from catalog stills. It retains a five-state path from landing through failed lookup, recovery with **HTML**, and the first useful official answer.

## Local evidence

- [`media/motion.webm`](media/motion.webm): 640×360, 13.92 seconds, 348 decoded frames, 280099 bytes, SHA-256 `53302c76564d988730b23a0162f6bc96e81eba0fa4227acf2bfa3b28263e8813`, measured with `ffprobe -count_frames`, provenance class `local-browser-recording`.
- [`reference.json`](reference.json): machine-readable journey, interaction, accessibility, motion, and provenance record, with every unclosed gap named in `evidence_gaps`.

| Journey state | Observed frame content | Local frame | Dimensions | Relationship to the motion asset |
|---|---|---|---:|---|
| landing | Web technology for developers article at rest, header search box collapsed | [media/state-01-landing.png](media/state-01-landing.png) | 960×540 | frame of media/motion.webm at 1.5s (mean abs diff 0.3867/255) |
| search open | search overlay open over the article, query field empty showing the Search placeholder and a × close control | [media/state-02-search-open.png](media/state-02-search-open.png) | 960×540 | frame of media/motion.webm at 2s (mean abs diff 0.0703/255) |
| failed search | query zzwissent-no-result-7f3c9 in the search field, no page suggestions, only the fallback row Site search for zzwissent-no-result-7f3c9 | [media/state-03-failed-search.png](media/state-03-failed-search.png) | 960×540 | frame of media/motion.webm at 3s (mean abs diff 0.0898/255) |
| recovered results | query HTML in the search field, eight full suggestion rows plus a ninth clipped at the frame edge, matched term highlighted in yellow, first row HTML: HyperText Markup Language shaded as the selected row | [media/state-04-results.png](media/state-04-results.png) | 960×540 | frame of media/motion.webm at 8.5s (mean abs diff 0.207/255) |
| first answer | HTML: HyperText Markup Language reference article open, breadcrumb Web › HTML, In this article sidebar listing Beginner's tutorials through Related topics | [media/state-05-answer.png](media/state-05-answer.png) | 960×540 | frame of media/motion.webm at 11s (mean abs diff 0.418/255) |

## First-success journey

**Actor:** A public documentation visitor looking for one product concept  
**Goal:** Reach the first useful MDN Web Docs answer for “HTML”

| Step | User action | Product response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the official documentation URL | Render the public product landing/article surface | landing | media/state-01-landing.png |
| 2 | Invoke the product search or official learn route | Expose a query/result/navigation surface | search or learn open | media/state-02-search-open.png |
| 3 | Request the deliberately impossible term zzwissent-no-result-7f3c9 | Return no useful answer | failed search | media/state-03-failed-search.png |
| 4 | Cancel the failed route and request “HTML” | Recover with relevant documentation feedback | recovered results | media/state-04-results.png |
| 5 | Inspect the first relevant product-owned route | Keep the candidate state stable for confirmation | result ready | media/state-04-results.png and media/motion.webm |
| 6 | Open the captured canonical answer | Render the first useful documentation answer | first answer | media/state-05-answer.png and media/motion.webm |

### Failure and recovery

The failure is deliberately observable: `zzwissent-no-result-7f3c9` produces no useful answer. Recovery is also retained: that route is abandoned, **HTML** is requested, a relevant result/learn state appears, and the official canonical answer renders. Completion is `media/state-05-answer.png` plus the closing motion.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Load documentation | Navigate to the official documentation URL | The live documentation landing/article surface renders | Origin or network failure prevents the landing state | Reload the same canonical URL | media/state-01-landing.png and opening of media/motion.webm |
| Open search or learn route | Activate the visible native search control, keyboard convention, or official product search/learn route | A product-owned query/result surface appears | Some book-style sites expose no visible inline search field | Use the same product’s official search/result or concept navigation route | media/state-02-search-open.png and media/motion.webm |
| Focus query input | Target the visible query control when the product exposes one | The product accepts text input or a query-bearing official route | A hidden or absent field cannot accept direct text | Continue through the official product result/learn route retained in the recording | media/state-02-search-open.png and media/motion.webm |
| Enter impossible query | Enter or request zzwissent-no-result-7f3c9 | The product returns an unmatched/failed lookup state | No useful answer exists for the deliberately impossible query | Replace it with “HTML” | media/state-03-failed-search.png and media/motion.webm |
| Cancel failed query | Clear or leave the impossible-query route | The failed lookup is abandoned | Stale no-result feedback may persist | Load the recovered query/result state | transition from media/state-03-failed-search.png to media/state-04-results.png |
| Submit learn query | Enter or request “HTML” | The product renders relevant documentation feedback | A narrow query can yield no exact match | Use the broader recorded product term and official route | media/state-04-results.png and media/motion.webm |
| Inspect result feedback | Wait for the product result/learn surface to settle | Candidate content remains stable for inspection | An empty result surface is not treated as success | Follow the captured canonical concept route | media/state-04-results.png and media/motion.webm |
| Open first answer | Activate or navigate from the recovered product route to the first relevant answer | The official documentation answer renders | A route may land on missing or generic content | Return to results and choose the captured canonical answer route | media/state-05-answer.png and closing of media/motion.webm |

Confirmation is the answer transition. Cancellation/backtracking is the recorded abandonment of the impossible route before confirmation; browser Back is documented only as the available post-navigation path, not claimed as an unrecorded observation.

## Motion analysis

- **Trigger:** Loading https://developer.mozilla.org/en-US/docs/Web in the recorded Chromium session, then opening the header search box and typing into it.
- **Start state:** Frame 0 of media/motion.webm: the Web technology for developers article with the header search box collapsed and no advertisement slots painted yet.
- **End state:** Frames 279-347 of media/motion.webm: the HTML: HyperText Markup Language article with the breadcrumb Web › HTML, unchanged for the last 2.76 seconds of the recording.
- **Continuity:** One continuous 13.92 second, 348 frame recording at 25 fps of a single browser context; no decoded frame is uniform, so there is no cut, black frame or splice, and the median 16x16 grayscale difference between consecutive frames is 0.0/255.
- **Timing class:** instant
- **Interruption or reversal:** The failed query is reversed rather than submitted: frame 64 (2.56 s) holds zzwissent-no-result-7f3c9 with only the Site search fallback row, and frame 97 (3.88 s) holds HTML with a full suggestion list, so the impossible term was cleared inside the same overlay.
- **Feedback:** Feedback is a repaint, not an animation: the search overlay replaces the article in a single frame (17.0/255 at 1.80 s), the suggestion list swaps in a single frame (7.5/255 at 3.88 s), and the answer article paints in a single frame (17.5/255 at 10.72 s); only 2.3% of all frame-to-frame comparisons differ at all, and the longest change event spans 0.08 s.
- **Reduced-motion equivalent:** not shown by the retained asset; recorded as null in reference.json

## Accessibility

Observed in the retained frames:

- In media/state-01-landing.png the header controls carry visible text beside their icons — "Theme" next to the contrast glyph and "English (US)" next to the translate glyph — so the icon is not the only label.
- The failed lookup in media/state-03-failed-search.png is stated in text rather than by motion or colour alone: the field shows the term and the single row reads "Site search for zzwissent-no-result-7f3c9", so the absence of a matching page is readable in a still frame.
- Contrast measured from media/state-05-answer.png by counting decoded pixels: the page background #ffffff covers 75.8% of the frame and the body-text ink #000000 covers 2.7%, a WCAG ratio of 21.0:1; the in-body link colour #1e64d4 on that background measures 5.48:1.
- In media/state-04-results.png the matched substring is highlighted in yellow behind the text and the first suggestion additionally carries a pale-blue row fill, so the selected row is distinguished by more than the match highlight.
- The search overlay in media/state-02-search-open.png keeps a visible × close control and the text placeholder "Search" in the empty field, both readable without any animation.

Unknown, and not promoted to observation:

- Screen-reader names, roles, live-region announcements, and reading order were not audited.
- Focus styling and focus restoration after cancellation were not independently audited; no focus ring is visible in any retained frame.
- Reduced motion, 400% zoom reflow, high contrast, and switch-device access were not tested.

`accessibility.measured` is unset: nothing here came from an audit driven against the running product, only from the retained frames and the motion file.

## Provenance

- Upstream owner: MDN Web Docs (developer.mozilla.org).
- Product/source URL: https://developer.mozilla.org/en-US/docs/Web.
- Recording environment: live official product, Weles AsyncNewBrowser, checksum-verified Weles Chromium, Stado-selected dedicated host `charless-mac-mini`; no local GUI/browser was launched.
- Captured at: 2026-08-16T23:06:22.683Z.
- Product interface/content remains owned by the upstream owner; local evidence is retained for inspectable design reference.
