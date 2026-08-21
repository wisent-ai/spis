# Content structure of the 50 documentation references

Captured 2026-08-19 over HTTP (no browser): for each of the
[50 documentation references](../README.md), the landing-page navigation
(anchor text + path), the full page inventory (robots.txt sitemaps,
`sitemap.xml`, `llms.txt`, or the site's own table of contents where no
sitemap exists — the source is named per record), a histogram of first URL
path segments, and a keyword classification of content kinds over nav labels
and URL sections.

This is the content-inventory pass the original 2026-08-16 capture explicitly
could not answer (that capture records search journeys: one viewport, motion,
five states). One JSON per reference, named like `references/`.

**Total pages inventoried:** 941,331

## What the capture cannot claim

- Landing navigation misses items behind JavaScript-only menus; the page
  inventory compensates where a sitemap or llms.txt exists.
- Keyword classification is lexical. A product may ship a content kind under
  a name the patterns miss, and a matched label does not grade quality.
- Four sites have no machine inventory; their record says exactly which
  substitute was used (landing nav, book ToC, technologies.json).

## Content kinds across the 50

| Content kind | Sites | Examples |
|---|---|---|
| api reference | 36/50 | .NET Documentation, Android Developers Documentation, Angular Documentation, Anthropic API Documentation, Cloudflare Developer Documentation, Datadog Documentation, Django Documentation, Docker Docs … |
| community / support | 34/50 | .NET Documentation, Android Developers Documentation, Anthropic API Documentation, Carbon Design System, Cloudflare Developer Documentation, Django Documentation, Docker Docs, Elastic Documentation … |
| tutorials / learn | 31/50 | .NET Documentation, Android Developers Documentation, Angular Documentation, Anthropic API Documentation, Cloudflare Developer Documentation, Django Documentation, Elastic Documentation, Firebase Documentation … |
| getting started / quickstart | 30/50 | .NET Documentation, Android Developers Documentation, Angular Documentation, Anthropic API Documentation, Atlassian Design System, Carbon Design System, Cloudflare Developer Documentation, Datadog Documentation … |
| changelog / release notes | 30/50 | .NET Documentation, Android Developers Documentation, Anthropic API Documentation, Atlassian Design System, Cloudflare Developer Documentation, Django Documentation, Docker Docs, Elastic Documentation … |
| guides / how-to | 29/50 | .NET Documentation, Android Developers Documentation, Angular Documentation, Anthropic API Documentation, Carbon Design System, Cloudflare Developer Documentation, Django Documentation, Docker Docs … |
| security | 26/50 | .NET Documentation, Android Developers Documentation, Angular Documentation, Cloudflare Developer Documentation, Datadog Documentation, Django Documentation, Docker Docs, Elastic Documentation … |
| migration / upgrade | 23/50 | .NET Documentation, Carbon Design System, Django Documentation, Elastic Documentation, Firebase Documentation, GitHub Docs, GitLab Docs, Go Documentation … |
| cli reference | 22/50 | .NET Documentation, Android Developers Documentation, Angular Documentation, Anthropic API Documentation, GitHub Docs, GitLab Docs, Google Cloud Documentation, Hugging Face Documentation … |
| integrations | 18/50 | Android Developers Documentation, Cloudflare Developer Documentation, Datadog Documentation, Django Documentation, GitHub Docs, GitLab Docs, Google Cloud Documentation, Grafana Documentation … |
| deployment / production | 18/50 | .NET Documentation, Django Documentation, Elastic Documentation, Firebase Documentation, GitLab Docs, Go Documentation, Google Cloud Documentation, Hugging Face Documentation … |
| troubleshooting / errors | 15/50 | .NET Documentation, Angular Documentation, Anthropic API Documentation, Django Documentation, Elastic Documentation, Go Documentation, Laravel Documentation, Netlify Docs … |
| sdk / client libraries | 15/50 | .NET Documentation, Android Developers Documentation, Anthropic API Documentation, Cloudflare Developer Documentation, Firebase Documentation, Google Cloud Documentation, Hugging Face Documentation, Laravel Documentation … |
| pricing / limits | 14/50 | Anthropic API Documentation, Elastic Documentation, Firebase Documentation, GitLab Docs, Google Cloud Documentation, Grafana Documentation, Hugging Face Documentation, Laravel Documentation … |
| configuration / settings | 13/50 | Android Developers Documentation, Django Documentation, Firebase Documentation, Google Cloud Documentation, Laravel Documentation, Microsoft Azure Documentation, Netlify Docs, Next.js Documentation … |
| versioned docs | 12/50 | .NET Documentation, Android Developers Documentation, Angular Documentation, Elastic Documentation, Go Documentation, Microsoft Azure Documentation, MongoDB Documentation, Next.js Documentation … |
| examples / samples | 11/50 | .NET Documentation, Android Developers Documentation, Anthropic API Documentation, Elastic Documentation, Firebase Documentation, GitLab Docs, Kubernetes Documentation, Netlify Docs … |
| contributing | 11/50 | .NET Documentation, Carbon Design System, Django Documentation, Elastic Documentation, GitHub Docs, Go Documentation, Laravel Documentation, React Documentation … |
| playground / sandbox | 10/50 | Android Developers Documentation, Angular Documentation, Cloudflare Developer Documentation, Docker Docs, Go Documentation, Redis Documentation, Svelte Documentation, Swift Documentation … |
| concepts / architecture | 10/50 | .NET Documentation, Android Developers Documentation, Cloudflare Developer Documentation, Django Documentation, Go Documentation, Google Cloud Documentation, Microsoft Azure Documentation, MongoDB Documentation … |
| faq | 9/50 | .NET Documentation, Django Documentation, Firebase Documentation, Go Documentation, Microsoft Azure Documentation, PostgreSQL Documentation, Python 3 Documentation, Storybook Documentation … |
| videos | 8/50 | Android Developers Documentation, Elastic Documentation, Google Cloud Documentation, Grafana Documentation, Microsoft Azure Documentation, MongoDB Documentation, Next.js Documentation, Twilio Documentation |
| accessibility | 8/50 | Android Developers Documentation, Angular Documentation, Atlassian Design System, Grafana Documentation, MDN Web Docs, MongoDB Documentation, Slack API Documentation, Storybook Documentation |
| glossary / definitions | 7/50 | MDN Web Docs, Python 3 Documentation, Redis Documentation, Terraform Documentation, Twilio Documentation, Vercel Documentation, Vue.js Guide |
| best practices | 5/50 | Angular Documentation, Anthropic API Documentation, Storybook Documentation, Svelte Documentation, Terraform Documentation |
| cookbook / recipes | 3/50 | GitHub Docs, Storybook Documentation, Twilio Documentation |
| roadmap | 2/50 | Angular Documentation, Storybook Documentation |


## Second pass: complete inventories and page anatomy (2026-08-19, later)

- Inventories completed: scoped re-pulls for the nine capped giants (MDN 54,594 · Azure/.NET/Google Cloud scoped but still >200k each and capped · Django 2,748 · Elastic 13,742 · MongoDB 24,292 · PostgreSQL 2,299 · Hugging Face filtered to /docs), Python via the full table-of-contents page (472), OpenAI via developers.openai.com (1,105; platform docs are JS-only), Material 3 via its sitemap (262) with section labels recovered from the shipped route table.
- Page anatomy: h1/h2 headings sampled across top sections — 647 pages, 50/50 sites (Material 3's anatomy is bundle-derived route titles; the site is Angular client-rendered and only its inventory is HTML-verifiable).
- In-page section vocabulary across the 50 sites: installation/setup 21, next-steps/related 20, overview/introduction 20, prerequisites 14, code/API-surface sections 14, versions/compatibility 14, FAQ 12, pricing/cost 12, in-page examples 11, parameters 7, usage 7, returns/output 6, limitations/notes 6.
