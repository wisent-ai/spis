# Full reference: onboarding and authentication

This synthesis is derived from all 50 evidence-complete per-product records in [`references.json`](references.json). It does not substitute for them. Each record retains authentic local motion, six directly extracted states, an eight-part interaction map, a six-state first-success journey, failure and recovery routes, accessibility observations and unknowns, and media provenance with SHA-256.

## Evidence field

| Group | Records | Shared first-success boundary |
|---|---|---|
| Workspace and collaboration setup | [01](references/01-slack-workspace-creation/README.md), [02](references/02-notion-create-and-switch-workspaces/README.md), [03](references/03-asana-account-setup/README.md), [04](references/04-atlassian-invite-users-to-a-site/README.md), [05](references/05-miro-getting-started/README.md), [06](references/06-figma-create-a-team/README.md), [07](references/07-airtable-onboarding/README.md), [08](references/08-clickup-onboarding/README.md), [09](references/09-monday-com-getting-started/README.md), [10](references/10-trello-create-a-workspace/README.md) | A persisted workspace, board, page, project, or task—not merely account creation. |
| Communication, storage, and productivity entry | [11](references/11-zoom-sign-up-and-activate-an-account/README.md), [12](references/12-cisco-webex-get-started/README.md), [13](references/13-discord-getting-started/README.md), [14](references/14-dropbox-import-files-and-folders/README.md), [15](references/15-google-workspace-setup-guide/README.md), [16](references/16-microsoft-365-setup-wizard/README.md), [17](references/17-canva-sign-up-or-log-in/README.md), [18](references/18-postman-create-and-manage-workspaces/README.md) | A sent message, uploaded file, provisioned user, saved design, or usable API request. |
| Developer platform first success | [19](references/19-github-creating-an-account/README.md), [20](references/20-gitlab-user-account/README.md), [21](references/21-vercel-import-a-git-repository/README.md), [22](references/22-netlify-import-an-existing-project/README.md), [23](references/23-render-deploy-from-a-git-repository/README.md), [24](references/24-heroku-deploying-with-git/README.md), [25](references/25-docker-desktop-sign-in/README.md), [26](references/26-sentry-guided-onboarding/README.md), [27](references/27-datadog-getting-started/README.md), [28](references/28-twilio-account-setup/README.md) | A repository, deployed URL, running container, captured event, telemetry source, or approved verification. |
| Hosted identity, device authorization, and network trust | [29](references/29-auth0-universal-login/README.md), [30](references/30-okta-device-authorization-flow/README.md), [31](references/31-microsoft-identity-platform-device-authorization-grant/README.md), [32](references/32-aws-iam-identity-center-device-authorization/README.md), [33](references/33-cloudflare-zero-trust-setup/README.md), [34](references/34-tailscale-quickstart/README.md) | An authenticated callback, token-bearing device, enrolled endpoint, or allowed connection. |
| Secrets, authenticators, and privacy accounts | [35](references/35-1password-get-started/README.md), [36](references/36-bitwarden-get-started/README.md), [37](references/37-dashlane-get-started/README.md), [38](references/38-yubico-set-up-a-security-key/README.md), [39](references/39-duo-security-device-enrollment/README.md), [40](references/40-proton-create-an-account/README.md), [41](references/41-signal-register-a-phone-number/README.md) | A saved vault item, enrolled factor, activated device, or first private communication. |
| Consumer device and media onboarding | [42](references/42-apple-set-up-an-iphone-or-ipad/README.md), [43](references/43-android-set-up-a-new-device/README.md), [44](references/44-spotify-sign-up/README.md), [45](references/45-netflix-sign-up/README.md) | A usable launcher/Home Screen or actual playback. |
| Financial connection and commerce activation | [46](references/46-stripe-connect-embedded-onboarding/README.md), [47](references/47-plaid-link-overview/README.md), [48](references/48-paypal-create-an-account/README.md), [49](references/49-shopify-initial-setup/README.md), [50](references/50-square-account-setup-guide/README.md) | A submitted requirement set, linked account, verified funding setup, test order, or first payment. |

## Recurring patterns

1. **Identity is a checkpoint, not the outcome.** The collaboration records [01](references/01-slack-workspace-creation/reference.json)–[10](references/10-trello-create-a-workspace/reference.json) continue from authentication into a persisted workspace artifact. Developer records [19](references/19-github-creating-an-account/reference.json)–[28](references/28-twilio-account-setup/reference.json) continue into a repository, deployment, container, event, metric, or verified API result.
2. **Verification must be resumable.** Email, phone, domain, provider, hardware, and device-code checks recur across [11 Zoom](references/11-zoom-sign-up-and-activate-an-account/reference.json), [15 Google Workspace](references/15-google-workspace-setup-guide/reference.json), [30 Okta](references/30-okta-device-authorization-flow/reference.json), [31 Microsoft](references/31-microsoft-identity-platform-device-authorization-grant/reference.json), [32 AWS](references/32-aws-iam-identity-center-device-authorization/reference.json), [38 Yubico](references/38-yubico-set-up-a-security-key/reference.json), and [39 Duo](references/39-duo-security-device-enrollment/reference.json). The durable state is the pending requirement plus expiry/resend/retry—not a spinner.
3. **Optional work must have a safe exit.** Invitations, personalization, imports, recovery contacts, data transfer, and profile work appear before value in many products, but the strong patterns expose Back, Skip, or “later” without fabricating completion. The interaction maps in every record separate cancellation from failure and recovery.
4. **First success is product-specific and observable.** Messages, pages, boards, tasks, files, repositories, deploys, telemetry, tokens, vault items, authenticators, playback, linked accounts, test orders, and payments are not interchangeable. Each completion claim names one retained state and one motion timestamp.
5. **High-trust flows explain changing requirements.** [46 Stripe Connect](references/46-stripe-connect-embedded-onboarding/reference.json), [47 Plaid Link](references/47-plaid-link-overview/reference.json), [48 PayPal](references/48-paypal-create-an-account/reference.json), [49 Shopify](references/49-shopify-initial-setup/reference.json), and [50 Square](references/50-square-account-setup-guide/reference.json) progressively reveal identity, financial, and operational requirements and require a durable reviewed or transaction state.

## Disagreements and applicability boundaries

- **Linear wizard vs. checklist:** device setup and device-code grants are naturally ordered; admin and merchant setup remains requirement-driven and resumable. Do not force Stripe, Shopify, Square, Google Workspace, or Microsoft 365 into a false percentage.
- **Embedded vs. redirected identity:** Auth0 and Stripe retain application context in hosted or embedded components; Docker Desktop, Tailscale, Git-provider deployment tools, and device-code clients intentionally hand control to a browser. The callback must restore the originating context.
- **Password vs. passwordless/hardware:** ordinary account flows coexist with provider sign-in, Signal phone verification, YubiKey WebAuthn, Duo enrollment, and device authorization. Recovery must match the chosen factor rather than default to password reset.
- **Immediate vs. asynchronous success:** a saved task or message is immediate; uploads, builds, telemetry arrival, financial verification, and domain activation have durable pending states. Loading is not success.
- **Consumer vs. administrator actor:** Spotify, Netflix, Signal, Proton, Apple, and Android optimize a single-person entry; Atlassian, Google Workspace, Microsoft 365, Cloudflare, Stripe, Shopify, and Square expose organization scope, delegated access, and compliance consequences.

## Motion findings

The retained media are owner-published real-product recordings (or, for Signal, a direct official Signal Blog MP4). Local copies are time-bounded visual transcodes rather than animations synthesized from stills. Editorial cuts establish ordered states but do not establish unedited transition timing. Every record therefore pairs motion with six exact extracted frames and treats interruption, cancellation, failure, and recovery as product-state transitions rather than decorative easing.

Reduced-motion behavior was not exposed in the owner recordings. The per-record six-frame sequences are the nonanimated inspection equivalent; product implementations should also preserve focus, labels, current requirement, and completion feedback when motion is disabled.

## Accessibility findings

- Persistent on-screen task labels and adjacent primary actions recur across the field and are inspectable in local states.
- Recordings do not prove screen-reader names, focus order, live-region behavior, one-time-code autofill, password-manager integration, text scaling, or reduced-motion preferences; every record names those unknowns instead of converting them to assumptions.
- Verification and asynchronous progress require text status, expiry/retry context, and focus-managed state changes; color or motion alone is insufficient.
- Cross-device flows need a readable code, copy support where appropriate, clear destination, expiry, and a return state on the originating device.

## All 50 records

1. [Slack — workspace creation](references/01-slack-workspace-creation/README.md) — [structured record](references/01-slack-workspace-creation/reference.json)
2. [Notion — create and switch workspaces](references/02-notion-create-and-switch-workspaces/README.md) — [structured record](references/02-notion-create-and-switch-workspaces/reference.json)
3. [Asana — account setup](references/03-asana-account-setup/README.md) — [structured record](references/03-asana-account-setup/reference.json)
4. [Atlassian — invite users to a site](references/04-atlassian-invite-users-to-a-site/README.md) — [structured record](references/04-atlassian-invite-users-to-a-site/reference.json)
5. [Miro — getting started](references/05-miro-getting-started/README.md) — [structured record](references/05-miro-getting-started/reference.json)
6. [Figma — create a team](references/06-figma-create-a-team/README.md) — [structured record](references/06-figma-create-a-team/reference.json)
7. [Airtable — onboarding](references/07-airtable-onboarding/README.md) — [structured record](references/07-airtable-onboarding/reference.json)
8. [ClickUp — onboarding](references/08-clickup-onboarding/README.md) — [structured record](references/08-clickup-onboarding/reference.json)
9. [monday.com — getting started](references/09-monday-com-getting-started/README.md) — [structured record](references/09-monday-com-getting-started/reference.json)
10. [Trello — create a Workspace](references/10-trello-create-a-workspace/README.md) — [structured record](references/10-trello-create-a-workspace/reference.json)
11. [Zoom — sign up and activate an account](references/11-zoom-sign-up-and-activate-an-account/README.md) — [structured record](references/11-zoom-sign-up-and-activate-an-account/reference.json)
12. [Cisco Webex — get started](references/12-cisco-webex-get-started/README.md) — [structured record](references/12-cisco-webex-get-started/reference.json)
13. [Discord — getting started](references/13-discord-getting-started/README.md) — [structured record](references/13-discord-getting-started/reference.json)
14. [Dropbox — import files and folders](references/14-dropbox-import-files-and-folders/README.md) — [structured record](references/14-dropbox-import-files-and-folders/reference.json)
15. [Google Workspace — setup guide](references/15-google-workspace-setup-guide/README.md) — [structured record](references/15-google-workspace-setup-guide/reference.json)
16. [Microsoft 365 — setup wizard](references/16-microsoft-365-setup-wizard/README.md) — [structured record](references/16-microsoft-365-setup-wizard/reference.json)
17. [Canva — sign up or log in](references/17-canva-sign-up-or-log-in/README.md) — [structured record](references/17-canva-sign-up-or-log-in/reference.json)
18. [Postman — create and manage workspaces](references/18-postman-create-and-manage-workspaces/README.md) — [structured record](references/18-postman-create-and-manage-workspaces/reference.json)
19. [GitHub — creating an account](references/19-github-creating-an-account/README.md) — [structured record](references/19-github-creating-an-account/reference.json)
20. [GitLab — user account](references/20-gitlab-user-account/README.md) — [structured record](references/20-gitlab-user-account/reference.json)
21. [Vercel — import a Git repository](references/21-vercel-import-a-git-repository/README.md) — [structured record](references/21-vercel-import-a-git-repository/reference.json)
22. [Netlify — import an existing project](references/22-netlify-import-an-existing-project/README.md) — [structured record](references/22-netlify-import-an-existing-project/reference.json)
23. [Render — deploy from a Git repository](references/23-render-deploy-from-a-git-repository/README.md) — [structured record](references/23-render-deploy-from-a-git-repository/reference.json)
24. [Heroku — deploying with Git](references/24-heroku-deploying-with-git/README.md) — [structured record](references/24-heroku-deploying-with-git/reference.json)
25. [Docker Desktop — sign in](references/25-docker-desktop-sign-in/README.md) — [structured record](references/25-docker-desktop-sign-in/reference.json)
26. [Sentry — guided onboarding](references/26-sentry-guided-onboarding/README.md) — [structured record](references/26-sentry-guided-onboarding/reference.json)
27. [Datadog — getting started](references/27-datadog-getting-started/README.md) — [structured record](references/27-datadog-getting-started/reference.json)
28. [Twilio — account setup](references/28-twilio-account-setup/README.md) — [structured record](references/28-twilio-account-setup/reference.json)
29. [Auth0 — Universal Login](references/29-auth0-universal-login/README.md) — [structured record](references/29-auth0-universal-login/reference.json)
30. [Okta — device authorization flow](references/30-okta-device-authorization-flow/README.md) — [structured record](references/30-okta-device-authorization-flow/reference.json)
31. [Microsoft identity platform — device authorization grant](references/31-microsoft-identity-platform-device-authorization-grant/README.md) — [structured record](references/31-microsoft-identity-platform-device-authorization-grant/reference.json)
32. [AWS IAM Identity Center — device authorization](references/32-aws-iam-identity-center-device-authorization/README.md) — [structured record](references/32-aws-iam-identity-center-device-authorization/reference.json)
33. [Cloudflare Zero Trust — setup](references/33-cloudflare-zero-trust-setup/README.md) — [structured record](references/33-cloudflare-zero-trust-setup/reference.json)
34. [Tailscale — quickstart](references/34-tailscale-quickstart/README.md) — [structured record](references/34-tailscale-quickstart/reference.json)
35. [1Password — get started](references/35-1password-get-started/README.md) — [structured record](references/35-1password-get-started/reference.json)
36. [Bitwarden — get started](references/36-bitwarden-get-started/README.md) — [structured record](references/36-bitwarden-get-started/reference.json)
37. [Dashlane — get started](references/37-dashlane-get-started/README.md) — [structured record](references/37-dashlane-get-started/reference.json)
38. [Yubico — set up a security key](references/38-yubico-set-up-a-security-key/README.md) — [structured record](references/38-yubico-set-up-a-security-key/reference.json)
39. [Duo Security — device enrollment](references/39-duo-security-device-enrollment/README.md) — [structured record](references/39-duo-security-device-enrollment/reference.json)
40. [Proton — create an account](references/40-proton-create-an-account/README.md) — [structured record](references/40-proton-create-an-account/reference.json)
41. [Signal — register a phone number](references/41-signal-register-a-phone-number/README.md) — [structured record](references/41-signal-register-a-phone-number/reference.json)
42. [Apple — set up an iPhone or iPad](references/42-apple-set-up-an-iphone-or-ipad/README.md) — [structured record](references/42-apple-set-up-an-iphone-or-ipad/reference.json)
43. [Android — set up a new device](references/43-android-set-up-a-new-device/README.md) — [structured record](references/43-android-set-up-a-new-device/reference.json)
44. [Spotify — sign up](references/44-spotify-sign-up/README.md) — [structured record](references/44-spotify-sign-up/reference.json)
45. [Netflix — sign up](references/45-netflix-sign-up/README.md) — [structured record](references/45-netflix-sign-up/reference.json)
46. [Stripe Connect — embedded onboarding](references/46-stripe-connect-embedded-onboarding/README.md) — [structured record](references/46-stripe-connect-embedded-onboarding/reference.json)
47. [Plaid Link — overview](references/47-plaid-link-overview/README.md) — [structured record](references/47-plaid-link-overview/reference.json)
48. [PayPal — create an account](references/48-paypal-create-an-account/README.md) — [structured record](references/48-paypal-create-an-account/reference.json)
49. [Shopify — initial setup](references/49-shopify-initial-setup/README.md) — [structured record](references/49-shopify-initial-setup/reference.json)
50. [Square — account setup guide](references/50-square-account-setup-guide/README.md) — [structured record](references/50-square-account-setup-guide/reference.json)

## Provenance boundary

Product and recording ownership remains with the upstream owners named in each record. `captured_at`, source URL, original owner, local dimensions, duration, frame count, byte size, and SHA-256 are recorded per asset. An upstream tutorial cut is evidence of the shown states only; it is not evidence for an unshown animation, timing, keyboard behavior, assistive-technology behavior, or recovery outcome.
