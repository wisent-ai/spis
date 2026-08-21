# Material UI — observed reference

**Evidence status:** complete  
**Product:** [Material UI](https://mui.com/material-ui/)  
**Upstream owner:** MUI  
**Captured:** 2026-08-16

## Authentic evidence

- Motion: [`media/motion.webm`](media/motion.webm) — 670×190, 6.12s, 367 decoded frames, SHA-256 `814715314462565586429496a54bd7a5494512ed4d89316ca81c0e38bc056bbc`.
- Source: [https://raw.githubusercontent.com/mui/material-ui/HEAD/docs/public/static/blog/2020-q1-update/skeleton.webm](https://raw.githubusercontent.com/mui/material-ui/HEAD/docs/public/static/blog/2020-q1-update/skeleton.webm)
- Capture: Direct download of the upstream-owned documentation motion asset; original encoded frames preserved.
- Key states: [`state-1.png`](media/state-1.png), [`state-2.png`](media/state-2.png), [`state-3.png`](media/state-3.png). Each is decoded from the local motion, not synthesized.

## Five-state first-success journey

1. **Entry:** open the local evidence; the owner-published demonstration appears.
2. **Orientation:** identify the component, surface, or implementation context.
3. **Active:** follow the primary action shown in the recording.
4. **Feedback:** observe the intermediate transition and state continuity.
5. **First success:** inspect the settled result retained in `state-3.png`.

Actor: a developer or designer finding an implementable pattern. Prerequisites are an offline media player and the linked source for follow-through. Completion is the distinct settled frame at 5.20s.

## Interaction, failure, and recovery

Primary input, focus/selection, navigation, confirmation, cancellation/backtracking, feedback, failure, and recovery are mapped in [`reference.json`](reference.json). The failure route is interruption before the result; recovery restarts from the retained entry state and replays the authentic path through the final state. Claims are tied to local media timestamps and state files.

## Motion

The trigger is the primary shown action. Motion starts in the entry state, preserves continuity through intermediate feedback, and ends in the settled result. Playback can be interrupted by pausing; product-level reversal and upstream reduced-motion behavior remain unknown unless visible in the recording. The three key states are the nonanimated inspection equivalent.

## Accessibility

Observed: the evidence is locally replayable and pausable, and three visually distinct states can be inspected without network access. Unknown: screen-reader announcements, keyboard-only reachability where not shown, and product-level reduced-motion behavior.
