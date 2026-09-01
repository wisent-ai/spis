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
        "Crawl the real {surface} for {name}. {goal}. Required coverage: {coverage}. Systematically inspect every reachable non-destructive control and retain the accessibility and visual state before and after every interaction. Execute and retain distinct cancellation, failure and recovery variants only when the real product exposes them. Retain animations, transitions, loading states and the first-success result with exact browser-history event IDs and artifact URIs. Exercise keyboard focus order, live regions, a screen-reader-relevant accessibility tree and reduced-motion media preference; name any variant that could not be executed instead of inferring it. Open destructive flows only through their final confirmation screen and never commit the final destructive control. {source_guard} Finish with one machine-readable JSON object named spis_evidence. It must contain observed_url, surface_kind, visible_pricing_comparison, canonical_interactions, canonical_journey, canonical_motion_analysis, canonical_accessibility, and artifacts. Every canonical claim must cite an exact retained event ID or stado:// artifact URI; use null or an explicit gap rather than inventing evidence. For pricing pages, visible_pricing_comparison is true only after at least two visible plans or price alternatives were actually observed. For landing pages, observed_url must be the exact requested landing URL after normalization."
    )
}

fn make_plan(
    catalog: &str,
    selected: Option<&str>,
    accounts_path: Option<&Path>,
    batch: &str,
) -> Result<Value> {
    let bindings = account_bindings(accounts_path)?;
    let source_revision = revision()?;
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
        let account_id = binding.and_then(|value| value.get("account_id")).and_then(Value::as_str).map(str::to_string);
        let credential_refs = binding.and_then(|value| value.get("credential_refs")).cloned().unwrap_or_else(|| json!([]));
        let action = binding.and_then(|value| value.get("action")).and_then(Value::as_str)
            .unwrap_or("generic_browser_task");
        let constraints = binding.and_then(|value| value.get("constraints")).cloned().unwrap_or_else(|| json!({}));
        let objective = objective(catalog, name, goal);
        let origin = url::Url::parse(url)?.origin().ascii_serialization();
        let idempotency_key = crate::sha256_hex(
            format!("{source_revision}\n{catalog}\n{slug}\n{action}\n{objective}").as_bytes()
        );
        tasks.push(json!({
            "slug": slug,
            "name": name,
            "url": url,
            "origin": origin,
            "account_id": account_id,
            "credential_refs": credential_refs,
            "action": action,
            "objective": objective,
            "justification": format!("Spis evidence capture for {catalog}/{slug}"),
            "flow_name": format!("spis:{catalog}:{slug}"),
            "idempotency_key": idempotency_key,
            "source_revision": source_revision,
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

fn revision() -> Result<String> { super::crawl::build_revision() }


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
    let output_uri = format!("stado://spis-crawls/{catalog}/{batch}/enqueue-output");
    let output = super::crawl::stado_command()
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
            &output_uri,
        ])
        .output()
        .context("submit web crawl through Stado")?;
    if !output.status.success() {
        bail!(
            "Stado refused web crawl: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    super::crawl::print_submission(
        catalog,
        "web",
        host,
        None,
        &output_uri,
        &String::from_utf8_lossy(&output.stdout),
    )
}

fn persisted_weles_id(uri: &str, path: &Path, plan_hash: &str, slug: &str) -> Option<String> {
    let output = super::crawl::stado_command().args(["storage", "get", uri]).arg(path).output().ok()?;
    if !output.status.success() { return None; }
    let receipt: Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    (receipt.get("plan_sha256").and_then(Value::as_str) == Some(plan_hash)
        && receipt.get("slug").and_then(Value::as_str) == Some(slug))
        .then(|| receipt.get("job_id").and_then(Value::as_str).map(str::to_string))
        .flatten()
}

fn persist_weles_id(uri: &str, path: &Path, receipt: &Value) -> Result<()> {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    std::fs::write(path, serde_json::to_string_pretty(receipt)? + "\n")?;
    let output = super::crawl::stado_command()
        .args(["storage", "put", "--if-absent", "--content-type", "application/json", uri])
        .arg(path)
        .output()?;
    if !output.status.success() {
        bail!("persist Weles correlation receipt {uri}: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
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
    let plan_bytes = serde_json::to_vec(plan)?;
    let plan_hash = crate::sha256_hex(&plan_bytes);
    let batch = plan.get("batch").and_then(Value::as_str).context("web plan has no batch")?;
    let catalog = plan.get("catalog").and_then(Value::as_str).context("web plan has no catalog")?;
    let receipt_root = Path::new("target").join("spis-weles-receipts").join(batch);
    let mut ids = Vec::new();
    let mut task_ids: Vec<(String, String)> = Vec::new();
    for task in tasks {
        let slug = task.get("slug").and_then(Value::as_str).context("web crawl task has no slug")?;
        let receipt_uri = format!("stado://spis-crawls/{catalog}/{batch}/weles-receipts/{slug}.json");
        let receipt_path = receipt_root.join(format!("{slug}.json"));
        let id = if let Some(id) = persisted_weles_id(&receipt_uri, &receipt_path, &plan_hash, slug) {
            id
        } else {
            let job = json!({
                "account_id": task.get("account_id").cloned().unwrap_or(Value::Null),
                "action": task.get("action").cloned().unwrap_or(Value::Null),
                "origin": task.get("origin").cloned().unwrap_or(Value::Null),
                "justification": task.get("justification").cloned().unwrap_or(Value::Null),
                "credential_refs": task.get("credential_refs").cloned().unwrap_or_else(|| json!([])),
                "idempotency_key": task.get("idempotency_key").cloned().unwrap_or(Value::Null),
                "platform": "generic",
                "params": {
                    "url": task.get("url").cloned().unwrap_or(Value::Null),
                    "objective": task.get("objective").cloned().unwrap_or(Value::Null),
                    "flow_name": task.get("flow_name").cloned().unwrap_or(Value::Null),
                    "client_correlation_id": task.get("idempotency_key").cloned().unwrap_or(Value::Null),
                    "constraints": task.get("constraints").cloned().unwrap_or_else(|| json!({})),
                    "headless": true,
                    "browser": "chromium",
                },
            });
            let response = match agent
                .post(&format!("{admission_url}/v1/echo/jobs/enqueue-batch"))
                .set("Authorization", &format!("Bearer {token}"))
                .send_json(json!({"jobs": [job]}))
            {
                Ok(response) => response,
                Err(ureq::Error::Status(status, response)) => {
                    let detail = response.into_string().unwrap_or_default();
                    bail!("Weles admission refused {slug} with HTTP {status}: {detail}");
                }
                Err(error) => bail!("Weles admission refused {slug}: {error}"),
            };
            let response: Value = response.into_json()?;
            let accepted = response.get("job_ids").and_then(Value::as_array)
                .filter(|ids| ids.len() == 1)
                .context("single Weles enqueue returned other than one job id")?;
            let id = accepted[0].as_str().context("Weles admission returned a non-string job id")?.to_string();
            persist_weles_id(&receipt_uri, &receipt_path, &json!({
                "schema": "wisent.weles-correlation-receipt.v1",
                "batch": batch,
                "slug": slug,
                "flow_name": task.get("flow_name"),
                "plan_sha256": plan_hash,
                "idempotency_key": task.get("idempotency_key"),
                "origin": task.get("origin"),
                "action": task.get("action"),
                "credential_refs": task.get("credential_refs"),
                "job_id": id,
                "accepted_at": crate::now_iso_utc(),
            }))?;
            id
        };
        if task_ids.iter().any(|(_, existing)| existing == &id) {
            bail!("Weles admission returned duplicate job id {id}");
        }
        ids.push(json!(id));
        task_ids.push((slug.to_string(), id));
    }
    let deadline = Instant::now() + Duration::from_secs(wait_seconds);
    let jobs = loop {
        let response = match agent
            .post(&format!("{admission_url}/v1/echo/jobs/get-many"))
            .set("Authorization", &format!("Bearer {token}"))
            .send_json(json!({"job_ids": ids}))
        {
            Ok(response) => response,
            Err(ureq::Error::Status(status, response)) => {
                let detail = response.into_string().unwrap_or_default();
                bail!("Weles crawl status refused with HTTP {status}: {detail}");
            }
            Err(error) => bail!("read Weles crawl jobs: {error}"),
        };
        let response: Value = response.into_json()?;
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
            break jobs;
        }
        std::thread::sleep(Duration::from_secs(5));
    };
    let active = jobs.iter().filter(|job| {
        matches!(job.get("status").and_then(Value::as_str), Some("queued" | "running"))
    }).count();
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
    let mut jobs_by_id: std::collections::HashMap<&str, &Value> =
        std::collections::HashMap::new();
    for job in &jobs {
        let id = job
            .get("id")
            .or_else(|| job.get("job_id"))
            .and_then(Value::as_str)
            .context("Weles status returned a job without id")?;
        if jobs_by_id.insert(id, job).is_some() {
            bail!("Weles status returned duplicate job id {id}");
        }
    }
    let tasks_by_slug: std::collections::HashMap<&str, &Value> = tasks
        .iter()
        .map(|task| {
            task.get("slug")
                .and_then(Value::as_str)
                .map(|slug| (slug, task))
                .context("web crawl task has no slug")
        })
        .collect::<Result<_>>()?;
    let records: Vec<Value> = task_ids
        .iter()
        .map(|(slug, id)| {
            let task = tasks_by_slug
                .get(slug.as_str())
                .ok_or_else(|| anyhow!("no web crawl task matches slug {slug}"))?;
            let job = jobs_by_id
                .get(id.as_str())
                .ok_or_else(|| anyhow!("Weles status omitted accepted job id {id} for {slug}"))?;
            Ok(json!({
                "record": slug,
                "name": task.get("name"),
                "job_id": id,
                "job": job,
                "origin": task.get("origin"),
                "action": task.get("action"),
                "idempotency_key": task.get("idempotency_key"),
                "credential_refs": task.get("credential_refs"),
            }))
        })
        .collect::<Result<_>>()?;
    Ok(json!({
        "schema": "wisent.web-crawl-run.v1",
        "batch": plan.get("batch"),
        "catalog": plan.get("catalog"),
        "task_count": tasks.len(),
        "action_ids": ids,
        "jobs": jobs,
        "records": records,
        "failed": failed,
        "pending_review": pending_review,
        "active": active,
        "completed_at": crate::now_iso_utc(),
        "evidence_observations": {
            "canonical_interactions": [],
            "canonical_journey": Value::Null,
            "canonical_accessibility": Value::Null,
            "canonical_motion_analysis": Value::Null,
            "gaps": [
                "Canonical semantic fields remain empty unless each Weles record result carries explicit linked variant observations."
            ]
        },
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
        let active = report.get("active").and_then(Value::as_u64).unwrap_or(0);
        println!("{}", serde_json::to_string(&report)?);
        if failures > 0 || active > 0 {
            bail!("{failures} Weles crawl jobs failed and {active} remain active; correlation receipts were persisted for idempotent resume");
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
