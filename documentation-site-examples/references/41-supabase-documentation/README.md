# 41. Supabase Documentation — observed search/learn-to-first-answer reference

**Evidence status:** complete  
**Official product:** [https://supabase.com/docs](https://supabase.com/docs)  
**Playable motion:** [motion.webm](media/motion.webm)  
**Captured:** 2026-08-16T23:06:22.683Z

This is a real live-product recording from a Weles browser on the Stado-selected dedicated `charless-mac-mini` host, not an animation synthesized from catalog stills. It retains a five-state path from landing through failed lookup, recovery with **row level security**, and the first useful official answer.

## Local evidence

- [`media/motion.webm`](media/motion.webm): 640×360, 9.16 seconds, 229 decoded frames, 261534 bytes, SHA-256 `fd845113adc172e247aecfca31393b57265cfdccd4f1a321e099f4435ce442c0`.
- [`reference.json`](reference.json): machine-readable journey, interaction, accessibility, motion, and provenance record.
- Every PNG below came from the same real browser context represented by the motion file.

| Observed state | Local frame | Dimensions | SHA-256 |
|---|---|---:|---|
| landing | [media/state-01-landing.png](media/state-01-landing.png) | 960×540 | `a73640f2586fa96983ad8ca284a2341acd7a0d32cc1ca7bdbef876df0d1ce765` |
| search open | [media/state-02-search-open.png](media/state-02-search-open.png) | 960×540 | `a093b5f929f67a92a1e2cda6b3125c06716306970be5dbd9e8bb37381ed0dadf` |
| failed search | [media/state-03-failed-search.png](media/state-03-failed-search.png) | 960×540 | `dbc39d6255a073bbf552301109a8ab4f6ddc7e1cbd8a0896fa1c6bea6e0fe85d` |
| recovered results | [media/state-04-results.png](media/state-04-results.png) | 960×540 | `4b08f0cefa15cb6b98653c0a4a49744b418f275e297d38881e0eb293506f6f7b` |
| first answer | [media/state-05-answer.png](media/state-05-answer.png) | 960×540 | `ee12883ea2882989bbc9831268964b69f8c65e910d628391aa3ceb880b30a87d` |

## First-success journey

**Actor:** A public documentation visitor looking for one product concept  
**Goal:** Reach the first useful Supabase Documentation answer for “row level security”

| Step | User action | Product response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the official documentation URL | Render the public product landing/article surface | landing | media/state-01-landing.png |
| 2 | Invoke the product search or official learn route | Expose a query/result/navigation surface | search or learn open | media/state-02-search-open.png |
| 3 | Request the deliberately impossible term zzwissent-no-result-7f3c9 | Return no useful answer | failed search | media/state-03-failed-search.png |
| 4 | Cancel the failed route and request “row level security” | Recover with relevant documentation feedback | recovered results | media/state-04-results.png |
| 5 | Inspect the first relevant product-owned route | Keep the candidate state stable for confirmation | result ready | media/state-04-results.png and media/motion.webm |
| 6 | Open the captured canonical answer | Render the first useful documentation answer | first answer | media/state-05-answer.png and media/motion.webm |

### Failure and recovery

The failure is deliberately observable: `zzwissent-no-result-7f3c9` produces no useful answer. Recovery is also retained: that route is abandoned, **row level security** is requested, a relevant result/learn state appears, and the official canonical answer renders. Completion is `media/state-05-answer.png` plus the closing motion.

## Interaction map

| Interaction | Trigger | Response | Failure | Recovery | Evidence |
|---|---|---|---|---|---|
| Load documentation | Navigate to the official documentation URL | The live documentation landing/article surface renders | Origin or network failure prevents the landing state | Reload the same canonical URL | media/state-01-landing.png and opening of media/motion.webm |
| Open search or learn route | Activate the visible native search control, keyboard convention, or official product search/learn route | A product-owned query/result surface appears | Some book-style sites expose no visible inline search field | Use the same product’s official search/result or concept navigation route | media/state-02-search-open.png and media/motion.webm |
| Focus query input | Target the visible query control when the product exposes one | The product accepts text input or a query-bearing official route | A hidden or absent field cannot accept direct text | Continue through the official product result/learn route retained in the recording | media/state-02-search-open.png and media/motion.webm |
| Enter impossible query | Enter or request zzwissent-no-result-7f3c9 | The product returns an unmatched/failed lookup state | No useful answer exists for the deliberately impossible query | Replace it with “row level security” | media/state-03-failed-search.png and media/motion.webm |
| Cancel failed query | Clear or leave the impossible-query route | The failed lookup is abandoned | Stale no-result feedback may persist | Load the recovered query/result state | transition from media/state-03-failed-search.png to media/state-04-results.png |
| Submit learn query | Enter or request “row level security” | The product renders relevant documentation feedback | A narrow query can yield no exact match | Use the broader recorded product term and official route | media/state-04-results.png and media/motion.webm |
| Inspect result feedback | Wait for the product result/learn surface to settle | Candidate content remains stable for inspection | An empty result surface is not treated as success | Follow the captured canonical concept route | media/state-04-results.png and media/motion.webm |
| Open first answer | Activate or navigate from the recovered product route to the first relevant answer | The official documentation answer renders | A route may land on missing or generic content | Return to results and choose the captured canonical answer route | media/state-05-answer.png and closing of media/motion.webm |

Confirmation is the answer transition. Cancellation/backtracking is the recorded abandonment of the impossible route before confirmation; browser Back is documented only as the available post-navigation path, not claimed as an unrecorded observation.

## Motion analysis

- **Trigger:** Official page navigation followed by product search/learn interaction.
- **Start → end:** public documentation landing/article → first useful canonical answer.
- **Continuity:** single real-browser context video; no synthesized or interpolated frames.
- **Timing:** brief; 9.16 seconds total.
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

- Upstream owner: Supabase Documentation (supabase.com).
- Product/source URL: https://supabase.com/docs.
- Recording environment: live official product, Weles AsyncNewBrowser, checksum-verified Weles Chromium, Stado-selected dedicated host `charless-mac-mini`; no local GUI/browser was launched.
- Captured at: 2026-08-16T23:06:22.683Z.
- Product interface/content remains owned by the upstream owner; local evidence is retained for inspectable design reference.
