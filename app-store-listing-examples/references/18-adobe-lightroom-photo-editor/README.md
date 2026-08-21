# Adobe Lightroom: Photo Editor (Apple App Store)

**Evidence status:** complete  
**Product URL:** [https://apps.apple.com/us/app/id878783582](https://apps.apple.com/us/app/id878783582)  
**Category:** Apple App Store — Photo & Video  
**Upstream owner:** Adobe Inc.  
**Captured:** 2026-08-17T00:32:42Z

Study the before-and-after visual demonstrations, tool-specific screenshot sequencing, recognizable Adobe branding, subscription details, and privacy information for creator acquisition.

## Authentic local motion

| Local asset | Kind | Dimensions | Duration / frames | Bytes | SHA-256 | Provenance |
|---|---|---:|---:|---:|---|---|
| [`media/listing-interaction.webm`](media/listing-interaction.webm) | webm | 1280×720 | 17.24 s / 431 frames | 1739250 | `e6fef0c3a363708517cd77fb85edfa58d51f837649e26cd8f37b638678fd084e` | [source](https://apps.apple.com/us/app/id878783582) |
| [`media/official-store-preview.mp4`](media/official-store-preview.mp4) | mp4 | 1024×576 | 5.001 s / 146 frames | 1363148 | `9cc069da619614591605b39d0a2dd35def65086ab001c4335f9bc5330dec0a5f` | [source](https://apptrailers.itunes.apple.com/itunes-assets/PurpleVideo221/v4/c8/17/16/c81716cc-e7a6-87f4-798e-e4b17aada906/P1406972692_default.m3u8) |

The listing recording is a real browser interaction captured on the dedicated `charless-mac-mini` through Stado and Weles. It is not animation synthesized from stills. The capture begins at canonical navigation, shows direct scroll feedback, records the exact media-target result `no-target-found`, continues into listing details, deliberately overshoots the target, reverses, and ends after returning to the product header.
The second motion asset is the listing publisher's official store preview. Its retained frames at 12%, 50%, and 88% are direct decodes of that authentic MP4, not a synthesized animation. The preview supplies product-native visual states while the Weles recording supplies the observable store interaction path.


## Ordered product and listing states

| Order | State | Local visual | Motion relationship | Dimensions | Bytes | SHA-256 |
|---:|---|---|---|---:|---:|---|
| 1 | listing-open | [`media/listing-state-01.png`](media/listing-state-01.png) | `media/listing-interaction.webm` | 900×506 | 425324 | `33b7119465cb0d098c2a8fefc03a7a18c3a73fa70b6bef318a905dd14ba4277b` |
| 2 | media-shelf | [`media/listing-state-02.png`](media/listing-state-02.png) | `media/listing-interaction.webm` | 900×506 | 435294 | `e0019658530a64d4a0c59810d35c9f713b170730c151ddcc2a1fc920748e1b6f` |
| 3 | selection-unavailable | [`media/listing-state-03.png`](media/listing-state-03.png) | `media/listing-interaction.webm` | 900×506 | 435296 | `b005754ad13fa2542d5844190ca31b02dee9e7a086e067bf1b43e1457d1c7d87` |
| 4 | details-reached | [`media/listing-state-04.png`](media/listing-state-04.png) | `media/listing-interaction.webm` | 900×506 | 214122 | `8c24c6193eafadd1d2e31e0f33317c0b669f4463f8458206ff3bb84d8064ec8d` |
| 5 | target-overshot | [`media/listing-state-05.png`](media/listing-state-05.png) | `media/listing-interaction.webm` | 900×506 | 220992 | `6e30b87c772472e09ffd0018c57f9fbdc545f3e802a665b4dd3619a708b197d3` |
| 6 | recovered-to-media | [`media/listing-state-06.png`](media/listing-state-06.png) | `media/listing-interaction.webm` | 900×506 | 435636 | `d3a676b1c67bfdfa215af5008a22e121544296015a65dabd788f7511aec785d1` |
| 7 | returned-to-header | [`media/listing-state-07.png`](media/listing-state-07.png) | `media/listing-interaction.webm` | 900×506 | 428467 | `19e0bfba4d059b55d175e8b54d54d45f9de39a79214161d0012575749fb3c7d1` |
| 8 | official-preview-key-state-1 | [`media/preview-state-01.png`](media/preview-state-01.png) | `media/official-store-preview.mp4` | 1024×576 | 800672 | `ca48d258a5d9b59f875481f52601155723669ab624215b2ba13db36ab3c6bfcc` |
| 9 | official-preview-key-state-2 | [`media/preview-state-02.png`](media/preview-state-02.png) | `media/official-store-preview.mp4` | 1024×576 | 737720 | `c2465d874aa347cdef33fabdb276cce9b08f030b61105657f87c06312e42946c` |
| 10 | official-preview-key-state-3 | [`media/preview-state-03.png`](media/preview-state-03.png) | `media/official-store-preview.mp4` | 1024×576 | 824578 | `0a16c910d40629557c0700e0cd13d02ad38ca5121ea36eed0365132049d5304e` |

## Discovery-to-open journey

**Actor:** Prospective mobile-app user evaluating a store listing before installation.  
**Goal:** Discover Adobe Lightroom: Photo Editor (Apple App Store), inspect its product media and listing details, recover from a navigation overshoot, and return to the product header ready for the next store decision.

**Prerequisites**

- Network access to the canonical Apple App Store or Google Play listing.
- A browser capable of rendering the listing, scrolling, and exposing store controls.
- No authentication or purchase is required for this observed inspection path.

| Step | User action | System response | State | Evidence |
|---:|---|---|---|---|
| 1 | Open the canonical store URL. | The store resolves the product listing and renders its header. | `listing-open` | media/listing-state-01.png; opening of media/listing-interaction.webm |
| 2 | Wait for deferred artwork and modules to settle. | The listing reaches a stable, inspectable header state. | `listing-settled` | opening segment of media/listing-interaction.webm |
| 3 | Scroll down 520 pixels. | The product screenshot/preview region enters the viewport. | `media-shelf` | media/listing-state-02.png |
| 4 | Attempt to activate the target named Screenshot. | Weles reports no-target-found; no unrelated content is activated. | `selection-unavailable` | media/listing-state-03.png and the selection segment of media/listing-interaction.webm |
| 5 | Continue downward by 760 pixels. | The listing advances to descriptive and trust metadata. | `details-reached` | media/listing-state-04.png |
| 6 | Scroll down a further 1800 pixels. | The intended media/detail target is overshot and later sections appear. | `target-overshot` | media/listing-state-05.png |
| 7 | Reverse upward by 2400 pixels. | Earlier media/detail modules return in reverse order. | `recovered-to-media` | media/listing-state-06.png |
| 8 | Continue upward by 2400 pixels. | The original product header returns, completing the discovery-to-open inspection loop. | `returned-to-header` | media/listing-state-07.png; ending of media/listing-interaction.webm |

**Failure route:** the Screenshot target attempt returns `no-target-found` without activating unrelated content; the later 1800-pixel scroll then overshoots the intended region.  
**Recovery route:** preserve or safely leave the media state, reverse upward by 2400 pixels to restore the media/detail region, then reverse by another 2400 pixels to return to the header.  
**Completion evidence:** media/listing-state-07.png and the final segment of media/listing-interaction.webm show the product header restored after the observed failure and recovery route.

## Interaction map

| Interaction | Trigger | Response | Feedback | Cancellation | Failure | Recovery | Evidence |
|---|---|---|---|---|---|---|---|
| Open listing | Navigate to the canonical store URL. | The store renders the product header and acquisition context. | The product name, icon, rating/context, and listing chrome become visible. | Leave or close the page before interacting. | A network or store response can prevent the header from rendering. | Reload the canonical URL and wait for the header state. | media/listing-interaction.webm and media/listing-state-01.png |
| Confirm load | Pause for two seconds after navigation. | Deferred artwork and listing modules finish rendering. | Stable header and media geometry confirm readiness. | Navigate away before the settle interval ends. | Unsettled or blank media remains visibly incomplete. | Wait for the page to settle before continuing. | media/listing-interaction.webm opening segment |
| Scroll to media | Scroll downward by 520 pixels. | The viewport moves from the header toward the screenshot or preview shelf. | The media region replaces part of the header in the viewport. | Reverse the scroll upward. | A scroll that is too short leaves the media target below the fold. | Continue a measured downward scroll until the shelf is visible. | media/listing-interaction.webm and media/listing-state-02.png |
| Select preview | Activate the visible target named Screenshot. | No matching interactive Screenshot target is exposed at this observed viewport. | no-target-found | Remain on the shelf, dismiss selected media, or reverse to the prior scroll position. | Weles returns no-target-found rather than activating unrelated content. | Continue through the listing with scrolling; the listing remains inspectable. | media/listing-interaction.webm and media/listing-state-03.png |
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
- **Feedback:** changing modules, re-entering content, stable screenshots, and the explicit `no-target-found` result make each outcome visible.
- **Reduced motion/nonanimated equivalent:** the seven retained listing frames preserve the ordered path without playback. Whether the stores honor an operating-system reduced-motion preference was not exercised.

## Accessibility observations and unknowns

Observed:

- The product name and listing content are visibly rendered at the opening state.
- Continuous scrolling preserves context and can be reversed without a modal trap.
- The Screenshot selection attempt produced the explicit Weles result no-target-found, rather than silently activating another element.
- The recording retains stable visual feedback after each navigation action.

Unknown:

- Screen-reader announcements and accessible-name completeness were not audited with assistive technology.
- Keyboard-only focus order and focus visibility were not exercised.
- Caption availability for every upstream preview and autoplay policy are not established.
- Reduced-motion preferences and nonanimated equivalents were not exercised.

## Provenance

The product and listing media remain owned by **Adobe Inc.** and the applicable store. Static page research used the canonical listing. The Weles recording ran on the Stado-selected dedicated Mac mini with Weles Chromium release `147.0.7727.108-weles.1`; its local media metadata and SHA-256 are recorded above and in [`reference.json`](reference.json). No local browser or GUI was launched on the operator workstation.
