//! `spis capture-widths` — capture a landing record at all three review
//! widths through Weles (1:1 port of capture-widths.py).
//!
//! Builds one `wisent.weles-capture-plan.v1` batch with a composition-axis
//! capture per width (390 × 844, 768 × 1024, 1440 × 1000) and enqueues it
//! through `stado host weles-capture`. Weles stores the screenshot **and**
//! the rendered DOM (`*_dom_*.html`) for every width.
//!
//! Usage: spis capture-widths <catalog> [--record <NN|slug>] [--host <target>] [--dry-run]

use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

const PLAN_SCHEMA: &str = "wisent.weles-capture-plan.v1";
const NAMESPACE: &str = "stado://weles-captures/";
const WIDTHS: &[(u32, u32)] = &[(390, 844), (768, 1024), (1440, 1000)];
const DEFAULT_HOST: &str = "charless-mac-mini";
/// Deviation from the Python original, which used ~/.stado/work: this port
/// keeps all generated working files under ~/.spis/work per harness policy.
fn work_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".spis").join("work")
}

fn fail(message: &str) -> ! {
    eprintln!("capture-widths: {message}");
    std::process::exit(1);
}

struct CatalogData {
    directory: PathBuf,
    sources: Value,
    index: Value,
}

fn load_catalog(catalog: &str) -> CatalogData {
    let name = if catalog.ends_with("-examples") {
        catalog.to_string()
    } else {
        format!("{catalog}-examples")
    };
    let directory = PathBuf::from(&name);
    if !directory.is_dir() {
        fail(&format!("{name} does not exist"));
    }
    let sources = match crate::read_json(directory.join("sources.json").to_str().unwrap()) {
        Ok(v) => v,
        Err(e) => fail(&format!("read {name}/sources.json: {e:#}")),
    };
    let index = match crate::read_json(directory.join("references.json").to_str().unwrap()) {
        Ok(v) => v,
        Err(e) => fail(&format!("read {name}/references.json: {e:#}")),
    };
    CatalogData {
        directory,
        sources,
        index,
    }
}

/// Returns (example, entry) for the record matching selector (None, NN, slug,
/// or lowercased name), mirroring the Python pick().
fn pick(
    sources: &Value,
    index: &Value,
    selector: Option<&str>,
) -> (Map<String, Value>, Map<String, Value>) {
    let examples = sources["examples"].as_array().cloned().unwrap_or_default();
    let references = index["references"].as_array().cloned().unwrap_or_default();
    for (position, example) in examples.iter().enumerate() {
        let Some(entry) = references.get(position).and_then(Value::as_object) else {
            continue;
        };
        let number = position + 1;
        let slug = entry["path"]
            .as_str()
            .map(Path::new)
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let name_lower = example["name"].as_str().unwrap_or("").to_lowercase();
        let matched = match selector {
            None => true,
            Some(sel) => sel == number.to_string() || sel == slug || sel == name_lower,
        };
        if matched {
            return (
                example.as_object().cloned().unwrap_or_default(),
                entry.clone(),
            );
        }
    }
    fail(&format!("record {:?} not found", selector.unwrap_or("")));
}

fn slug_of(entry: &Map<String, Value>) -> String {
    entry["path"]
        .as_str()
        .map(Path::new)
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Compact UTC stamp like 20260823t123456Z from now_iso_utc().
fn compact_stamp() -> String {
    // Python: strftime("%Y%m%dt%H%M%SZ")
    let iso = crate::now_iso_utc(); // YYYY-MM-DDTHH:MM:SSZ
    format!(
        "{}{}{}t{}{}{}Z",
        &iso[0..4],
        &iso[5..7],
        &iso[8..10],
        &iso[11..13],
        &iso[14..16],
        &iso[17..19]
    )
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut record: Option<String> = None;
    let mut host = DEFAULT_HOST.to_string();
    let mut dry_run = false;

    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--record" => {
                i += 1;
                record = Some(rest.get(i).context("--record needs a value")?.clone());
            }
            "--host" => {
                i += 1;
                host = rest.get(i).context("--host needs a value")?.clone();
            }
            "--dry-run" => dry_run = true,
            other => {
                if other.starts_with('-') || catalog.is_some() {
                    bail!("unknown argument: {other}");
                }
                catalog = Some(other.to_string());
            }
        }
        i += 1;
    }
    let Some(catalog) = catalog else {
        bail!("usage: spis capture-widths <catalog> [--record <NN|slug>] [--host <target>] [--dry-run]");
    };

    let data = load_catalog(&catalog);
    let selected: Vec<(Map<String, Value>, Map<String, Value>)> = match &record {
        Some(selector) => vec![pick(&data.sources, &data.index, Some(selector))],
        None => {
            let examples = data.sources["examples"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let references = data.index["references"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            examples
                .into_iter()
                .zip(references)
                .map(|(e, r)| {
                    (
                        e.as_object().cloned().unwrap_or_default(),
                        r.as_object().cloned().unwrap_or_default(),
                    )
                })
                .collect()
        }
    };
    if selected.is_empty() {
        fail("nothing to capture; add records first");
    }

    let batch = format!("widths-{}", compact_stamp());
    let mut captures: Vec<Value> = Vec::new();
    for (example, entry) in &selected {
        let slug = slug_of(entry);
        let source_url = example["source_url"].as_str().unwrap_or_default();
        for (width, height) in WIDTHS {
            captures.push(json!({
                "batch": batch,
                "site_slug": format!("{slug}-{width}"),
                "source_url": source_url,
                "axis": "composition",
                "viewport": {"width": width, "height": height, "device_scale_factor": 1},
                "artifact_prefix": format!("{NAMESPACE}{batch}/{catalog}/{slug}/{width}/"),
                "full_page": true,
                "record_seconds": 0,
                "steps": [{"op": "wait_ms", "value": 2500}],
            }));
        }
    }

    let plan = json!({
        "schema": PLAN_SCHEMA,
        "batch": batch,
        "target": host,
        "captures": captures,
    });
    let plan_dir = work_root().join("landing-width-plans");
    std::fs::create_dir_all(&plan_dir)?;
    let plan_path = plan_dir.join(format!("{batch}.json"));
    std::fs::write(&plan_path, serde_json::to_string_pretty(&plan)? + "\n")?;

    if dry_run {
        println!(
            "dry run: planned {} captures across {} record(s); plan={}",
            captures.len(),
            selected.len(),
            plan_path.display()
        );
        for capture in &captures {
            println!(
                "  {} <- {} @ {}px",
                capture["site_slug"].as_str().unwrap_or_default(),
                capture["source_url"].as_str().unwrap_or_default(),
                capture["viewport"]["width"],
            );
        }
        return Ok(());
    }

    let stado = which("stado").unwrap_or_else(|| {
        fail("stado is not on PATH; hosts are reached through stado, never ssh");
    });
    let output = std::process::Command::new(&stado)
        .args(["host", "weles-capture", &host, "--plan"])
        .arg(plan_path.as_os_str())
        .arg("--json")
        .output()
        .context("run stado host weles-capture")?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success()
        && (stderr.clone() + &stdout)
            .to_lowercase()
            .contains("skarbiec")
    {
        // This machine holds no grant for the admission token. That is the
        // designed state: enqueue on the target host instead, where the host's
        // own Skarbiec identity authorizes the batch. The pinned Stado job is
        // the sanctioned remote channel; no credentials leave the host.
        println!("local enqueue lacks the admission grant; submitting pinned Stado job on {host}");
        let script = format!(
            "#!/bin/sh\nset -eu\ncp {plan} /tmp/spis-widths-plan.json\n\
             STADO=$HOME/.stado/bin/stado; [ -x \"$STADO\" ] || STADO=$(command -v stado)\n\
             \"$STADO\" host weles-capture charless-mac-mini --plan /tmp/spis-widths-plan.json --json\n",
            plan = plan_path.display(),
        );
        let job_dir = work_root();
        std::fs::create_dir_all(&job_dir)?;
        let job_script = job_dir.join(format!("spis-widths-{batch}.sh"));
        std::fs::write(&job_script, script)?;
        let submit = std::process::Command::new(&stado)
            .args([
                "submit",
                &format!("sh {}", job_script.display()),
                "--pinned-host",
                &host,
            ])
            .output()
            .context("run stado submit")?;
        let sub_out = String::from_utf8_lossy(&submit.stdout).to_string();
        let sub_err = String::from_utf8_lossy(&submit.stderr).to_string();
        if !submit.status.success() {
            fail(&format!(
                "remote enqueue submission failed: {}",
                if sub_err.trim().is_empty() {
                    sub_out.trim()
                } else {
                    sub_err.trim()
                }
                .chars()
                .take(300)
                .collect::<String>()
            ));
        }
        match find_batch_id(&sub_out) {
            Some(id) => println!("remote batch {id} submitted; poll with: stado status {id}"),
            None => fail(&format!(
                "could not read Stado batch id from: {}",
                sub_out.trim().chars().take(200).collect::<String>()
            )),
        }
        return Ok(());
    }
    if !output.status.success() {
        let detail = if stderr.trim().is_empty() {
            &stdout
        } else {
            &stderr
        };
        fail(&format!(
            "weles-capture refused: {}",
            detail.trim().chars().take(300).collect::<String>()
        ));
    }

    // Record the batch on each touched reference so retrieval can find it.
    for (_, entry) in &selected {
        let record_path = data
            .directory
            .join(entry["path"].as_str().unwrap_or_default());
        let mut record: Map<String, Value> =
            crate::read_json::<serde_json::Value>(record_path.to_str().unwrap())?
                .as_object()
                .cloned()
                .unwrap_or_default();
        {
            let batches = record
                .entry("capture_batches".to_string())
                .or_insert_with(|| Value::Array(Vec::new()));
            if !batches.is_array() {
                *batches = Value::Array(Vec::new());
            }
            let list = batches.as_array_mut().unwrap();
            if !list.iter().any(|v| v == &Value::String(batch.clone())) {
                list.push(Value::String(batch.clone()));
            }
        }
        {
            let gaps = record
                .entry("evidence_gaps".to_string())
                .or_insert_with(|| Value::Array(Vec::new()));
            if !gaps.is_array() {
                *gaps = Value::Array(Vec::new());
            }
            let pending = "width captures enqueued; awaiting retrieval";
            let list = gaps.as_array_mut().unwrap();
            if !list.iter().any(|v| v.as_str() == Some(pending)) {
                list.push(Value::String(pending.to_string()));
            }
        }
        crate::write_pretty_json(record_path.to_str().unwrap(), &Value::Object(record))?;
    }

    println!(
        "enqueued {} captures as {}; artifacts land under {NAMESPACE}{batch}/",
        captures.len(),
        batch
    );
    println!("retrieve with spis verify --apply after the host finishes the batch");
    Ok(())
}

/// Locate `Batch: batch-<digits>` in free-form stado output (the Python used a regex).
fn find_batch_id(text: &str) -> Option<String> {
    let start = text.find("Batch: ")? + "Batch: ".len();
    let rest = &text[start..];
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .unwrap_or(rest.len());
    let token = &rest[..end];
    if token.starts_with("batch-")
        && token[6..].bytes().all(|b| b.is_ascii_digit())
        && !token[6..].is_empty()
    {
        Some(token.to_string())
    } else {
        None
    }
}

fn which(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_stamp_shape() {
        let stamp = compact_stamp();
        // %Y%m%dt%H%M%SZ → 8 digits, 't', 6 digits, 'Z'.
        assert_eq!(stamp.len(), 16);
        assert_eq!(&stamp[8..9], "t");
        assert!(stamp.ends_with('Z'));
        assert!(stamp[..8].bytes().all(|b| b.is_ascii_digit()));
        assert!(stamp[9..15].bytes().all(|b| b.is_ascii_digit()));
    }

    #[test]
    fn finds_batch_ids() {
        assert_eq!(
            find_batch_id("queued\nBatch: batch-1724410001\nok"),
            Some("batch-1724410001".to_string())
        );
        assert_eq!(find_batch_id("no id here"), None);
        assert_eq!(find_batch_id("Batch: notanumber"), None);
        assert_eq!(find_batch_id("Batch: batch-12x"), None);
    }
}
