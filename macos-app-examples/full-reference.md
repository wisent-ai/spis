# Full macOS app interaction reference

This synthesis is derived from all 50 complete per-product records in [`references.json`](references.json). It does not substitute for them: every row below resolves to a self-contained directory with playable local motion, five frame-derived states, an observed five-step first-success sequence, eight interaction records, failure/recovery boundaries, accessibility observations, and cryptographic provenance.

## Completion summary

- **Reference records:** 50 / 50
- **Complete records:** 50 / 50
- **Playable local motion assets:** 50 MP4 excerpts
- **Local key states:** 250 PNG frames (five per product)
- **Observed first-success steps:** 250 (five per product)
- **Interaction records:** 400 (eight per product)
- **Motion provenance:** 10 direct official product/App Store sources, 38 video-source recordings, and 2 Vimeo-source recordings; each record identifies its exact source and recording owner.
- **Integrity:** every motion and state file has byte size, dimensions, and SHA-256 in its local `reference.json`.

## Coverage by catalog category

| Category | Records |
|---|---:|
| Automation and productivity | 1 |
| Communication | 4 |
| Creative software | 7 |
| Database tool | 5 |
| Developer tool | 9 |
| Document and knowledge management | 2 |
| Document app | 6 |
| Launcher and productivity | 2 |
| Menu-bar utility | 4 |
| Operations | 5 |
| Productivity | 4 |
| Window management | 1 |

## Recurring interaction patterns

1. **Invoke, focus, act, confirm, verify.** Across utilities, launchers, editors, database tools, and operations clients, the most stable first-success shape is a five-state progression from an inert or entry surface to a locally visible result.
2. **Selection is the dominant feedback mechanism.** Menu items, search results, sidebar rows, canvas objects, tracks, branches, tables, and tasks all make progress legible by changing the active selection before a consequential action.
3. **Dense apps preserve context instead of navigating away.** Developer tools, database clients, and creative software usually keep navigation, working content, and an inspector or result pane visible together. This reduces recovery cost because the user can return to the last valid selection.
4. **Compact apps compress the same grammar.** Bartender, Ice, Maccy, Stats, Raycast, Alfred, and Fantastical use popovers or menu-sized surfaces, but still expose invocation, focused choice, commitment, and confirmation.
5. **First success is locally evidenced, not inferred.** The fifth state in each directory is decoded from the same motion asset as the preceding states; no journey is built by animating gallery stills.
6. **Non-completion is the common recoverable failure boundary.** Many polished first-run recordings do not stage an error dialog. The records therefore distinguish a truthful observed non-success state—stopping with incomplete input or before confirmation—from an invented system failure, then trace recovery through the retained confirmation and result states.
7. **Static equivalents matter.** Five ordered PNGs per record make the same transition inspectable when video playback or motion is unsuitable, while product-level Reduce Motion behavior remains explicitly unknown unless shown.

## Disagreements and product-specific choices

- **Keyboard-first versus direct manipulation:** launchers, editors, terminals, and task apps reward typing and shortcuts; creative tools and file managers rely more heavily on pointer selection, drag, canvas manipulation, and spatial preview.
- **Implicit versus explicit commit:** window managers and live previews respond continuously, while database connections, Git actions, exports, transfers, and network rules use a visible confirmation boundary.
- **Single-surface versus multi-pane:** menu-bar utilities minimize persistent structure; developer, knowledge, database, and operations apps keep sidebars, tables, editors, and inspectors concurrently visible.
- **Onboarding depth:** template-based products such as Scrivener and Pixelmator Pro expose setup choices before the workspace. Other products enter a ready surface immediately and establish first success through a command, search, connection, or content edit.
- **Failure presentation:** some products provide explicit validation or connection feedback; others in the retained recordings only reveal failure as the absence of the intended result. The per-product records preserve that distinction rather than normalizing every app to an invented error alert.
- **Motion purpose:** some recordings use native interface transitions, some use cursor-led tutorials, and some use edited product demonstrations. The timing and interruption limits are stated locally instead of assigning a universal duration.

## Applicability boundaries

- These references document the exact retained macOS/product recordings, not every current version, account state, feature flag, permission prompt, or localization.
- An ordered frame proves visible state and motion continuity; it does not prove VoiceOver naming, keyboard focus order, Reduce Motion behavior, or contrast compliance. Those remain in each record’s accessibility unknowns.
- A tutorial or edited demonstration can prove the shown product states but not interactions outside its retained interval. No record claims unseen undo, error, or reversal behavior.
- Remote services, repositories, databases, mailboxes, and collaboration spaces require their own prerequisites. The reference isolates the client interaction pattern from any claim that a particular external account or server remains available.
- The existing gallery remains useful for broad visual comparison. The per-example references are the offline, traceable source for motion and journey claims.

## All 50 product references

| # | Product | Category | Structured record |
|---:|---|---|---|
| 01 | [Bartender](references/01-bartender/README.md) | Menu-bar utility | [`reference.json`](references/01-bartender/reference.json) |
| 02 | [Ice](references/02-ice/README.md) | Menu-bar utility | [`reference.json`](references/02-ice/reference.json) |
| 03 | [Maccy](references/03-maccy/README.md) | Menu-bar utility | [`reference.json`](references/03-maccy/reference.json) |
| 04 | [Stats](references/04-stats/README.md) | Menu-bar utility | [`reference.json`](references/04-stats/reference.json) |
| 05 | [Rectangle](references/05-rectangle/README.md) | Window management | [`reference.json`](references/05-rectangle/reference.json) |
| 06 | [Raycast](references/06-raycast/README.md) | Launcher and productivity | [`reference.json`](references/06-raycast/reference.json) |
| 07 | [Alfred](references/07-alfred/README.md) | Launcher and productivity | [`reference.json`](references/07-alfred/reference.json) |
| 08 | [Keyboard Maestro](references/08-keyboard-maestro/README.md) | Automation and productivity | [`reference.json`](references/08-keyboard-maestro/reference.json) |
| 09 | [Pages](references/09-pages/README.md) | Document app | [`reference.json`](references/09-pages/reference.json) |
| 10 | [Ulysses](references/10-ulysses/README.md) | Document app | [`reference.json`](references/10-ulysses/reference.json) |
| 11 | [Bear](references/11-bear/README.md) | Document app | [`reference.json`](references/11-bear/reference.json) |
| 12 | [iA Writer](references/12-ia-writer/README.md) | Document app | [`reference.json`](references/12-ia-writer/reference.json) |
| 13 | [Craft](references/13-craft/README.md) | Document app | [`reference.json`](references/13-craft/reference.json) |
| 14 | [DEVONthink](references/14-devonthink/README.md) | Document and knowledge management | [`reference.json`](references/14-devonthink/reference.json) |
| 15 | [Scrivener](references/15-scrivener/README.md) | Document app | [`reference.json`](references/15-scrivener/reference.json) |
| 16 | [Obsidian](references/16-obsidian/README.md) | Document and knowledge management | [`reference.json`](references/16-obsidian/reference.json) |
| 17 | [Xcode](references/17-xcode/README.md) | Developer tool | [`reference.json`](references/17-xcode/reference.json) |
| 18 | [Nova](references/18-nova/README.md) | Developer tool | [`reference.json`](references/18-nova/reference.json) |
| 19 | [BBEdit](references/19-bbedit/README.md) | Developer tool | [`reference.json`](references/19-bbedit/reference.json) |
| 20 | [Tower](references/20-tower/README.md) | Developer tool | [`reference.json`](references/20-tower/reference.json) |
| 21 | [Fork](references/21-fork/README.md) | Developer tool | [`reference.json`](references/21-fork/reference.json) |
| 22 | [Kaleidoscope](references/22-kaleidoscope/README.md) | Developer tool | [`reference.json`](references/22-kaleidoscope/reference.json) |
| 23 | [Dash](references/23-dash/README.md) | Developer tool | [`reference.json`](references/23-dash/reference.json) |
| 24 | [Proxyman](references/24-proxyman/README.md) | Developer tool | [`reference.json`](references/24-proxyman/reference.json) |
| 25 | [Zed](references/25-zed/README.md) | Developer tool | [`reference.json`](references/25-zed/reference.json) |
| 26 | [Sketch](references/26-sketch/README.md) | Creative software | [`reference.json`](references/26-sketch/reference.json) |
| 27 | [Pixelmator Pro](references/27-pixelmator-pro/README.md) | Creative software | [`reference.json`](references/27-pixelmator-pro/reference.json) |
| 28 | [Affinity Designer](references/28-affinity-designer/README.md) | Creative software | [`reference.json`](references/28-affinity-designer/reference.json) |
| 29 | [Final Cut Pro](references/29-final-cut-pro/README.md) | Creative software | [`reference.json`](references/29-final-cut-pro/reference.json) |
| 30 | [Logic Pro](references/30-logic-pro/README.md) | Creative software | [`reference.json`](references/30-logic-pro/reference.json) |
| 31 | [ScreenFlow](references/31-screenflow/README.md) | Creative software | [`reference.json`](references/31-screenflow/reference.json) |
| 32 | [Acorn](references/32-acorn/README.md) | Creative software | [`reference.json`](references/32-acorn/reference.json) |
| 33 | [Mimestream](references/33-mimestream/README.md) | Communication | [`reference.json`](references/33-mimestream/reference.json) |
| 34 | [Spark](references/34-spark/README.md) | Communication | [`reference.json`](references/34-spark/reference.json) |
| 35 | [Slack](references/35-slack/README.md) | Communication | [`reference.json`](references/35-slack/reference.json) |
| 36 | [Telegram for macOS](references/36-telegram-for-macos/README.md) | Communication | [`reference.json`](references/36-telegram-for-macos/reference.json) |
| 37 | [TablePlus](references/37-tableplus/README.md) | Database tool | [`reference.json`](references/37-tableplus/reference.json) |
| 38 | [Postico 2](references/38-postico-2/README.md) | Database tool | [`reference.json`](references/38-postico-2/reference.json) |
| 39 | [Sequel Ace](references/39-sequel-ace/README.md) | Database tool | [`reference.json`](references/39-sequel-ace/reference.json) |
| 40 | [SQLPro Studio](references/40-sqlpro-studio/README.md) | Database tool | [`reference.json`](references/40-sqlpro-studio/reference.json) |
| 41 | [Base](references/41-base/README.md) | Database tool | [`reference.json`](references/41-base/reference.json) |
| 42 | [Transmit](references/42-transmit/README.md) | Operations | [`reference.json`](references/42-transmit/reference.json) |
| 43 | [ForkLift](references/43-forklift/README.md) | Operations | [`reference.json`](references/43-forklift/reference.json) |
| 44 | [Cyberduck](references/44-cyberduck/README.md) | Operations | [`reference.json`](references/44-cyberduck/reference.json) |
| 45 | [Little Snitch](references/45-little-snitch/README.md) | Operations | [`reference.json`](references/45-little-snitch/reference.json) |
| 46 | [iTerm2](references/46-iterm2/README.md) | Operations | [`reference.json`](references/46-iterm2/reference.json) |
| 47 | [Things](references/47-things/README.md) | Productivity | [`reference.json`](references/47-things/reference.json) |
| 48 | [OmniFocus](references/48-omnifocus/README.md) | Productivity | [`reference.json`](references/48-omnifocus/reference.json) |
| 49 | [Fantastical](references/49-fantastical/README.md) | Productivity | [`reference.json`](references/49-fantastical/reference.json) |
| 50 | [MindNode](references/50-mindnode/README.md) | Productivity | [`reference.json`](references/50-mindnode/reference.json) |

## Proxyman integration

The Proxyman row integrates the existing [`proxyman-reference`](proxyman-reference/README.md) rather than copying its 26-screen static set. [`references/24-proxyman`](references/24-proxyman/README.md) adds the missing authentic motion, five-state journey, interaction map, and hashed frame evidence while retaining the deeper screen anatomy in its original directory.
