# Picsart AI Photo Editor, Video (Apple App Store)

**Evidence status:** complete  
**Product URL:** [https://apps.apple.com/us/app/id587366035](https://apps.apple.com/us/app/id587366035)  
**Category:** Apple App Store — Photo & Video  
**Upstream owner:** PicsArt, Inc.  
**Captured:** 2026-08-17T00:32:58Z

Study how vivid creative outcomes lead the screenshot set before feature controls appear, linking aspirational acquisition media to concrete editing interactions.

## Authentic local motion

| Local asset | Kind | Dimensions | Duration / frames | Bytes | SHA-256 | Provenance |
|---|---|---:|---:|---:|---|---|
| [`media/listing-interaction.webm`](media/listing-interaction.webm) | webm | 1280×720 | 15.56 s / 389 frames | 1550303 | `9580a0b071345c7d0384e3fb2e50c53af2f2485f66dde3b99e9f01bae7b7e0f1` | [source](https://apps.apple.com/us/app/id587366035) |
| [`media/official-store-preview.mp4`](media/official-store-preview.mp4) | mp4 | 1024×576 | 3.2 s / 96 frames | 772960 | `985a664bd6d32daea043d9bf6a4aabd9580fa476665dfc5def48550488211bdc` | [source](https://apptrailers.itunes.apple.com/itunes-assets/PurpleVideo211/v4/90/08/17/9008175e-e89f-5b1e-0b3f-e8b978f33d8b/P1465183426_default.m3u8) |

The listing recording is a real browser interaction captured on the dedicated `charless-mac-mini` through Stado and Weles. It is not animation synthesized from stills. The capture begins at canonical navigation, shows direct scroll feedback, records the exact media-target result `no-target-found`, continues into listing details, deliberately overshoots the target, reverses, and ends after returning to the product header.
The second motion asset is the listing publisher's official store preview. Its retained frames at 12%, 50%, and 88% are direct decodes of that authentic MP4, not a synthesized animation. The preview supplies product-native visual states while the Weles recording supplies the observable store interaction path.


## Ordered product and listing states

| Order | State | Local visual | Motion relationship | Dimensions | Bytes | SHA-256 |
|---:|---|---|---|---:|---:|---|
| 1 | listing-open | [`media/listing-state-01.png`](media/listing-state-01.png) | `media/listing-interaction.webm` | 900×506 | 423336 | `4a97653e437d56a12d0ceffa6c536eba7d53527ef46605a0cf39fe5a6fd65c9f` |
| 2 | media-shelf | [`media/listing-state-02.png`](media/listing-state-02.png) | `media/listing-interaction.webm` | 900×506 | 355807 | `c475aa4192597cde619e72c973b41e96fae61d1fe3baed63cf895bf9c1752d7a` |
| 3 | selection-unavailable | [`media/listing-state-03.png`](media/listing-state-03.png) | `media/listing-interaction.webm` | 900×506 | 355789 | `9289b267ce327ca6674ef53a2fb5a9e528f67195156d466631145868f607f7d4` |
| 4 | details-reached | [`media/listing-state-04.png`](media/listing-state-04.png) | `media/listing-interaction.webm` | 900×506 | 153548 | `a125762e8c1895e449066ff8e4b9576aa8b3a9784ba1946a2dfd5bffee7a598f` |
| 5 | target-overshot | [`media/listing-state-05.png`](media/listing-state-05.png) | `media/listing-interaction.webm` | 900×506 | 151289 | `79cae52332b929fc7fa20316b781ae25188cfba740693c1881107f048b74e9c9` |
| 6 | recovered-to-media | [`media/listing-state-06.png`](media/listing-state-06.png) | `media/listing-interaction.webm` | 900×506 | 377994 | `804f11c0ec24b4806d451588abc2c9f727585475063a278444d18b139b6d69e1` |
| 7 | returned-to-header | [`media/listing-state-07.png`](media/listing-state-07.png) | `media/listing-interaction.webm` | 900×506 | 426712 | `a6a4a232e3a4528d22453d6cdb5a8a95b198c60d7494e296634ec73cb847d202` |
| 8 | official-preview-key-state-1 | [`media/preview-state-01.png`](media/preview-state-01.png) | `media/official-store-preview.mp4` | 1024×576 | 386253 | `942a5f0ce3d292d0ef74859cceafe4c0477491e8d86339c19f1f5a4acabf46a2` |
| 9 | official-preview-key-state-2 | [`media/preview-state-02.png`](media/preview-state-02.png) | `media/official-store-preview.mp4` | 1024×576 | 310267 | `50371c82e984b72b0763639eaa18eaa4cc5305974fb1563c9149bb34df239374` |
| 10 | official-preview-key-state-3 | [`media/preview-state-03.png`](media/preview-state-03.png) | `media/official-store-preview.mp4` | 1024×576 | 287522 | `02be28e5bfa7f9a5a677e64d04cf49e04ed67fa06cf486e2dcb61186d894b76b` |

## Discovery-to-open journey

**Actor:** Prospective mobile-app user evaluating a store listing before installation.  
**Goal:** Discover Picsart AI Photo Editor, Video (Apple App Store), inspect its product media and listing details, recover from a navigation overshoot, and return to the product header ready for the next store decision.

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

The product and listing media remain owned by **PicsArt, Inc.** and the applicable store. Static page research used the canonical listing. The Weles recording ran on the Stado-selected dedicated Mac mini with Weles Chromium release `147.0.7727.108-weles.1`; its local media metadata and SHA-256 are recorded above and in [`reference.json`](reference.json). No local browser or GUI was launched on the operator workstation.
