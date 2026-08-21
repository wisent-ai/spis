# Amazon Shopping (Google Play)

**Evidence status:** complete  
**Product URL:** [https://play.google.com/store/apps/details?id=com.amazon.mShop.android.shopping](https://play.google.com/store/apps/details?id=com.amazon.mShop.android.shopping)  
**Category:** Google Play — Shopping  
**Upstream owner:** Amazon Mobile LLC  
**Captured:** 2026-08-16T23:35:21Z

Study the conversion continuity between product-discovery screenshots and familiar shopping interactions, supplemented by review scale, data-safety details, and frequent maintenance updates.

## Authentic local motion

| Local asset | Kind | Dimensions | Duration / frames | Bytes | SHA-256 | Provenance |
|---|---|---:|---:|---:|---|---|
| [`media/listing-interaction.webm`](media/listing-interaction.webm) | webm | 1280×720 | 17.16 s / 429 frames | 1339107 | `43ea335ae4d74adc7b589d30f93fb030cca7d3ac9f40db1cf03c00700f96b230` | [source](https://play.google.com/store/apps/details?id=com.amazon.mShop.android.shopping) |
| [`media/official-store-preview.mp4`](media/official-store-preview.mp4) | mp4 | 1280×720 | 6.95 s / 139 frames | 289825 | `df50e8374dd4684069bcb7e9afa937fc6cfd50517965e8eed1cd19b6f7f16bbe` | [source](https://play-games.googleusercontent.com/vp/mp4/1280x720/0cXihLp1aIQ.mp4) |

The listing recording is a real browser interaction captured on the dedicated `charless-mac-mini` through Stado and Weles. It is not animation synthesized from stills. The capture begins at canonical navigation, shows direct scroll feedback, records the exact media-target result `clicked button: Screenshot`, continues into listing details, deliberately overshoots the target, reverses, and ends after returning to the product header.
The second motion asset is the listing publisher's official store preview. Its retained frames at 12%, 50%, and 88% are direct decodes of that authentic MP4, not a synthesized animation. The preview supplies product-native visual states while the Weles recording supplies the observable store interaction path.


## Ordered product and listing states

| Order | State | Local visual | Motion relationship | Dimensions | Bytes | SHA-256 |
|---:|---|---|---|---:|---:|---|
| 1 | listing-open | [`media/listing-state-01.png`](media/listing-state-01.png) | `media/listing-interaction.webm` | 1366×768 | 491844 | `b21a124f26fedc8d92621c7d66d2b87b1fa896cfbaf81a6aec57be9cc05505c9` |
| 2 | media-shelf | [`media/listing-state-02.png`](media/listing-state-02.png) | `media/listing-interaction.webm` | 1366×768 | 282044 | `8cef880e0a56e18255e5724f3c435041010f6387866c180b86269deb1ff13275` |
| 3 | media-selected | [`media/listing-state-03.png`](media/listing-state-03.png) | `media/listing-interaction.webm` | 1366×768 | 253368 | `9acf979efc5478892895e25d52f198abb38c18d666a362f8c92b21ee82bbaeda` |
| 4 | details-reached | [`media/listing-state-04.png`](media/listing-state-04.png) | `media/listing-interaction.webm` | 1366×768 | 214859 | `6e61f4c37ad95c92531011f18aa74644364ad17f70b78e5f5680fd08e03686ee` |
| 5 | target-overshot | [`media/listing-state-05.png`](media/listing-state-05.png) | `media/listing-interaction.webm` | 1366×768 | 69294 | `6599123ce8dacd4573380e5df3f41b1bafa7d13165537b8b3ae0b487a658bb24` |
| 6 | recovered-to-media | [`media/listing-state-06.png`](media/listing-state-06.png) | `media/listing-interaction.webm` | 1366×768 | 283440 | `b8a582f7a5508baffefff99cfad934d6c05476bcd2e565e6a0aa37cceca83644` |
| 7 | returned-to-header | [`media/listing-state-07.png`](media/listing-state-07.png) | `media/listing-interaction.webm` | 1366×768 | 491844 | `b21a124f26fedc8d92621c7d66d2b87b1fa896cfbaf81a6aec57be9cc05505c9` |
| 8 | official-preview-key-state-1 | [`media/preview-state-01.png`](media/preview-state-01.png) | `media/official-store-preview.mp4` | 1280×720 | 68602 | `1296e796b54ef667c542aa95e1fd0ce7c4a1a21f4df14f1604711e4dd3149e09` |
| 9 | official-preview-key-state-2 | [`media/preview-state-02.png`](media/preview-state-02.png) | `media/official-store-preview.mp4` | 1280×720 | 93276 | `a36691582f109ba3e8c031380d0cc029fdb39ab23115fa58e3cdd4505f0be744` |
| 10 | official-preview-key-state-3 | [`media/preview-state-03.png`](media/preview-state-03.png) | `media/official-store-preview.mp4` | 1280×720 | 111296 | `d83999349f2fc18648713d00d247191acbe15c7304cede2fc3ad9f4ff2d61570` |

## Discovery-to-open journey

**Actor:** Prospective mobile-app user evaluating a store listing before installation.  
**Goal:** Discover Amazon Shopping (Google Play), inspect its product media and listing details, recover from a navigation overshoot, and return to the product header ready for the next store decision.

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

The product and listing media remain owned by **Amazon Mobile LLC** and the applicable store. Static page research used the canonical listing. The Weles recording ran on the Stado-selected dedicated Mac mini with Weles Chromium release `147.0.7727.108-weles.1`; its local media metadata and SHA-256 are recorded above and in [`reference.json`](reference.json). No local browser or GUI was launched on the operator workstation.
