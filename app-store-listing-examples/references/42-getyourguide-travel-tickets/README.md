# GetYourGuide: Travel & Tickets (Google Play)

**Evidence status:** complete  
**Product URL:** [https://play.google.com/store/apps/details?id=com.getyourguide.android](https://play.google.com/store/apps/details?id=com.getyourguide.android)  
**Category:** Google Play — Travel & Local  
**Upstream owner:** GetYourGuide  
**Captured:** 2026-08-16T23:40:32Z

Study how experience photography creates desire before screenshots introduce search, availability, reviews, mobile tickets, and booking-management interactions.

## Authentic local motion

| Local asset | Kind | Dimensions | Duration / frames | Bytes | SHA-256 | Provenance |
|---|---|---:|---:|---:|---|---|
| [`media/listing-interaction.webm`](media/listing-interaction.webm) | webm | 1280×720 | 40.32 s / 1008 frames | 2398237 | `d3fb2c90b17f356fd39a66b3239b218d027370d7621f34e65e68dbf94e3532ac` | [source](https://play.google.com/store/apps/details?id=com.getyourguide.android) |
| [`media/official-store-preview.mp4`](media/official-store-preview.mp4) | mp4 | 1280×720 | 6.95 s / 139 frames | 514299 | `b77c844faac51af08dbc4556e97173cf491fad08b174cc072b9af906b0b0ab2f` | [source](https://play-games.googleusercontent.com/vp/mp4/1280x720/X02MK1hOr4M.mp4) |

The listing recording is a real browser interaction captured on the dedicated `charless-mac-mini` through Stado and Weles. It is not animation synthesized from stills. The capture begins at canonical navigation, shows direct scroll feedback, records the exact media-target result `clicked button: Screenshot`, continues into listing details, deliberately overshoots the target, reverses, and ends after returning to the product header.
The second motion asset is the listing publisher's official store preview. Its retained frames at 12%, 50%, and 88% are direct decodes of that authentic MP4, not a synthesized animation. The preview supplies product-native visual states while the Weles recording supplies the observable store interaction path.


## Ordered product and listing states

| Order | State | Local visual | Motion relationship | Dimensions | Bytes | SHA-256 |
|---:|---|---|---|---:|---:|---|
| 1 | listing-open | [`media/listing-state-01.png`](media/listing-state-01.png) | `media/listing-interaction.webm` | 5120×3200 | 3411260 | `434b806093c4040ad0034212878272e7bf880f79e90d0086f5c89e05e942fa08` |
| 2 | media-shelf | [`media/listing-state-02.png`](media/listing-state-02.png) | `media/listing-interaction.webm` | 5120×3200 | 2092626 | `3f82962bf8efa95017fd29555d15f4d8bb85ade127b904eb2bd46d1c41dab3fa` |
| 3 | media-selected | [`media/listing-state-03.png`](media/listing-state-03.png) | `media/listing-interaction.webm` | 5120×3200 | 3344884 | `d0a3dcb38968541148771c158199556b94db5b96ddca9a2c1ba30e9aec4b10fa` |
| 4 | details-reached | [`media/listing-state-04.png`](media/listing-state-04.png) | `media/listing-interaction.webm` | 5120×3200 | 1142151 | `cd50d0446166467e6c017f672b720792f50aac29bd0cabeacafb136fa6464e45` |
| 5 | target-overshot | [`media/listing-state-05.png`](media/listing-state-05.png) | `media/listing-interaction.webm` | 5120×3200 | 485312 | `aad8ac39bbfd1385a00d73a0ff0be21f392facf7aa873375fa7b5a7c65eeb9d3` |
| 6 | recovered-to-media | [`media/listing-state-06.png`](media/listing-state-06.png) | `media/listing-interaction.webm` | 5120×3200 | 3107662 | `7d7e0741bc26c7f9432c2fe98506f8268ff6d0fedc94d75df50b0c8f6bc49a9f` |
| 7 | returned-to-header | [`media/listing-state-07.png`](media/listing-state-07.png) | `media/listing-interaction.webm` | 5120×3200 | 3412381 | `38ea2c4a98bc3603b73d7a3cfd02f1c3378ea361d5418c0b8ee0c6bb53dcb842` |
| 8 | official-preview-key-state-1 | [`media/preview-state-01.png`](media/preview-state-01.png) | `media/official-store-preview.mp4` | 1280×720 | 563697 | `3ba0bc184e988f17b487d4282ad507c9e77ec4fd43699e48c8440d52766ef0ca` |
| 9 | official-preview-key-state-2 | [`media/preview-state-02.png`](media/preview-state-02.png) | `media/official-store-preview.mp4` | 1280×720 | 642546 | `5559b0195f8c91a451ee8107ae3f1553f20388064c5b641ccc1b43dd1ca4e4e5` |
| 10 | official-preview-key-state-3 | [`media/preview-state-03.png`](media/preview-state-03.png) | `media/official-store-preview.mp4` | 1280×720 | 567679 | `069bb62005973f9142599431887e0aaad5b40163e6e4e57d4d533f51275a80bc` |

## Discovery-to-open journey

**Actor:** Prospective mobile-app user evaluating a store listing before installation.  
**Goal:** Discover GetYourGuide: Travel & Tickets (Google Play), inspect its product media and listing details, recover from a navigation overshoot, and return to the product header ready for the next store decision.

**Prerequisites**

- Network access to the canonical Apple App Store or Google Play listing.
- A browser capable of rendering the listing, scrolling, and exposing store controls.
- No authentication or purchase is required for this observed inspection path.

| Step | User action | System response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the canonical store URL. | The store resolves the product listing and renders its header. | `listing-open` | media/listing-state-01.png; opening of media/listing-interaction.webm |
| 2 | Wait for deferred artwork and modules to settle. | The listing reaches a stable, inspectable header state. | `listing-settled` | opening segment of media/listing-interaction.webm |
| 3 | Scroll down 520 pixels. | The product screenshot/preview region enters the viewport. | `media-shelf` | media/listing-state-02.png |
| 4 | Attempt to activate the target named Screenshot. | Weles reports clicked button: Screenshot; the visible screenshot control is activated. | `media-selected` | media/listing-state-03.png and the selection segment of media/listing-interaction.webm |
| 5 | Continue downward by 760 pixels. | The listing advances to descriptive and trust metadata. | `details-reached` | media/listing-state-04.png |
| 6 | Scroll down a further 1800 pixels. | The intended media/detail target is overshot and later sections appear. | `target-overshot` | media/listing-state-05.png |
| 7 | Reverse upward by 2400 pixels. | Earlier media/detail modules return in reverse order. | `recovered-to-media` | media/listing-state-06.png |
| 8 | Continue upward by 2400 pixels. | The original product header returns, completing the discovery-to-open inspection loop. | `returned-to-header` | media/listing-state-07.png; ending of media/listing-interaction.webm |

**Failure route:** the Screenshot control is selected with result `clicked button: Screenshot`; the later 1800-pixel scroll then overshoots the intended region.  
**Recovery route:** preserve or safely leave the media state, reverse upward by 2400 pixels to restore the media/detail region, then reverse by another 2400 pixels to return to the header.  
**Completion evidence:** media/listing-state-07.png and the final segment of media/listing-interaction.webm show the product header restored after the observed failure and recovery route.

## Interaction map

| Interaction | Trigger | Response | Feedback | Cancellation | Failure | Recovery | Evidence |
|---|---|---|---|---|---|---|---|
| Open listing | Navigate to the canonical store URL. | The store renders the product header and acquisition context. | The product name, icon, rating/context, and listing chrome become visible. | Leave or close the page before interacting. | A network or store response can prevent the header from rendering. | Reload the canonical URL and wait for the header state. | media/listing-interaction.webm and media/listing-state-01.png |
| Confirm load | Pause for two seconds after navigation. | Deferred artwork and listing modules finish rendering. | Stable header and media geometry confirm readiness. | Navigate away before the settle interval ends. | Unsettled or blank media remains visibly incomplete. | Wait for the page to settle before continuing. | media/listing-interaction.webm opening segment |
| Scroll to media | Scroll downward by 520 pixels. | The viewport moves from the header toward the screenshot or preview shelf. | The media region replaces part of the header in the viewport. | Reverse the scroll upward. | A scroll that is too short leaves the media target below the fold. | Continue a measured downward scroll until the shelf is visible. | media/listing-interaction.webm and media/listing-state-02.png |
| Select preview | Activate the visible target named Screenshot. | The store activates the visible screenshot media control. | clicked button: Screenshot | Remain on the shelf, dismiss selected media, or reverse to the prior scroll position. | No selection failure is observed; the later scroll overshoot supplies the recorded failure route. | Dismiss or leave the selected media context by continuing the reversible listing path. | media/listing-interaction.webm and media/listing-state-03.png |
| Continue to details | Scroll downward by 760 pixels. | Description, metadata, ratings, privacy, or data-safety content enters the viewport. | The viewport changes continuously with the scroll. | Scroll upward to return to media. | The target detail can remain below the fold on a taller listing. | Use another measured downward scroll. | media/listing-interaction.webm and media/listing-state-04.png |
| Overshoot target | Scroll downward by 1800 pixels after reaching details. | The viewport moves beyond the intended media/detail target. | The target shelf is no longer visible and later listing sections appear. | Stop the gesture before its end. | The intended inspection target is overshot. | Reverse direction with an upward scroll. | media/listing-interaction.webm and media/listing-state-05.png |
| Recover position | Scroll upward by 2400 pixels after the overshoot. | The viewport returns toward the media and detail region. | Previously seen modules re-enter the viewport in reverse order. | Reverse downward again. | A partial reverse can stop short of the target. | Continue upward until the recognizable shelf returns. | media/listing-interaction.webm and media/listing-state-06.png |
| Backtrack to header | Scroll upward by a further 2400 pixels. | The listing returns to its product header and acquisition context. | The original product header is visible again. | Stop at any intermediate listing section. | Large pages may require another upward gesture. | Repeat the upward gesture until the header is restored. | media/listing-interaction.webm and media/listing-state-07.png |

## Motion analysis

- **Trigger:** canonical URL navigation starts the real listing recording; scroll and selection gestures trigger subsequent transitions.
- **Start/end:** the listing begins at initial navigation and ends with the product header restored after reversal.
- **Continuity:** viewport motion is continuous during scrolls; one-second settle intervals make state boundaries inspectable.
- **Timing class:** direct manipulation feedback is immediate; explicit waits are one second after each gesture and two seconds after load.
- **Interruption/reversal:** scrolling can be stopped or reversed; the recorded overshoot is recovered through two upward gestures.
- **Feedback:** changing modules, re-entering content, stable screenshots, and the explicit `clicked button: Screenshot` result make each outcome visible.
- **Reduced motion/nonanimated equivalent:** the seven retained listing frames preserve the ordered path without playback. Whether the stores honor an operating-system reduced-motion preference was not exercised.

## Accessibility observations and unknowns

Observed:

- The product name and listing content are visibly rendered at the opening state.
- Continuous scrolling preserves context and can be reversed without a modal trap.
- The Screenshot selection attempt produced the explicit Weles result clicked button: Screenshot, rather than silently activating another element.
- The recording retains stable visual feedback after each navigation action.

Unknown:

- Screen-reader announcements and accessible-name completeness were not audited with assistive technology.
- Keyboard-only focus order and focus visibility were not exercised.
- Caption availability for every upstream preview and autoplay policy are not established.
- Reduced-motion preferences and nonanimated equivalents were not exercised.

## Provenance

The product and listing media remain owned by **GetYourGuide** and the applicable store. Static page research used the canonical listing. The Weles recording ran on the Stado-selected dedicated Mac mini with Weles Chromium release `147.0.7727.108-weles.1`; its local media metadata and SHA-256 are recorded above and in [`reference.json`](reference.json). No local browser or GUI was launched on the operator workstation.
