//! `spis analyze-readme-examples` — measure recurring structural patterns in
//! the curated README snapshots under readme-examples/.
//!
//! Writes readme-examples/analysis.json and prints the summary block that the
//! former `analyze-readme-examples.py` printed (pretty JSON of source_count,
//! length, prevalence).

use crate as lib;
use anyhow::{bail, Result};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::path::Path;

const EXAMPLES: &str = "readme-examples";
const OUTPUT: &str = "readme-examples/analysis.json";

fn section_patterns() -> Vec<(&'static str, String)> {
    vec![
        (
            "installation_or_quick_start",
            r"(?im)^.{0,8}(install|installation|setup|getting started|quick ?start|get started)"
                .into(),
        ),
        (
            "usage_or_examples",
            r"(?im)^.{0,8}(usage|how to use|examples?|tutorial)".into(),
        ),
        (
            "features_or_capabilities",
            r"(?im)^.{0,8}(features?|capabilities|what .* does)".into(),
        ),
        (
            "documentation_links",
            r"(?im)^.{0,8}(documentation|docs|learn more)".into(),
        ),
        (
            "contribution_guidance",
            r"(?im)^.{0,8}(contribut|development)".into(),
        ),
        ("license_section", r"(?im)^.{0,8}(licen[cs]e)".into()),
        (
            "security_guidance",
            r"(?im)^.{0,8}(security|vulnerabilit)".into(),
        ),
        (
            "support_or_community",
            r"(?im)^.{0,8}(support|help|community|getting help)".into(),
        ),
        (
            "architecture_or_how_it_works",
            r"(?im)^.{0,8}(architecture|how it works|design|internals)".into(),
        ),
        (
            "status_or_roadmap",
            r"(?im)^.{0,8}(status|roadmap|maturity|stability)".into(),
        ),
        (
            "requirements_or_prerequisites",
            r"(?im)^.{0,8}(requirements?|prerequisites?|compatibility)".into(),
        ),
        (
            "alternatives_or_comparison",
            r"(?im)^.{0,8}(alternatives?|comparison|why )".into(),
        ),
    ]
}

/// Python's round(): halves go to the nearest even value.
fn py_round(value: f64, digits: i32) -> f64 {
    let factor = 10f64.powi(digits);
    let scaled = value * factor;
    let floor = scaled.floor();
    let fract = scaled - floor;
    let rounded = if (fract - 0.5).abs() < 1e-9 {
        if floor % 2.0 == 0.0 {
            floor
        } else {
            floor + 1.0
        }
    } else {
        scaled.round()
    };
    rounded / factor
}

/// Python format `{x:.0%}`: percentage with round-half-to-even at 0 decimals.
fn py_share(count: usize, total: usize) -> String {
    let pct = count as f64 / total as f64 * 100.0;
    format!("{}%", py_round(pct, 0))
}

/// statistics.median: middle value (int-shaped) when odd, mean when even.
fn median(values: &[usize]) -> Value {
    let mut ordered = values.to_vec();
    ordered.sort_unstable();
    let n = ordered.len();
    match n {
        0 => Value::Null,
        n if n % 2 == 1 => json!(ordered[n / 2]),
        _ => {
            let mid = (ordered[n / 2 - 1] + ordered[n / 2]) as f64 / 2.0;
            json!(mid)
        }
    }
}

fn percentile(values: &[usize], fraction: f64) -> f64 {
    let mut ordered = values.to_vec();
    ordered.sort_unstable();
    let position = (ordered.len() - 1) as f64 * fraction;
    let lower = position as usize;
    let upper = std::cmp::min(lower + 1, ordered.len() - 1);
    py_round(
        ordered[lower] as f64
            + (ordered[upper] - ordered[lower]) as f64 * (position - lower as f64),
        2,
    )
}

fn headings(text: &str, markdown_re: &Regex, underline_re: &Regex) -> Vec<String> {
    let mut found: Vec<String> = markdown_re
        .captures_iter(text)
        .map(|c| c["title"].trim().to_string())
        .collect();
    // reStructuredText titles: a non-empty line followed by an underline of >=3
    // punctuation characters (= - ~ ^ ` : # * +).
    let lines: Vec<&str> = text.lines().collect();
    for pair in lines.windows(2) {
        let (title, underline) = (pair[0], pair[1]);
        if !title.trim().is_empty() && underline_re.is_match(underline.trim()) {
            found.push(title.trim().to_string());
        }
    }
    found
}

struct Patterns {
    markdown_heading: Regex,
    rst_underline: Regex,
    badge: Regex,
    visual: Regex,
    animated_gif: Regex,
    video: Regex,
    code_examples: Regex,
    code_fence_line: Regex,
    mermaid: Regex,
    table: Regex,
    toc: Regex,
    sections: Vec<(&'static str, Regex)>,
}

impl Patterns {
    fn compile() -> Patterns {
        Patterns {
            markdown_heading: Regex::new(r"(?m)^#{1,6}\s+(?P<title>.+?)\s*$").unwrap(),
            rst_underline: Regex::new(r"^[=\-~^`:#*+]{3,}$").unwrap(),
            badge: Regex::new(
                r#"(?i)shields\.io|badge\.svg|actions/workflows|badge\.fury|badgen\.net|/badge/"#,
            )
            .unwrap(),
            visual: Regex::new(r#"<img\b[^>]*>|!\[[^]]*\]\([^)]+\)|\.\.\s+image::\s*\S+"#).unwrap(),
            // Python `$` also matches just before a trailing newline; the extra
            // `\n\z` alternative reproduces that here exactly.
            animated_gif: Regex::new(r#"(?i)\.gif(?:[?#)"'\s]|\z|\n\z)"#).unwrap(),
            video: Regex::new(r"(?i)<video\b").unwrap(),
            code_examples: Regex::new(r"(?i)```|\.\.\s+(code-block|sourcecode)::").unwrap(),
            code_fence_line: Regex::new(r"(?m)^```").unwrap(),
            mermaid: Regex::new(r"(?im)^```mermaid").unwrap(),
            table: Regex::new(r"(?m)^\s*\|?(?:\s*:?-{3,}:?\s*\|)+\s*:?-{3,}:?\s*\|?\s*$").unwrap(),
            toc: Regex::new(r"(?im)^.{0,8}(table of contents|contents)\s*$").unwrap(),
            sections: section_patterns()
                .into_iter()
                .map(|(name, pattern)| (name, Regex::new(&pattern).unwrap()))
                .collect(),
        }
    }

    /// Count matches of `\b\w+\b`.
    fn word_count(text: &str) -> usize {
        // Python's \w is str.isalnum() or underscore: letters and numbers but
        // NOT combining marks (\p{M}), which Rust's \w includes. Maximal runs
        // of this class equal Python findall(r"\b\w+\b").
        static WORDS: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new(r"[\p{L}\p{N}_]+").unwrap());
        WORDS.find_iter(text).count()
    }

    fn inspect(&self, path: &Path) -> Result<Value> {
        let text = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = text.lines().collect();
        let found_headings = headings(&text, &self.markdown_heading, &self.rst_underline);
        let has_visual = self.visual.is_match(&text);

        let mut object = Map::new();
        object.insert(
            "file".into(),
            json!(path.file_name().unwrap().to_string_lossy()),
        );
        object.insert("lines".into(), json!(lines.len()));
        object.insert("words".into(), json!(Self::word_count(&text)));
        object.insert("headings".into(), json!(found_headings.len()));
        object.insert(
            "first_heading".into(),
            found_headings.first().map_or(Value::Null, |h| json!(h)),
        );
        object.insert("badges".into(), json!(self.badge.is_match(&text)));
        object.insert("visuals".into(), json!(has_visual));
        object.insert(
            "visuals_first_30_lines".into(),
            json!(self
                .visual
                .is_match(&lines[..lines.len().min(30)].join("\n"))),
        );
        object.insert(
            "animated_gif".into(),
            json!(self.animated_gif.is_match(&text)),
        );
        object.insert("video".into(), json!(self.video.is_match(&text)));
        object.insert(
            "code_examples".into(),
            json!(self.code_examples.is_match(&text)),
        );
        object.insert(
            "code_block_count".into(),
            json!(self.code_fence_line.find_iter(&text).count() / 2),
        );
        object.insert("mermaid".into(), json!(self.mermaid.is_match(&text)));
        object.insert("markdown_table".into(), json!(self.table.is_match(&text)));
        object.insert("table_of_contents".into(), json!(self.toc.is_match(&text)));
        for (name, re) in &self.sections {
            object.insert((*name).into(), json!(re.is_match(&text)));
        }
        Ok(Value::Object(object))
    }
}

pub fn run(rest: &[String]) -> Result<()> {
    for arg in rest {
        bail!("unknown argument: {arg}");
    }
    let patterns = Patterns::compile();

    let examples_dir = Path::new(EXAMPLES);
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(examples_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .map(|n| {
                        let name = n.to_string_lossy();
                        name.as_bytes().len() >= 3
                            && name.as_bytes()[0].is_ascii_digit()
                            && name.as_bytes()[1].is_ascii_digit()
                            && name.as_bytes()[2] == b'-'
                    })
                    .unwrap_or(false)
        })
        .collect();
    paths.sort();

    let records: Vec<Value> = paths
        .iter()
        .map(|p| patterns.inspect(p))
        .collect::<Result<Vec<_>>>()?;
    let total = records.len();
    if total == 0 {
        bail!("no readme-examples/[0-9][0-9]-* snapshots found");
    }

    let boolean_fields: Vec<&str> = [
        "badges",
        "visuals",
        "visuals_first_30_lines",
        "animated_gif",
        "video",
        "code_examples",
        "mermaid",
        "markdown_table",
        "table_of_contents",
    ]
    .iter()
    .copied()
    .chain(section_patterns().into_iter().map(|(name, _)| name))
    .collect();

    let mut prevalence = Map::new();
    for field in &boolean_fields {
        let count = records
            .iter()
            .filter(|r| r.get(*field).and_then(Value::as_bool) == Some(true))
            .count();
        prevalence.insert(
            (*field).to_string(),
            json!({ "count": count, "share": py_share(count, total) }),
        );
    }

    let line_values: Vec<usize> = records
        .iter()
        .filter_map(|r| r.get("lines").and_then(Value::as_u64))
        .map(|v| v as usize)
        .collect();
    let word_values: Vec<usize> = records
        .iter()
        .filter_map(|r| r.get("words").and_then(Value::as_u64))
        .map(|v| v as usize)
        .collect();
    let heading_counts: Vec<usize> = records
        .iter()
        .filter_map(|r| r.get("headings").and_then(Value::as_u64))
        .map(|v| v as usize)
        .collect();
    let code_block_counts: Vec<usize> = records
        .iter()
        .filter_map(|r| r.get("code_block_count").and_then(Value::as_u64))
        .map(|v| v as usize)
        .collect();

    let bands: [(&str, usize, usize); 4] = [
        ("compact_75_lines_or_fewer", 0, 75),
        ("standard_76_to_200_lines", 76, 200),
        ("extended_201_to_400_lines", 201, 400),
        ("manual_over_400_lines", 401, usize::MAX),
    ];
    let mut bands_json = Map::new();
    for (name, lower, upper) in bands {
        let count = line_values
            .iter()
            .filter(|&&v| lower <= v && v <= upper)
            .count();
        bands_json.insert(
            name.to_string(),
            json!({ "count": count, "share": py_share(count, total) }),
        );
    }

    let result = json!({
        "schema": "wisent.readme-example-analysis",
        "source_count": total,
        "length": {
            "median_lines": median(&line_values),
            "p25_lines": percentile(&line_values, 0.25),
            "p75_lines": percentile(&line_values, 0.75),
            "p90_lines": percentile(&line_values, 0.90),
            "median_words": median(&word_values),
            "p25_words": percentile(&word_values, 0.25),
            "p75_words": percentile(&word_values, 0.75),
            "p90_words": percentile(&word_values, 0.90),
            "shortest_lines": line_values.iter().min(),
            "longest_lines": line_values.iter().max(),
            "median_headings": median(&heading_counts),
            "median_code_blocks": median(&code_block_counts),
            "bands": Value::Object(bands_json),
        },
        "prevalence": Value::Object(prevalence),
        "files": records,
    });

    lib::write_pretty_json(OUTPUT, &result)?;
    let summary = json!({
        "source_count": result["source_count"],
        "length": result["length"],
        "prevalence": result["prevalence"],
    });
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
