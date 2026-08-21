# PageSpeed Insights — full product reference

**Evidence status:** complete  
**Product:** [PageSpeed Insights](https://developers.google.com/speed/docs/insights/v5/about)  
**Upstream owner:** Google for Developers  
**Captured:** 2026-08-16T23:30:33Z

## Authentic motion

<video controls muted preload="metadata" src="media/journey.mp4" width="640"></video>

The local motion is an authentic upstream product demonstration retained through Downloaded the upstream-published YouTube video through the Cobalt media gateway, then transcoded a continuous 12-second excerpt beginning at source timestamp 180s; no frames were synthesized. It is not an animation synthesized from the state images.

| Property | Value |
|---|---|
| Source | [https://www.youtube.com/watch?v=BaWokURhU_k&t=180s](https://www.youtube.com/watch?v=BaWokURhU_k&t=180s) |
| Local file | [`media/journey.mp4`](media/journey.mp4) |
| Kind | `video/mp4` |
| Dimensions | 256 × 144 |
| Duration / frames | 12.012 seconds / 360 frames |
| Bytes | 89398 |
| SHA-256 | `42b80f8c5aaaeb06d7b77eb0b3c7abb88cb104a096603291db4facf23f403216` |
| Capture method | Downloaded the upstream-published YouTube video through the Cobalt media gateway, then transcoded a continuous 12-second excerpt beginning at source timestamp 180s; no frames were synthesized |

## Retained product states

All five states were retained from, or while capturing, the authentic motion source.

| State | Local visual | SHA-256 |
|---|---|---|
| evidence ready | ![PageSpeed Insights evidence ready](media/state-01.png) | `fae7600affb677b5857a19691ed81619e2d2e2a0ef2803edda0219af94c1b3e6` |
| overview visible | ![PageSpeed Insights overview visible](media/state-02.png) | `a9533c9ce8f3e9711b9c2d796336f7964ca0075728478b1222c51aa372d19d87` |
| paused detail | ![PageSpeed Insights paused detail](media/state-03.png) | `dea793349d444a486aa7bed79e4c80d94902c53a6ae611bc0944ac806dbc67dd` |
| deeper evidence | ![PageSpeed Insights deeper evidence](media/state-04.png) | `99d1a2e97117aa9fd51e59ed18eecf0325f3b8488c503d43dc142a45dd1c85ac` |
| completion evidence | ![PageSpeed Insights completion evidence](media/state-05.png) | `7db3189717ea5f17d51255d7f4bb8905c518c75b6993c018401dce3dca5c3f8d` |

## First-success investigation journey

**Actor:** A reviewer validating a summary claim against product-native evidence  
**Goal:** Study the separation of field and lab data, including population provenance, metric thresholds, diagnostics, and actionable opportunities.

| Step | User action | System response | Evidence |
|---:|---|---|---|
| 1 | Open the local evidence recording. | The authentic upstream demonstration is ready at the chosen investigation segment. | [evidence ready](media/state-01.png) |
| 2 | Start playback and orient to the report or evidence surface. | The product moves from its summary context toward an inspectable evidence view. | [overview visible](media/state-02.png) |
| 3 | Pause when the selected detail becomes visible. | The viewer freezes the product-native state for inspection. | [paused detail](media/state-03.png) |
| 4 | Resume and navigate farther into the recorded investigation. | A later product state reveals deeper or differently scoped evidence. | [deeper evidence](media/state-04.png) |
| 5 | Continue to the retained completion point. | The source journey leaves a final visible state that supports the investigation outcome. | [completion evidence](media/state-05.png) |

**Failure route:** Playback is paused or a seek skips the intended evidence transition. The reviewer does not treat the interrupted or skipped state as completed evidence.  
**Recovery route:** Resume from the paused position or seek backward. Use the retained state images to re-establish context, then continue to media/state-05.png.  
**Completion evidence:** `media/journey.mp4 together with media/state-05.png`

## Interaction map

| Interaction | Trigger | Response and feedback | Failure | Recovery |
|---|---|---|---|---|
| open evidence | Open the local motion asset at its beginning. | The evidence viewer presents the upstream product demonstration at the selected investigation segment. | A missing or unreadable asset prevents the product state from appearing. | Reopen the hash-verified local asset. |
| start playback | Activate play. | The playhead advances through authentic upstream product motion. | Playback can remain on the ready frame when interrupted. | Activate play again. |
| focus or select | Follow the source recording as its pointer or focus changes the visible evidence surface. | The product binds a selected summary, row, finding, span, report item, or visual mark to more specific evidence. | A transient frame can make the selected target ambiguous. | Step backward and replay the transition. |
| pause for inspection | Pause at the retained detail state. | Motion stops while the selected product evidence remains visible. | The evidence flow is intentionally interrupted at the paused state. | Resume playback from the same playhead position. |
| resume after interruption | Activate play after the pause. | The source journey continues from the retained detail. | Resumption may not advance when the player remains paused. | Activate play and confirm the next visible frame. |
| navigate forward | Seek forward within the local recording. | The viewer reveals a later product state without changing evidence provenance. | An over-large seek can skip the intended intermediate state. | Use the retained state images or seek backward. |
| backtrack | Seek backward or return to an earlier retained state. | The prior evidence context is restored. | A viewer without precise seeking can land between documented states. | Open the corresponding local state image directly. |
| confirm completion | Continue to the retained completion point. | A later, more specific product evidence state is visible. | Stopping early leaves the summary claim unsupported by the final retained state. | Resume or seek to the completion frame. |

Cancellation and backtracking are preserved by pause and reverse seeking; these operations do not alter the source evidence or its hash.

## Motion behavior

- **Trigger:** explicit play or resume in the evidence viewer.
- **Start state:** `media/state-01.png`; **end state:** `media/state-05.png`.
- **Continuity:** continuous recorded product motion, with discrete state images as the nonanimated inspection equivalent.
- **Timing class:** user-controlled media time; the captured duration is 12.012 seconds.
- **Interruption and reversal:** pause interrupts without losing position; seek backward reverses the inspection path; resume recovers it.
- **Feedback:** visible product-state changes and the player's playhead jointly show progress.
- **Reduced motion:** the five local states are available without playback. The product's own reduced-motion behavior is unknown.

## Accessibility

**Observed**

- Playback can be paused, resumed, and revisited without changing the underlying evidence identity.
- Five discrete local states provide a nonanimated inspection path for the recorded visual sequence.

**Unknown from this visual recording**

- The source recording does not expose the product accessibility tree, screen-reader announcements, or exact focus order.
- Reduced-motion behavior inside the recorded product is not established by this visual evidence.
- Audio narration and caption quality were not used as evidence in this reference.

## Provenance boundary

The cited recording proves only the visible states and transitions retained here. Product semantics not visible in the motion or state images remain unknown; marketing claims and inference are not promoted to observation.
