//! `spis audit-reference-accessibility` — measure reference accessibility with
//! axe-core through Weles on a Stado host (port of audit-reference-accessibility.py).
//!
//! Plans a `wisent.weles-capture-plan.v1` batch of generic_accessibility_audit
//! actions, enqueues it via `stado host weles-capture`, polls status, retrieves
//! axe artifacts through `stado storage get`, validates them, installs them
//! under each record's media/accessibility/, updates reference.json, and runs
//! the `verify-reference-evidence` subcommand per completed catalog.
//!
//! Deviations from the Python original (reported, deliberate):
//! * Plan/staging directories live under ~/.spis/work instead of ~/.stado/work.
//! * The verifier is invoked as a spis subcommand (`spis
//!   verify-reference-evidence`) on the current executable rather than
//!   `python3 verify-reference-evidence.py`.
//! * The strict JSON reader is a hand-rolled parser with duplicate-key
//!   detection (serde_json alone accepts duplicates silently).

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const INDEX: &str = "accessibility-audit-index.json";

const PLAN_SCHEMA: &str = "wisent.weles-capture-plan.v1";
const INDEX_SCHEMA: &str = "wisent.accessibility-audit-index.v1";
const ACTION: &str = "generic_accessibility_audit";
const NAMESPACE: &str = "stado://weles-captures/";
const DEFAULT_TARGET: &str = "charless-mac-mini";
const DEFAULT_CATALOGS: &[&str] = &[
    "web-app-examples",
    "dashboard-console-examples",
    "documentation-site-examples",
    "design-system-examples",
    "onboarding-auth-examples",
];
const ACTION_KEYS: &[&str] = &[
    "batch",
    "site_slug",
    "source_url",
    "viewport",
    "artifact_prefix",
];
const PLAN_KEYS: &[&str] = &["schema", "batch", "target", "captures"];
const SUMMARY_FIELDS: &[&str] = &[
    "source_url",
    "viewport",
    "captured_at",
    "renderer",
    "weles_version",
    "axe_version",
    "violation_count",
    "violations",
    "passes_count",
    "incomplete_count",
    "bytes",
    "sha256",
];

/// Deviation from the Python original, which used ~/.stado/work: generated
/// working files stay under ~/.spis/work per harness policy.
fn work_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".spis").join("work")
}

fn plan_dir() -> PathBuf {
    work_root().join("accessibility-audit-plans")
}

fn staging_root() -> PathBuf {
    work_root().join("accessibility-audits")
}

// ---------------------------------------------------------------------------
// Small utilities

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

fn now() -> String {
    crate::now_iso_utc()
}

fn default_batch() -> String {
    // Python: strftime("accessibility-%Y%m%dt%H%M%Sz")
    let iso = now();
    format!(
        "accessibility-{}{}{}t{}{}{}z",
        &iso[0..4],
        &iso[5..7],
        &iso[8..10],
        &iso[11..13],
        &iso[14..16],
        &iso[17..19]
    )
}

fn pid() -> u32 {
    std::process::id()
}

/// Lexical normalization that works for not-yet-existing paths (Path::resolve
/// equivalent for our purposes).
fn lex_norm(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn is_relative_to(path: &Path, base: &Path) -> bool {
    path.starts_with(base)
}

fn basename_of(key: &str) -> String {
    key.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string()
}

fn looks_like_slug(value: &str) -> bool {
    // fullmatch r"[a-z0-9][a-z0-9._-]{0,80}"
    let mut chars = value.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    value.chars().count() <= 81
        && value.chars().skip(1).all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-'
        })
}

fn is_hex64_lower(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// ---------------------------------------------------------------------------
// Strict JSON parsing with duplicate-key detection

struct StrictParser<'a> {
    text: &'a [u8],
    pos: usize,
    depth: usize,
}

impl<'a> StrictParser<'a> {
    fn new(text: &'a str) -> Self {
        StrictParser {
            text: text.as_bytes(),
            pos: 0,
            depth: 0,
        }
    }

    fn err(&self, what: &str) -> String {
        format!("{what} at byte {}", self.pos)
    }

    fn skip_ws(&mut self) {
        while self.pos < self.text.len()
            && matches!(self.text[self.pos], b' ' | b'\t' | b'\n' | b'\r')
        {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.text.get(self.pos).copied()
    }

    fn expect(&mut self, byte: u8, what: &str) -> Result<(), String> {
        match self.peek() {
            Some(b) if b == byte => {
                self.pos += 1;
                Ok(())
            }
            _ => Err(self.err(what)),
        }
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        self.depth += 1;
        if self.depth > 200 {
            return Err("JSON nesting deeper than 200 levels".to_string());
        }
        let value = match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b't') => self.parse_literal("true", Value::Bool(true)),
            Some(b'f') => self.parse_literal("false", Value::Bool(false)),
            Some(b'n') => self.parse_literal("null", Value::Null),
            Some(_) => self.parse_number(),
            None => Err(self.err("unexpected end of input")),
        };
        self.depth -= 1;
        value
    }

    fn parse_literal(&mut self, word: &str, value: Value) -> Result<Value, String> {
        if self.text[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(value)
        } else {
            Err(self.err("invalid literal"))
        }
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        while self.pos < self.text.len()
            && matches!(
                self.text[self.pos],
                b'-' | b'+' | b'.' | b'0'..=b'9' | b'e' | b'E'
            )
        {
            self.pos += 1;
        }
        let token = std::str::from_utf8(&self.text[start..self.pos])
            .map_err(|_| self.err("invalid number"))?;
        if token.is_empty() {
            return Err(self.err("invalid value"));
        }
        if let Ok(i) = token.parse::<i64>() {
            return Ok(Value::Number(i.into()));
        }
        token
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| self.err("invalid number"))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect(b'"', "expected string")?;
        let mut out = String::new();
        loop {
            let Some(&byte) = self.text.get(self.pos) else {
                return Err(self.err("unterminated string"));
            };
            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let esc = *self
                        .text
                        .get(self.pos)
                        .ok_or_else(|| self.err("unterminated escape"))?;
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let cp = self.parse_hex4()?;
                            if (0xD800..0xDC00).contains(&cp) {
                                // High surrogate: require a following \uXXXX low surrogate.
                                if self.text.get(self.pos) == Some(&b'\\')
                                    && self.text.get(self.pos + 1) == Some(&b'u')
                                {
                                    self.pos += 2;
                                    let low = self.parse_hex4()?;
                                    if !(0xDC00..0xE000).contains(&low) {
                                        return Err(self.err("invalid low surrogate"));
                                    }
                                    let combined = 0x10000 + ((cp - 0xD800) << 10) + (low - 0xDC00);
                                    out.push(
                                        char::from_u32(combined)
                                            .ok_or_else(|| self.err("invalid surrogate pair"))?,
                                    );
                                } else {
                                    return Err(self.err("lone high surrogate"));
                                }
                            } else if (0xDC00..0xE000).contains(&cp) {
                                return Err(self.err("lone low surrogate"));
                            } else {
                                out.push(
                                    char::from_u32(cp)
                                        .ok_or_else(|| self.err("invalid \\u escape"))?,
                                );
                            }
                        }
                        _ => return Err(self.err("invalid escape")),
                    }
                }
                0x00..=0x1F => return Err(self.err("control character in string")),
                _ => {
                    // Copy one UTF-8 encoded scalar.
                    let remaining = &self.text[self.pos..];
                    let s = std::str::from_utf8(remaining)
                        .map_err(|_| self.err("invalid UTF-8 in string"))?;
                    let ch = s.chars().next().ok_or_else(|| self.err("empty string"))?;
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u32, String> {
        let slice = self
            .text
            .get(self.pos..self.pos + 4)
            .ok_or_else(|| self.err("truncated \\u escape"))?;
        let hex = std::str::from_utf8(slice).map_err(|_| self.err("bad \\u escape"))?;
        let cp = u32::from_str_radix(hex, 16).map_err(|_| self.err("bad \\u escape"))?;
        self.pos += 4;
        Ok(cp)
    }

    fn parse_object(&mut self) -> Result<Value, String> {
        self.expect(b'{', "expected object")?;
        let mut map = Map::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Object(map));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.expect(b':', "expected ':' after key")?;
            let value = self.parse_value()?;
            if map.contains_key(&key) {
                return Err(format!("duplicate key {key:?}"));
            }
            map.insert(key, value);
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Value::Object(map));
                }
                _ => return Err(self.err("expected ',' or '}'")),
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value, String> {
        self.expect(b'[', "expected array")?;
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Value::Array(items));
                }
                _ => return Err(self.err("expected ',' or ']'")),
            }
        }
    }
}

fn strict_json(text: &str, what: &str) -> Result<Value> {
    let mut parser = StrictParser::new(text);
    let value = parser
        .parse_value()
        .map_err(|detail| anyhow!("{what}: not readable JSON: {detail}"))?;
    parser.skip_ws();
    if parser.pos != text.len() {
        bail!("{what}: trailing characters after JSON document");
    }
    Ok(value)
}

// ---------------------------------------------------------------------------
// Row helpers

fn pick<'a>(row: &'a Value, names: &[&str]) -> Option<&'a Value> {
    for name in names {
        if let Some(value) = row.get(*name) {
            if !value.is_null() {
                return Some(value);
            }
        }
    }
    None
}

fn action_rows(payload: &Value, what: &str) -> Result<Vec<Value>> {
    let rows_from = |list: &Value| -> Option<Vec<Value>> {
        list.as_array().map(|items| {
            items
                .iter()
                .filter(|item| item.is_object())
                .cloned()
                .collect()
        })
    };
    if let Some(rows) = rows_from(payload) {
        return Ok(rows);
    }
    if let Some(map) = payload.as_object() {
        for key in ["actions", "jobs", "captures", "items", "results"] {
            if let Some(value) = map.get(key) {
                if let Some(rows) = rows_from(value) {
                    return Ok(rows);
                }
            }
        }
    }
    bail!("{what}: no per-action list in the response")
}

fn action_id(row: &Value) -> Option<String> {
    pick(row, &["action_id", "actionId", "id", "job_id", "jobId"]).map(|v| v.to_string())
}

fn canonical_url(value: &Value, record_id: &str, field: &str) -> Result<String> {
    let raw = value
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow!("{record_id}: {field}: expected a non-empty http(s) URL"))?;
    let (scheme, rest) = raw
        .split_once("://")
        .ok_or_else(|| anyhow!("{record_id}: {field}: expected an http(s) URL, got {raw:?}"))?;
    if scheme != "http" && scheme != "https" {
        bail!("{record_id}: {field}: expected an http(s) URL, got {raw:?}");
    }
    let netloc_end = rest
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(rest.len());
    let netloc = &rest[..netloc_end];
    if netloc.is_empty() {
        bail!("{record_id}: {field}: expected an http(s) URL, got {raw:?}");
    }
    let tail = &rest[netloc_end..];
    let (before_fragment, fragment) = match tail.split_once('#') {
        Some((head, frag)) => (head, Some(frag)),
        None => (tail, None),
    };
    let (path, query) = match before_fragment.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (before_fragment, None),
    };
    let path = if path.is_empty() { "/" } else { path };
    let mut out = format!("{scheme}://{netloc}{path}");
    if let Some(query) = query {
        out.push('?');
        out.push_str(query);
    }
    if let Some(fragment) = fragment {
        out.push('#');
        out.push_str(fragment);
    }
    Ok(out)
}

fn parse_record_selection(raw: Option<&String>) -> Result<Option<HashSet<usize>>> {
    let Some(raw) = raw else { return Ok(None) };
    let mut selected: HashSet<usize> = HashSet::new();
    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            bail!("--records: empty item in the comma-separated selection");
        }
        let invalid = || anyhow!("--records: {token:?} is not a positive record number or range");
        let (first_s, last_s) = match token.split_once('-') {
            Some((a, b)) => (a, Some(b)),
            None => (token, None),
        };
        let parse_num = |value: &str| -> Result<usize> {
            if value.is_empty()
                || !value.bytes().all(|b| b.is_ascii_digit())
                || value.starts_with('0')
            {
                return Err(invalid());
            }
            value.parse::<usize>().map_err(|_| invalid())
        };
        let first = parse_num(first_s)?;
        let last = match last_s {
            Some(value) => parse_num(value)?,
            None => first,
        };
        if last < first {
            bail!("--records: descending range {token:?} is not allowed");
        }
        for number in first..=last {
            selected.insert(number);
        }
    }
    Ok(Some(selected))
}

fn normalize_catalog(value: &str) -> Result<String> {
    let mut catalog = value.trim().to_string();
    if catalog.is_empty() {
        bail!("--catalog: catalog name must not be empty");
    }
    if !catalog.ends_with("-examples") {
        catalog.push_str("-examples");
    }
    // fullmatch r"[a-z0-9][a-z0-9.-]*-examples"
    let stem = catalog.strip_suffix("-examples").unwrap_or(&catalog);
    let valid = !stem.is_empty()
        && stem
            .chars()
            .enumerate()
            .all(|(i, c)| c.is_ascii_lowercase() || c.is_ascii_digit() || (i > 0 && c == '-'))
        && stem.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit());
    if !valid {
        bail!("--catalog: invalid catalog name {value:?}");
    }
    Ok(catalog)
}

// ---------------------------------------------------------------------------
// References

#[derive(Clone)]
struct Reference {
    catalog: String,
    index: usize,
    name: String,
    slug: String,
    path: PathBuf,
    source_url: String,
}

impl Reference {
    fn id(&self) -> String {
        format!("{}/{}", self.catalog, self.slug)
    }

    fn action(&self, batch: &str) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert("batch".into(), Value::String(batch.to_string()));
        map.insert("site_slug".into(), Value::String(self.slug.clone()));
        map.insert("source_url".into(), Value::String(self.source_url.clone()));
        map.insert(
            "viewport".into(),
            serde_json::json!({"width": 1440, "height": 1000, "device_scale_factor": 1}),
        );
        map.insert(
            "artifact_prefix".into(),
            Value::String(format!(
                "{NAMESPACE}{batch}/{}/{}/accessibility/",
                self.catalog, self.slug
            )),
        );
        map
    }
}

fn load_references(
    catalogs: &[String],
    selection: Option<&HashSet<usize>>,
) -> Result<Vec<Reference>> {
    let mut references: Vec<Reference> = Vec::new();
    for catalog in catalogs {
        let catalog_dir = PathBuf::from(catalog);
        let catalog_path = catalog_dir.join("references.json");
        if !catalog_path.is_file() {
            bail!("{catalog}: references.json: catalog does not exist");
        }
        let text = std::fs::read_to_string(&catalog_path)
            .with_context(|| format!("{catalog}: references.json: unreadable"))?;
        let payload = strict_json(&text, &format!("{catalog}: references.json"))?;
        let pointers = payload
            .get("references")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("{catalog}: references.json: references must be an array"))?;
        let mut known: HashSet<usize> = HashSet::new();
        for (position, pointer) in pointers.iter().enumerate() {
            let position = position + 1;
            let pointer_id = format!("{catalog}/record-{position}");
            let pointer = pointer
                .as_object()
                .ok_or_else(|| anyhow!("{pointer_id}: catalog entry: expected an object"))?;
            let index = pointer
                .get("index")
                .and_then(Value::as_u64)
                .filter(|v| (1..=usize::MAX as u64).contains(v))
                .ok_or_else(|| anyhow!("{pointer_id}: index: expected a positive integer"))?
                as usize;
            if known.contains(&index) {
                bail!("{catalog}/{index}: index: duplicate catalog record");
            }
            known.insert(index);
            if let Some(selection) = selection {
                if !selection.contains(&index) {
                    continue;
                }
            }
            let relative = pointer
                .get("path")
                .and_then(Value::as_str)
                .filter(|p| !p.is_empty())
                .ok_or_else(|| anyhow!("{catalog}/{index:02}: path: expected a reference path"))?;
            let record_id = format!("{catalog}/{index:02}");
            let joined = catalog_dir.join(relative);
            let resolved = lex_norm(&joined);
            let catalog_resolved = lex_norm(&catalog_dir);
            if !is_relative_to(&resolved, &catalog_resolved)
                || resolved.file_name().and_then(|n| n.to_str()) != Some("reference.json")
            {
                bail!("{record_id}: path: {relative:?} is outside the catalog reference layout");
            }
            if !resolved.is_file() {
                bail!("{record_id}: path: {relative:?} does not exist");
            }
            let document_text = std::fs::read_to_string(&resolved)
                .with_context(|| format!("{record_id}: reference.json: unreadable"))?;
            let document = strict_json(&document_text, &format!("{record_id}: reference.json"))?;
            if !document.is_object() {
                bail!("{record_id}: reference.json: expected an object");
            }
            let slug = resolved
                .parent()
                .and_then(Path::file_name)
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            if !looks_like_slug(&slug) {
                bail!("{record_id}: site_slug: directory name {slug:?} is not a Weles slug");
            }
            let name = document
                .get("name")
                .and_then(Value::as_str)
                .filter(|n| !n.is_empty())
                .ok_or_else(|| anyhow!("{record_id}: name: expected a non-empty string"))?
                .to_string();
            let source_field = if document.get("product_url").map(Value::is_null) == Some(false) {
                "product_url"
            } else {
                "source_url"
            };
            let source_url = canonical_url(
                document.get(source_field).unwrap_or(&Value::Null),
                &record_id,
                source_field,
            )?;
            references.push(Reference {
                catalog: catalog.clone(),
                index,
                name,
                slug,
                path: resolved,
                source_url,
            });
        }
        if let Some(selection) = selection {
            let mut missing: Vec<usize> = selection.difference(&known).copied().collect();
            missing.sort_unstable();
            if !missing.is_empty() {
                let joined = missing
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                bail!("{catalog}: --records: record(s) {joined} do not exist");
            }
        }
    }
    Ok(references)
}

// ---------------------------------------------------------------------------
// Stado plumbing

/// Match capture-landing-pages.py: preserve the first useful Stado refusal
/// rather than replacing it with the usage banner printed after it.
fn stado(args: &[&str], parse_json: bool) -> Result<(Option<Value>, String)> {
    let Some(stado_bin) = which("stado") else {
        bail!("stado is not on PATH; hosts are reached through stado, never ssh");
    };
    let output = Command::new(stado_bin)
        .args(args)
        .output()
        .with_context(|| format!("run stado {}", args.join(" ")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        let combined = if stderr.trim().is_empty() {
            &stdout
        } else {
            &stderr
        };
        let lines: Vec<&str> = combined
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        let said: Vec<&str> = lines
            .iter()
            .copied()
            .filter(|line| {
                let lower = line.to_lowercase();
                lower.starts_with("error") || lower.starts_with("warning")
            })
            .collect();
        let detail = said.first().is_some().then_some(said).unwrap_or(lines);
        let tail = if detail.is_empty() {
            format!("exit {}", output.status.code().unwrap_or(-1))
        } else {
            detail
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        };
        bail!("stado {}: {}", args.join(" "), tail);
    }
    if !parse_json {
        return Ok((None, stdout));
    }
    let value = load_json(&stdout, &format!("stado {}", args.join(" ")))?;
    Ok((Some(value), stdout))
}

fn load_json(text: &str, what: &str) -> Result<Value> {
    match serde_json::from_str::<Value>(text) {
        Ok(value) => Ok(value),
        Err(_) => {
            let start = text.find('{');
            let end = text.rfind('}');
            if let (Some(start), Some(end)) = (start, end) {
                if end > start {
                    if let Ok(value) = serde_json::from_str::<Value>(&text[start..=end]) {
                        return Ok(value);
                    }
                }
            }
            bail!(
                "{what}: expected JSON on stdout, got {:?}",
                text.trim().chars().take(200).collect::<String>()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Plan

fn validate_plan(
    document: &Value,
    expected_target: &str,
    expected: &[Reference],
) -> Result<Map<String, Value>> {
    let document = document
        .as_object()
        .ok_or_else(|| anyhow!("plan: document must be a JSON object"))?;
    let mut keys: Vec<&String> = document.keys().collect();
    keys.sort();
    let mut wanted: Vec<&str> = PLAN_KEYS.to_vec();
    wanted.sort_unstable();
    let keys_match = keys.len() == wanted.len()
        && keys
            .iter()
            .zip(wanted.iter())
            .all(|(k, w)| k.as_str() == *w);
    if !keys_match {
        bail!(
            "plan: keys must be exactly {}; got {}",
            wanted.join(", "),
            keys.iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if document.get("schema").and_then(Value::as_str) != Some(PLAN_SCHEMA) {
        bail!("plan: schema must be {PLAN_SCHEMA}");
    }
    let batch = document
        .get("batch")
        .and_then(Value::as_str)
        .filter(|b| looks_like_slug(b))
        .ok_or_else(|| anyhow!("plan: batch must be a lowercase Weles slug"))?;
    if document.get("target").and_then(Value::as_str) != Some(expected_target) {
        bail!("plan: target must be {expected_target:?}");
    }
    let captures = document
        .get("captures")
        .and_then(Value::as_array)
        .filter(|c| !c.is_empty())
        .ok_or_else(|| anyhow!("plan: captures must be a non-empty array"))?;
    if captures.len() != expected.len() {
        bail!(
            "plan: expected {} actions, got {}",
            expected.len(),
            captures.len()
        );
    }
    let mut prefixes: HashSet<String> = HashSet::new();
    for (position, (action, reference)) in captures.iter().zip(expected.iter()).enumerate() {
        let position = position + 1;
        let label = format!("{}: action {position}", reference.id());
        let action = action
            .as_object()
            .ok_or_else(|| anyhow!("{label}: expected an object"))?;
        let action_keys: HashSet<&String> = action.keys().collect();
        if action_keys.len() != ACTION_KEYS.len()
            || !ACTION_KEYS
                .iter()
                .all(|key| action_keys.contains(&key.to_string()))
        {
            bail!("{label}: keys must be exactly {}", ACTION_KEYS.join(", "));
        }
        let expected_action = reference.action(batch);
        for field in ACTION_KEYS {
            let expected_value = expected_action
                .get(*field)
                .ok_or_else(|| anyhow!("{label}: internal: missing {field}"))?;
            let got = action.get(*field).unwrap_or(&Value::Null);
            if expected_value != got {
                bail!("{label}: {field}: expected {expected_value}, got {got}");
            }
        }
        let prefix = action
            .get("artifact_prefix")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if prefixes.contains(prefix) {
            bail!("{label}: artifact_prefix: duplicate prefix {prefix:?}");
        }
        prefixes.insert(prefix.to_string());
    }
    Ok(document.clone())
}

fn atomic_json(path: &Path, payload: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_file_name(format!(".{}.{}.part", path_name(path), pid()));
    std::fs::write(&temporary, serde_json::to_string_pretty(payload)? + "\n")?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

fn path_name(path: &Path) -> &str {
    path.file_name().and_then(|n| n.to_str()).unwrap_or("file")
}

fn plan_path_for(plan_arg: Option<&String>, batch: &str) -> Result<PathBuf> {
    let raw = match plan_arg {
        Some(arg) => PathBuf::from(expand_home(arg)),
        None => plan_dir().join(format!("{batch}.json")),
    };
    let resolved = lex_norm(Path::new(&raw));
    let work = lex_norm(&work_root());
    if !is_relative_to(&resolved, &work) {
        bail!(
            "--plan: {} is outside {}; plans belong under ~/.spis/work",
            resolved.display(),
            work.display()
        );
    }
    Ok(resolved)
}

fn expand_home(input: &str) -> String {
    if input == "~" {
        return std::env::var("HOME").unwrap_or(input.to_string());
    }
    if let Some(rest) = input.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    input.to_string()
}

// ---------------------------------------------------------------------------
// Enqueue / poll / retrieve

fn enqueue(
    target: &str,
    plan_path: &Path,
    plan: &Map<String, Value>,
) -> Result<(String, Vec<String>)> {
    let plan_arg = plan_path.to_string_lossy().to_string();
    let (payload, _) = stado(
        &[
            "host",
            "weles-capture",
            target,
            "--plan",
            plan_arg.as_str(),
            "--json",
        ],
        true,
    )?;
    let payload = payload.unwrap_or(Value::Null);
    let rows = action_rows(&payload, "weles-capture")?;
    let captures = plan
        .get("captures")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if rows.len() != captures.len() {
        bail!(
            "weles-capture enqueued {} actions for a plan of {}; \
             refusing to attribute artifacts to records on a mismatched list",
            rows.len(),
            captures.len()
        );
    }
    if let Some(returned_action) = payload.get("action") {
        if returned_action.as_str() != Some(ACTION) {
            bail!("weles-capture: action: expected {ACTION}, got {returned_action}");
        }
    }
    let returned_batch = pick(&payload, &["batch", "batch_id", "id"])
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            plan.get("batch")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        })
        .trim_matches('"')
        .to_string();
    let planned_batch = plan
        .get("batch")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if returned_batch != planned_batch {
        bail!("weles-capture: batch: expected {planned_batch:?}, got {returned_batch:?}");
    }
    let mut ids: Vec<String> = Vec::new();
    for (position, (row, capture)) in rows.iter().zip(captures.iter()).enumerate() {
        let position = position + 1;
        let Some(identifier) = action_id(row) else {
            bail!("weles-capture action {position}: action_id: missing");
        };
        let identifier_trimmed = identifier.trim_matches('"').to_string();
        if let Some(site_slug) = row.get("site_slug").and_then(Value::as_str) {
            if let Some(expected_slug) = capture.get("site_slug").and_then(Value::as_str) {
                if site_slug != expected_slug {
                    bail!(
                        "weles-capture action {position}: site_slug: expected {expected_slug:?}, got {site_slug:?}"
                    );
                }
            }
        }
        if let Some(prefix) = row.get("artifact_prefix").and_then(Value::as_str) {
            if let Some(expected_prefix) = capture.get("artifact_prefix").and_then(Value::as_str) {
                if prefix != expected_prefix {
                    bail!(
                        "weles-capture action {position}: artifact_prefix: expected {expected_prefix:?}, got {prefix:?}"
                    );
                }
            }
        }
        ids.push(identifier_trimmed);
    }
    let unique = ids.iter().cloned().collect::<HashSet<_>>();
    if unique.len() != ids.len() {
        bail!("weles-capture: action_id: duplicate ids prevent record attribution");
    }
    Ok((returned_batch, ids))
}

fn state_of(row: &Value) -> String {
    pick(row, &["state", "status"])
        .map(|v| v.to_string().trim_matches('"').to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

fn poll(
    target: &str,
    batch: &str,
    expected_ids: &HashSet<String>,
    interval: u64,
    timeout_seconds: u64,
    log: &dyn Fn(&str),
) -> Result<std::collections::HashMap<String, Value>> {
    let terminal: HashSet<&str> = [
        "done",
        "failed",
        "error",
        "cancelled",
        "canceled",
        "skipped",
    ]
    .into_iter()
    .collect();
    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    let mut latest: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    loop {
        let (payload, _) = stado(
            &[
                "host",
                "weles-capture-status",
                target,
                "--batch",
                batch,
                "--json",
            ],
            true,
        )?;
        let payload = payload.unwrap_or(Value::Null);
        if let Some(returned_action) = payload.get("action") {
            if returned_action.as_str() != Some(ACTION) {
                bail!("weles-capture-status: action: expected {ACTION}, got {returned_action}");
            }
        }
        let rows = action_rows(&payload, "weles-capture-status")?;
        latest.clear();
        for row in &rows {
            if let Some(identifier) = action_id(row) {
                let identifier = identifier.trim_matches('"').to_string();
                if expected_ids.contains(&identifier) {
                    latest.insert(identifier, row.clone());
                }
            }
        }
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for row in latest.values() {
            *counts.entry(state_of(row)).or_insert(0) += 1;
        }
        let counts_text = counts
            .iter()
            .map(|(state, count)| format!("{state}={count}"))
            .collect::<Vec<_>>()
            .join(", ");
        log(&format!(
            "  {counts_text} ({}/{})",
            latest.len(),
            expected_ids.len()
        ));
        let all_terminal = latest.len() == expected_ids.len()
            && latest
                .values()
                .all(|row| terminal.contains(state_of(row).as_str()));
        if all_terminal {
            return Ok(latest);
        }
        if Instant::now() > deadline {
            log(&format!(
                "  timed out after {timeout_seconds}s; unresolved actions remain pending"
            ));
            return Ok(latest);
        }
        std::thread::sleep(Duration::from_secs(interval));
    }
}

fn artifact_keys(row: &Value) -> Vec<String> {
    let raw = match pick(row, &["artifacts", "artefacts", "objects", "keys"]) {
        Some(value) => value.clone(),
        None => return Vec::new(),
    };
    let mut result: Vec<String> = Vec::new();
    match raw {
        Value::String(item) => result.push(item),
        Value::Array(items) => {
            for item in items {
                match item {
                    Value::String(text) => result.push(text),
                    Value::Object(map) => {
                        let holder = Value::Object(map);
                        if let Some(value) = pick(
                            &holder,
                            &[
                                "key", "uri", "url", "artifact", "artefact", "object", "path",
                            ],
                        ) {
                            result.push(value.to_string().trim_matches('"').to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    result
}

fn named_artifact(keys: &[String], name: &str, reference: &Reference) -> Result<String> {
    let matches: Vec<&String> = keys.iter().filter(|key| basename_of(key) == name).collect();
    if matches.len() != 1 {
        bail!(
            "{}: artifacts.{name}: expected exactly one storage object, got {}",
            reference.id(),
            matches.len()
        );
    }
    Ok(matches[0].clone())
}

fn fetch_artifact(key: &str, destination: &Path) -> Result<()> {
    destination
        .parent()
        .map(std::fs::create_dir_all)
        .transpose()?;
    if destination.exists() {
        std::fs::remove_file(destination)?;
    }
    let dest = destination.to_string_lossy().to_string();
    stado(&["storage", "get", key, dest.as_str()], false)?;
    if !destination.is_file() {
        bail!(
            "stado storage get {key}: nothing was written to {}",
            destination.display()
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Artifact validation

fn require_summary_field<'a>(
    summary: &'a Map<String, Value>,
    field: &str,
    reference: &Reference,
) -> Result<&'a Value> {
    summary.get(field).ok_or_else(|| {
        anyhow!(
            "{}: axe-summary.json.{field}: field is missing",
            reference.id()
        )
    })
}

fn nonempty_text(value: &Value, field: &str, reference: &Reference) -> Result<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            anyhow!(
                "{}: axe-summary.json.{field}: expected a non-empty string",
                reference.id()
            )
        })
}

fn count_field(summary: &Map<String, Value>, field: &str, reference: &Reference) -> Result<u64> {
    let value = require_summary_field(summary, field, reference)?;
    value.as_u64().ok_or_else(|| {
        anyhow!(
            "{}: axe-summary.json.{field}: expected a non-negative integer",
            reference.id()
        )
    })
}

fn sha256_of(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(crate::sha256_hex(&bytes))
}

fn validate_artifacts(
    reference: &Reference,
    action: &Map<String, Value>,
    raw_path: &Path,
    summary_path: &Path,
) -> Result<Map<String, Value>> {
    let rid = reference.id();
    let summary_text = std::fs::read_to_string(summary_path)
        .map_err(|e| anyhow!("{rid}: axe-summary.json: not UTF-8/readable: {e}"))?;
    let summary_doc = strict_json(&summary_text, &format!("{rid}: axe-summary.json"))?;
    let summary = summary_doc
        .as_object()
        .ok_or_else(|| anyhow!("{rid}: axe-summary.json: expected an object"))?
        .clone();
    for field in SUMMARY_FIELDS {
        require_summary_field(&summary, field, reference)?;
    }
    if summary.get("source_url") != action.get("source_url") {
        bail!(
            "{rid}: axe-summary.json.source_url: expected {}, got {}",
            action.get("source_url").unwrap_or(&Value::Null),
            summary.get("source_url").unwrap_or(&Value::Null)
        );
    }
    if summary.get("viewport") != action.get("viewport") {
        bail!(
            "{rid}: axe-summary.json.viewport: expected {}, got {}",
            action.get("viewport").unwrap_or(&Value::Null),
            summary.get("viewport").unwrap_or(&Value::Null)
        );
    }
    for field in ["captured_at", "renderer", "weles_version", "axe_version"] {
        nonempty_text(
            require_summary_field(&summary, field, reference)?,
            field,
            reference,
        )?;
    }
    let raw_size = std::fs::metadata(raw_path)
        .map_err(|e| anyhow!("{rid}: staged axe.json: {e}"))?
        .len();
    let raw_hash = sha256_of(raw_path)?;
    let expected_size = count_field(&summary, "bytes", reference)?;
    if raw_size != expected_size {
        bail!(
            "{rid}: axe-summary.json.bytes: downloaded axe.json has {raw_size} bytes, summary records {expected_size}"
        );
    }
    let expected_hash = require_summary_field(&summary, "sha256", reference)?
        .as_str()
        .filter(|h| is_hex64_lower(h))
        .ok_or_else(|| {
            anyhow!("{rid}: axe-summary.json.sha256: expected 64 lowercase hex characters")
        })?;
    if raw_hash != expected_hash {
        bail!(
            "{rid}: axe-summary.json.sha256: downloaded axe.json hashes to {raw_hash}, summary records {expected_hash}"
        );
    }
    let violation_count = count_field(&summary, "violation_count", reference)?;
    let passes_count = count_field(&summary, "passes_count", reference)?;
    let incomplete_count = count_field(&summary, "incomplete_count", reference)?;
    let violations = require_summary_field(&summary, "violations", reference)?
        .as_array()
        .ok_or_else(|| anyhow!("{rid}: axe-summary.json.violations: expected an array"))?;
    if violations.len() as u64 != violation_count {
        bail!(
            "{rid}: axe-summary.json.violation_count: records {violation_count}, but violations has {} entries",
            violations.len()
        );
    }
    for (position, violation) in violations.iter().enumerate() {
        let field = format!("violations[{position}]");
        let violation = violation
            .as_object()
            .ok_or_else(|| anyhow!("{rid}: axe-summary.json.{field}: expected an object"))?;
        let id_ok = violation
            .get("id")
            .and_then(Value::as_str)
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        if !id_ok {
            bail!("{rid}: axe-summary.json.{field}.id: expected a non-empty string");
        }
        if let Some(impact) = violation.get("impact") {
            if !(impact.is_null() || impact.is_string()) {
                bail!("{rid}: axe-summary.json.{field}.impact: expected a string or null");
            }
        }
        if !violation.get("help").map(Value::is_string).unwrap_or(false) {
            bail!("{rid}: axe-summary.json.{field}.help: expected a string");
        }
        let nodes_ok = violation
            .get("node_count")
            .and_then(Value::as_u64)
            .is_some();
        if !nodes_ok {
            bail!("{rid}: axe-summary.json.{field}.node_count: expected a non-negative integer");
        }
    }
    let raw_text = std::fs::read_to_string(raw_path)
        .map_err(|e| anyhow!("{rid}: axe.json: not UTF-8/readable: {e}"))?;
    let raw = strict_json(&raw_text, &format!("{rid}: axe.json"))?;
    if !raw.is_object() {
        bail!("{rid}: axe.json: expected an object");
    }
    for (field, count) in [
        ("violations", violation_count),
        ("passes", passes_count),
        ("incomplete", incomplete_count),
    ] {
        let length = raw
            .get(field)
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("{rid}: axe.json.{field}: expected an array"))?
            .len() as u64;
        if length != count {
            bail!("{rid}: axe.json.{field}: has {length} entries, summary records {count}");
        }
    }
    let mut summary = summary;
    summary.insert("bytes".to_string(), Value::Number(raw_size.into()));
    summary.insert("sha256".to_string(), Value::String(raw_hash));
    Ok(summary)
}

fn install_artifacts(
    reference: &Reference,
    raw_stage: &Path,
    summary_stage: &Path,
) -> Result<(PathBuf, PathBuf)> {
    let directory = reference
        .path
        .parent()
        .context("reference path has no parent")?
        .join("media")
        .join("accessibility");
    std::fs::create_dir_all(&directory)?;
    let raw_path = directory.join("axe.json");
    let summary_path = directory.join("axe-summary.json");
    let raw_part = directory.join(format!(".axe.json.{}.part", pid()));
    let summary_part = directory.join(format!(".axe-summary.json.{}.part", pid()));
    std::fs::copy(raw_stage, &raw_part)?;
    std::fs::copy(summary_stage, &summary_part)?;
    std::fs::rename(raw_part, &raw_path)?;
    std::fs::rename(summary_part, &summary_path)?;
    Ok((raw_path, summary_path))
}

fn axe_observations(summary: &Map<String, Value>) -> Vec<String> {
    let version = summary
        .get("axe_version")
        .and_then(Value::as_str)
        .unwrap_or("");
    let viewport = summary.get("viewport").cloned().unwrap_or(Value::Null);
    let width = viewport
        .get("width")
        .map(|v| v.to_string())
        .unwrap_or_default();
    let height = viewport
        .get("height")
        .map(|v| v.to_string())
        .unwrap_or_default();
    let dsf = viewport
        .get("device_scale_factor")
        .map(|v| v.to_string())
        .unwrap_or_default();
    let captured_at = summary
        .get("captured_at")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut observations = vec![format!(
        "[axe-core] axe-core {version} reported {} violation rules, {} passing rules, and {} incomplete rules against the live product at {width}x{height}@{dsf} on {captured_at}.",
        summary.get("violation_count").map(|v| v.to_string()).unwrap_or_default(),
        summary.get("passes_count").map(|v| v.to_string()).unwrap_or_default(),
        summary.get("incomplete_count").map(|v| v.to_string()).unwrap_or_default(),
    )];
    if let Some(violations) = summary.get("violations").and_then(Value::as_array) {
        for violation in violations {
            let Some(violation) = violation.as_object() else {
                continue;
            };
            let rule = violation
                .get("id")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("unnamed-rule");
            let impact_text = match violation.get("impact") {
                Some(Value::Null) | None => "impact not reported".to_string(),
                Some(other) => other.to_string().trim_matches('"').to_string(),
            };
            let nodes = violation
                .get("node_count")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0".to_string());
            let help_text = violation
                .get("help")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let suffix = if help_text.is_empty() {
                String::new()
            } else {
                format!(": {help_text}")
            };
            observations.push(format!(
                "[axe-core] Rule {rule} ({impact_text}) affected {nodes} nodes{suffix}."
            ));
        }
    }
    observations
}

fn update_record(reference: &Reference, summary: &Map<String, Value>) -> Result<(String, String)> {
    let rid = reference.id();
    let text = std::fs::read_to_string(&reference.path)
        .with_context(|| format!("{rid}: reference.json: unreadable"))?;
    let document_doc = strict_json(&text, &format!("{rid}: reference.json"))?;
    let mut document = document_doc
        .as_object()
        .ok_or_else(|| anyhow!("{rid}: reference.json: expected an object"))?
        .clone();

    let current_field = if document.get("product_url").map(Value::is_null) == Some(false) {
        "product_url"
    } else {
        "source_url"
    };
    let current_url = canonical_url(
        document.get(current_field).unwrap_or(&Value::Null),
        &rid,
        current_field,
    )?;
    if current_url != reference.source_url {
        bail!(
            "{rid}: {current_field}: changed from {:?} to {current_url:?} while audit ran",
            reference.source_url
        );
    }

    let accessibility_missing = document
        .get("accessibility")
        .map(Value::is_null)
        .unwrap_or(true);
    let mut accessibility: Map<String, Value> = if accessibility_missing {
        Map::new()
    } else {
        document
            .get("accessibility")
            .and_then(Value::as_object)
            .cloned()
            .ok_or_else(|| anyhow!("{rid}: accessibility: expected an object"))?
    };

    let observations_value = accessibility
        .get("observations")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));
    let observations_list = observations_value.as_array().ok_or_else(|| {
        anyhow!("{rid}: accessibility.observations: expected an array of strings")
    })?;
    if observations_list.iter().any(|item| !item.is_string()) {
        bail!("{rid}: accessibility.observations: expected an array of strings");
    }
    let unknowns_value = accessibility
        .get("unknowns")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));
    let unknowns_list = unknowns_value
        .as_array()
        .ok_or_else(|| anyhow!("{rid}: accessibility.unknowns: expected an array of strings"))?;
    if unknowns_list.iter().any(|item| !item.is_string()) {
        bail!("{rid}: accessibility.unknowns: expected an array of strings");
    }

    let mut observations: Vec<Value> = observations_list
        .iter()
        .filter(|item| !item.as_str().unwrap_or("").starts_with("[axe-core]"))
        .cloned()
        .collect();
    for observation in axe_observations(summary) {
        observations.push(Value::String(observation));
    }
    accessibility.insert("observations".into(), Value::Array(observations));
    accessibility.insert("unknowns".into(), unknowns_value);
    accessibility.insert("measured".into(), Value::Bool(true));

    let get_str = |key: &str| -> String {
        summary
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let get_num = |key: &str| -> Value { summary.get(key).cloned().unwrap_or(Value::Null) };
    let measurement = json!({
        "tool": "axe-core",
        "version": get_str("axe_version"),
        "captured_at": get_str("captured_at"),
        "renderer": get_str("renderer"),
        "weles_version": get_str("weles_version"),
        "source_url": get_str("source_url"),
        "viewport": summary.get("viewport").cloned().unwrap_or(Value::Null),
        "raw_path": "media/accessibility/axe.json",
        "summary_path": "media/accessibility/axe-summary.json",
        "raw_bytes": get_num("bytes"),
        "raw_sha256": get_num("sha256"),
        "violation_count": get_num("violation_count"),
        "passes_count": get_num("passes_count"),
        "incomplete_count": get_num("incomplete_count"),
    });
    accessibility.insert("measurement".into(), measurement);
    document.insert("accessibility".into(), Value::Object(accessibility));
    atomic_json(&reference.path, &Value::Object(document))?;
    Ok((
        "media/accessibility/axe.json".to_string(),
        "media/accessibility/axe-summary.json".to_string(),
    ))
}

fn retrieve(
    reference: &Reference,
    action: &Map<String, Value>,
    row: &Value,
    batch: &str,
) -> Result<Value> {
    let rid = reference.id();
    if let Some(site_slug) = row.get("site_slug").and_then(Value::as_str) {
        if site_slug != reference.slug {
            bail!(
                "{rid}: status.site_slug: expected {:?}, got {site_slug:?}",
                reference.slug
            );
        }
    }
    if let Some(prefix) = row.get("artifact_prefix").and_then(Value::as_str) {
        let expected = action
            .get("artifact_prefix")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if prefix != expected {
            bail!("{rid}: status.artifact_prefix: expected {expected:?}, got {prefix:?}");
        }
    }
    let keys = artifact_keys(row);
    let raw_key = named_artifact(&keys, "axe.json", reference)?;
    let summary_key = named_artifact(&keys, "axe-summary.json", reference)?;
    let stage = staging_root()
        .join(batch)
        .join(&reference.catalog)
        .join(&reference.slug);
    let raw_stage = stage.join("axe.json");
    let summary_stage = stage.join("axe-summary.json");
    fetch_artifact(&summary_key, &summary_stage)
        .map_err(|e| anyhow!("{rid}: staging.axe-summary.json: {e:#}"))?;
    fetch_artifact(&raw_key, &raw_stage).map_err(|e| anyhow!("{rid}: staging.axe.json: {e:#}"))?;
    let summary = validate_artifacts(reference, action, &raw_stage, &summary_stage)
        .map_err(|e| anyhow!("{e:#}"))?;
    let (raw_path, summary_path) = install_artifacts(reference, &raw_stage, &summary_stage)
        .map_err(|e| anyhow!("{rid}: media/accessibility: {e}"))?;
    let (raw_relative, summary_relative) =
        update_record(reference, &summary).map_err(|e| anyhow!("{rid}: reference.json: {e:#}"))?;
    let repo_relative = |path: &Path| -> String { path.to_string_lossy().to_string() };
    Ok(json!({
        "id": rid,
        "catalog": reference.catalog,
        "index": reference.index,
        "name": reference.name,
        "site_slug": reference.slug,
        "source_url": reference.source_url,
        "status": "complete",
        "reason": Value::Null,
        "raw_path": repo_relative(&raw_path),
        "summary_path": repo_relative(&summary_path),
        "record_raw_path": raw_relative,
        "record_summary_path": summary_relative,
        "raw_storage_key": raw_key,
        "summary_storage_key": summary_key,
    }))
}

fn initial_row(reference: &Reference, action: &Map<String, Value>) -> Value {
    json!({
        "id": reference.id(),
        "catalog": reference.catalog,
        "index": reference.index,
        "name": reference.name,
        "site_slug": reference.slug,
        "source_url": reference.source_url,
        "status": "pending",
        "artifact_prefix": action.get("artifact_prefix").cloned().unwrap_or(Value::Null),
        "raw_path": Value::Null,
        "summary_path": Value::Null,
        "reason": "not dispatched",
    })
}

fn write_index(
    rows: &[Value],
    batch: &str,
    target: &str,
    plan_path: &Path,
    verifier_errors: &[String],
) -> Result<Value> {
    let count = |status: &str| {
        rows.iter()
            .filter(|row| row.get("status").and_then(Value::as_str) == Some(status))
            .count()
    };
    let totals = json!({
        "planned": rows.len(),
        "complete": count("complete"),
        "failed": count("failed"),
        "pending": count("pending"),
    });
    let payload = json!({
        "schema": INDEX_SCHEMA,
        "generated_at": now(),
        "batch": batch,
        "target": target,
        "plan": plan_path.to_string_lossy(),
        "totals": totals,
        "records": rows,
        "verifier_errors": verifier_errors,
    });
    atomic_json(Path::new(INDEX), &payload)?;
    Ok(payload)
}

fn run_verifier(catalog: &str) -> Result<()> {
    let exe = std::env::current_exe().context("locate the spis executable")?;
    let output = Command::new(exe)
        .args([
            "verify-reference-evidence",
            "--catalog",
            catalog,
            "--apply",
            "--no-state-match",
        ])
        .output()
        .context("spawn spis verify-reference-evidence")?;
    if !output.status.success() {
        let combined = if output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };
        let lines: Vec<&str> = combined
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        let detail = if lines.is_empty() {
            format!("exit {}", output.status.code().unwrap_or(-1))
        } else {
            lines
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        };
        bail!("spis verify-reference-evidence --catalog {catalog}: {detail}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------

pub fn run(rest: &[String]) -> Result<()> {
    let mut catalogs_arg: Vec<String> = Vec::new();
    let mut records: Option<String> = None;
    let mut batch_arg: Option<String> = None;
    let mut target = DEFAULT_TARGET.to_string();
    let mut plan_arg: Option<String> = None;
    let mut dry_run = false;
    let mut poll_seconds: u64 = 15;
    let mut timeout_minutes: u64 = 120;

    let mut i = 0usize;
    while i < rest.len() {
        match rest[i].as_str() {
            "--catalog" => {
                i += 1;
                catalogs_arg.push(rest.get(i).context("--catalog needs a value")?.clone());
            }
            "--records" => {
                i += 1;
                records = Some(rest.get(i).context("--records needs a value")?.clone());
            }
            "--batch" => {
                i += 1;
                batch_arg = Some(rest.get(i).context("--batch needs a value")?.clone());
            }
            "--target" => {
                i += 1;
                target = rest.get(i).context("--target needs a value")?.clone();
            }
            "--plan" => {
                i += 1;
                plan_arg = Some(rest.get(i).context("--plan needs a value")?.clone());
            }
            "--dry-run" => dry_run = true,
            "--poll-seconds" => {
                i += 1;
                poll_seconds = rest
                    .get(i)
                    .context("--poll-seconds needs a value")?
                    .parse()?;
            }
            "--timeout-minutes" => {
                i += 1;
                timeout_minutes = rest
                    .get(i)
                    .context("--timeout-minutes needs a value")?
                    .parse()?;
            }
            "--help" | "-h" => {
                println!("usage: spis audit-reference-accessibility [--catalog NAME]... [--records SEL] [--batch ID] [--target HOST] [--plan PATH] [--dry-run] [--poll-seconds N] [--timeout-minutes N]");
                return Ok(());
            }
            other => bail!("unknown argument: {other}"),
        }
        i += 1;
    }

    let log = |line: &str| {
        eprintln!("{line}");
        use std::io::Write;
        let _ = std::io::stderr().flush();
    };

    // ---- planning phase: failures exit 2 --------------------------------
    let plan: Map<String, Value>;
    let plan_path: PathBuf;
    let references: Vec<Reference>;
    let batch: String;
    {
        let outcome: Result<(Map<String, Value>, PathBuf, Vec<Reference>, String)> = (|| {
            if poll_seconds < 1 {
                bail!("--poll-seconds: must be at least 1");
            }
            if timeout_minutes < 1 {
                bail!("--timeout-minutes: must be at least 1");
            }
            let sources: Vec<String> = if catalogs_arg.is_empty() {
                DEFAULT_CATALOGS.iter().map(|s| s.to_string()).collect()
            } else {
                catalogs_arg.clone()
            };
            let mut catalogs: Vec<String> = Vec::new();
            for value in &sources {
                let normalized = normalize_catalog(value)?;
                if !catalogs.contains(&normalized) {
                    catalogs.push(normalized);
                }
            }
            let selection = parse_record_selection(records.as_ref())?;
            let refs = load_references(&catalogs, selection.as_ref())?;
            if refs.is_empty() {
                bail!("selection: no records selected");
            }
            let batch = batch_arg.clone().unwrap_or_else(default_batch);
            let captures: Vec<Value> = refs
                .iter()
                .map(|r| Value::Object(r.action(&batch)))
                .collect();
            let plan = serde_json::json!({
                "schema": PLAN_SCHEMA,
                "batch": batch,
                "target": target,
                "captures": captures,
            });
            let validated = validate_plan(&plan, &target, &refs)?;
            let plan_path = plan_path_for(plan_arg.as_ref(), &batch)?;
            atomic_json(&plan_path, &Value::Object(validated.clone()))?;
            let text = std::fs::read_to_string(&plan_path)
                .with_context(|| "plan: unreadable after write")?;
            let parsed = strict_json(&text, "plan")?;
            let validated_again = validate_plan(&parsed, &target, &refs)?;
            Ok((validated_again, plan_path, refs, batch))
        })();
        match outcome {
            Ok(value) => {
                plan = value.0;
                plan_path = value.1;
                references = value.2;
                batch = value.3;
            }
            Err(e) => {
                eprintln!("error: {e:#}");
                std::process::exit(2);
            }
        }
    }

    if dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&Value::Object(plan.clone()))?
        );
        println!(
            "dry run: planned={} complete=0 failed=0 pending={}; no host was contacted; plan={}",
            references.len(),
            references.len(),
            plan_path.display()
        );
        return Ok(());
    }

    let captures = plan
        .get("captures")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut rows: Vec<Value> = references
        .iter()
        .zip(captures.iter())
        .map(|(reference, capture)| initial_row(reference, capture.as_object().unwrap()))
        .collect();
    let mut verifier_errors: Vec<String> = Vec::new();

    // ---- enqueue + poll: failures mark every row and exit 2 -------------
    let (ids, states): (Vec<String>, std::collections::HashMap<String, Value>) = {
        let outcome: Result<(
            String,
            Vec<String>,
            std::collections::HashMap<String, Value>,
        )> = (|| {
            let (enqueued_batch, ids) = enqueue(&target, &plan_path, &plan)?;
            log(&format!(
                "batch {enqueued_batch}: {} {ACTION} actions enqueued",
                ids.len()
            ));
            let id_set: HashSet<String> = ids.iter().cloned().collect();
            let states = poll(
                &target,
                &enqueued_batch,
                &id_set,
                poll_seconds,
                timeout_minutes * 60,
                &log,
            )?;
            Ok((enqueued_batch, ids, states))
        })();
        match outcome {
            Ok((_, ids, states)) => (ids, states),
            Err(e) => {
                let reason = format!("{e:#}");
                for row in rows.iter_mut() {
                    if let Some(obj) = row.as_object_mut() {
                        obj.insert("reason".into(), Value::String(reason.clone()));
                    }
                }
                let payload = write_index(&rows, &batch, &target, &plan_path, &verifier_errors)?;
                let totals = payload.get("totals").cloned().unwrap_or(Value::Null);
                log(&format!("error: {reason}"));
                log(&format!("{INDEX}: {totals}"));
                std::process::exit(2);
            }
        }
    };

    // ---- per-reference retrieve -----------------------------------------
    let mut completed_catalogs: Vec<String> = Vec::new();
    for position in 0..references.len() {
        let reference = &references[position];
        let action = captures[position].as_object().cloned().unwrap_or_default();
        let identifier = &ids[position];
        let Some(state_row) = states.get(identifier) else {
            if let Some(obj) = rows[position].as_object_mut() {
                obj.insert(
                    "reason".into(),
                    Value::String(format!(
                        "action {identifier} had not reported a state when polling stopped"
                    )),
                );
            }
            continue;
        };
        let state = state_of(state_row);
        if state != "done" {
            let exact_error = match pick(state_row, &["error", "message", "reason"]) {
                Some(error) => {
                    let text = error.to_string();
                    let trimmed = text.trim_matches('"').to_string();
                    if trimmed.is_empty() {
                        format!("action ended in state {state}")
                    } else {
                        trimmed
                    }
                }
                None => format!("action ended in state {state}"),
            };
            if let Some(obj) = rows[position].as_object_mut() {
                obj.insert("status".into(), Value::String("failed".into()));
                obj.insert("reason".into(), Value::String(exact_error.clone()));
            }
            log(&format!(
                "FAILED {}: action.error: {exact_error}",
                reference.id()
            ));
            continue;
        }
        match retrieve(reference, &action, state_row, &batch) {
            Ok(result_row) => {
                rows[position] = result_row;
                if !completed_catalogs.contains(&reference.catalog) {
                    completed_catalogs.push(reference.catalog.clone());
                }
                let raw_path = rows[position]
                    .get("raw_path")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                log(&format!("COMPLETE {}: {raw_path}", reference.id()));
            }
            Err(e) => {
                let reason = format!("{e:#}");
                if let Some(obj) = rows[position].as_object_mut() {
                    obj.insert("status".into(), Value::String("failed".into()));
                    obj.insert("reason".into(), Value::String(reason.clone()));
                }
                log(&format!("REFUSED {reason}"));
            }
        }
    }

    completed_catalogs.sort();
    for catalog in &completed_catalogs {
        match run_verifier(catalog) {
            Ok(()) => log(&format!("verified {catalog} with --apply --no-state-match")),
            Err(e) => {
                verifier_errors.push(format!("{e:#}"));
                log(&format!("error: {e:#}"));
            }
        }
    }

    let payload = write_index(&rows, &batch, &target, &plan_path, &verifier_errors)?;
    let totals = payload.get("totals").cloned().unwrap_or(Value::Null);
    let total = |key: &str| totals.get(key).and_then(Value::as_u64).unwrap_or(0);
    log(&format!(
        "{INDEX}: planned={} complete={} failed={} pending={}",
        total("planned"),
        total("complete"),
        total("failed"),
        total("pending")
    ));
    if total("failed") == 0 && total("pending") == 0 && verifier_errors.is_empty() {
        Ok(())
    } else {
        std::process::exit(3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_json_rejects_duplicate_keys() {
        assert!(strict_json(r#"{"a": 1, "b": 2}"#, "x").is_ok());
        let err = strict_json(r#"{"a": 1, "a": 2}"#, "x");
        assert!(err.is_err(), "{err:?}");
        assert!(strict_json(r#"{"a": [1, {"b": null}]}"#, "x").is_ok());
        assert!(strict_json(r#"{"a": 1,}"#, "x").is_err()); // trailing comma
        assert!(strict_json(r#"{"a": 1} extra"#, "x").is_err()); // trailing junk
        assert_eq!(
            strict_json(r#""é😀""#, "x").unwrap(),
            Value::String("é😀".to_string())
        );
    }

    #[test]
    fn parses_record_selection() {
        assert_eq!(parse_record_selection(None).unwrap(), None);
        let sel = parse_record_selection(Some(&"3,5-7,10".to_string()))
            .unwrap()
            .unwrap();
        assert_eq!(sel, HashSet::from([3, 5, 6, 7, 10]));
        assert!(parse_record_selection(Some(&"".to_string())).is_err());
        assert!(parse_record_selection(Some(&"0".to_string())).is_err());
        assert!(parse_record_selection(Some(&"7-4".to_string())).is_err());
        assert!(parse_record_selection(Some(&"2,".to_string())).is_err());
    }

    #[test]
    fn normalizes_catalog_names() {
        assert_eq!(normalize_catalog("web-app").unwrap(), "web-app-examples");
        assert_eq!(
            normalize_catalog("design-system-examples").unwrap(),
            "design-system-examples"
        );
        assert!(normalize_catalog("").is_err());
        assert!(normalize_catalog("Bad_Catalog").is_err());
        assert!(normalize_catalog("-examples-examples").is_err());
    }

    #[test]
    fn canonicalizes_urls() {
        let v = Value::String("https://example.com/a?b=1#frag".into());
        assert_eq!(
            canonical_url(&v, "r", "f").unwrap(),
            "https://example.com/a?b=1#frag"
        );
        let v = Value::String("https://example.com".into());
        assert_eq!(canonical_url(&v, "r", "f").unwrap(), "https://example.com/");
        assert!(canonical_url(&Value::String("ftp://example.com/".into()), "r", "f").is_err());
        assert!(canonical_url(&Value::String("https://".into()), "r", "f").is_err());
        assert!(canonical_url(&Value::Null, "r", "f").is_err());
    }

    #[test]
    fn slug_checks() {
        assert!(looks_like_slug("01-linear"));
        assert!(looks_like_slug("a"));
        assert!(looks_like_slug("a.b_c-d"));
        assert!(!looks_like_slug(""));
        assert!(!looks_like_slug("-lead"));
        assert!(!looks_like_slug("Has-Caps"));
        assert!(!looks_like_slug(&"a".repeat(82)));
        assert!(looks_like_slug(&"a".repeat(81)));
    }

    #[test]
    fn hex_checks() {
        assert!(is_hex64_lower(&"a".repeat(64)));
        assert!(!is_hex64_lower(&"A".repeat(64)));
        assert!(!is_hex64_lower(&"g".repeat(64)));
        assert!(!is_hex64_lower("abc"));
    }

    #[test]
    fn validates_plan_shape() {
        let reference = Reference {
            catalog: "web-app-examples".into(),
            index: 3,
            name: "Example".into(),
            slug: "03-example".into(),
            path: PathBuf::from("web-app-examples/03-example/reference.json"),
            source_url: "https://example.com/".into(),
        };
        let action = reference.action("batch-1");
        let plan = json!({
            "schema": PLAN_SCHEMA,
            "batch": "batch-1",
            "target": DEFAULT_TARGET,
            "captures": [Value::Object(action.clone())],
        });
        assert!(validate_plan(&plan, DEFAULT_TARGET, &[reference.clone()]).is_ok());

        // Extra key → rejected.
        let mut bad = plan.clone();
        bad.as_object_mut()
            .unwrap()
            .insert("extra".into(), Value::Null);
        assert!(validate_plan(&bad, DEFAULT_TARGET, &[reference.clone()]).is_err());

        // Wrong target → rejected.
        assert!(validate_plan(&plan, "other-host", &[reference.clone()]).is_err());

        // Mutated field → rejected.
        let mut mutated = plan.clone();
        mutated["captures"][0]["source_url"] = Value::String("https://evil.example/".into());
        assert!(validate_plan(&mutated, DEFAULT_TARGET, &[reference]).is_err());
    }

    #[test]
    fn batch_stamp_format() {
        let batch = default_batch();
        // accessibility-YYYYMMDDtHHMMSSz
        assert_eq!(batch.len(), "accessibility-20260823t123456z".len());
        assert!(batch.starts_with("accessibility-"));
        assert!(batch.ends_with('z'));
        let stamp = &batch["accessibility-".len()..];
        assert!(stamp[8..9].as_bytes() == b"t");
        assert!(stamp[..8].bytes().all(|b| b.is_ascii_digit()));
        assert!(stamp[9..15].bytes().all(|b| b.is_ascii_digit()));
    }
}
