# Medium (Google Play)

**Evidence status:** complete  
**Product URL:** [https://play.google.com/store/apps/details?id=com.medium.reader](https://play.google.com/store/apps/details?id=com.medium.reader)  
**Category:** Google Play — News & Magazines  
**Upstream owner:** A Medium Corporation  
**Captured:** 2026-08-16T23:42:45Z

Study how article typography and topic discovery are allowed to carry the visual story, connecting acquisition media directly to the reading experience.

## Authentic local motion

| Local asset | Kind | Dimensions | Duration / frames | Bytes | SHA-256 | Provenance |
|---|---|---:|---:|---:|---|---|
| [`media/listing-interaction.webm`](media/listing-interaction.webm) | webm | 1280×720 | 18.8 s / 470 frames | 1153153 | `2a4ae1e8867aedfcf6852d4195d250ef4545ab2e3469368f86ac0127bcd65aea` | [source](https://play.google.com/store/apps/details?id=com.medium.reader) |

The listing recording is a real browser interaction captured on the dedicated `charless-mac-mini` through Stado and Weles. It is not animation synthesized from stills. The capture begins at canonical navigation, shows direct scroll feedback, records the exact media-target result `clicked button: Screenshot`, continues into listing details, deliberately overshoots the target, reverses, and ends after returning to the product header.

## Ordered product and listing states

| Order | State | Local visual | Motion relationship | Dimensions | Bytes | SHA-256 |
|---:|---|---|---|---:|---:|---|
| 1 | listing-open | [`media/listing-state-01.png`](media/listing-state-01.png) | `media/listing-interaction.webm` | 1920×1080 | 347383 | `86af6bdb294a2f3841c807821917ddd3d449aa0c38598b29ff2e09f2ea76e5a1` |
| 2 | media-shelf | [`media/listing-state-02.png`](media/listing-state-02.png) | `media/listing-interaction.webm` | 1920×1080 | 289489 | `276cc3a334fc8ccbc0aca7d1db3d1c2cee8a8ac6173487186b88afda3df54eb8` |
| 3 | media-selected | [`media/listing-state-03.png`](media/listing-state-03.png) | `media/listing-interaction.webm` | 1920×1080 | 440822 | `12415f057f6482a1b98f1d9bb38965f6a4281af853f923f7eb45f8786aa44ced` |
| 4 | details-reached | [`media/listing-state-04.png`](media/listing-state-04.png) | `media/listing-interaction.webm` | 1920×1080 | 138517 | `102d7cbad872a81f44ef23c71fd1dc735633c04b1875a13497f14e289c9f5c2c` |
| 5 | target-overshot | [`media/listing-state-05.png`](media/listing-state-05.png) | `media/listing-interaction.webm` | 1920×1080 | 111651 | `09db2aab0f5cb9e655111fb4c782f9586bd0651824771d1eaff4e94ae6c91e7d` |
| 6 | recovered-to-media | [`media/listing-state-06.png`](media/listing-state-06.png) | `media/listing-interaction.webm` | 1920×1080 | 347383 | `86af6bdb294a2f3841c807821917ddd3d449aa0c38598b29ff2e09f2ea76e5a1` |
| 7 | returned-to-header | [`media/listing-state-07.png`](media/listing-state-07.png) | `media/listing-interaction.webm` | 1920×1080 | 347383 | `86af6bdb294a2f3841c807821917ddd3d449aa0c38598b29ff2e09f2ea76e5a1` |

## Discovery-to-open journey

**Actor:** Prospective mobile-app user evaluating a store listing before installation.  
**Goal:** Discover Medium (Google Play), inspect its product media and listing details, recover from a navigation overshoot, and return to the product header ready for the next store decision.

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

The product and listing media remain owned by **A Medium Corporation** and the applicable store. Static page research used the canonical listing. The Weles recording ran on the Stado-selected dedicated Mac mini with Weles Chromium release `147.0.7727.108-weles.1`; its local media metadata and SHA-256 are recorded above and in [`reference.json`](reference.json). No local browser or GUI was launched on the operator workstation.
