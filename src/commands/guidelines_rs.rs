//! `spis guidelines` — derive a DRAFT guidelines document from a catalog's
//! measured records.
//!
//! Every statement in the draft is an aggregate over records and carries its
//! own count (n/m). Nothing is invented: a pattern only appears if the records
//! show it, and a family with no measured records produces no guidelines.
//! The output is a draft for human review — it becomes guidelines only after
//! a human edits, confirms, and moves it into product-guidelines.
//!
//! Rust port of the former `guidelines.py`.

use crate as lib;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fail(message: &str) -> ! {
    eprintln!("guidelines: {message}");
    std::process::exit(1);
}

/// First-seen-insertion-order counter, matching Python's `collections.Counter`.
struct Counter {
    pairs: Vec<(String, usize)>,
}

impl Counter {
    fn new() -> Self {
        Counter { pairs: Vec::new() }
    }

    fn add(&mut self, name: &str) {
        if let Some((_, count)) = self.pairs.iter_mut().find(|(k, _)| k == name) {
            *count += 1;
            return;
        }
        self.pairs.push((name.to_string(), 1));
    }

    /// Pairs sorted by descending count; stable, so ties stay in first-seen order.
    fn sorted(&self) -> Vec<(String, usize)> {
        let mut pairs = self.pairs.clone();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs
    }
}

/// Today's date as YYYY-MM-DD (UTC civil date from the Unix clock).
fn today_iso_date() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_secs();
    let days = (secs / 86_400) as i64;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    format!("{y:04}-{m:02}-{d:02}")
}

fn load_records(catalog: &str) -> (Value, Vec<Value>) {
    let slug = if catalog.ends_with("-examples") {
        catalog.to_string()
    } else {
        format!("{catalog}-examples")
    };
    let directory = PathBuf::from(&slug);
    let sources_path = directory.join("sources.json");
    if !sources_path.is_file() {
        fail(&format!("{} is not a managed catalog", slug));
    }
    let sources: Value = match lib::read_json(sources_path.to_str().unwrap()) {
        Ok(v) => v,
        Err(e) => fail(&format!("{e:#}")),
    };
    let mut records = Vec::new();
    if let Ok(index) = lib::read_json::<Value>(directory.join("references.json").to_str().unwrap())
    {
        for entry in index
            .get("references")
            .and_then(Value::as_array)
            .map(|a| a.as_slice())
            .unwrap_or(&[])
        {
            let Some(path) = entry.get("path").and_then(Value::as_str) else {
                continue;
            };
            let record_path = directory.join(path);
            if record_path.is_file() {
                if let Ok(record) = lib::read_json::<Value>(record_path.to_str().unwrap()) {
                    records.push(record);
                }
            }
        }
    }
    (sources, records)
}

fn counter_block(title: &str, pairs: &[(String, usize)], total: usize, lines: &mut Vec<String>) {
    if pairs.is_empty() {
        return;
    }
    lines.push(format!("### {title}"));
    lines.push(String::new());
    for (name, count) in pairs {
        lines.push(format!("- {name} — {count}/{total} records"));
    }
    lines.push(String::new());
}

fn truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) | Some(Value::Bool(false)) => false,
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        Some(Value::Bool(true)) => true,
    }
}
pub fn run(rest: &[String]) -> Result<()> {
    let mut catalog: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--out" => {
                i += 1;
                out = Some(PathBuf::from(rest.get(i).context("--out needs a value")?));
            }
            "--help" | "-h" => {
                println!("usage: spis guidelines <catalog> [--out <file>]");
                return Ok(());
            }
            other => {
                if other.starts_with('-') {
                    bail!("unknown argument: {other}");
                }
                if catalog.is_some() {
                    bail!("unexpected extra argument: {other}");
                }
                catalog = Some(other.to_string());
            }
        }
        i += 1;
    }
    let Some(catalog) = catalog else {
        bail!("usage: spis guidelines <catalog> [--out <file>]");
    };

    let (sources, records) = load_records(&catalog);
    let total = records.len();
    if total == 0 {
        fail(&format!(
            "{catalog} has no measured records; guidelines require evidence"
        ));
    }

    let complete = records
        .iter()
        .filter(|r| r.get("evidence_status").and_then(Value::as_str) == Some("complete"))
        .count();

    let mut provenance = Counter::new();
    for r in &records {
        if let Some(classes) = r.get("motion_provenance").and_then(Value::as_array) {
            for cls in classes {
                if let Some(name) = cls.as_str() {
                    provenance.add(name);
                }
            }
        }
    }

    let mut interactions = Counter::new();
    for r in &records {
        if let Some(items) = r.get("interactions").and_then(Value::as_array) {
            for item in items {
                let name = item
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("unnamed");
                interactions.add(name);
            }
        }
    }

    let mut timing = Counter::new();
    for r in &records {
        if let Some(entries) = r.get("motion_analysis").and_then(Value::as_array) {
            for m in entries {
                let name = m
                    .get("timing_class")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .unwrap_or("unspecified");
                timing.add(name);
            }
        }
    }

    let accessibility_measured = records
        .iter()
        .filter(|r| truthy(r.get("accessibility").and_then(|a| a.get("measured"))))
        .count();

    let mut categories = Counter::new();
    for e in sources
        .get("examples")
        .and_then(Value::as_array)
        .map(|a| a.as_slice())
        .unwrap_or(&[])
    {
        let category = e
            .get("category")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or("uncategorized");
        categories.add(category);
    }

    let mut gap_counter = Counter::new();
    for r in &records {
        if let Some(gaps) = r.get("evidence_gaps").and_then(Value::as_array) {
            for g in gaps {
                if let Some(name) = g.as_str() {
                    gap_counter.add(name);
                }
            }
        }
    }

    let title = sources
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(&catalog);
    let mut lines = vec![
        format!("# {title} — derived guidelines (DRAFT)"),
        String::new(),
        format!(
            "Machine-derived from `{catalog}` on {}. Every line cites its record count; \
             a line without a count is not from this corpus.",
            today_iso_date()
        ),
        String::new(),
        "**This is a DRAFT.** It becomes guidelines only after a human reviews it, \
         edits it, and moves the confirmed rules into product-guidelines. Counts \
         below quote only what the records measure; the corpus does not score taste."
            .to_string(),
        String::new(),
        "## Coverage".to_string(),
        String::new(),
        format!(
            "- records: {total} ({complete} complete, {} partial)",
            total - complete
        ),
        format!("- accessibility measured on the product: {accessibility_measured}/{total}"),
        String::new(),
    ];
    counter_block("Record categories", &categories.sorted(), total, &mut lines);
    counter_block("Motion provenance", &provenance.sorted(), total, &mut lines);
    counter_block(
        "Observed interactions (how often each appeared)",
        &interactions.sorted(),
        total,
        &mut lines,
    );
    counter_block("Motion timing classes", &timing.sorted(), total, &mut lines);
    counter_block(
        "Named evidence gaps across records",
        &gap_counter.sorted(),
        total,
        &mut lines,
    );
    lines.extend([
        "## Review checklist".to_string(),
        String::new(),
        "- [ ] every rule kept above still cites a count I accept".to_string(),
        "- [ ] rules I reject are deleted here before promotion".to_string(),
        "- [ ] promoted copy lands in product-guidelines with this file cited as source"
            .to_string(),
        String::new(),
    ]);

    let out = out.unwrap_or_else(|| Path::new(&catalog).join("guidelines-draft.md"));
    std::fs::write(&out, lines.join("\n")).with_context(|| format!("write {}", out.display()))?;
    println!("wrote {}", out.display());
    Ok(())
}
