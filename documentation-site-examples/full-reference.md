# Full reference: documentation sites

This synthesis is derived from all **50 complete per-product records** in [`references.json`](references.json). Each row below points to authentic local WebM motion recorded from the live official product in Weles on a Stado-selected dedicated host, five retained states, an observed failed route and recovery, and a structured interaction/journey record. It supplements rather than replaces the original [50-image gallery](README.md).

## Evidence set

- 50/50 products have a local playable WebM, SHA-256, dimensions, decoded frame count, duration, byte size, source URL, upstream owner, capture method, and capture time.
- 50/50 products retain landing, search/learn-open, failed-search, recovered-results, and first-answer frames from the same browser context as the motion.
- 34/50 exposed a usable inline/modal search interaction to the generic Weles observer; 16/50 required the product's official query-bearing search or learn route because no visible inline query field was available.
- Median recording duration: 9.820 seconds; range: 8.000–15.080 seconds.
- Every journey has six ordered states and every interaction map has eight observed or explicitly bounded interaction entries.

## Recurring patterns derived from the 50 records

1. **Stable landing context precedes lookup.** Product identity and the current documentation scope remain visible before search or concept navigation. This makes a failed lookup recoverable without losing the reader's origin.
2. **Search commonly opens as a transient layer.** The native-search subset favors buttons, command-style shortcuts, or header fields that reveal a focused query surface without replacing the underlying article immediately.
3. **No-result feedback is a real state, not silence.** The deliberately impossible query distinguishes an empty/failure state from loading and from successful answer navigation.
4. **Recovery is replacement, not restart.** Across the records, the useful route follows by clearing/replacing the failed term or by moving from the official result/learn route to a canonical concept page; the browser session is continuous.
5. **First success ends in addressable content.** Completion is a changed official URL/title/content surface, not merely a highlighted suggestion. That boundary makes the result shareable and lets the reader use browser history.
6. **State persistence makes motion inspectable.** Search-open, failed, recovered, and answer surfaces persist long enough to retain independent frames; animation never carries the only evidence.

## Disagreements across products

- **Modal versus inline versus route-based lookup:** modern framework/API sites often use modal search; older books and guide indexes more often expose hierarchical navigation or a separate official search/result URL.
- **Live filtering versus submitted navigation:** some products update results on each input event, while others submit to a results page. A documentation system should not assume one timing model.
- **Result selection versus canonical-route handoff:** native overlays allow direct result activation; book-style sites may require choosing a guide, module, or official concept route before the final answer.
- **Failure vocabulary:** products variously show “no results,” retain unchanged content, or present a missing/empty route. The records preserve this disagreement instead of normalizing it into one invented message.
- **Backtracking ownership:** query clearing belongs to the documentation interaction; browser Back belongs to navigation history. The per-product records distinguish the observed cancellation from the merely available history path.

## Applicability boundaries

- These are public, unauthenticated first-answer journeys. They do not establish behavior for private organization docs, signed-in personalization, region-restricted content, or paid support portals.
- The generic actor seeks one concept, not a complete tutorial, deploy, or code execution. A first answer proves findability; it does not prove technical correctness of every page.
- Reduced-motion preferences, screen readers, high contrast, 400% zoom, focus restoration, and switch access were not varied. The records state these as unknowns rather than inferred compliance.
- For products without a visible inline query field, the evidence uses that same product's official search/learn/result route. This boundary matters when applying modal-search lessons to book-style references.
- Motion timings include network/render latency on the dedicated capture host and are classified as brief/bounded/extended observations, not performance budgets.

## Catalog composition

| Documentation category | Records |
|---|---:|
| Programming language documentation | 5 |
| Frontend framework documentation | 4 |
| Backend framework documentation | 4 |
| Design system documentation | 4 |
| Cloud platform documentation | 3 |
| Database documentation | 3 |
| Web framework documentation | 2 |
| Application platform documentation | 2 |
| Deployment platform documentation | 2 |
| Infrastructure documentation | 2 |
| AI API documentation | 2 |
| Backend platform documentation | 2 |
| Observability documentation | 2 |
| Web platform documentation | 1 |
| Mobile platform documentation | 1 |
| Edge platform documentation | 1 |
| Infrastructure as code documentation | 1 |
| Developer platform documentation | 1 |
| DevOps platform documentation | 1 |
| Payments API documentation | 1 |
| Communications API documentation | 1 |
| Commerce platform documentation | 1 |
| Collaboration API documentation | 1 |
| Machine learning platform documentation | 1 |
| Search and observability documentation | 1 |
| UI development tooling documentation | 1 |

## All 50 derived records

| # | Product | Category | Successful term | Observed lookup mode | Motion | Evidence |
|---:|---|---|---|---|---:|---|
| 01 | [MDN Web Docs](references/01-mdn-web-docs/README.md) | Web platform documentation | `HTML` | native search | 13.92 s | [record](references/01-mdn-web-docs/reference.json) · [motion](references/01-mdn-web-docs/media/motion.webm) |
| 02 | [React Documentation](references/02-react-documentation/README.md) | Frontend framework documentation | `state` | native search | 8.68 s | [record](references/02-react-documentation/reference.json) · [motion](references/02-react-documentation/media/motion.webm) |
| 03 | [Vue.js Guide](references/03-vue-js-guide/README.md) | Frontend framework documentation | `reactivity` | native search | 9.84 s | [record](references/03-vue-js-guide/reference.json) · [motion](references/03-vue-js-guide/media/motion.webm) |
| 04 | [Angular Documentation](references/04-angular-documentation/README.md) | Frontend framework documentation | `components` | native search | 14.48 s | [record](references/04-angular-documentation/reference.json) · [motion](references/04-angular-documentation/media/motion.webm) |
| 05 | [Svelte Documentation](references/05-svelte-documentation/README.md) | Frontend framework documentation | `runes` | native search | 13.48 s | [record](references/05-svelte-documentation/reference.json) · [motion](references/05-svelte-documentation/media/motion.webm) |
| 06 | [Next.js Documentation](references/06-next-js-documentation/README.md) | Web framework documentation | `routing` | native search | 9.56 s | [record](references/06-next-js-documentation/reference.json) · [motion](references/06-next-js-documentation/media/motion.webm) |
| 07 | [Nuxt Documentation](references/07-nuxt-documentation/README.md) | Web framework documentation | `routing` | native search | 10.16 s | [record](references/07-nuxt-documentation/reference.json) · [motion](references/07-nuxt-documentation/media/motion.webm) |
| 08 | [Django Documentation](references/08-django-documentation/README.md) | Backend framework documentation | `models` | native search | 9.0 s | [record](references/08-django-documentation/reference.json) · [motion](references/08-django-documentation/media/motion.webm) |
| 09 | [Ruby on Rails Guides](references/09-ruby-on-rails-guides/README.md) | Backend framework documentation | `routing` | official search/learn route | 8.88 s | [record](references/09-ruby-on-rails-guides/reference.json) · [motion](references/09-ruby-on-rails-guides/media/motion.webm) |
| 10 | [Laravel Documentation](references/10-laravel-documentation/README.md) | Backend framework documentation | `routing` | native search | 10.6 s | [record](references/10-laravel-documentation/reference.json) · [motion](references/10-laravel-documentation/media/motion.webm) |
| 11 | [Spring Boot Reference Documentation](references/11-spring-boot-reference-documentation/README.md) | Backend framework documentation | `configuration` | native search | 9.72 s | [record](references/11-spring-boot-reference-documentation/reference.json) · [motion](references/11-spring-boot-reference-documentation/media/motion.webm) |
| 12 | [.NET Documentation](references/12-net-documentation/README.md) | Application platform documentation | `dependency injection` | official search/learn route | 9.84 s | [record](references/12-net-documentation/reference.json) · [motion](references/12-net-documentation/media/motion.webm) |
| 13 | [The Rust Programming Language](references/13-the-rust-programming-language/README.md) | Programming language documentation | `ownership` | native search | 8.92 s | [record](references/13-the-rust-programming-language/reference.json) · [motion](references/13-the-rust-programming-language/media/motion.webm) |
| 14 | [Go Documentation](references/14-go-documentation/README.md) | Programming language documentation | `modules` | official search/learn route | 9.76 s | [record](references/14-go-documentation/reference.json) · [motion](references/14-go-documentation/media/motion.webm) |
| 15 | [Python 3 Documentation](references/15-python-3-documentation/README.md) | Programming language documentation | `asyncio` | native search | 8.36 s | [record](references/15-python-3-documentation/reference.json) · [motion](references/15-python-3-documentation/media/motion.webm) |
| 16 | [Kotlin Documentation](references/16-kotlin-documentation/README.md) | Programming language documentation | `coroutines` | official search/learn route | 14.44 s | [record](references/16-kotlin-documentation/reference.json) · [motion](references/16-kotlin-documentation/media/motion.webm) |
| 17 | [Swift Documentation](references/17-swift-documentation/README.md) | Programming language documentation | `concurrency` | official search/learn route | 8.0 s | [record](references/17-swift-documentation/reference.json) · [motion](references/17-swift-documentation/media/motion.webm) |
| 18 | [Android Developers Documentation](references/18-android-developers-documentation/README.md) | Mobile platform documentation | `compose` | native search | 10.56 s | [record](references/18-android-developers-documentation/reference.json) · [motion](references/18-android-developers-documentation/media/motion.webm) |
| 19 | [Apple Developer Documentation](references/19-apple-developer-documentation/README.md) | Application platform documentation | `SwiftUI` | native search | 14.44 s | [record](references/19-apple-developer-documentation/reference.json) · [motion](references/19-apple-developer-documentation/media/motion.webm) |
| 20 | [Amazon Web Services Documentation](references/20-amazon-web-services-documentation/README.md) | Cloud platform documentation | `Lambda` | native search | 9.16 s | [record](references/20-amazon-web-services-documentation/reference.json) · [motion](references/20-amazon-web-services-documentation/media/motion.webm) |
| 21 | [Google Cloud Documentation](references/21-google-cloud-documentation/README.md) | Cloud platform documentation | `Cloud Run` | native search | 10.16 s | [record](references/21-google-cloud-documentation/reference.json) · [motion](references/21-google-cloud-documentation/media/motion.webm) |
| 22 | [Microsoft Azure Documentation](references/22-microsoft-azure-documentation/README.md) | Cloud platform documentation | `Functions` | official search/learn route | 9.24 s | [record](references/22-microsoft-azure-documentation/reference.json) · [motion](references/22-microsoft-azure-documentation/media/motion.webm) |
| 23 | [Cloudflare Developer Documentation](references/23-cloudflare-developer-documentation/README.md) | Edge platform documentation | `Workers` | native search | 9.16 s | [record](references/23-cloudflare-developer-documentation/reference.json) · [motion](references/23-cloudflare-developer-documentation/media/motion.webm) |
| 24 | [Vercel Documentation](references/24-vercel-documentation/README.md) | Deployment platform documentation | `deployments` | native search | 8.76 s | [record](references/24-vercel-documentation/reference.json) · [motion](references/24-vercel-documentation/media/motion.webm) |
| 25 | [Netlify Docs](references/25-netlify-docs/README.md) | Deployment platform documentation | `deploys` | native search | 14.6 s | [record](references/25-netlify-docs/reference.json) · [motion](references/25-netlify-docs/media/motion.webm) |
| 26 | [Kubernetes Documentation](references/26-kubernetes-documentation/README.md) | Infrastructure documentation | `pods` | official search/learn route | 9.16 s | [record](references/26-kubernetes-documentation/reference.json) · [motion](references/26-kubernetes-documentation/media/motion.webm) |
| 27 | [Docker Docs](references/27-docker-docs/README.md) | Infrastructure documentation | `containers` | native search | 14.76 s | [record](references/27-docker-docs/reference.json) · [motion](references/27-docker-docs/media/motion.webm) |
| 28 | [Terraform Documentation](references/28-terraform-documentation/README.md) | Infrastructure as code documentation | `providers` | native search | 9.16 s | [record](references/28-terraform-documentation/reference.json) · [motion](references/28-terraform-documentation/media/motion.webm) |
| 29 | [GitHub Docs](references/29-github-docs/README.md) | Developer platform documentation | `pull requests` | native search | 9.88 s | [record](references/29-github-docs/reference.json) · [motion](references/29-github-docs/media/motion.webm) |
| 30 | [GitLab Docs](references/30-gitlab-docs/README.md) | DevOps platform documentation | `merge requests` | official search/learn route | 15.08 s | [record](references/30-gitlab-docs/reference.json) · [motion](references/30-gitlab-docs/media/motion.webm) |
| 31 | [Stripe Documentation](references/31-stripe-documentation/README.md) | Payments API documentation | `payment intents` | official search/learn route | 12.6 s | [record](references/31-stripe-documentation/reference.json) · [motion](references/31-stripe-documentation/media/motion.webm) |
| 32 | [Twilio Documentation](references/32-twilio-documentation/README.md) | Communications API documentation | `messaging` | native search | 9.68 s | [record](references/32-twilio-documentation/reference.json) · [motion](references/32-twilio-documentation/media/motion.webm) |
| 33 | [Shopify Developer Documentation](references/33-shopify-developer-documentation/README.md) | Commerce platform documentation | `webhooks` | native search | 9.36 s | [record](references/33-shopify-developer-documentation/reference.json) · [motion](references/33-shopify-developer-documentation/media/motion.webm) |
| 34 | [Slack API Documentation](references/34-slack-api-documentation/README.md) | Collaboration API documentation | `Block Kit` | official search/learn route | 10.08 s | [record](references/34-slack-api-documentation/reference.json) · [motion](references/34-slack-api-documentation/media/motion.webm) |
| 35 | [OpenAI API Documentation](references/35-openai-api-documentation/README.md) | AI API documentation | `responses` | official search/learn route | 11.24 s | [record](references/35-openai-api-documentation/reference.json) · [motion](references/35-openai-api-documentation/media/motion.webm) |
| 36 | [Anthropic API Documentation](references/36-anthropic-api-documentation/README.md) | AI API documentation | `messages` | native search | 9.88 s | [record](references/36-anthropic-api-documentation/reference.json) · [motion](references/36-anthropic-api-documentation/media/motion.webm) |
| 37 | [Hugging Face Documentation](references/37-hugging-face-documentation/README.md) | Machine learning platform documentation | `transformers` | native search | 9.12 s | [record](references/37-hugging-face-documentation/reference.json) · [motion](references/37-hugging-face-documentation/media/motion.webm) |
| 38 | [PostgreSQL Documentation](references/38-postgresql-documentation/README.md) | Database documentation | `indexes` | official search/learn route | 12.92 s | [record](references/38-postgresql-documentation/reference.json) · [motion](references/38-postgresql-documentation/media/motion.webm) |
| 39 | [MongoDB Documentation](references/39-mongodb-documentation/README.md) | Database documentation | `aggregation` | native search | 8.92 s | [record](references/39-mongodb-documentation/reference.json) · [motion](references/39-mongodb-documentation/media/motion.webm) |
| 40 | [Redis Documentation](references/40-redis-documentation/README.md) | Database documentation | `data types` | native search | 8.76 s | [record](references/40-redis-documentation/reference.json) · [motion](references/40-redis-documentation/media/motion.webm) |
| 41 | [Supabase Documentation](references/41-supabase-documentation/README.md) | Backend platform documentation | `row level security` | official search/learn route | 9.16 s | [record](references/41-supabase-documentation/reference.json) · [motion](references/41-supabase-documentation/media/motion.webm) |
| 42 | [Firebase Documentation](references/42-firebase-documentation/README.md) | Backend platform documentation | `authentication` | native search | 8.96 s | [record](references/42-firebase-documentation/reference.json) · [motion](references/42-firebase-documentation/media/motion.webm) |
| 43 | [Elastic Documentation](references/43-elastic-documentation/README.md) | Search and observability documentation | `search` | native search | 9.44 s | [record](references/43-elastic-documentation/reference.json) · [motion](references/43-elastic-documentation/media/motion.webm) |
| 44 | [Grafana Documentation](references/44-grafana-documentation/README.md) | Observability documentation | `dashboards` | official search/learn route | 10.12 s | [record](references/44-grafana-documentation/reference.json) · [motion](references/44-grafana-documentation/media/motion.webm) |
| 45 | [Datadog Documentation](references/45-datadog-documentation/README.md) | Observability documentation | `monitors` | native search | 9.68 s | [record](references/45-datadog-documentation/reference.json) · [motion](references/45-datadog-documentation/media/motion.webm) |
| 46 | [Material Design 3](references/46-material-design-3/README.md) | Design system documentation | `color` | official search/learn route | 11.08 s | [record](references/46-material-design-3/reference.json) · [motion](references/46-material-design-3/media/motion.webm) |
| 47 | [Carbon Design System](references/47-carbon-design-system/README.md) | Design system documentation | `button` | native search | 15.04 s | [record](references/47-carbon-design-system/reference.json) · [motion](references/47-carbon-design-system/media/motion.webm) |
| 48 | [Atlassian Design System](references/48-atlassian-design-system/README.md) | Design system documentation | `tokens` | official search/learn route | 13.36 s | [record](references/48-atlassian-design-system/reference.json) · [motion](references/48-atlassian-design-system/media/motion.webm) |
| 49 | [Shopify Polaris](references/49-shopify-polaris/README.md) | Design system documentation | `button` | native search | 9.8 s | [record](references/49-shopify-polaris/reference.json) · [motion](references/49-shopify-polaris/media/motion.webm) |
| 50 | [Storybook Documentation](references/50-storybook-documentation/README.md) | UI development tooling documentation | `controls` | native search | 10.44 s | [record](references/50-storybook-documentation/reference.json) · [motion](references/50-storybook-documentation/media/motion.webm) |

## Interaction model

The reusable sequence is: **land → open search/learn route → request impossible term → observe failed lookup → cancel/replace → inspect recovered result → confirm canonical answer**. Confirmation must produce content the reader can identify by title and URL. Cancellation must not destroy the underlying documentation context. If a site has no query control, its navigation/search route should still expose failure, recovery, and canonical completion rather than silently redirecting to a generic home page.

## Motion model

- **Trigger:** navigation and product-owned search/query/learn interaction.
- **Start/end:** stable landing/article to stable first-answer content.
- **Continuity:** one live Weles browser context per record; no synthesized, interpolated, or still-image animation.
- **Interruption/reversal:** an intentionally impossible query/route is abandoned before the valid term/route.
- **Feedback:** open, failure, recovered result, and answer states are visible both in motion and retained local frames.
- **Reduced-motion equivalent:** retained state frames provide inspection without playback; actual product handling of `prefers-reduced-motion` remains unknown unless visible in the recording.

## Accessibility observations and unknowns

Observed evidence is deliberately narrow: the paths used focused controls or official links without drag-only gestures, and feedback persisted visually. The recordings do not establish semantic role/name correctness, live-region announcements, reading order, focus styling/restoration, reduced motion, reflow, high contrast, or assistive-technology compatibility. Those remain explicit unknowns in every record.

## Wisent adoption

Adopt addressable completion, visible empty states, reversible query replacement, persistent result feedback, and a stable answer URL. Support both modal search and route-based/book navigation rather than forcing one interaction model across all documentation. Never encode the only critical state in animation, never treat an unchanged page as successful search feedback, and never infer accessibility compliance from visual motion alone.
