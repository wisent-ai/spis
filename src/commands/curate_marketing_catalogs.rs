//! Deterministic selector for the two browser marketing families.
//!
//! This command creates attributed candidates only. Visual, structure and semantic
//! evidence stay explicitly pending until Weles artifacts are imported and measured.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

const PRICING: &[(&str, &str)] = &[
    ("GitHub", "https://github.com/pricing"),
    ("GitLab", "https://about.gitlab.com/pricing/"),
    ("Linear", "https://linear.app/pricing"),
    ("Notion", "https://www.notion.com/pricing"),
    ("Slack", "https://slack.com/pricing"),
    ("Figma", "https://www.figma.com/pricing/"),
    ("Canva", "https://www.canva.com/pricing/"),
    ("Dropbox", "https://www.dropbox.com/plans"),
    ("Box", "https://www.box.com/pricing"),
    ("Asana", "https://asana.com/pricing"),
    ("Monday", "https://monday.com/pricing"),
    ("ClickUp", "https://clickup.com/pricing"),
    ("Airtable", "https://airtable.com/pricing"),
    ("Miro", "https://miro.com/pricing/"),
    ("Jira", "https://www.atlassian.com/software/jira/pricing"),
    ("Bitbucket", "https://www.atlassian.com/software/bitbucket/pricing"),
    ("Vercel", "https://vercel.com/pricing"),
    ("Netlify", "https://www.netlify.com/pricing/"),
    ("Cloudflare", "https://www.cloudflare.com/plans/"),
    ("DigitalOcean", "https://www.digitalocean.com/pricing"),
    ("Render", "https://render.com/pricing"),
    ("Fly.io", "https://fly.io/pricing/"),
    ("Railway", "https://railway.com/pricing"),
    ("Supabase", "https://supabase.com/pricing"),
    ("Neon", "https://neon.com/pricing"),
    ("MongoDB", "https://www.mongodb.com/pricing"),
    ("Elastic", "https://www.elastic.co/pricing/"),
    ("Datadog", "https://www.datadoghq.com/pricing/"),
    ("Sentry", "https://sentry.io/pricing/"),
    ("New Relic", "https://newrelic.com/pricing"),
    ("Grafana", "https://grafana.com/pricing/"),
    ("PostHog", "https://posthog.com/pricing"),
    ("Amplitude", "https://amplitude.com/pricing"),
    ("Mixpanel", "https://mixpanel.com/pricing/"),
    ("Segment", "https://segment.com/pricing"),
    ("HubSpot", "https://www.hubspot.com/pricing"),
    ("Mailchimp", "https://mailchimp.com/pricing/"),
    ("Intercom", "https://www.intercom.com/pricing"),
    ("Zendesk", "https://www.zendesk.com/pricing/"),
    ("Stripe", "https://stripe.com/pricing"),
    ("Shopify", "https://www.shopify.com/pricing"),
    ("Squarespace", "https://www.squarespace.com/pricing"),
    ("Webflow", "https://webflow.com/pricing"),
    ("Framer", "https://www.framer.com/pricing/"),
    ("Zapier", "https://zapier.com/pricing"),
    ("Twilio", "https://www.twilio.com/en-us/pricing"),
    ("Calendly", "https://calendly.com/pricing"),
    ("Loom", "https://www.loom.com/pricing"),
    ("Typeform", "https://www.typeform.com/pricing/"),
    ("Grammarly", "https://www.grammarly.com/plans"),
];

const LANDING: &[(&str, &str)] = &[
    ("Airbnb", "https://www.airbnb.com/"),
    ("Stripe", "https://stripe.com/"),
    ("Linear", "https://linear.app/"),
    ("Notion", "https://www.notion.com/"),
    ("Slack", "https://slack.com/"),
    ("Figma", "https://www.figma.com/"),
    ("Canva", "https://www.canva.com/"),
    ("GitHub", "https://github.com/"),
    ("GitLab", "https://about.gitlab.com/"),
    ("Vercel", "https://vercel.com/"),
    ("Netlify", "https://www.netlify.com/"),
    ("Cloudflare", "https://www.cloudflare.com/"),
    ("DigitalOcean", "https://www.digitalocean.com/"),
    ("Render", "https://render.com/"),
    ("Fly.io", "https://fly.io/"),
    ("Railway", "https://railway.com/"),
    ("Supabase", "https://supabase.com/"),
    ("Neon", "https://neon.com/"),
    ("PlanetScale", "https://planetscale.com/"),
    ("MongoDB", "https://www.mongodb.com/"),
    ("Elastic", "https://www.elastic.co/"),
    ("Datadog", "https://www.datadoghq.com/"),
    ("Sentry", "https://sentry.io/"),
    ("New Relic", "https://newrelic.com/"),
    ("Grafana", "https://grafana.com/"),
    ("PostHog", "https://posthog.com/"),
    ("Amplitude", "https://amplitude.com/"),
    ("Mixpanel", "https://mixpanel.com/"),
    ("Segment", "https://segment.com/"),
    ("HubSpot", "https://www.hubspot.com/"),
    ("Mailchimp", "https://mailchimp.com/"),
    ("Intercom", "https://www.intercom.com/"),
    ("Zendesk", "https://www.zendesk.com/"),
    ("Shopify", "https://www.shopify.com/"),
    ("Squarespace", "https://www.squarespace.com/"),
    ("Webflow", "https://webflow.com/"),
    ("Framer", "https://www.framer.com/"),
    ("Zapier", "https://zapier.com/"),
    ("Twilio", "https://www.twilio.com/"),
    ("Asana", "https://asana.com/"),
    ("Monday", "https://monday.com/"),
    ("ClickUp", "https://clickup.com/"),
    ("Airtable", "https://airtable.com/"),
    ("Miro", "https://miro.com/"),
    ("Zoom", "https://www.zoom.com/"),
    ("Dropbox", "https://www.dropbox.com/"),
    ("Box", "https://www.box.com/"),
    ("Loom", "https://www.loom.com/"),
    ("Calendly", "https://calendly.com/"),
    ("Typeform", "https://www.typeform.com/"),
];

fn slug(value: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            out.push(character);
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn validate(category: &str, entries: &[(&str, &str)]) -> Result<()> {
    if entries.len() != 50 { bail!("{category}: selector must carry exactly 50 sources"); }
    let mut names = BTreeSet::new();
    let mut urls = BTreeSet::new();
    for (name, url) in entries {
        if !names.insert(name.to_ascii_lowercase()) || !urls.insert(*url) { bail!("{category}: duplicate {name} / {url}"); }
        if !url.starts_with("https://") { bail!("{category}: source is not HTTPS: {url}"); }
        if category == "pricing" {
            let lower = url.to_ascii_lowercase();
            if !["pricing", "plans", "plan"].iter().any(|needle| lower.contains(needle)) {
                bail!("pricing source does not identify pricing/plans: {url}");
            }
        }
    }
    Ok(())
}

fn write_catalog(catalog: &str, title: &str, description: &str, category: &str, entries: &[(&str, &str)], replace: bool) -> Result<()> {
    validate(category, entries)?;
    let root = Path::new(catalog);
    let references = root.join("references");
    if references.read_dir()?.next().is_some() && !replace { bail!("{catalog}: references are non-empty; pass --replace"); }
    if references.exists() { std::fs::remove_dir_all(&references)?; }
    std::fs::create_dir_all(&references)?;
    let mut examples = Vec::new();
    let mut index = Vec::new();
    for (offset, (name, url)) in entries.iter().enumerate() {
        let number = offset + 1;
        let directory_name = format!("{number:02}-{}", slug(name));
        let directory = references.join(&directory_name);
        std::fs::create_dir_all(&directory)?;
        let gaps = json!([
            "Weles capture not imported",
            "motion evidence absent",
            "state visuals below the three-state floor",
            "interaction variants absent",
            "journey variants absent",
            "motion analysis absent",
            "accessibility variants absent"
        ]);
        let record = json!({
            "schema": "wisent.full-product-reference.v2",
            "name": name,
            "product_url": url,
            "evidence_status": "partial",
            "upstream_owner": name,
            "captured_at": "2026-09-01",
            "motion": [],
            "states": [],
            "interactions": [],
            "journey": Value::Null,
            "motion_analysis": Value::Null,
            "accessibility": {"measured": false, "observations": [], "unknowns": ["Weles accessibility variants not executed yet"]},
            "motion_provenance": [],
            "evidence_gaps": gaps,
        });
        std::fs::write(directory.join("reference.json"), serde_json::to_string_pretty(&record)? + "\n")?;
        examples.push(json!({
            "name": name,
            "source_url": url,
            "category": category,
            "selection_note": "Official destination selected by the deterministic marketing-family selector; Weles must prove the family surface before completeness.",
            "visual": {"capture_status": "pending-weles"},
            "interface_structure": {"analysis_status": "pending-weles"}
        }));
        index.push(json!({
            "index": number,
            "name": name,
            "path": format!("references/{directory_name}/reference.json"),
            "evidence_status": "partial",
            "evidence_gap_count": 7
        }));
    }
    crate::write_pretty_json(root.join("sources.json").to_str().context("sources path is not UTF-8")?, &json!({
        "schema": "wisent.example-catalog.v2",
        "catalog": catalog,
        "slug": catalog,
        "title": title,
        "description": description,
        "status": "capture-pending",
        "curated_at": "2026-09-01",
        "count": entries.len(),
        "examples": examples,
        "visual_count": 0,
        "structure_count": 0
    }))?;
    crate::write_pretty_json(root.join("references.json").to_str().context("index path is not UTF-8")?, &json!({
        "schema": "wisent.full-reference-catalog.v2",
        "catalog": catalog,
        "reference_count": entries.len(),
        "references": index
    }))?;
    Ok(())
}

pub fn run(rest: &[String]) -> Result<()> {
    let apply = rest.iter().any(|value| value == "--apply");
    let replace = rest.iter().any(|value| value == "--replace");
    if rest.iter().any(|value| !matches!(value.as_str(), "--apply" | "--replace")) {
        bail!("usage: spis curate-marketing-catalogs --apply [--replace]");
    }
    validate("pricing", PRICING)?;
    validate("landing", LANDING)?;
    if !apply {
        println!("pricing-page-examples: 50 attributable candidates\nlanding-page-examples: 50 attributable candidates\npass --apply to write capture-pending records");
        return Ok(());
    }
    write_catalog("pricing-page-examples", "Pricing & plans examples", "Measured pricing pages with visible price and plan comparison.", "pricing", PRICING, replace)?;
    write_catalog("landing-page-examples", "Landing page examples", "Measured exact landing destinations at responsive widths.", "landing", LANDING, replace)?;
    println!("wrote 50 pricing and 50 landing capture-pending records");
    Ok(())
}
