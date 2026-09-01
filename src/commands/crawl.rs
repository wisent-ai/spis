//! Durable coordinator for every Spis crawler.
//!
//! The six surface-specific commands remain the execution engines. This command
//! is the single operator and desktop contract for planning, submission, status,
//! resumption, artifact retrieval and idempotent record import.

use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const OP_SCHEMA: &str = "wisent.crawl-operation.v1";
const RUN_SCHEMA: &str = "wisent.crawl-run.v1";
const SUBMISSION_SCHEMA: &str = "wisent.crawl-submission.v1";
const RUN_ROOT: &str = ".wisent-output/crawl-runs";

const CATALOGS: &[(&str, &str)] = &[
    ("ios-app-examples", "mobile"),
    ("android-app-examples", "mobile"),
    ("macos-app-examples", "desktop"),
    ("desktop-app-examples", "desktop"),
    ("web-app-examples", "web"),
    ("dashboard-console-examples", "web"),
    ("tui-examples", "tui"),
    ("cli-examples", "cli"),
    ("onboarding-auth-examples", "web"),
    ("documentation-site-examples", "docs"),
    ("app-store-listing-examples", "web"),
    ("design-system-examples", "web"),
    ("report-evidence-examples", "web"),
    ("pricing-page-examples", "web"),
    ("landing-page-examples", "web"),
];

fn run_path(run_id: &str) -> PathBuf {
    Path::new(RUN_ROOT).join(run_id).join("run.json")
}

fn exact_revision() -> Result<String> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() || value.len() != 40 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("Spis checkout has no exact Git revision");
    }
    Ok(value)
}

fn persist(run: &Value) -> Result<()> {
    let run_id = run.get("run_id").and_then(Value::as_str).context("run has no run_id")?;
    let path = run_path(run_id);
    std::fs::create_dir_all(path.parent().context("run path has no parent")?)?;
    std::fs::write(path, serde_json::to_string_pretty(run)? + "\n")?;
    Ok(())
}

fn load(run_id: Option<&str>) -> Result<Value> {
    let selected = match run_id {
        Some(value) => value.to_string(),
        None => {
            let mut ids: Vec<String> = std::fs::read_dir(RUN_ROOT)
                .with_context(|| format!("no crawl runs exist under {RUN_ROOT}"))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().join("run.json").is_file())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect();
            ids.sort();
            ids.pop().context("no persisted crawl run exists")?
        }
    };
    crate::read_json(run_path(&selected).to_str().context("run path is not UTF-8")?)
}

fn compact_submission(catalog: &str, engine: &str, host: &str, artifact_uri: Option<&str>, output_uri: &str, stado_stdout: &str) -> Result<Value> {
    let job_id = stado_stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix("Job ID:").map(str::trim))
        .filter(|value| !value.is_empty())
        .context("Stado accepted the command but returned no Job ID")?;
    Ok(json!({
        "schema": SUBMISSION_SCHEMA,
        "catalog": catalog,
        "engine": engine,
        "host": host,
        "job_id": job_id,
        "artifact_uri": artifact_uri,
        "output_uri": output_uri,
        "state": "queued",
    }))
}

/// Surface-specific coordinators call this after Stado accepts a job. The final
/// compact line is stable machine input while the preceding Stado text remains
/// useful to a person invoking the low-level engine directly.
pub fn print_submission(catalog: &str, engine: &str, host: &str, artifact_uri: Option<&str>, output_uri: &str, stado_stdout: &str) -> Result<()> {
    print!("{stado_stdout}");
    let report = compact_submission(catalog, engine, host, artifact_uri, output_uri, stado_stdout)?;
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn parse_submission(stdout: &[u8]) -> Result<Value> {
    let text = String::from_utf8_lossy(stdout);
    text.lines()
        .rev()
        .find_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .filter(|value| value.get("schema").and_then(Value::as_str) == Some(SUBMISSION_SCHEMA))
        .context("crawler returned no machine-readable submission")
}

fn engine_command(catalog: &str, engine: &str, host: &str, admission_url: &str, record: Option<&str>) -> Result<Vec<String>> {
    let mut args = match engine {
        "mobile" => vec!["crawl-mobile".into(), catalog.into(), "--host".into(), host.into()],
        "desktop" => vec!["crawl-desktop".into(), catalog.into(), "--host".into(), host.into()],
        "web" => vec!["crawl-web".into(), catalog.into(), "--host".into(), host.into(), "--admission-url".into(), admission_url.into()],
        "tui" => vec!["crawl-tui".into(), "--host".into(), host.into()],
        "cli" => vec!["crawl-cli".into(), "--host".into(), host.into()],
        "docs" => vec!["crawl-docs".into(), "--all".into(), "--host".into(), host.into()],
        _ => bail!("unknown crawler engine {engine}"),
    };
    if let Some(record) = record {
        if engine == "docs" {
            bail!("--record is not supported by the documentation corpus crawler");
        }
        args.push("--record".into());
        args.push(record.into());
    }
    Ok(args)
}

fn invoke_engine(args: &[String]) -> Result<Output> {
    let executable = std::env::current_exe().context("locate running spis binary")?;
    Command::new(executable).args(args).output().context("launch crawler coordinator")
}

fn selected_specs(selected: &[String]) -> Result<Vec<(&'static str, &'static str)>> {
    if selected.is_empty() {
        return Ok(CATALOGS.to_vec());
    }
    let mut out = Vec::new();
    for wanted in selected {
        let spec = CATALOGS.iter().find(|(catalog, _)| catalog == wanted).copied()
            .ok_or_else(|| anyhow!("unknown crawl catalog {wanted}"))?;
        if !out.contains(&spec) {
            out.push(spec);
        }
    }
    Ok(out)
}
fn registry_placements() -> Result<(HashMap<String, String>, Option<String>)> {
    let output = Command::new("stado").args(["registry", "pull"]).output()?;
    if !output.status.success() {
        bail!(
            "Stado registry could not select crawler hosts: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let registry: Value = serde_json::from_slice(&output.stdout)?;
    let targets = registry
        .get("targets")
        .and_then(Value::as_array)
        .context("Stado registry has no targets")?;
    let web = registry
        .pointer("/service_directory/services/weles-admission/active_host")
        .and_then(Value::as_str)
        .map(str::to_string);
    let admission_url = web.as_deref().and_then(|host| {
        registry.pointer(&format!("/service_directory/services/weles-admission/endpoints/{host}/url"))
            .and_then(Value::as_str).map(str::to_string)
    });
    let always_on = targets.iter().find(|target| {
        target.get("role").and_then(Value::as_str) == Some("always-on")
            && target.pointer("/weles/enabled").and_then(Value::as_bool) == Some(true)
    }).and_then(|target| target.get("name")).and_then(Value::as_str).map(str::to_string);
    let cpu = targets.iter().find(|target| {
        target.get("role").and_then(Value::as_str) == Some("always-on")
    }).and_then(|target| target.get("name")).and_then(Value::as_str).map(str::to_string);
    let mobile = targets.iter().find(|target| {
        target.get("services").and_then(Value::as_array).is_some_and(|services| {
            services.iter().any(|service| {
                service.get("name").and_then(Value::as_str).is_some_and(|name| {
                    name.to_ascii_lowercase().contains("appium")
                })
            })
        })
    }).and_then(|target| target.get("name")).and_then(Value::as_str).map(str::to_string);
    let mut placements = HashMap::new();
    if let Some(host) = web { placements.insert("web".into(), host); }
    if let Some(host) = always_on {
        placements.insert("desktop".into(), host);
    }
    if let Some(host) = mobile { placements.insert("mobile".into(), host); }
    if let Some(host) = cpu {
        placements.insert("cli".into(), host.clone());
        placements.insert("tui".into(), host.clone());
        placements.insert("docs".into(), host);
    }
    Ok((placements, admission_url))
}

fn host_for(
    catalog: &str,
    engine: &str,
    explicit: &HashMap<String, String>,
    discovered: &HashMap<String, String>,
) -> Result<String> {
    explicit
        .get(catalog)
        .or_else(|| explicit.get(engine))
        .or_else(|| explicit.get("*"))
        .or_else(|| discovered.get(engine))
        .cloned()
        .ok_or_else(|| anyhow!(
            "no Stado host advertises the {engine} execution boundary for {catalog}; pass --host {engine}=TARGET after registering that capability"
        ))
}


fn preflight_catalog(catalog: &str, selected_record: Option<&str>) -> Result<()> {
    let root = Path::new(catalog);
    let sources: Value = crate::read_json(root.join("sources.json").to_str().context("sources path is not UTF-8")?)
        .with_context(|| format!("{catalog}: read source manifest"))?;
    let examples = sources.get("examples").and_then(Value::as_array)
        .context(format!("{catalog}: sources.json has no examples"))?;
    let references = root.join("references");
    let mut record_count = 0usize;
    for entry in std::fs::read_dir(&references).with_context(|| format!("{catalog}: read references"))?.flatten() {
        let path = entry.path().join("reference.json");
        if !path.is_file() { continue; }
        let record: Value = crate::read_json(path.to_str().context("reference path is not UTF-8")?)?;
        let directory = entry.file_name().to_string_lossy().to_string();
        if selected_record.is_some_and(|wanted| wanted != directory && directory.split_once('-').map(|(_, slug)| slug) != Some(wanted)) {
            continue;
        }
        let url = record.get("product_url").and_then(Value::as_str)
            .filter(|value| value.starts_with("https://") || value.starts_with("http://"))
            .ok_or_else(|| anyhow!("{catalog}/{directory}: product_url must be HTTP(S)"))?;
        let source = examples.iter().find(|example| example.get("source_url").and_then(Value::as_str) == Some(url))
            .ok_or_else(|| anyhow!("{catalog}/{directory}: product_url is absent from sources.json"))?;
        if catalog == "pricing-page-examples" {
            if source.get("category").and_then(Value::as_str) != Some("pricing") {
                bail!("{catalog}/{directory}: category must be exactly pricing");
            }
            let lower = url.to_ascii_lowercase();
            if !["pricing", "plans", "plan"].iter().any(|needle| lower.contains(needle)) {
                bail!("{catalog}/{directory}: URL does not identify a pricing/plans surface");
            }
        }
        if catalog == "landing-page-examples" && source.get("category").and_then(Value::as_str) != Some("landing") {
            bail!("{catalog}/{directory}: category must be exactly landing");
        }
        record_count += 1;
    }
    if record_count == 0 {
        bail!("{catalog}: selected family is empty");
    }
    Ok(())
}

fn host_probe(host: &str, arguments: &[&str]) -> Value {
    let stado = std::env::var("SPIS_STADO_BIN").unwrap_or_else(|_| "stado".to_string());
    let output = Command::new(stado).args(["host", "exec", host, "--"]).args(arguments).output();
    match output {
        Ok(output) => json!({
            "command": arguments,
            "ready": output.status.success(),
            "stdout": String::from_utf8_lossy(&output.stdout).trim(),
            "stderr": String::from_utf8_lossy(&output.stderr).trim(),
        }),
        Err(error) => json!({"command": arguments, "ready": false, "error": error.to_string()}),
    }
}

fn preflight_record_diagnostics(catalog: &str, engine: &str, selected: Option<&str>, ready: bool) -> Vec<Value> {
    let references = Path::new(catalog).join("references");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(references).into_iter().flatten().flatten()
        .map(|entry| entry.path()).filter(|path| path.join("reference.json").is_file()).collect();
    paths.sort();
    paths.into_iter().filter_map(|path| {
        let slug = path.file_name()?.to_str()?.to_string();
        if selected.is_some_and(|wanted| wanted != slug && wanted != slug.split_once('-').map(|(_, tail)| tail).unwrap_or(&slug)) {
            return None;
        }
        let record: Value = crate::read_json(path.join("reference.json").to_str()?).ok()?;
        let name = record.get("name").and_then(Value::as_str).unwrap_or_default();
        let product_url = record.get("product_url").and_then(Value::as_str).unwrap_or_default();
        let runtime_id = if catalog == "android-app-examples" {
            url::Url::parse(product_url).ok().and_then(|url| url.query_pairs().find(|(key, _)| key == "id").map(|(_, value)| value.into_owned()))
        } else if catalog == "ios-app-examples" {
            product_url.rsplit('/').next().and_then(|tail| tail.strip_prefix("id")).map(str::to_string)
        } else {
            Some(name.to_string())
        };
        Some(json!({
            "record": slug,
            "name": name,
            "product_url": product_url,
            "engine": engine,
            "required_runtime_product": runtime_id,
            "account_binding": if engine == "web" { "anonymous" } else { "not-applicable" },
            "ready": ready,
            "diagnostic": if ready { "host-level prerequisites passed; worker must bind this exact product" } else { "host-level prerequisite failed; record was not submitted" },
        }))
    }).collect()
}

fn host_preflight(catalog: &str, engine: &str, host: &str, admission_url: &str, selected: Option<&str>) -> Value {
    let commands: Vec<Vec<&str>> = match (engine, catalog) {
        ("mobile", "ios-app-examples") => vec![
            vec!["appium", "--version"],
            vec!["appium", "driver", "list", "--installed"],
            vec!["xcrun", "simctl", "list", "devices", "available"],
            vec!["xcrun", "simctl", "listapps", "booted"],
        ],
        ("mobile", _) => vec![
            vec!["appium", "--version"],
            vec!["appium", "driver", "list", "--installed"],
            vec!["adb", "version"],
            vec!["adb", "devices", "-l"],
            vec!["adb", "shell", "pm", "list", "packages"],
        ],
        ("desktop", _) => vec![vec!["cua-driver", "doctor", "--json"], vec!["ls", "/Applications"]],
        ("tui" | "cli", _) => vec![vec!["tmux", "-V"], vec!["cargo", "--version"]],
        ("web", _) => vec![vec!["node", "--version"], vec!["curl", "--version"]],
        ("docs", _) => vec![vec!["git", "--version"], vec!["curl", "--version"], vec!["df", "-h"]],
        _ => vec![],
    };
    let checks: Vec<Value> = commands.iter().map(|command| host_probe(host, command)).collect();
    let admission_ready = if engine == "web" {
        url::Url::parse(admission_url).ok().and_then(|url| {
            let host = url.host_str()?.to_string();
            let port = url.port_or_known_default()?;
            Some(std::net::TcpStream::connect_timeout(
                &format!("{host}:{port}").parse().ok()?,
                std::time::Duration::from_secs(3),
            ).is_ok())
        }).unwrap_or(false)
    } else {
        true
    };
    let ready = checks.iter().all(|check| check.get("ready").and_then(Value::as_bool) == Some(true)) && admission_ready;
    let records = preflight_record_diagnostics(catalog, engine, selected, ready);
    json!({
        "schema": "wisent.crawl-host-preflight.v1",
        "catalog": catalog,
        "engine": engine,
        "host": host,
        "ready": ready,
        "checks": checks,
        "records": records,
        "weles": if engine == "web" { json!({"admission_url": admission_url, "admission_transport_ready": admission_ready, "account_binding": "anonymous unless an engine account manifest is supplied"}) } else { Value::Null },
        "no_permission_prompts_requested": true,
    })
}

fn start(rest: &[String]) -> Result<()> {
    let mut hosts: HashMap<String, String> = HashMap::new();
    let mut admission_url = None;
    let mut catalogs = Vec::new();
    let mut record = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                let value = rest.get(i).context("--host needs a value")?;
                if let Some((scope, target)) = value.split_once('=') {
                    if scope.is_empty() || target.is_empty() {
                        bail!("--host mapping must be ENGINE=TARGET or CATALOG=TARGET");
                    }
                    hosts.insert(scope.to_string(), target.to_string());
                } else {
                    hosts.insert("*".into(), value.clone());
                }
            }
            "--catalog" => { i += 1; catalogs.push(rest.get(i).context("--catalog needs a value")?.clone()); }
            "--record" => { i += 1; record = Some(rest.get(i).context("--record needs a value")?.clone()); }
            "--admission-url" => { i += 1; admission_url = Some(rest.get(i).context("--admission-url needs a value")?.clone()); }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    let (discovered_hosts, discovered_admission_url) = registry_placements()?;
    let admission_url = admission_url.or(discovered_admission_url)
        .context("Stado registry does not expose the Weles admission endpoint; pass --admission-url URL")?;
    let specs = selected_specs(&catalogs)?;
    if record.is_some() && specs.len() != 1 {
        bail!("--record requires exactly one --catalog");
    }
    for (catalog, _) in &specs {
        preflight_catalog(catalog, record.as_deref())?;
    }
    let run_id = format!("crawl-{}", crate::now_iso_utc().replace(':', "-").replace('T', "-"));
    let mut preflight_cache: HashMap<String, Value> = HashMap::new();
    let mut entries = Vec::new();
    for (catalog, engine) in specs {
        let host = host_for(catalog, engine, &hosts, &discovered_hosts)?;
        let command = engine_command(catalog, engine, &host, &admission_url, record.as_deref())?;
        let preflight_key = if engine == "mobile" { format!("{engine}:{catalog}:{host}") } else { format!("{engine}:{host}") };
        let preflight = if let Some(cached) = preflight_cache.get(&preflight_key) {
            let mut reused = cached.clone();
            let ready = reused.get("ready").and_then(Value::as_bool).unwrap_or(false);
            reused["catalog"] = json!(catalog);
            reused["records"] = Value::Array(preflight_record_diagnostics(catalog, engine, record.as_deref(), ready));
            reused
        } else {
            let measured = host_preflight(catalog, engine, &host, &admission_url, record.as_deref());
            preflight_cache.insert(preflight_key, measured.clone());
            measured
        };
        let mut entry = json!({
            "catalog": catalog,
            "engine": engine,
            "host": host,
            "command": command,
            "job_id": null,
            "artifact_uri": null,
            "output_uri": null,
            "state": "submission_failed",
            "records": [],
            "error": null,
            "preflight": preflight,
            "selected_record": record,
        });
        if entry.pointer("/preflight/ready").and_then(Value::as_bool) != Some(true) {
            entry["state"] = json!("preflight_failed");
            entry["error"] = json!("host preflight failed; no crawler job was submitted");
            entries.push(entry);
            continue;
        }
        let output = invoke_engine(&command)?;
        if output.status.success() {
            match parse_submission(&output.stdout) {
                Ok(receipt) => {
                    entry["job_id"] = receipt["job_id"].clone();
                    entry["artifact_uri"] = receipt["artifact_uri"].clone();
                    entry["output_uri"] = receipt["output_uri"].clone();
                    entry["state"] = json!("queued");
                }
                Err(error) => entry["error"] = json!(error.to_string()),
            }
        } else {
            entry["error"] = json!(String::from_utf8_lossy(&output.stderr).trim());
        }
        entries.push(entry);
    }
    let mut run = json!({
        "schema": RUN_SCHEMA,
        "run_id": run_id,
        "source_revision": exact_revision()?,
        "created_at": crate::now_iso_utc(),
        "updated_at": crate::now_iso_utc(),
        "hosts": hosts,
        "admission_url": admission_url,
        "state": "queued",
        "catalogs": entries,
    });
    update_run_state(&mut run);
    persist(&run)?;
    print_operation("start", &run, None)?;
    if has_failures(&run) { bail!("one or more crawler submissions failed"); }
    Ok(())
}

fn machine_status(job_id: &str) -> Result<Value> {
    let output = Command::new("stado").args(["machine", "status", job_id]).output()?;
    let document: Value = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("Stado machine status for {job_id} returned invalid JSON"))?;
    if !output.status.success() || document.get("ok").and_then(Value::as_bool) != Some(true) {
        bail!("{}", document.get("error").cloned().unwrap_or_else(|| json!(String::from_utf8_lossy(&output.stderr).trim())));
    }
    Ok(document.pointer("/result/job").cloned().context("Stado status has no result.job")?)
}

fn refresh(run: &mut Value) {
    if let Some(catalogs) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in catalogs {
            let Some(job_id) = entry.get("job_id").and_then(Value::as_str).map(str::to_string) else { continue; };
            match machine_status(&job_id) {
                Ok(job) => {
                    if entry.get("state").and_then(Value::as_str) != Some("imported") {
                        entry["state"] = job.get("state").cloned().unwrap_or_else(|| json!("failed"));
                    }
                    entry["job"] = job;
                    if entry.get("state").and_then(Value::as_str) != Some("partial") {
                        entry["error"] = Value::Null;
                    }
                }
                Err(error) => entry["error"] = json!(error.to_string()),
            }
        }
    }
    run["updated_at"] = json!(crate::now_iso_utc());
    update_run_state(run);
}

fn update_run_state(run: &mut Value) {
    let states: Vec<&str> = run.get("catalogs").and_then(Value::as_array).into_iter().flatten()
        .filter_map(|entry| entry.get("state").and_then(Value::as_str)).collect();
    let state = if states.iter().any(|s| matches!(
        *s,
        "preflight_failed" | "submission_failed" | "failed" | "cancelled" | "partial"
    )) {
        if states.iter().all(|s| matches!(*s, "preflight_failed" | "submission_failed" | "failed" | "cancelled")) {
            "failed"
        } else {
            "partial"
        }
    } else if states.iter().all(|s| *s == "imported") && !states.is_empty() {
        "imported"
    } else if states.iter().all(|s| matches!(*s, "completed" | "uploaded" | "imported")) && !states.is_empty() {
        "completed"
    } else if states.iter().any(|s| *s == "pending_review") {
        "pending_review"
    } else if states.iter().any(|s| *s == "running") {
        "running"
    } else {
        "queued"
    };
    run["state"] = json!(state);
}

fn status(rest: &[String]) -> Result<()> {
    let (run_id, record) = parse_run_and_record(rest, false)?;
    let mut run = load(run_id.as_deref())?;
    refresh(&mut run);
    let id = run.get("run_id").and_then(Value::as_str).context("run has no id")?.to_string();
    let run_dir = run_path(&id).parent().context("run path has no parent")?.to_path_buf();
    import_ready(&mut run, &id, &run_dir)?;
    persist(&run)?;
    print_operation("status", &run, record.as_deref())
}

fn parse_run_and_record(rest: &[String], require_run: bool) -> Result<(Option<String>, Option<String>)> {
    let mut run = None;
    let mut record = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--run" => { i += 1; run = Some(rest.get(i).context("--run needs a value")?.clone()); }
            "--record" => { i += 1; record = Some(rest.get(i).context("--record needs a value")?.clone()); }
            value => bail!("unknown argument: {value}"),
        }
        i += 1;
    }
    if require_run && run.is_none() { bail!("--run is required"); }
    Ok((run, record))
}

fn rerun_job(job_id: &str) -> Result<String> {
    let output = Command::new("stado").args(["job", "rerun", job_id, "--json"]).output()?;
    if !output.status.success() {
        bail!("Stado refused rerun of {job_id}: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    let value: Value = serde_json::from_slice(&output.stdout)?;
    value.get("new_job_id").or_else(|| value.get("job_id")).and_then(Value::as_str).map(str::to_string)
        .context("Stado rerun returned no new job id")
}

fn resume(rest: &[String]) -> Result<()> {
    let (run_id, _) = parse_run_and_record(rest, true)?;
    let mut run = load(run_id.as_deref())?;
    refresh(&mut run);
    let admission_url = run.get("admission_url").and_then(Value::as_str).unwrap_or_default().to_string();
    if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in entries {
            let state = entry.get("state").and_then(Value::as_str).unwrap_or("submission_failed");
            if state == "preflight_failed" {
                let catalog = entry.get("catalog").and_then(Value::as_str).unwrap_or_default();
                let engine = entry.get("engine").and_then(Value::as_str).unwrap_or_default();
                let host = entry.get("host").and_then(Value::as_str).unwrap_or_default();
                let selected = entry.get("selected_record").and_then(Value::as_str);
                let preflight = host_preflight(catalog, engine, host, &admission_url, selected);
                entry["preflight"] = preflight;
                if entry.pointer("/preflight/ready").and_then(Value::as_bool) != Some(true) {
                    entry["error"] = json!("host preflight still fails; no crawler job was submitted");
                    continue;
                }
            } else if !matches!(state, "submission_failed" | "failed" | "cancelled") {
                continue;
            }
            let result = if let Some(job_id) = entry.get("job_id").and_then(Value::as_str) {
                rerun_job(job_id).map(|fresh| json!({"job_id": fresh}))
            } else {
                entry
                    .get("command")
                    .and_then(Value::as_array)
                    .context("failed submission retained no original command")
                    .and_then(|values| {
                        values
                            .iter()
                            .map(|value| {
                                value
                                    .as_str()
                                    .map(str::to_string)
                                    .context("retained command argument is not a string")
                            })
                            .collect::<Result<Vec<_>>>()
                    })
                    .and_then(|arguments| invoke_engine(&arguments))
                    .and_then(|output| {
                        if output.status.success() {
                            parse_submission(&output.stdout)
                        } else {
                            Err(anyhow!(
                                String::from_utf8_lossy(&output.stderr)
                                    .trim()
                                    .to_string()
                            ))
                        }
                    })
            };
            match result {
                Ok(receipt) => {
                    entry["job_id"] = receipt.get("job_id").cloned().unwrap_or(Value::Null);
                    if receipt.get("artifact_uri").is_some() { entry["artifact_uri"] = receipt["artifact_uri"].clone(); }
                    if receipt.get("output_uri").is_some() { entry["output_uri"] = receipt["output_uri"].clone(); }
                    entry["state"] = json!("queued");
                    entry["error"] = Value::Null;
                }
                Err(error) => entry["error"] = json!(error.to_string()),
            }
        }
    }
    run["updated_at"] = json!(crate::now_iso_utc());
    update_run_state(&mut run);
    persist(&run)?;
    print_operation("resume", &run, None)?;
    if has_failures(&run) { bail!("one or more crawler jobs could not be resumed"); }
    Ok(())
}

fn download_uri(uri: &str, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() { std::fs::create_dir_all(parent)?; }
    let output = Command::new("stado").args(["storage", "get", uri]).arg(destination).output()?;
    if !output.status.success() {
        bail!("download {uri}: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

fn unpack(archive: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    let decoder = GzDecoder::new(File::open(archive)?);
    let mut archive = tar::Archive::new(decoder);
    for member in archive.entries()? {
        let mut member = member?;
        let relative = member.path()?.into_owned();
        if relative.is_absolute() || relative.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
            bail!("crawl archive contains an unsafe path");
        }
        member.unpack_in(destination)?;
    }
    Ok(())
}

fn collect_named(root: &Path, name: &str, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() { collect_named(&path, name, out)?; }
        else if path.file_name().and_then(|value| value.to_str()) == Some(name) { out.push(path); }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() { copy_tree(&from, &to)?; } else { std::fs::copy(from, to)?; }
    }
    Ok(())
}

fn find_record_dir(catalog: &str, slug: &str) -> Option<PathBuf> {
    let root = Path::new(catalog).join("references");
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let tail = name.split_once('-').map(|(_, value)| value).unwrap_or(&name);
        if name == slug || tail == slug { return Some(entry.path()); }
    }
    None
}

fn record_slug(report: &Value, crawl_path: &Path) -> Option<String> {
    report.get("record").or_else(|| report.get("slug")).and_then(Value::as_str).map(str::to_string)
        .or_else(|| crawl_path.parent()?.file_name()?.to_str().map(str::to_string))
}

fn artifact_record(report: &Value, relative: &str, run_id: &str, job_id: Option<&str>, artifact_uri: Option<&str>) -> Value {
    json!({
        "schema": "wisent.crawl-import.v1",
        "run_id": run_id,
        "job_id": job_id,
        "artifact_uri": artifact_uri,
        "raw_report": relative,
        "imported_at": crate::now_iso_utc(),
        "states_seen": report.get("states_seen").or_else(|| report.get("commands_crawled")).cloned().unwrap_or(json!(0)),
        "status": report.get("status").cloned().unwrap_or_else(|| json!("completed")),
        "error": report.get("error").cloned().unwrap_or(Value::Null),
    })
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(root) else { return; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { walk(&path, out); } else { out.push(path); }
        }
    }
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    files
}

fn media_kind(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" => Some("state"),
        "gif" | "mp4" | "webm" | "cast" => Some("motion"),
        _ => None,
    }
}

fn capture_method(engine: &str) -> &'static str {
    match engine {
        "mobile" => "Local product run through Appium with XCUITest or UiAutomator2; screen recording and accessibility source retained",
        "desktop" => "Local product run through Cua Driver; snapshot-bound actions, screenshots, action recording and accessibility tree retained",
        "web" => "Local browser recording through Weles on a Stado-selected host; browser history, screenshots, recordings and signed result retained",
        "tui" => "Local product run in an isolated tmux pseudo-terminal; raw terminal bytes and distinct screens retained",
        "cli" => "Local product run of the real executable in an isolated tmux pseudo-terminal; stdout/stderr, argv and exit status retained",
        _ => "Unclassified Spis crawl",
    }
}

fn copy_evidence_media(
    engine: &str,
    raw_source: &Path,
    raw_destination: &Path,
    record_dir: &Path,
    run_id: &str,
    source_url: &str,
) -> Result<(Vec<Value>, Vec<Value>)> {
    let media_root = record_dir.join("media").join(run_id);
    if media_root.exists() { std::fs::remove_dir_all(&media_root)?; }
    std::fs::create_dir_all(&media_root)?;
    let mut motion = Vec::new();
    let mut states = Vec::new();
    let fallback_motion = files_under(raw_source).into_iter()
        .find(|candidate| media_kind(candidate) == Some("motion"))
        .and_then(|candidate| candidate.strip_prefix(raw_source).ok().map(Path::to_path_buf))
        .map(|relative| format!("media/{run_id}/{}", relative.to_string_lossy()));
    for source in files_under(raw_source) {
        let Some(kind) = media_kind(&source) else { continue; };
        let relative = source.strip_prefix(raw_source).unwrap_or(&source);
        let destination = media_root.join(relative);
        if let Some(parent) = destination.parent() { std::fs::create_dir_all(parent)?; }
        std::fs::copy(&source, &destination)?;
        let local_path = destination.strip_prefix(record_dir).unwrap_or(&destination).to_string_lossy().to_string();
        if kind == "motion" {
            let declared = destination.extension().and_then(|value| value.to_str()).map(|ext| match ext.to_ascii_lowercase().as_str() {
                "mp4" => "video-mp4",
                "webm" => "video-webm",
                "gif" => "animated-gif",
                "webp" => "animated-webp",
                "cast" => "terminal-cast",
                _ => "unknown",
            });
            motion.push(json!({
                "local_path": local_path,
                "source_url": source_url,
                "media_kind": declared,
                "capture_method": capture_method(engine),
                "crawl_evidence_path": raw_destination.strip_prefix(record_dir).unwrap_or(raw_destination).to_string_lossy(),
            }));
        } else {
            let sibling_motion = files_under(source.parent().unwrap_or(raw_source))
                .into_iter()
                .find(|candidate| media_kind(candidate) == Some("motion"))
                .and_then(|candidate| candidate.strip_prefix(raw_source).ok().map(Path::to_path_buf))
                .map(|relative| format!("media/{run_id}/{}", relative.to_string_lossy()))
                .or_else(|| fallback_motion.clone());
            states.push(json!({
                "name": format!("Observed {}", relative.to_string_lossy()),
                "local_path": local_path,
                "source_motion_path": sibling_motion,
            }));
        }
    }
    if motion.is_empty() { states.clear(); }
    Ok((motion, states))
}


fn evidence_interactions(report: &Value) -> Vec<Value> {
    report.pointer("/evidence_observations/canonical_interactions")
        .and_then(Value::as_array).cloned().unwrap_or_default()
}

fn report_accessibility(report: &Value) -> Option<Value> {
    report.pointer("/evidence_observations/canonical_accessibility").cloned()
}

fn accessibility_evidence(raw_source: &Path, run_id: &str, report: &Value) -> Value {
    if let Some(measurement) = report_accessibility(report) {
        return measurement;
    }
    let files = files_under(raw_source);
    let trees: Vec<&PathBuf> = files.iter().filter(|path| {
        matches!(path.extension().and_then(|value| value.to_str()), Some("xml" | "html"))
            || matches!(path.file_name().and_then(|value| value.to_str()), Some("snapshot.json" | "source.json" | "axe.json"))
    }).collect();
    let bytes: u64 = trees.iter().filter_map(|path| std::fs::metadata(path).ok().map(|metadata| metadata.len())).sum();
    json!({
        "measured": false,
        "observations": if trees.is_empty() { vec![] } else { vec![format!("Retained {} accessibility/DOM source files totalling {bytes} bytes under crawl/{run_id}.", trees.len())] },
        "unknowns": [
            "No engine-supplied canonical accessibility measurement was retained.",
            "Screen-reader traversal, focus order, live regions and reduced-motion preference remain unmeasured.",
        ],
    })
}

fn journey_evidence(report: &Value) -> Value {
    report.pointer("/evidence_observations/canonical_journey")
        .cloned().unwrap_or(Value::Null)
}

fn motion_analysis(report: &Value) -> Value {
    report.pointer("/evidence_observations/canonical_motion_analysis")
        .cloned().unwrap_or(Value::Null)
}

fn adapt_canonical_record(engine: &str, run_id: &str, raw_source: &Path, raw_destination: &Path, record_dir: &Path, report: &Value, record: &mut Value) -> Result<()> {
    let source_url = record.get("product_url").and_then(Value::as_str)
        .context("reference record has no product_url")?.to_string();
    let (motion, states) = copy_evidence_media(engine, raw_source, raw_destination, record_dir, run_id, &source_url)?;
    let interactions = evidence_interactions(report);
    let journey = journey_evidence(report);
    let accessibility = accessibility_evidence(raw_source, run_id, report);
    let analysis = motion_analysis(report);
    let object = record.as_object_mut().context("reference record is not an object")?;
    object.insert("captured_at".into(), json!(crate::now_iso_utc()));
    object.insert("motion".into(), Value::Array(motion));
    object.insert("states".into(), Value::Array(states));
    object.insert("interactions".into(), Value::Array(interactions));
    object.insert("journey".into(), journey);
    object.insert("motion_analysis".into(), analysis);
    object.insert("accessibility".into(), accessibility);
    object.insert("evidence_status".into(), json!("partial"));
    object.insert("evidence_gaps".into(), json!(["crawl evidence has not yet passed verify-reference-evidence"]));
    Ok(())
}

fn merge_report(catalog: &str, engine: &str, run_id: &str, job_id: Option<&str>, artifact_uri: Option<&str>, crawl_path: &Path, report: &Value) -> Result<Value> {
    let slug = record_slug(report, crawl_path).context("crawl report has no record slug")?;
    let record_dir = find_record_dir(catalog, &slug).ok_or_else(|| anyhow!("{catalog}: no record matches {slug}"))?;
    let raw_source = crawl_path.parent().context("crawl report has no parent")?;
    let raw_destination = record_dir.join("crawl").join(run_id);
    if raw_destination.exists() { std::fs::remove_dir_all(&raw_destination)?; }
    copy_tree(raw_source, &raw_destination)?;
    let record_path = record_dir.join("reference.json");
    let mut record: Value = crate::read_json(record_path.to_str().context("record path is not UTF-8")?)?;
    adapt_canonical_record(engine, run_id, raw_source, &raw_destination, &record_dir, report, &mut record)?;
    let relative_report = format!("crawl/{run_id}/{}", crawl_path.file_name().and_then(|value| value.to_str()).unwrap_or("crawl.json"));
    let imported = artifact_record(report, &relative_report, run_id, job_id, artifact_uri);
    let runs = record.as_object_mut().context("reference record is not an object")?.entry("crawl_runs").or_insert_with(|| json!([])).as_array_mut().context("crawl_runs is not a list")?;
    if let Some(existing) = runs.iter_mut().find(|value| value.get("run_id").and_then(Value::as_str) == Some(run_id)) { *existing = imported; } else { runs.push(imported); }
    std::fs::write(&record_path, serde_json::to_string_pretty(&record)? + "\n")?;
    let gaps = record.get("evidence_gaps").and_then(Value::as_array).cloned().unwrap_or_default();
    Ok(json!({
        "record": record_dir.file_name().and_then(|value| value.to_str()).unwrap_or(&slug),
        "state": if report.get("status").and_then(Value::as_str) == Some("failed") { "failed" } else { "imported" },
        "states": record.get("states").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
        "interactions": record.get("interactions").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
        "media": count_media(raw_source),
        "gaps": gaps,
        "error": report.get("error").cloned().unwrap_or(Value::Null),
    }))
}

fn count_media(root: &Path) -> usize {
    fn walk(path: &Path, count: &mut usize) {
        let Ok(entries) = std::fs::read_dir(path) else { return; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { walk(&path, count); }
            else if matches!(path.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase).as_deref(), Some("png" | "jpg" | "jpeg" | "webp" | "gif" | "mp4" | "webm" | "cast")) { *count += 1; }
        }
    }
    let mut count = 0;
    walk(root, &mut count);
    count
}

fn find_directory_named(root: &Path, name: &str) -> Option<PathBuf> {
    if root.file_name().and_then(|value| value.to_str()) == Some(name) && root.is_dir() {
        return Some(root.to_path_buf());
    }
    for entry in std::fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_directory_named(&path, name) { return Some(found); }
        }
    }
    None
}

fn import_docs_corpus(catalog: &str, run_id: &str, job_id: Option<&str>, artifact_uri: Option<&str>, root: &Path) -> Result<Vec<Value>> {
    let references = Path::new(catalog).join("references");
    let mut record_dirs: Vec<PathBuf> = std::fs::read_dir(&references)?
        .filter_map(Result::ok).map(|entry| entry.path())
        .filter(|path| path.join("reference.json").is_file()).collect();
    record_dirs.sort();
    let mut out = Vec::new();
    for record_dir in record_dirs {
        let directory_name = record_dir.file_name().and_then(|value| value.to_str()).context("record directory is not UTF-8")?;
        let slug = directory_name.split_once('-').map(|(_, tail)| tail).unwrap_or(directory_name);
        let source = find_directory_named(root, slug);
        let destination = record_dir.join("crawl").join(run_id);
        if destination.exists() { std::fs::remove_dir_all(&destination)?; }
        std::fs::create_dir_all(&destination)?;
        let (state, error) = if let Some(source) = source {
            copy_tree(&source, &destination)?;
            ("imported", Value::Null)
        } else {
            ("missing", json!(format!("documentation crawl archive has no corpus directory for {slug}")))
        };
        let report = json!({
            "schema": "wisent.docs-crawl-record.v1",
            "record": slug,
            "status": state,
            "files": files_under(&destination).len(),
        });
        std::fs::write(destination.join("crawl.json"), serde_json::to_string_pretty(&report)? + "\n")?;
        let record_path = record_dir.join("reference.json");
        let mut record: Value = crate::read_json(record_path.to_str().context("record path is not UTF-8")?)?;
        let imported = artifact_record(&report, &format!("crawl/{run_id}/crawl.json"), run_id, job_id, artifact_uri);
        let runs = record.as_object_mut().context("record is not an object")?.entry("crawl_runs").or_insert_with(|| json!([])).as_array_mut().context("crawl_runs is not a list")?;
        if let Some(existing) = runs.iter_mut().find(|value| value.get("run_id").and_then(Value::as_str) == Some(run_id)) { *existing = imported; } else { runs.push(imported); }
        std::fs::write(record_path, serde_json::to_string_pretty(&record)? + "\n")?;
        out.push(json!({
            "record": directory_name,
            "state": state,
            "states": record.get("states").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "interactions": record.get("interactions").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "media": files_under(&destination).len(),
            "gaps": record.get("evidence_gaps").cloned().unwrap_or_else(|| json!([])),
            "error": error,
        }));
    }
    Ok(out)
}

fn import_catalog(run_id: &str, entry: &mut Value, run_dir: &Path) -> Result<()> {
    let catalog = entry.get("catalog").and_then(Value::as_str).context("catalog entry has no catalog")?.to_string();
    let engine = entry.get("engine").and_then(Value::as_str).unwrap_or_default().to_string();
    let job_id = entry.get("job_id").and_then(Value::as_str).map(str::to_string);
    let artifact_uri = entry.get("artifact_uri").and_then(Value::as_str).map(str::to_string);
    let destination = run_dir.join("downloads").join(&catalog);
    std::fs::create_dir_all(&destination)?;
    if let Some(uri) = artifact_uri.as_deref() {
        let archive = destination.join("crawl.tar.gz");
        download_uri(uri, &archive)?;
        let extracted = destination.join("extracted");
        if extracted.exists() { std::fs::remove_dir_all(&extracted)?; }
        unpack(&archive, &extracted)?;
    } else if let Some(job_id) = job_id.as_deref() {
        let output = Command::new("stado").args(["machine", "artifacts", job_id, "--output-dir"]).arg(&destination).output()?;
        if !output.status.success() {
            bail!("download canonical artifacts for {catalog}: {}", String::from_utf8_lossy(&output.stderr).trim());
        }
    }
    if engine == "docs" {
        let records = import_docs_corpus(&catalog, run_id, job_id.as_deref(), artifact_uri.as_deref(), &destination)?;
        entry["records"] = Value::Array(records);
        entry["state"] = json!("imported");
        entry["error"] = Value::Null;
        return Ok(());
    }
    let mut reports = Vec::new();
    collect_named(&destination, "crawl.json", &mut reports)?;
    if engine == "web" && reports.is_empty() {
        collect_named(&destination, "command_output.log", &mut reports)?;
    }
    let mut records = Vec::new();
    for path in reports {
        if path.file_name().and_then(|value| value.to_str()) == Some("command_output.log") {
            let text = std::fs::read_to_string(&path)?;
            let candidate = text.lines().rev().find_map(|line| serde_json::from_str::<Value>(line).ok());
            if let Some(report) = candidate { records.extend(import_web_report(&catalog, run_id, job_id.as_deref(), &path, &report)?); }
        } else {
            let report: Value = crate::read_json(path.to_str().context("crawl report path is not UTF-8")?)?;
            records.push(merge_report(&catalog, &engine, run_id, job_id.as_deref(), artifact_uri.as_deref(), &path, &report)?);
        }
    }
    if records.is_empty() { bail!("{catalog}: downloaded artifacts contain no importable record reports"); }
    entry["records"] = Value::Array(records);
    entry["state"] = json!("imported");
    entry["error"] = Value::Null;
    Ok(())
}

fn collect_weles_uris(value: &Value, uris: &mut Vec<String>) {
    match value {
        Value::String(text) if text.starts_with("stado://weles/recordings/") => {
            if !uris.contains(text) { uris.push(text.clone()); }
        }
        Value::Array(values) => values.iter().for_each(|value| collect_weles_uris(value, uris)),
        Value::Object(values) => values.values().for_each(|value| collect_weles_uris(value, uris)),
        _ => {}
    }
}

fn find_spis_evidence(value: &Value) -> Option<Value> {
    match value {
        Value::Object(object) => {
            if let Some(evidence) = object.get("spis_evidence").filter(|evidence| evidence.is_object()) {
                return Some(evidence.clone());
            }
            object.values().find_map(find_spis_evidence)
        }
        Value::Array(values) => values.iter().find_map(find_spis_evidence),
        Value::String(text) => serde_json::from_str::<Value>(text.trim()).ok().and_then(|parsed| find_spis_evidence(&parsed)),
        _ => None,
    }
}

fn web_observation(job: &Value) -> Value {
    let history = job.pointer("/result/generic_browser_task/history")
        .or_else(|| job.pointer("/result/history"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let path: Vec<Value> = history.iter().enumerate().map(|(index, step)| {
        json!({
            "label": step.get("tool").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| format!("browser action {}", index + 1)),
            "args": step.get("args").cloned().unwrap_or_else(|| json!({})),
        })
    }).collect();
    let evidence = find_spis_evidence(job);
    json!({
        "schema": "wisent.web-crawl-record.v1",
        "states": if path.is_empty() { vec![] } else { vec![json!({"path": path})] },
        "states_seen": history.len(),
        "blocked_edges": job.get("error").into_iter().cloned().collect::<Vec<_>>(),
        "status": job.get("status").cloned().unwrap_or_else(|| json!("failed")),
        "error": job.get("error").cloned().unwrap_or(Value::Null),
        "evidence_observations": {
            "canonical_interactions": evidence.as_ref().and_then(|value| value.get("canonical_interactions")).cloned().unwrap_or_else(|| json!([])),
            "canonical_journey": evidence.as_ref().and_then(|value| value.get("canonical_journey")).cloned().unwrap_or(Value::Null),
            "canonical_motion_analysis": evidence.as_ref().and_then(|value| value.get("canonical_motion_analysis")).cloned().unwrap_or(Value::Null),
            "canonical_accessibility": evidence.as_ref().and_then(|value| value.get("canonical_accessibility")).cloned().unwrap_or(Value::Null),
            "surface_proof": evidence,
        },
    })
}

fn normalized_surface_url(value: &str) -> Result<String> {
    let mut url = url::Url::parse(value).context("surface proof URL is invalid")?;
    url.set_fragment(None);
    if url.path() != "/" {
        let trimmed = url.path().trim_end_matches('/').to_string();
        url.set_path(&trimmed);
    }
    Ok(url.to_string())
}

fn validate_web_surface(catalog: &str, product_url: &str, observation: &Value) -> Result<()> {
    let proof = observation.pointer("/evidence_observations/surface_proof")
        .filter(|value| value.is_object())
        .ok_or_else(|| anyhow!("{catalog}: Weles result has no machine-readable spis_evidence surface proof"))?;
    let observed_url = proof.get("observed_url").and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{catalog}: spis_evidence has no observed_url"))?;
    if catalog == "landing-page-examples" {
        if proof.get("surface_kind").and_then(Value::as_str) != Some("landing") {
            bail!("{catalog}: Weles did not identify the retained surface as a landing page");
        }
        if normalized_surface_url(observed_url)? != normalized_surface_url(product_url)? {
            bail!("{catalog}: Weles observed {observed_url}, expected exact landing URL {product_url}");
        }
    }
    if catalog == "pricing-page-examples" {
        if proof.get("surface_kind").and_then(Value::as_str) != Some("pricing") {
            bail!("{catalog}: Weles did not identify the retained surface as a pricing page");
        }
        if proof.get("visible_pricing_comparison").and_then(Value::as_bool) != Some(true) {
            bail!("{catalog}: Weles did not prove a visible comparison of at least two plans or prices");
        }
    }
    Ok(())
}

fn import_web_report(catalog: &str, run_id: &str, job_id: Option<&str>, _path: &Path, report: &Value) -> Result<Vec<Value>> {
    let records = report.get("records").and_then(Value::as_array).context("web report has no records mapping")?;
    let mut out = Vec::new();
    for item in records {
        let slug = item.get("record").and_then(Value::as_str).context("web record mapping has no record")?;
        let record_dir = find_record_dir(catalog, slug).ok_or_else(|| anyhow!("{catalog}: no record matches {slug}"))?;
        let destination = record_dir.join("crawl").join(run_id);
        if destination.exists() { std::fs::remove_dir_all(&destination)?; }
        std::fs::create_dir_all(&destination)?;
        let relative = format!("crawl/{run_id}/weles-result.json");
        std::fs::write(destination.join("weles-result.json"), serde_json::to_string_pretty(item)? + "\n")?;
        let job = item.get("job").unwrap_or(item);
        let mut uris = Vec::new();
        collect_weles_uris(job, &mut uris);
        let artifacts = destination.join("artifacts");
        std::fs::create_dir_all(&artifacts)?;
        let mut downloaded = Vec::new();
        for (index, uri) in uris.iter().enumerate() {
            let basename = uri.rsplit('/').find(|part| !part.is_empty()).unwrap_or("artifact");
            let safe: String = basename.chars().map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') { character } else { '_' }
            }).collect();
            let local = artifacts.join(format!("{index:04}-{safe}"));
            download_uri(uri, &local)?;
            downloaded.push((uri.clone(), local));
        }
        let observation = web_observation(job);
        std::fs::write(destination.join("crawl.json"), serde_json::to_string_pretty(&observation)? + "\n")?;
        let record_path = record_dir.join("reference.json");
        let mut record: Value = crate::read_json(record_path.to_str().context("record path is not UTF-8")?)?;
        validate_web_surface(catalog, record.get("product_url").and_then(Value::as_str).unwrap_or_default(), &observation)?;
        update_web_source_visual(catalog, &record_dir, &record, &downloaded)?;
        adapt_canonical_record("web", run_id, &destination, &destination, &record_dir, &observation, &mut record)?;
        let imported = artifact_record(item, &relative, run_id, job_id, None);
        let runs = record.as_object_mut().context("record is not an object")?.entry("crawl_runs").or_insert_with(|| json!([])).as_array_mut().context("crawl_runs is not a list")?;
        if let Some(existing) = runs.iter_mut().find(|value| value.get("run_id").and_then(Value::as_str) == Some(run_id)) { *existing = imported; } else { runs.push(imported); }
        std::fs::write(record_path, serde_json::to_string_pretty(&record)? + "\n")?;
        let state = job.get("status").and_then(Value::as_str).unwrap_or("failed");
        let gaps = record.get("evidence_gaps").and_then(Value::as_array).cloned().unwrap_or_default();
        out.push(json!({
            "record": slug,
            "state": if state == "completed" { "imported" } else { state },
            "states": record.get("states").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "interactions": record.get("interactions").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "media": count_media(&destination),
            "gaps": gaps,
            "error": job.get("error").cloned().unwrap_or(Value::Null),
        }));
    }
    Ok(out)
}

fn run_spis_command(arguments: &[&str]) -> Result<String> {
    let executable = std::env::current_exe().context("resolve current Spis executable")?;
    let output = Command::new(executable).args(arguments).output()?;
    if !output.status.success() {
        bail!(
            "spis {} failed: {}{}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn update_web_source_visual(catalog: &str, record_dir: &Path, record: &Value, artifacts: &[(String, PathBuf)]) -> Result<()> {
    let Some((source_uri, source_path)) = artifacts.iter().find(|(_, path)| media_kind(path) == Some("state")) else {
        return Ok(());
    };
    let extension = source_path.extension().and_then(|value| value.to_str()).unwrap_or("png").to_ascii_lowercase();
    let image_name = format!("{}.{}", record_dir.file_name().and_then(|value| value.to_str()).unwrap_or("capture"), extension);
    let image_path = Path::new(catalog).join("images").join(image_name);
    if let Some(parent) = image_path.parent() { std::fs::create_dir_all(parent)?; }
    std::fs::copy(source_path, &image_path)?;
    let bytes = std::fs::read(&image_path)?;
    let decoded = image::open(&image_path)?;
    let sources_path = Path::new(catalog).join("sources.json");
    let mut sources: Value = crate::read_json(sources_path.to_str().context("sources path is not UTF-8")?)?;
    let product_url = record.get("product_url").and_then(Value::as_str).context("record has no product_url")?;
    let examples = sources.get_mut("examples").and_then(Value::as_array_mut).context("sources examples are not a list")?;
    let example = examples.iter_mut().find(|example| example.get("source_url").and_then(Value::as_str) == Some(product_url))
        .ok_or_else(|| anyhow!("{catalog}: no source example matches {product_url}"))?;
    example["visual"] = json!({
        "source_page_url": product_url,
        "source_artifact_uri": source_uri,
        "local_path": image_path.strip_prefix(catalog).unwrap_or(&image_path).to_string_lossy(),
        "capture_kind": "local-browser-screenshot",
        "captured_at": crate::now_iso_utc(),
        "format": extension,
        "width": decoded.width(),
        "height": decoded.height(),
        "bytes": bytes.len(),
        "sha256": crate::sha256_hex(&bytes),
    });
    let visual_count = examples.iter().filter(|example| {
        example.pointer("/visual/capture_status").and_then(Value::as_str) != Some("pending-weles")
    }).count();
    sources["visual_count"] = json!(visual_count);
    std::fs::write(sources_path, serde_json::to_string_pretty(&sources)? + "\n")?;
    Ok(())
}

fn summarize_catalog_records(catalog: &str, run_id: &str, entry: &mut Value) -> Result<()> {
    let reference = Path::new(catalog).join("references");
    let mut summaries = Vec::new();
    if reference.is_dir() {
        let mut directories: Vec<PathBuf> = std::fs::read_dir(&reference)?
            .filter_map(Result::ok)
            .map(|item| item.path())
            .filter(|path| path.join("reference.json").is_file())
            .collect();
        directories.sort();
        for record_dir in directories {
            let record: Value = crate::read_json(record_dir.join("reference.json").to_str().context("record path is not UTF-8")?)?;
            let imported = record.get("crawl_runs").and_then(Value::as_array).is_some_and(|runs| {
                runs.iter().any(|run| run.get("run_id").and_then(Value::as_str) == Some(run_id))
            });
            let complete = record.get("evidence_status").and_then(Value::as_str) == Some("complete");
            let gaps = record.get("evidence_gaps").and_then(Value::as_array).cloned().unwrap_or_default();
            summaries.push(json!({
                "record": record_dir.file_name().and_then(|value| value.to_str()).unwrap_or("unknown"),
                "state": if imported && complete { "complete" } else if imported { "partial" } else { "missing" },
                "states": record.get("states").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
                "interactions": record.get("interactions").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
                "media": record.get("motion").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
                "gaps": gaps,
                "error": Value::Null,
            }));
        }
    }
    entry["records"] = Value::Array(summaries);
    Ok(())
}

fn import_ready(run: &mut Value, run_id: &str, run_dir: &Path) -> Result<()> {
    let mut imported_catalogs = Vec::new();
    if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in entries {
            let state = entry.get("state").and_then(Value::as_str).unwrap_or_default();
            if state == "imported" { continue; }
            if !matches!(state, "completed" | "uploaded") { continue; }
            let catalog = entry.get("catalog").and_then(Value::as_str).context("catalog entry has no catalog")?.to_string();
            let engine = entry.get("engine").and_then(Value::as_str).unwrap_or_default().to_string();
            match import_catalog(run_id, entry, run_dir)
                .and_then(|_| {
                    if engine == "web" {
                        run_spis_command(&["analyze-example-structures", &catalog]).map(|_| ())
                    } else {
                        Ok(())
                    }
                })
                .and_then(|_| run_spis_command(&["verify-reference-evidence", "--catalog", &catalog, "--apply"]).map(|_| ()))
                .and_then(|_| summarize_catalog_records(&catalog, run_id, entry))
            {
                Ok(()) => imported_catalogs.push(catalog),
                Err(error) => {
                    entry["state"] = json!("partial");
                    entry["error"] = json!(error.to_string());
                }
            }
        }
    }
    if !imported_catalogs.is_empty() {
        run_spis_command(&["generate-example-catalogs"])?;
    }
    run["updated_at"] = json!(crate::now_iso_utc());
    update_run_state(run);
    Ok(())
}

fn import(rest: &[String]) -> Result<()> {
    let (run_id, _) = parse_run_and_record(rest, true)?;
    let mut run = load(run_id.as_deref())?;
    refresh(&mut run);
    let id = run.get("run_id").and_then(Value::as_str).context("run has no id")?.to_string();
    let run_dir = run_path(&id).parent().context("run path has no parent")?.to_path_buf();
    import_ready(&mut run, &id, &run_dir)?;
    if let Some(entries) = run.get_mut("catalogs").and_then(Value::as_array_mut) {
        for entry in entries {
            let state = entry.get("state").and_then(Value::as_str).unwrap_or_default();
            if !matches!(state, "imported" | "partial" | "failed" | "cancelled" | "submission_failed" | "preflight_failed") {
                entry["error"] = json!(format!("artifact import requires a terminal successful job, found {state}"));
            }
        }
    }
    update_run_state(&mut run);
    persist(&run)?;
    print_operation("import", &run, None)?;
    if has_failures(&run) || run.get("state").and_then(Value::as_str) != Some("imported") {
        bail!("one or more crawl artifacts were not imported");
    }
    Ok(())
}

fn has_failures(run: &Value) -> bool {
    run.get("catalogs").and_then(Value::as_array).into_iter().flatten().any(|entry| {
        matches!(entry.get("state").and_then(Value::as_str), Some("preflight_failed" | "submission_failed" | "failed" | "cancelled" | "partial"))
    })
}

fn print_operation(operation: &str, run: &Value, record_filter: Option<&str>) -> Result<()> {
    let mut catalogs = run.get("catalogs").and_then(Value::as_array).cloned().unwrap_or_default();
    if let Some(record) = record_filter {
        for catalog in &mut catalogs {
            if let Some(records) = catalog.get_mut("records").and_then(Value::as_array_mut) {
                records.retain(|item| item.get("record").and_then(Value::as_str).is_some_and(|value| value == record || value.split_once('-').map(|(_, tail)| tail) == Some(record)));
            }
        }
        catalogs.retain(|catalog| catalog.get("records").and_then(Value::as_array).is_some_and(|records| !records.is_empty()));
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for catalog in &catalogs {
        *counts.entry(catalog.get("state").and_then(Value::as_str).unwrap_or("unknown").to_string()).or_default() += 1;
        if let Some(records) = catalog.get("records").and_then(Value::as_array) {
            for record in records {
                *counts.entry(format!("record_{}", record.get("state").and_then(Value::as_str).unwrap_or("unknown"))).or_default() += 1;
            }
        }
    }
    let document = json!({
        "schema": OP_SCHEMA,
        "operation": operation,
        "run_id": run.get("run_id"),
        "state": run.get("state"),
        "source_revision": run.get("source_revision"),
        "updated_at": run.get("updated_at"),
        "counts": counts,
        "catalogs": catalogs,
    });
    println!("{}", serde_json::to_string(&document)?);
    Ok(())
}

fn usage() {
    println!("usage:\n  spis crawl start --host TARGET [--catalog SLUG ...] [--record SLUG] [--admission-url URL]\n  spis crawl status [--run RUN_ID] [--record SLUG]\n  spis crawl resume --run RUN_ID\n  spis crawl import --run RUN_ID\n\nAll commands emit one wisent.crawl-operation.v1 JSON document on stdout. The CLI is the process API; Spis does not expose a second HTTP /v1/crawl surface.");
}

pub fn run(rest: &[String]) -> Result<()> {
    match rest.first().map(String::as_str) {
        Some("start") => start(&rest[1..]),
        Some("status") => status(&rest[1..]),
        Some("resume") => resume(&rest[1..]),
        Some("import") => import(&rest[1..]),
        Some("--help" | "-h") | None => { usage(); Ok(()) }
        Some(other) => bail!("unknown crawl operation: {other}"),
    }
}
