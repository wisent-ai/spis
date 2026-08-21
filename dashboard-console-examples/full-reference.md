# Full reference: dashboard and console examples

This synthesis is derived from the 50 complete per-product records indexed by [`references.json`](references.json). It is not a substitute for those records: every claim below is bounded by the owner-published motion excerpt, three retained product states, and five-state observed journey stored with each example.

## Evidence corpus

- **50/50 products** have a local playable H.264 MP4 excerpt of an owner-published real-product recording.
- **150 retained state images** are direct frame extractions from those local motion files (three per product).
- Every record includes eight observed interaction categories, a five-state first-success journey, explicit failure/recovery handling, motion analysis, accessibility observations/unknowns, original-source ownership, exact media metadata, and SHA-256 hashes.
- Capture used static media retrieval and ffmpeg only; no browser or GUI was launched on the operator workstation.

### Coverage by catalog category

| Category | Records |
|---|---:|
| CI/CD | 8 |
| Observability | 6 |
| Internal tools | 5 |
| Business intelligence | 4 |
| Cloud control plane | 4 |
| Customer support operations | 4 |
| Container orchestration console | 2 |
| Deployment control plane | 2 |
| Finance operations | 2 |
| Incident management | 2 |
| Product analytics | 2 |
| Application monitoring | 1 |
| Commerce analytics | 1 |
| Container management console | 1 |
| GitOps control plane | 1 |
| Identity administration | 1 |
| Network and security control plane | 1 |
| Observability and analytics | 1 |
| Service operations | 1 |
| Workspace administration | 1 |

## Recurring patterns across the 50 records

### 1. Stable shell, changing work region

Across cloud control planes, observability tools, CI/CD consoles, support workspaces, internal-tool builders, analytics products, and admin consoles, global context stays comparatively stable while the central work region changes. Side navigation, project or tenant scope, and account controls provide orientation; the result area carries most state change. The applicability boundary is focused full-screen authoring or presentation modes, where the shell intentionally recedes.

### 2. Selection precedes commitment

The common first-success rhythm is: enter a product-ready shell, establish scope, select a target, inspect an intermediate state, then accept a populated result. Even when a product autosaves, selection and scope changes remain visually distinguishable from consequential confirmation. This matters most in cloud, identity, finance, and service operations, where “selected” must not be mistaken for “committed.”

### 3. Fast local feedback, slower remote settling

Selection highlights, opened panels, and navigation changes are immediate; data queries, deployments, pipeline runs, monitoring results, and remote updates settle asynchronously. Good consoles expose both phases. The evidence does not justify a universal duration: the appropriate timing class depends on whether the product is changing local view state or waiting on a remote system.

### 4. Recovery preserves context

The strongest recovery pattern is not a reset. Persistent navigation, visible scope, and retained configuration let the operator backtrack, correct a selection, retry, or wait without reconstructing the entire task. This pattern recurs across the five catalog families with the highest operational consequence: cloud/container control, observability, CI/CD, customer support, and administration.

### 5. Completion is a populated, inspectable state

A spinner, empty frame, unchanged status, or marketing promise is not completion. The records use the final retained product frame only when content, state, chart, table, queue, resource, or settings feedback is visible. Each product README makes the non-result and recovery rule explicit.

## Product disagreements worth preserving

- **Search-first vs. navigation-first:** AWS, Google Cloud, Azure, and broad admin suites emphasize global discovery, while focused inboxes, builders, and pipelines often favor persistent collections and tabs. Neither should be universalized.
- **Inline detail vs. route transition:** Support tools and dense observability products frequently preserve list context beside detail; cloud and CI/CD products more often dedicate the work area to the selected resource or run.
- **Autosave vs. explicit confirmation:** Builders and analytics surfaces commonly preview or autosave low-risk changes. Finance, identity, cloud, and incident operations require stronger confirmation and result status.
- **Optimistic feedback vs. explicit waiting:** Local view configuration can update optimistically; remote deployments, queries, syncs, and administrative changes need pending/error/success states.
- **Compact density vs. progressive disclosure:** Data-heavy consoles reward compact tables and persistent filters, while onboarding-oriented or cross-functional tools disclose controls gradually. The operator’s frequency, expertise, and consequence of error determine the appropriate density.
- **Motion as continuity vs. motion as status:** Some products animate spatial transition to preserve context; others minimize spatial motion and rely on spinners, progress, updated counts, or status colors. Reduced-motion behavior remains unknown unless the record explicitly observes it.

## Applicability boundaries

- These records describe the observed desktop-oriented product surfaces in the retained owner recordings; they do not establish mobile parity.
- Role-based visibility, entitlement, tenant policy, sample data, and authentication can materially change a console. Reuse the structural pattern, not product-specific permission assumptions.
- Recorded excerpts prove the described local transitions and states, not every destructive path or every backend failure mode.
- Accessibility observations are visual and structural. Keyboard order, assistive-technology semantics, announcements, measured contrast, zoom/reflow, and reduced-motion preferences remain explicitly unknown unless exercised.
- Product UI and terminology can change after capture; the original source URL, owner, capture date, hashes, and local evidence make the observation auditable.

## All 50 records

| # | Product | Category | Owner-published motion | Complete record |
|---:|---|---|---|---|
| 1 | AWS Management Console | Cloud control plane | [Introduction to the AWS Management Console for New AWS Users](https://www.youtube.com/watch?v=i331jNgsL_4) — Amazon Web Services | [`references/01-aws-management-console/reference.json`](references/01-aws-management-console/reference.json) |
| 2 | Microsoft Azure Portal | Cloud control plane | [Getting started in the Azure Portal](https://www.youtube.com/watch?v=leJRc0JWzSY) — Microsoft Azure Developers | [`references/02-microsoft-azure-portal/reference.json`](references/02-microsoft-azure-portal/reference.json) |
| 3 | Google Cloud Console | Cloud control plane | [How to use the Google Cloud Console](https://www.youtube.com/watch?v=27Pb5g7bEAA) — Google Cloud Tech | [`references/03-google-cloud-console/reference.json`](references/03-google-cloud-console/reference.json) |
| 4 | Cloudflare Dashboard | Network and security control plane | [Cloudflare Access: Product Demo](https://www.youtube.com/watch?v=eshlmtPh4m4) — Cloudflare | [`references/04-cloudflare-dashboard/reference.json`](references/04-cloudflare-dashboard/reference.json) |
| 5 | DigitalOcean Cloud Control Panel | Cloud control plane | [How To Create a Cloud Server on DigitalOcean](https://www.youtube.com/watch?v=vqZ7eKM0WS8) — DigitalOcean | [`references/05-digitalocean-cloud-control-panel/reference.json`](references/05-digitalocean-cloud-control-panel/reference.json) |
| 6 | Vercel Dashboard | Deployment control plane | [Vercel Product Walkthrough](https://www.youtube.com/watch?v=sPmat30SE4k) — Vercel | [`references/06-vercel-dashboard/reference.json`](references/06-vercel-dashboard/reference.json) |
| 7 | Netlify Team Dashboard | Deployment control plane | [Add forms to your project with AI + Netlify](https://www.youtube.com/watch?v=B4PfKu-e3Uk) — Netlify | [`references/07-netlify-team-dashboard/reference.json`](references/07-netlify-team-dashboard/reference.json) |
| 8 | Kubernetes Dashboard | Container orchestration console | [State of the UI: Leveraging Kubernetes Dashboard](https://www.youtube.com/watch?v=_AIL1QENv04) — CNCF | [`references/08-kubernetes-dashboard/reference.json`](references/08-kubernetes-dashboard/reference.json) |
| 9 | SUSE Rancher | Container orchestration console | [SUSE Rancher Prime AI Crew Demo](https://www.youtube.com/watch?v=lBPp3gDfc40) — SUSE | [`references/09-suse-rancher/reference.json`](references/09-suse-rancher/reference.json) |
| 10 | Portainer | Container management console | [Portainer 101 - deploy a container using Portainer](https://www.youtube.com/watch?v=UsutybgCrVI) — Portainer IO | [`references/10-portainer/reference.json`](references/10-portainer/reference.json) |
| 11 | Datadog Dashboards | Observability | [Datadog Service Catalog Demo](https://www.youtube.com/watch?v=1r3J3WCmah0) — Datadog | [`references/11-datadog-dashboards/reference.json`](references/11-datadog-dashboards/reference.json) |
| 12 | Grafana Dashboards | Observability | [Understanding Dashboards in Grafana](https://www.youtube.com/watch?v=vTiIkdDwT-0) — Grafana | [`references/12-grafana-dashboards/reference.json`](references/12-grafana-dashboards/reference.json) |
| 13 | New Relic Dashboards | Observability | [Enhance your New Relic dashboards with Markdown](https://www.youtube.com/watch?v=_dLwy7xskBk) — New Relic | [`references/13-new-relic-dashboards/reference.json`](references/13-new-relic-dashboards/reference.json) |
| 14 | Splunk Observability Cloud | Observability | [Splunk Synthetic Monitoring product demo](https://www.youtube.com/watch?v=frqXK0W777k) — Splunk | [`references/14-splunk-observability-cloud/reference.json`](references/14-splunk-observability-cloud/reference.json) |
| 15 | Kibana Dashboards | Observability and analytics | [Creating your first visualization with Kibana Lens](https://www.youtube.com/watch?v=DzGwmr8nKPg) — Elastic | [`references/15-kibana-dashboards/reference.json`](references/15-kibana-dashboards/reference.json) |
| 16 | Sentry Issues | Application monitoring | [Sentry in Six Minutes](https://www.youtube.com/watch?v=4djseRVSan8) — Sentry | [`references/16-sentry-issues/reference.json`](references/16-sentry-issues/reference.json) |
| 17 | Honeycomb Query and Investigate | Observability | [Honeycomb AI SRE investigation](https://www.youtube.com/watch?v=09G2RUUd4cs) — Honeycomb | [`references/17-honeycomb-query-and-investigate/reference.json`](references/17-honeycomb-query-and-investigate/reference.json) |
| 18 | Dynatrace Dashboards and Notebooks | Observability | [What is Dynatrace in 15 minutes](https://www.youtube.com/watch?v=qo6vjyE-Ak0) — Dynatrace | [`references/18-dynatrace-dashboards-and-notebooks/reference.json`](references/18-dynatrace-dashboards-and-notebooks/reference.json) |
| 19 | PagerDuty Incidents | Incident management | [Manage incidents end-to-end with PagerDuty](https://www.youtube.com/watch?v=3AdG_vyiZig) — PagerDuty Inc. | [`references/19-pagerduty-incidents/reference.json`](references/19-pagerduty-incidents/reference.json) |
| 20 | incident.io Incident Management | Incident management | [How incident.io works](https://www.youtube.com/watch?v=uoSKbDsV-EY) — incident.io | [`references/20-incident-io-incident-management/reference.json`](references/20-incident-io-incident-management/reference.json) |
| 21 | GitHub Actions Workflow Monitoring | CI/CD | [How to use GitHub Actions](https://www.youtube.com/watch?v=BQrohJ3PT7I) — GitHub | [`references/21-github-actions-workflow-monitoring/reference.json`](references/21-github-actions-workflow-monitoring/reference.json) |
| 22 | GitLab CI/CD Pipelines | CI/CD | [Your First GitLab CI/CD Pipeline Explained](https://www.youtube.com/watch?v=IV5MQUEUx44) — GitLab | [`references/22-gitlab-ci-cd-pipelines/reference.json`](references/22-gitlab-ci-cd-pipelines/reference.json) |
| 23 | CircleCI Pipelines | CI/CD | [CircleCI Demo](https://www.youtube.com/watch?v=J1l-icYGyd0) — CircleCI | [`references/23-circleci-pipelines/reference.json`](references/23-circleci-pipelines/reference.json) |
| 24 | Buildkite Pipelines | CI/CD | [Bootstrapping a New Pipeline](https://www.youtube.com/watch?v=GUIK4AdcKM0) — Buildkite | [`references/24-buildkite-pipelines/reference.json`](references/24-buildkite-pipelines/reference.json) |
| 25 | Jenkins Blue Ocean Dashboard | CI/CD | [Getting Started with the Blue Ocean Dashboard](https://www.youtube.com/watch?v=sm1jLj5lbwk) — CloudBeesTV | [`references/25-jenkins-blue-ocean-dashboard/reference.json`](references/25-jenkins-blue-ocean-dashboard/reference.json) |
| 26 | Argo CD | GitOps control plane | [Argo CD Demo](https://www.youtube.com/watch?v=0WAm0y2vLIo) — Argo Project | [`references/26-argo-cd/reference.json`](references/26-argo-cd/reference.json) |
| 27 | Harness Continuous Delivery | CI/CD | [Harness Continuous Delivery demo](https://www.youtube.com/watch?v=akM9df69XmI) — Harness | [`references/27-harness-continuous-delivery/reference.json`](references/27-harness-continuous-delivery/reference.json) |
| 28 | JetBrains TeamCity | CI/CD | [Getting Started with TeamCity](https://www.youtube.com/watch?v=s68u2shSo6o) — JetBrains | [`references/28-jetbrains-teamcity/reference.json`](references/28-jetbrains-teamcity/reference.json) |
| 29 | Azure Pipelines | CI/CD | [Building and Deploying with Azure Pipelines](https://www.youtube.com/watch?v=NuYDAs3kNV8) — Microsoft Visual Studio | [`references/29-azure-pipelines/reference.json`](references/29-azure-pipelines/reference.json) |
| 30 | Zendesk Agent Workspace | Customer support operations | [Zendesk Agent Workspace](https://www.youtube.com/watch?v=6MlLcWapLF0) — Zendesk | [`references/30-zendesk-agent-workspace/reference.json`](references/30-zendesk-agent-workspace/reference.json) |
| 31 | Intercom Inbox | Customer support operations | [Intercom inbox overview](https://www.youtube.com/watch?v=7dFQzkeTe2g) — Intercom / Fin | [`references/31-intercom-inbox/reference.json`](references/31-intercom-inbox/reference.json) |
| 32 | Freshdesk Omnichannel | Customer support operations | [Manage Customer Inquiries in Freshdesk](https://www.youtube.com/watch?v=q_EjfEKijP4) — Freshworks | [`references/32-freshdesk-omnichannel/reference.json`](references/32-freshdesk-omnichannel/reference.json) |
| 33 | Salesforce Service Cloud | Customer support operations | [What Is Service Cloud?](https://www.youtube.com/watch?v=4iFHgOXmMQo) — Salesforce | [`references/33-salesforce-service-cloud/reference.json`](references/33-salesforce-service-cloud/reference.json) |
| 34 | Jira Service Management Queues | Service operations | [Customer Service Management Highlights](https://www.youtube.com/watch?v=qs3-hpXuAfM) — Atlassian | [`references/34-jira-service-management-queues/reference.json`](references/34-jira-service-management-queues/reference.json) |
| 35 | Retool Apps | Internal tools | [How to build multipage apps in Retool](https://www.youtube.com/watch?v=_TnHl8VYLs4) — Retool | [`references/35-retool-apps/reference.json`](references/35-retool-apps/reference.json) |
| 36 | Appsmith Internal Tools | Internal tools | [Building an Internal Tool in 5 Minutes on Appsmith](https://www.youtube.com/watch?v=mzqK0QIZRLs) — Appsmith | [`references/36-appsmith-internal-tools/reference.json`](references/36-appsmith-internal-tools/reference.json) |
| 37 | ToolJet | Internal tools | [ToolJet Introduction](https://www.youtube.com/watch?v=NnG9lZLsHaU) — ToolJet | [`references/37-tooljet/reference.json`](references/37-tooljet/reference.json) |
| 38 | Budibase | Internal tools | [Budibase digital maps release demo](https://www.youtube.com/watch?v=uigkFflJboc) — Budibase | [`references/38-budibase/reference.json`](references/38-budibase/reference.json) |
| 39 | Superblocks | Internal tools | [Live Superblocks Demo](https://www.youtube.com/watch?v=qnAtWBT1W7A) — Superblocks | [`references/39-superblocks/reference.json`](references/39-superblocks/reference.json) |
| 40 | Stripe Dashboard | Finance operations | [Stripe Dashboard for platforms demo](https://www.youtube.com/watch?v=UB_hdbpFNwk) — Stripe | [`references/40-stripe-dashboard/reference.json`](references/40-stripe-dashboard/reference.json) |
| 41 | Shopify Analytics and Reports | Commerce analytics | [The Official Shopify Tutorial For Beginners](https://www.youtube.com/watch?v=roM3wlSqk1c) — Learn With Shopify | [`references/41-shopify-analytics-and-reports/reference.json`](references/41-shopify-analytics-and-reports/reference.json) |
| 42 | Xero Accounting Dashboard | Finance operations | [Xero assurance dashboard](https://www.youtube.com/watch?v=-R6znS1BkZw) — Xero | [`references/42-xero-accounting-dashboard/reference.json`](references/42-xero-accounting-dashboard/reference.json) |
| 43 | Metabase Dashboards | Business intelligence | [See what Metabase can do](https://www.youtube.com/watch?v=j_4vI2bm6-8) — Metabase | [`references/43-metabase-dashboards/reference.json`](references/43-metabase-dashboards/reference.json) |
| 44 | Looker Dashboards | Business intelligence | [Creating a Looker dashboard from an Explore](https://www.youtube.com/watch?v=o3MYjg31rbM) — Looker | [`references/44-looker-dashboards/reference.json`](references/44-looker-dashboards/reference.json) |
| 45 | Tableau Dashboards | Business intelligence | [Tableau Online Product Tour](https://www.youtube.com/watch?v=yJI3dV2FWwU) — Tableau | [`references/45-tableau-dashboards/reference.json`](references/45-tableau-dashboards/reference.json) |
| 46 | Microsoft Power BI Dashboards | Business intelligence | [What is Power BI?](https://www.youtube.com/watch?v=yKTSLffVGbk) — Microsoft Power BI | [`references/46-microsoft-power-bi-dashboards/reference.json`](references/46-microsoft-power-bi-dashboards/reference.json) |
| 47 | Amplitude Analytics Charts | Product analytics | [Chart Types in Amplitude](https://www.youtube.com/watch?v=upUZS1i-MH4) — Amplitude | [`references/47-amplitude-analytics-charts/reference.json`](references/47-amplitude-analytics-charts/reference.json) |
| 48 | Mixpanel Reports | Product analytics | [Mixpanel Product Demo](https://www.youtube.com/watch?v=sRQCfmvh3vg) — Mixpanel | [`references/48-mixpanel-reports/reference.json`](references/48-mixpanel-reports/reference.json) |
| 49 | Okta Admin Console | Identity administration | [Govern Okta Admin Roles](https://www.youtube.com/watch?v=JyeJhv5E09E) — Okta | [`references/49-okta-admin-console/reference.json`](references/49-okta-admin-console/reference.json) |
| 50 | Google Admin Console | Workspace administration | [Chrome OS Demo: Google Admin Console](https://www.youtube.com/watch?v=jhHOfPL-DT0) — Google Chrome | [`references/50-google-admin-console/reference.json`](references/50-google-admin-console/reference.json) |

## Using this corpus

Start with the product whose actor, consequence, and latency most closely match the intended design. Play its local motion offline, inspect the three retained states, and trace the five-state journey plus failure/recovery route. Compare at least one product that disagrees on navigation, confirmation, or density before adopting a pattern.
