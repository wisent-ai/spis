# 34. Slack API Documentation — observed search/learn-to-first-answer reference

**Evidence status:** complete  
**Official product:** [https://api.slack.com/docs](https://api.slack.com/docs)  
**Playable motion:** [motion.webm](media/motion.webm)  
**Captured:** 2026-08-16T23:10:48.113Z

This is a real live-product recording from a Weles browser on the Stado-selected dedicated `charless-mac-mini` host, not an animation synthesized from catalog stills. It retains a five-state path from landing through failed lookup, recovery with **Block Kit**, and the first useful official answer.

## Local evidence

- [`media/motion.webm`](media/motion.webm): 640×360, 10.08 seconds, 252 decoded frames, 222444 bytes, SHA-256 `1a3e147addb2c699820c6a4cadf5ed80551f8f3b08aa883d273925465ef94db9`.
- [`reference.json`](reference.json): machine-readable journey, interaction, accessibility, motion, and provenance record.
- Every PNG below came from the same real browser context represented by the motion file.

| Observed state | Local frame | Dimensions | SHA-256 |
|---|---|---:|---|
| landing | [media/state-01-landing.png](media/state-01-landing.png) | 960×540 | `3357d32064ce0467b555863c0250d4523d0c22b09e0497a9f6eea364f6b7255b` |
| search open | [media/state-02-search-open.png](media/state-02-search-open.png) | 960×540 | `2c668a2b2fa66290f815052bc65aa5be03912020b44de57cf6d88bac04862bb3` |
| failed search | [media/state-03-failed-search.png](media/state-03-failed-search.png) | 960×540 | `a2963f083d19d9902d929449e357c86bbe5c453727cc9651620bac8d45980843` |
| recovered results | [media/state-04-results.png](media/state-04-results.png) | 960×540 | `acc7f4959fa6b8b96b3571504e859b6d4b71b225975082d22ff247c681ecf5f6` |
| first answer | [media/state-05-answer.png](media/state-05-answer.png) | 960×540 | `ee492cdf7880b04ca5cb3b9f08be11885613f89eb3fc9233f21cd5a0fe1e5f10` |

## First-success journey

**Actor:** A public documentation visitor looking for one product concept  
**Goal:** Reach the first useful Slack API Documentation answer for “Block Kit”

| Step | User action | Product response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the official documentation URL | Render the public product landing/article surface | landing | media/state-01-landing.png |
| 2 | Invoke the product search or official learn route | Expose a query/result/navigation surface | search or learn open | media/state-02-search-open.png |
| 3 | Request the deliberately impossible term zzwissent-no-result-7f3c9 | Return no useful answer | failed search | media/state-03-failed-search.png |
| 4 | Cancel the failed route and request “Block Kit” | Recover with relevant documentation feedback | recovered results | media/state-04-results.png |
| 5 | Inspect the first relevant product-owned route | Keep the candidate state stable for confirmation | result ready | media/state-04-results.png and media/motion.webm |
| 6 | Open the captured canonical answer | Render the first useful documentation answer | first answer | media/state-05-answer.png and media/motion.webm |

### Failure and recovery

The failure is deliberately observable: `zzwissent-no-result-7f3c9` produces no useful answer. Recovery is also retained: that route is abandoned, **Block Kit** is requested, a relevant result/learn state appears, and the official canonical answer renders. Completion is `media/state-05-answer.png` plus the closing motion.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Load documentation | Navigate to the official documentation URL | The live documentation landing/article surface renders | Origin or network failure prevents the landing state | Reload the same canonical URL | media/state-01-landing.png and opening of media/motion.webm |
| Open search or learn route | Activate the visible native search control, keyboard convention, or official product search/learn route | A product-owned query/result surface appears | Some book-style sites expose no visible inline search field | Use the same product’s official search/result or concept navigation route | media/state-02-search-open.png and media/motion.webm |
| Focus query input | Target the visible query control when the product exposes one | The product accepts text input or a query-bearing official route | A hidden or absent field cannot accept direct text | Continue through the official product result/learn route retained in the recording | media/state-02-search-open.png and media/motion.webm |
| Enter impossible query | Enter or request zzwissent-no-result-7f3c9 | The product returns an unmatched/failed lookup state | No useful answer exists for the deliberately impossible query | Replace it with “Block Kit” | media/state-03-failed-search.png and media/motion.webm |
| Cancel failed query | Clear or leave the impossible-query route | The failed lookup is abandoned | Stale no-result feedback may persist | Load the recovered query/result state | transition from media/state-03-failed-search.png to media/state-04-results.png |
| Submit learn query | Enter or request “Block Kit” | The product renders relevant documentation feedback | A narrow query can yield no exact match | Use the broader recorded product term and official route | media/state-04-results.png and media/motion.webm |
| Inspect result feedback | Wait for the product result/learn surface to settle | Candidate content remains stable for inspection | An empty result surface is not treated as success | Follow the captured canonical concept route | media/state-04-results.png and media/motion.webm |
| Open first answer | Activate or navigate from the recovered product route to the first relevant answer | The official documentation answer renders | A route may land on missing or generic content | Return to results and choose the captured canonical answer route | media/state-05-answer.png and closing of media/motion.webm |

Confirmation is the answer transition. Cancellation/backtracking is the recorded abandonment of the impossible route before confirmation; browser Back is documented only as the available post-navigation path, not claimed as an unrecorded observation.

## Motion analysis

- **Trigger:** Official page navigation followed by product search/learn interaction.
- **Start → end:** public documentation landing/article → first useful canonical answer.
- **Continuity:** single real-browser context video; no synthesized or interpolated frames.
- **Timing:** brief; 10.08 seconds total.
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

- Upstream owner: Slack API Documentation (api.slack.com).
- Product/source URL: https://api.slack.com/docs.
- Recording environment: live official product, Weles AsyncNewBrowser, checksum-verified Weles Chromium, Stado-selected dedicated host `charless-mac-mini`; no local GUI/browser was launched.
- Captured at: 2026-08-16T23:10:48.113Z.
- Product interface/content remains owned by the upstream owner; local evidence is retained for inspectable design reference.
