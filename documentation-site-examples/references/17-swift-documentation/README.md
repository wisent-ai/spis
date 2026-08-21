# 17. Swift Documentation — observed search/learn-to-first-answer reference

**Evidence status:** complete  
**Official product:** [https://www.swift.org/documentation/](https://www.swift.org/documentation/)  
**Playable motion:** [motion.webm](media/motion.webm)  
**Captured:** 2026-08-16T23:10:48.113Z

This is a real live-product recording from a Weles browser on the Stado-selected dedicated `charless-mac-mini` host, not an animation synthesized from catalog stills. It retains a five-state path from landing through failed lookup, recovery with **concurrency**, and the first useful official answer.

## Local evidence

- [`media/motion.webm`](media/motion.webm): 640×360, 8.0 seconds, 200 decoded frames, 247020 bytes, SHA-256 `12b3f30139a178bffae4a5b61bd07710b8e80086c53a2246a93a8551274b826c`.
- [`reference.json`](reference.json): machine-readable journey, interaction, accessibility, motion, and provenance record.
- Every PNG below came from the same real browser context represented by the motion file.

| Observed state | Local frame | Dimensions | SHA-256 |
|---|---|---:|---|
| landing | [media/state-01-landing.png](media/state-01-landing.png) | 960×540 | `554da84beed8505f3a7370895d19f0344e1747295df863f972c47b0396eab2bd` |
| search open | [media/state-02-search-open.png](media/state-02-search-open.png) | 960×540 | `c3f2268722c3e7508c1094a4ba3731efaaeb8b55a5a65e2ef9fe590cac066c71` |
| failed search | [media/state-03-failed-search.png](media/state-03-failed-search.png) | 960×540 | `ae33d6c9063647f008f521f8cb52ca1c617cad151d3a0aab8b958594177729f8` |
| recovered results | [media/state-04-results.png](media/state-04-results.png) | 960×540 | `d9d2e9d503eb10d0b6340c6ba980e1baec52f09650a214e4c658ad25cc370185` |
| first answer | [media/state-05-answer.png](media/state-05-answer.png) | 960×540 | `82f30cfe7bda1ec2deffa94b9928426162d2c663bb76a44526264a608325e28f` |

## First-success journey

**Actor:** A public documentation visitor looking for one product concept  
**Goal:** Reach the first useful Swift Documentation answer for “concurrency”

| Step | User action | Product response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the official documentation URL | Render the public product landing/article surface | landing | media/state-01-landing.png |
| 2 | Invoke the product search or official learn route | Expose a query/result/navigation surface | search or learn open | media/state-02-search-open.png |
| 3 | Request the deliberately impossible term zzwissent-no-result-7f3c9 | Return no useful answer | failed search | media/state-03-failed-search.png |
| 4 | Cancel the failed route and request “concurrency” | Recover with relevant documentation feedback | recovered results | media/state-04-results.png |
| 5 | Inspect the first relevant product-owned route | Keep the candidate state stable for confirmation | result ready | media/state-04-results.png and media/motion.webm |
| 6 | Open the captured canonical answer | Render the first useful documentation answer | first answer | media/state-05-answer.png and media/motion.webm |

### Failure and recovery

The failure is deliberately observable: `zzwissent-no-result-7f3c9` produces no useful answer. Recovery is also retained: that route is abandoned, **concurrency** is requested, a relevant result/learn state appears, and the official canonical answer renders. Completion is `media/state-05-answer.png` plus the closing motion.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Load documentation | Navigate to the official documentation URL | The live documentation landing/article surface renders | Origin or network failure prevents the landing state | Reload the same canonical URL | media/state-01-landing.png and opening of media/motion.webm |
| Open search or learn route | Activate the visible native search control, keyboard convention, or official product search/learn route | A product-owned query/result surface appears | Some book-style sites expose no visible inline search field | Use the same product’s official search/result or concept navigation route | media/state-02-search-open.png and media/motion.webm |
| Focus query input | Target the visible query control when the product exposes one | The product accepts text input or a query-bearing official route | A hidden or absent field cannot accept direct text | Continue through the official product result/learn route retained in the recording | media/state-02-search-open.png and media/motion.webm |
| Enter impossible query | Enter or request zzwissent-no-result-7f3c9 | The product returns an unmatched/failed lookup state | No useful answer exists for the deliberately impossible query | Replace it with “concurrency” | media/state-03-failed-search.png and media/motion.webm |
| Cancel failed query | Clear or leave the impossible-query route | The failed lookup is abandoned | Stale no-result feedback may persist | Load the recovered query/result state | transition from media/state-03-failed-search.png to media/state-04-results.png |
| Submit learn query | Enter or request “concurrency” | The product renders relevant documentation feedback | A narrow query can yield no exact match | Use the broader recorded product term and official route | media/state-04-results.png and media/motion.webm |
| Inspect result feedback | Wait for the product result/learn surface to settle | Candidate content remains stable for inspection | An empty result surface is not treated as success | Follow the captured canonical concept route | media/state-04-results.png and media/motion.webm |
| Open first answer | Activate or navigate from the recovered product route to the first relevant answer | The official documentation answer renders | A route may land on missing or generic content | Return to results and choose the captured canonical answer route | media/state-05-answer.png and closing of media/motion.webm |

Confirmation is the answer transition. Cancellation/backtracking is the recorded abandonment of the impossible route before confirmation; browser Back is documented only as the available post-navigation path, not claimed as an unrecorded observation.

## Motion analysis

- **Trigger:** Official page navigation followed by product search/learn interaction.
- **Start → end:** public documentation landing/article → first useful canonical answer.
- **Continuity:** single real-browser context video; no synthesized or interpolated frames.
- **Timing:** brief; 8.0 seconds total.
- **Interruption/reversal:** the deliberately impossible route is abandoned before the useful query/route.
- **Feedback:** landing, open, failed, recovered, and answer frames retain the visible state changes.
- **Reduced motion / nonanimated equivalent:** reduced-motion behavior was not observed; five local states are retained as nonanimated inspection equivalents.

## Accessibility

Observed:
- The captured query/navigation path was operable through focused controls or official product links without drag-only gestures.
- Failure, recovery, and answer feedback remained visually persistent long enough to retain inspectable frames.

Unknown, and not promoted to observation:
- Screen-reader names, roles, live-region announcements, and reading order were not audited.
- Focus styling and focus restoration after cancellation were not independently audited.
- Reduced motion, 400% zoom reflow, high contrast, and switch-device access were not tested.

## Provenance

- Upstream owner: Swift Documentation (www.swift.org).
- Product/source URL: https://www.swift.org/documentation/.
- Recording environment: live official product, Weles AsyncNewBrowser, checksum-verified Weles Chromium, Stado-selected dedicated host `charless-mac-mini`; no local GUI/browser was launched.
- Captured at: 2026-08-16T23:10:48.113Z.
- Product interface/content remains owned by the upstream owner; local evidence is retained for inspectable design reference.
