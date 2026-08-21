# Full interaction reference — design system examples

This synthesis is derived from all 50 complete per-example records in [`references.json`](references.json). It does not replace them: each linked directory retains authentic local motion, three source frames, an eight-part interaction map, and a five-state first-success journey.

## Coverage

- Records: **50 / 50 complete**
- Authentic local motion formats: **gif 11**, **mp4 34**, **webm 5**
- Timing classes: **micro (<3s): 9**, **brief (3–15s): 13**, **extended (>15s): 28**
- State frames: **150**, all decoded from their associated local motion assets.
- Every reference includes primary input, focus/selection, navigation, confirmation, cancellation/backtracking, feedback, failure, recovery, accessibility observations and explicit unknowns.

## Recurring patterns

1. **Entry → active → settled is the common minimum.** All 50 records preserve those three key states, while their five-step journeys add orientation and intermediate feedback. This makes transition continuity inspectable offline rather than inferred from prose.
2. **Implementation guidance is coupled to visible feedback.** Component-oriented systems frequently use short demonstrations for hover, focus, loading, disclosure, navigation, and validation. Broader platform and government systems more often use extended guided walkthroughs to establish context before the result.
3. **Failure is usually absence of the expected state.** Across the records, interruption or omission of the primary action leaves the surface before first success. The dependable recovery pattern is to return to the retained entry state and repeat the evidenced path.
4. **Motion and static inspection are complementary.** Motion proves timing and continuity; the three hashed key states support comparison, review, and a nonanimated inspection path. The state frames are not substitutes for motion evidence.
5. **Accessibility evidence has a hard boundary.** Visual recordings support observations about distinguishable states and pause/replay, but do not prove screen-reader announcements, keyboard-only reachability, or reduced-motion behavior unless those are explicitly visible. Those facts remain unknown rather than inferred.

## Disagreements across systems

- **Timing:** micro-feedback assets finish in under three seconds, while guided recordings spend tens of seconds establishing context. Neither timing class should be copied without matching task complexity.
- **Styling versus semantics:** branded systems emphasize visual continuity and tone; headless and accessibility-first systems emphasize state contracts and focus behavior. Implementation should preserve both where the target product needs them.
- **Reversal:** some component transitions visibly reverse, but many guided recordings only show the forward success path. A reversible player is not evidence that the product interaction itself reverses.
- **Reduced motion:** retained key states provide an inspection equivalent, not proof of an upstream `prefers-reduced-motion` implementation. Each record separates the observed behavior from this unknown.
- **Scope of “first success”:** for a component asset it is the first stable component result; for a platform or service-system walkthrough it is the first implementable pattern the viewer can identify. The per-record actor, goal, and prerequisites define that boundary.

## Applicability boundaries

Use these references to compare state models, feedback, timing class, and find-to-implement flow. Do not treat a design-system recording as proof of production performance, assistive-technology output, browser compatibility, localization behavior, or reduced-motion support beyond what the recording exposes. Do not lift government or platform conventions into a different jurisdiction or operating system without checking local requirements. For implementation decisions, follow the product URL and upstream owner recorded beside the authentic media.

## All 50 records

| # | Reference | Local motion | Duration / frames | Status |
|---:|---|---|---:|---|
| 1 | [Material Design 3](references/01-material-design-3/README.md) | [gif](references/01-material-design-3/media/motion.gif) | 4.43s / 100 frames | complete |
| 2 | [Apple Human Interface Guidelines](references/02-apple-human-interface-guidelines/README.md) | [mp4](references/02-apple-human-interface-guidelines/media/motion.mp4) | 30.00s / 899 frames | complete |
| 3 | [Fluent 2 Design System](references/03-fluent-2-design-system/README.md) | [webm](references/03-fluent-2-design-system/media/motion.webm) | 2.45s / 147 frames | complete |
| 4 | [Carbon Design System](references/04-carbon-design-system/README.md) | [gif](references/04-carbon-design-system/media/motion.gif) | 9.52s / 136 frames | complete |
| 5 | [Salesforce Lightning Design System](references/05-salesforce-lightning-design-system/README.md) | [gif](references/05-salesforce-lightning-design-system/media/motion.gif) | 0.98s / 49 frames | complete |
| 6 | [Adobe Spectrum](references/06-adobe-spectrum/README.md) | [mp4](references/06-adobe-spectrum/media/motion.mp4) | 6.57s / 197 frames | complete |
| 7 | [Atlassian Design System](references/07-atlassian-design-system/README.md) | [mp4](references/07-atlassian-design-system/media/motion.mp4) | 30.00s / 899 frames | complete |
| 8 | [Shopify Polaris](references/08-shopify-polaris/README.md) | [mp4](references/08-shopify-polaris/media/motion.mp4) | 2.54s / 76 frames | complete |
| 9 | [GitHub Primer](references/09-github-primer/README.md) | [mp4](references/09-github-primer/media/motion.mp4) | 30.00s / 899 frames | complete |
| 10 | [GitLab Pajamas](references/10-gitlab-pajamas/README.md) | [mp4](references/10-gitlab-pajamas/media/motion.mp4) | 2.42s / 121 frames | complete |
| 11 | [SAP Fiori Design System](references/11-sap-fiori-design-system/README.md) | [gif](references/11-sap-fiori-design-system/media/motion.gif) | 6.45s / 63 frames | complete |
| 12 | [VMware Clarity Design System](references/12-vmware-clarity-design-system/README.md) | [mp4](references/12-vmware-clarity-design-system/media/motion.mp4) | 30.00s / 900 frames | complete |
| 13 | [AWS Cloudscape Design System](references/13-aws-cloudscape-design-system/README.md) | [mp4](references/13-aws-cloudscape-design-system/media/motion.mp4) | 128.00s / 3840 frames | complete |
| 14 | [PatternFly](references/14-patternfly/README.md) | [mp4](references/14-patternfly/media/motion.mp4) | 30.00s / 900 frames | complete |
| 15 | [Zendesk Garden](references/15-zendesk-garden/README.md) | [mp4](references/15-zendesk-garden/media/motion.mp4) | 30.00s / 720 frames | complete |
| 16 | [Twilio Paste](references/16-twilio-paste/README.md) | [gif](references/16-twilio-paste/media/motion.gif) | 8.00s / 80 frames | complete |
| 17 | [Base Web](references/17-base-web/README.md) | [gif](references/17-base-web/media/motion.gif) | 4.16s / 52 frames | complete |
| 18 | [Gestalt](references/18-gestalt/README.md) | [mp4](references/18-gestalt/media/motion.mp4) | 29.99s / 719 frames | complete |
| 19 | [Nord Design System](references/19-nord-design-system/README.md) | [mp4](references/19-nord-design-system/media/motion.mp4) | 20.73s / 622 frames | complete |
| 20 | [Elastic UI](references/20-elastic-ui/README.md) | [gif](references/20-elastic-ui/media/motion.gif) | 1.72s / 34 frames | complete |
| 21 | [MongoDB LeafyGreen](references/21-mongodb-leafygreen/README.md) | [gif](references/21-mongodb-leafygreen/media/motion.gif) | 8.80s / 88 frames | complete |
| 22 | [HashiCorp Helios Design System](references/22-hashicorp-helios-design-system/README.md) | [mp4](references/22-hashicorp-helios-design-system/media/motion.mp4) | 12.88s / 772 frames | complete |
| 23 | [Blueprint](references/23-blueprint/README.md) | [mp4](references/23-blueprint/media/motion.mp4) | 30.00s / 899 frames | complete |
| 24 | [JetBrains Ring UI](references/24-jetbrains-ring-ui/README.md) | [gif](references/24-jetbrains-ring-ui/media/motion.gif) | 0.24s / 4 frames | complete |
| 25 | [Mozilla Protocol](references/25-mozilla-protocol/README.md) | [webm](references/25-mozilla-protocol/media/motion.webm) | 9.00s / 240 frames | complete |
| 26 | [BBC Global Experience Language](references/26-bbc-global-experience-language/README.md) | [mp4](references/26-bbc-global-experience-language/media/motion.mp4) | 30.00s / 480 frames | complete |
| 27 | [Skyscanner Backpack](references/27-skyscanner-backpack/README.md) | [mp4](references/27-skyscanner-backpack/media/motion.mp4) | 2.04s / 61 frames | complete |
| 28 | [Vanilla Framework](references/28-vanilla-framework/README.md) | [mp4](references/28-vanilla-framework/media/motion.mp4) | 30.00s / 750 frames | complete |
| 29 | [Ant Design](references/29-ant-design/README.md) | [webm](references/29-ant-design/media/motion.webm) | 21.00s / 525 frames | complete |
| 30 | [Material UI](references/30-material-ui/README.md) | [webm](references/30-material-ui/media/motion.webm) | 6.12s / 367 frames | complete |
| 31 | [Chakra UI](references/31-chakra-ui/README.md) | [mp4](references/31-chakra-ui/media/motion.mp4) | 13.32s / 799 frames | complete |
| 32 | [Radix Primitives](references/32-radix-primitives/README.md) | [mp4](references/32-radix-primitives/media/motion.mp4) | 30.00s / 899 frames | complete |
| 33 | [React Aria](references/33-react-aria/README.md) | [mp4](references/33-react-aria/media/motion.mp4) | 6.38s / 175 frames | complete |
| 34 | [Headless UI](references/34-headless-ui/README.md) | [mp4](references/34-headless-ui/media/motion.mp4) | 30.00s / 900 frames | complete |
| 35 | [WAI-ARIA Authoring Practices Guide](references/35-wai-aria-authoring-practices-guide/README.md) | [mp4](references/35-wai-aria-authoring-practices-guide/media/motion.mp4) | 9.92s / 248 frames | complete |
| 36 | [U.S. Web Design System](references/36-u-s-web-design-system/README.md) | [gif](references/36-u-s-web-design-system/media/motion.gif) | 0.72s / 8 frames | complete |
| 37 | [GOV.UK Design System](references/37-gov-uk-design-system/README.md) | [mp4](references/37-gov-uk-design-system/media/motion.mp4) | 30.00s / 750 frames | complete |
| 38 | [NHS Digital Service Manual](references/38-nhs-digital-service-manual/README.md) | [mp4](references/38-nhs-digital-service-manual/media/motion.mp4) | 30.00s / 900 frames | complete |
| 39 | [Government of Canada Design System](references/39-government-of-canada-design-system/README.md) | [mp4](references/39-government-of-canada-design-system/media/motion.mp4) | 17.95s / 538 frames | complete |
| 40 | [Singapore Government Design System](references/40-singapore-government-design-system/README.md) | [mp4](references/40-singapore-government-design-system/media/motion.mp4) | 21.50s / 516 frames | complete |
| 41 | [French State Design System](references/41-french-state-design-system/README.md) | [mp4](references/41-french-state-design-system/media/motion.mp4) | 30.00s / 750 frames | complete |
| 42 | [Design System of the Country of Italy](references/42-design-system-of-the-country-of-italy/README.md) | [gif](references/42-design-system-of-the-country-of-italy/media/motion.gif) | 2.00s / 48 frames | complete |
| 43 | [European Commission Europa Component Library](references/43-european-commission-europa-component-library/README.md) | [mp4](references/43-european-commission-europa-component-library/media/motion.mp4) | 30.00s / 750 frames | complete |
| 44 | [Suomi.fi Design System](references/44-suomi-fi-design-system/README.md) | [webm](references/44-suomi-fi-design-system/media/motion.webm) | 18.88s / 472 frames | complete |
| 45 | [Scottish Government Design System](references/45-scottish-government-design-system/README.md) | [mp4](references/45-scottish-government-design-system/media/motion.mp4) | 24.33s / 584 frames | complete |
| 46 | [CMS Design System](references/46-cms-design-system/README.md) | [mp4](references/46-cms-design-system/media/motion.mp4) | 146.45s / 4389 frames | complete |
| 47 | [VA.gov Design System](references/47-va-gov-design-system/README.md) | [mp4](references/47-va-gov-design-system/media/motion.mp4) | 30.00s / 750 frames | complete |
| 48 | [California Design System](references/48-california-design-system/README.md) | [mp4](references/48-california-design-system/media/motion.mp4) | 30.00s / 900 frames | complete |
| 49 | [GNOME Human Interface Guidelines](references/49-gnome-human-interface-guidelines/README.md) | [mp4](references/49-gnome-human-interface-guidelines/media/motion.mp4) | 30.00s / 750 frames | complete |
| 50 | [KDE Human Interface Guidelines](references/50-kde-human-interface-guidelines/README.md) | [mp4](references/50-kde-human-interface-guidelines/media/motion.mp4) | 30.00s / 899 frames | complete |

## Provenance and verification model

Every structured record uses `wisent.full-product-reference.v1` and records product URL, upstream owner, capture date, source URL, capture method, dimensions, duration, frame count, byte size, and SHA-256. `references.json` is the exact 50-record catalog index. Local paths are relative to each example directory so a numbered example can be inspected and played without network access.
