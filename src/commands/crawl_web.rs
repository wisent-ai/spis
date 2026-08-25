//! Real browser-product crawler dispatched through Weles on a Stado-selected host.
//!
//! The coordinator never launches a local browser. It embeds an immutable task
//! plan in an exact-revision Stado job. The worker enqueues one
//! `generic_browser_task` per catalog record into the host-local Weles admission
//! API; Weles owns browser identity, login capabilities, interaction recording,
//! screenshots and receipts.

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const PLAN_SCHEMA: &str = "wisent.web-crawl-plan.v1";
const REPOSITORY: &str = "https://github.com/wisent-ai/spis.git";
const TOKEN_ITEM: &str = "weles-admission-api#token";
const CATALOGS: &[&str] = &[
    "web-app-examples",
    "dashboard-console-examples",
    "onboarding-auth-examples",
    "app-store-listing-examples",
    "design-system-examples",
    "report-evidence-examples",
    "pricing-page-examples",
    "landing-page-examples",
];

fn safe_component(value: &str, name: &str) -> Result<()> {
    if value.is_empty()
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        bail!("{name} must contain only letters, digits, '.', '-' or '_'");
    }
    Ok(())
}

fn canonical_admission_url(value: &str) -> Result<String> {
    let url = url::Url::parse(value).context("--admission-url must be a URL")?;
    let local = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    if url.scheme() != "https" && !(url.scheme() == "http" && local) {
        bail!("--admission-url must be HTTPS or loopback HTTP");
    }
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/")
    {
        bail!("--admission-url may contain only scheme, host and port");
    }
    Ok(value.trim_end_matches('/').to_string())
}

fn record_paths(catalog: &str, selected: Option<&str>) -> Result<Vec<PathBuf>> {
    if !CATALOGS.contains(&catalog) {
        bail!("crawl-web accepts {}", CATALOGS.join(", "));
    }
    let directory = Path::new(catalog).join("references");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&directory)
        .with_context(|| format!("read {}", directory.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    paths.sort();
    paths.retain(|path| {
        let slug = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        selected.is_none_or(|wanted| {
            wanted == slug || wanted == slug.split_once('-').map(|(_, tail)| tail).unwrap_or(slug)
        })
    });
    if paths.is_empty() {
        bail!("no matching records in {catalog}");
    }
    Ok(paths)
}

fn account_bindings(path: Option<&Path>) -> Result<Map<String, Value>> {
    let Some(path) = path else {
        return Ok(Map::new());
    };
    let document: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    if document.get("schema").and_then(Value::as_str) != Some("wisent.web-crawl-accounts.v1") {
        bail!("account bindings must declare wisent.web-crawl-accounts.v1");
    }
    document
        .get("records")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| anyhow!("account bindings must carry a records object"))
}

fn objective(catalog: &str, name: &str, goal: &str) -> String {
    let (surface, coverage, source_guard) = match catalog {
        "web-app-examples" => (
            "browser application",
            "global navigation, the primary create/read/update workflow, search and filters, empty/loading/error states, cancellation, recovery and the first successful result",
            "Do not replace the signed-in application with its marketing site, documentation, app-store page or a guessed flow.",
        ),
        "dashboard-console-examples" => (
            "dashboard or administrative console",
            "navigation hierarchy, date and scope filters, tables, sorting, search, drill-downs, charts, export previews, permission boundaries, empty/loading/error states and recovery",
            "Do not replace the live console with its public marketing site, documentation, screenshots or a guessed flow.",
        ),
        "onboarding-auth-examples" => (
            "onboarding and authentication journey",
            "sign-in, sign-up entry, SSO choices, password recovery, MFA when available, validation failures, backtracking, cancellation and the first authenticated success state",
            "Use only the account identity bound to this task; do not invent credentials or substitute a public product page.",
        ),
        "app-store-listing-examples" => (
            "application-store listing",
            "media carousel, device or platform variants, description expansion, release history, ratings and reviews, privacy and product information, in-app purchases and visible pricing",
            "Crawl the actual store listing named by product_url, not the installed app or the vendor landing page.",
        ),
        "design-system-examples" => (
            "design-system documentation and component explorer",
            "navigation, search, component examples, variants and properties, code or installation copy controls, theming, responsive examples, accessibility guidance and error or empty states",
            "Crawl the actual design-system reference or component explorer, not its owner’s corporate homepage.",
        ),
        "report-evidence-examples" => (
            "interactive report and its evidence surfaces",
            "filters, comparisons, drill-downs, source and evidence links, tables, charts, annotations, export previews, empty/loading/error states and recovery",
            "Crawl the actual report and its linked evidence surfaces, not a summary landing page.",
        ),
        "pricing-page-examples" => (
            "pricing page",
            "billing interval, currency or region controls, seat and usage calculators, plan comparisons, feature disclosure, FAQs, CTA transitions and checkout preview up to but excluding payment",
            "Crawl the actual pricing and plan-selection surface, not a generic product homepage.",
        ),
        "landing-page-examples" => (
            "landing page",
            "global navigation, product-information routes, CTA transitions, media and carousels, forms with validation and cancellation, and desktop, tablet and mobile responsive states",
            "Crawl the exact landing page named by product_url; do not substitute another vendor page, static screenshot or guessed flow.",
        ),
        _ => unreachable!("catalog validated before objective construction"),
    };
    let goal = if goal.trim().is_empty() {
        "Map the product's reachable functionality"
    } else {
        goal
    };
    format!(
        "Crawl the real {surface} for {name}. {goal}. Required coverage: {coverage}. Systematically inspect every reachable non-destructive control and record the accessibility state and visual state before and after every interaction. Retain animations, transitions, loading states, validation failures, cancellation paths, recovery paths and the first-success result. Open destructive flows only through their final confirmation screen and never commit the final destructive control. {source_guard}"
    )
}

fn make_plan(
    catalog: &str,
    selected: Option<&str>,
    accounts_path: Option<&Path>,
    batch: &str,
) -> Result<Value> {
    let bindings = account_bindings(accounts_path)?;
    let mut tasks = Vec::new();
    for path in record_paths(catalog, selected)? {
        let slug = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let record: Value = serde_json::from_slice(&std::fs::read(path.join("reference.json"))?)?;
        let name = record
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let url = record
            .get("product_url")
            .and_then(Value::as_str)
            .filter(|value| value.starts_with("https://") || value.starts_with("http://"))
            .ok_or_else(|| anyhow!("{slug} has no HTTP product_url"))?;
        let goal = record
            .pointer("/journey/goal")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let binding = bindings.get(&slug).and_then(Value::as_object);
        let account_id = binding
            .and_then(|value| value.get("account_id"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let constraints = binding
            .and_then(|value| value.get("constraints"))
            .cloned()
            .unwrap_or_else(|| json!({}));
        tasks.push(json!({
            "slug": slug,
            "name": name,
            "url": url,
            "account_id": account_id,
            "objective": objective(catalog, name, goal),
            "flow_name": format!("spis:{catalog}:{slug}"),
            "constraints": constraints,
        }));
    }
    Ok(json!({
        "schema": PLAN_SCHEMA,
        "batch": batch,
        "catalog": catalog,
        "tasks": tasks,
    }))
}

fn revision() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .context("read Spis source revision")?;
    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success()
        || revision.len() != 40
        || !revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("Spis checkout has no exact Git revision");
    }
    Ok(revision)
}


fn submit(
    host: &str,
    admission_url: &str,
    plan: &Value,
    batch: &str,
    catalog: &str,
    wait_seconds: u64,
) -> Result<()> {
    safe_component(host, "--host")?;
    safe_component(batch, "batch")?;
    let revision = revision()?;
    let encoded_plan = STANDARD.encode(serde_json::to_vec(plan)?);
    let command = format!(
        "cargo run --release -- crawl-web {catalog} --worker --plan-base64 '{encoded_plan}' --admission-url {admission_url} --wait-seconds {wait_seconds}"
    );
    let output = Command::new("stado")
        .args([
            "submit",
            &command,
            "--pinned-host",
            host,
            "--repo",
            REPOSITORY,
            "--repo-ref",
            &revision,
            "--repo-workdir",
            "spis",
            "--repo-extras",
            "",
            "--secret-env",
            &format!("WELES_ADMISSION_TOKEN={TOKEN_ITEM}"),
            "--output-uri",
            &format!("stado://spis-crawls/{catalog}/{batch}/enqueue-output"),
        ])
        .output()
        .context("submit web crawl through Stado")?;
    if !output.status.success() {
        bail!(
            "Stado refused web crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn enqueue(plan: &Value, admission_url: &str, wait_seconds: u64) -> Result<Value> {
    if plan.get("schema").and_then(Value::as_str) != Some(PLAN_SCHEMA) {
        bail!("worker plan must declare {PLAN_SCHEMA}");
    }
    let tasks = plan
        .get("tasks")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("worker plan carries no tasks"))?;
    let token = std::env::var("WELES_ADMISSION_TOKEN")
        .context("worker needs the Stado-scoped WELES_ADMISSION_TOKEN")?;
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(60))
        .build();
    let mut ids = Vec::new();
    for chunk in tasks.chunks(100) {
        let jobs: Vec<Value> = chunk
            .iter()
            .map(|task| {
                json!({
                    "account_id": task.get("account_id").cloned().unwrap_or(Value::Null),
                    "action": "generic_browser_task",
                    "platform": "generic",
                    "params": {
                        "url": task.get("url").cloned().unwrap_or(Value::Null),
                        "objective": task.get("objective").cloned().unwrap_or(Value::Null),
                        "flow_name": task.get("flow_name").cloned().unwrap_or(Value::Null),
                        "constraints": task.get("constraints").cloned().unwrap_or_else(|| json!({})),
                        "headless": true,
                        "browser": "chromium",
                    },
                })
            })
            .collect();
        let response: Value = agent
            .post(&format!("{admission_url}/v1/echo/jobs/enqueue-batch"))
            .set("Authorization", &format!("Bearer {token}"))
            .send_json(json!({"jobs": jobs}))
            .map_err(|error| anyhow!("Weles admission refused web crawl: {error}"))?
            .into_json()?;
        let accepted = response
            .get("job_ids")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("Weles admission returned no job_ids"))?;
        ids.extend(accepted.iter().cloned());
    }
    let deadline = Instant::now() + Duration::from_secs(wait_seconds);
    let jobs = loop {
        let response: Value = agent
            .post(&format!("{admission_url}/v1/echo/jobs/get-many"))
            .set("Authorization", &format!("Bearer {token}"))
            .send_json(json!({"job_ids": ids}))
            .map_err(|error| anyhow!("read Weles crawl jobs: {error}"))?
            .into_json()?;
        let jobs = response
            .get("jobs")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| anyhow!("Weles job read returned no jobs"))?;
        let active = jobs
            .iter()
            .filter(|job| {
                matches!(
                    job.get("status").and_then(Value::as_str),
                    Some("queued" | "running")
                )
            })
            .count();
        if active == 0 {
            break jobs;
        }
        if Instant::now() >= deadline {
            bail!(
                "Weles crawl timed out after {wait_seconds}s with {active} of {} jobs active",
                jobs.len()
            );
        }
        std::thread::sleep(Duration::from_secs(5));
    };
    let failed = jobs
        .iter()
        .filter(|job| {
            matches!(
                job.get("status").and_then(Value::as_str),
                Some("failed" | "rejected")
            )
        })
        .count();
    let pending_review = jobs
        .iter()
        .filter(|job| job.get("status").and_then(Value::as_str) == Some("pending_review"))
        .count();
    Ok(json!({
        "schema": "wisent.web-crawl-run.v1",
        "batch": plan.get("batch"),
        "catalog": plan.get("catalog"),
        "task_count": tasks.len(),
        "action_ids": ids,
        "jobs": jobs,
        "failed": failed,
        "pending_review": pending_review,
        "completed_at": crate::now_iso_utc(),
    }))
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut host: Option<String> = None;
    let mut admission_url: Option<String> = None;
    let mut record: Option<String> = None;
    let mut accounts: Option<PathBuf> = None;
    let mut plan: Option<PathBuf> = None;
    let mut plan_base64: Option<String> = None;
    let mut worker = false;
    let mut wait_seconds = 7_200u64;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                host = Some(rest.get(i).context("--host needs a value")?.clone());
            }
            "--admission-url" => {
                i += 1;
                admission_url = Some(canonical_admission_url(
                    rest.get(i).context("--admission-url needs a value")?,
                )?);
            }
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--accounts" => {
                i += 1;
                accounts = Some(PathBuf::from(
                    rest.get(i).context("--accounts needs a value")?,
                ));
            }
            "--plan" => {
                i += 1;
                plan = Some(PathBuf::from(rest.get(i).context("--plan needs a value")?));
            }
            "--plan-base64" => {
                i += 1;
                plan_base64 = Some(rest.get(i).context("--plan-base64 needs a value")?.clone());
            }
            "--wait-seconds" => {
                i += 1;
                wait_seconds = rest
                    .get(i)
                    .context("--wait-seconds needs a value")?
                    .parse()?;
            }
            "--worker" => worker = true,
            "--help" | "-h" => {
                println!("usage: spis crawl-web <catalog> --host TARGET --admission-url URL [--record SLUG] [--accounts FILE] [--wait-seconds N]\nworker mode: spis crawl-web <catalog> --worker (--plan FILE | --plan-base64 DATA) --admission-url URL --wait-seconds N");
                return Ok(());
            }
            value if value.starts_with('-') => bail!("unknown argument: {value}"),
            value if catalog.is_none() => catalog = Some(value.to_string()),
            value => bail!("unexpected argument: {value}"),
        }
        i += 1;
    }
    let catalog = catalog.context("catalog is required")?;
    if !CATALOGS.contains(&catalog.as_str()) {
        bail!("crawl-web accepts {}", CATALOGS.join(", "));
    }
    if !(30..=86_400).contains(&wait_seconds) {
        bail!("--wait-seconds must be 30..86400");
    }
    let admission_url = admission_url.context("--admission-url is required")?;
    if worker {
        let plan = match (plan, plan_base64) {
            (Some(path), None) => serde_json::from_slice(&std::fs::read(path)?)?,
            (None, Some(encoded)) => serde_json::from_slice(
                &STANDARD.decode(encoded).context("--plan-base64 is invalid")?,
            )?,
            (Some(_), Some(_)) => bail!("worker accepts exactly one of --plan or --plan-base64"),
            (None, None) => bail!("worker requires --plan or --plan-base64"),
        };
        let report = enqueue(&plan, &admission_url, wait_seconds)?;
        let failures = report.get("failed").and_then(Value::as_u64).unwrap_or(0);
        println!("{}", serde_json::to_string_pretty(&report)?);
        if failures > 0 {
            bail!("{failures} Weles crawl jobs failed");
        }
        return Ok(());
    }
    let host = host
        .context("--host is required; web crawls execute through Weles on a pinned Stado host")?;
    safe_component(&host, "--host")?;
    if plan.is_some() || plan_base64.is_some() {
        bail!("--plan and --plan-base64 are worker-only");
    }
    let batch = format!(
        "spis-{catalog}-{}",
        crate::now_iso_utc().replace(':', "-").replace('T', "-")
    );
    let document = make_plan(&catalog, record.as_deref(), accounts.as_deref(), &batch)?;
    let directory = Path::new("target").join("spis-crawl-plans");
    std::fs::create_dir_all(&directory)?;
    let path = directory.join(format!("{batch}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(&document)? + "\n")?;
    submit(
        &host,
        &admission_url,
        &document,
        &batch,
        &catalog,
        wait_seconds,
    )
}
