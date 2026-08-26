//! `spis generate-example-catalogs` — validate measured example catalogs and
//! write the machine-readable cross-catalog index.
//!
//! This generator is the gate. It refuses to index a catalog whose data
//! contradicts the files beside it, and records the measured numbers rather
//! than an intention: how many records are complete, how many are partial, and
//! how the motion evidence was actually obtained (a product we drove, a browser
//! we drove, or media its owner published).
//!
//! Rust port of the validation and JSON indexing behavior from the former
//! `generate-example-catalogs.py` pipeline.

use crate as lib;
use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// reference_contract vocabulary
// ---------------------------------------------------------------------------

const CATALOG_SCHEMA: &str = "wisent.example-catalog.v2";
const RECORD_SCHEMA: &str = "wisent.full-product-reference.v2";
const INDEX_SCHEMA: &str = "wisent.full-reference-catalog.v2";

/// Canonical motion-kind vocabulary (`CANONICAL_MOTION_KINDS`, sorted).
const CANONICAL_MOTION_KINDS: &[&str] = &[
    "animated-gif",
    "animated-webp",
    "terminal-cast",
    "video-mp4",
    "video-webm",
];

const MOTION_SUFFIXES: &[&str] = &[".gif", ".webp", ".mp4", ".webm", ".cast"];
const STATE_SUFFIXES: &[&str] = &[".png", ".webp", ".jpg", ".jpeg"];

const PROVENANCE_CLASSES: &[&str] = &[
    "local-product-run",
    "local-browser-recording",
    "upstream-owner-media",
    "unclassified",
];
const LOCAL_PROVENANCE: &[&str] = &["local-browser-recording", "local-product-run"];

const MIN_MOTION_SECONDS: f64 = 0.2;
const MIN_STATES: usize = 3;
const MIN_JOURNEY_STEPS: usize = 5;
const MIN_INTERACTIONS: usize = 8;

const INTERACTION_FIELDS: &[&str] = &[
    "name",
    "trigger",
    "response",
    "feedback",
    "cancellation",
    "failure",
    "recovery",
    "evidence",
];

const MOTION_ANALYSIS_FIELDS: &[&str] = &[
    "trigger",
    "start_state",
    "end_state",
    "continuity",
    "timing_class",
    "interruption_or_reversal",
    "feedback",
    "reduced_motion_equivalent",
];
const MOTION_ANALYSIS_OPTIONAL: &[&str] = &["source_title", "evidence", "timing_description"];

const TIMING_CLASSES: &[&str] = &[
    "continuous",
    "instant",
    "multi-second",
    "one-to-three-seconds",
    "sub-second",
];

const EVIDENCE_STATUSES: &[&str] = &["complete", "partial"];

const JOURNEY_FIELDS: &[&str] = &[
    "actor",
    "goal",
    "prerequisites",
    "steps",
    "failure_route",
    "recovery_route",
    "completion_evidence",
];
const JOURNEY_STEP_FIELDS: &[&str] = &[
    "index",
    "user_action",
    "system_response",
    "state",
    "evidence",
];

/// Curated third-party families, in reading order, followed by catalogs of our
/// own products. Any other directory that satisfies the contract is appended.
const CATALOGS: &[&str] = &[
    "ios-app-examples",
    "android-app-examples",
    "macos-app-examples",
    "desktop-app-examples",
    "web-app-examples",
    "dashboard-console-examples",
    "tui-examples",
    "cli-examples",
    "onboarding-auth-examples",
    "documentation-site-examples",
    "app-store-listing-examples",
    "design-system-examples",
    "report-evidence-examples",
    "wisent-product-examples",
];

fn provenance_label(name: &str) -> &'static str {
    match name {
        "local-product-run" => "product run here",
        "local-browser-recording" => "browser driven here",
        "upstream-owner-media" => "owner-published media",
        _ => "unclassified",
    }
}

// ---------------------------------------------------------------------------
// Small helpers mirroring Python semantics
// ---------------------------------------------------------------------------

/// Truthiness of a JSON value under Python rules (None/false/0/""/[]/{}) are falsey).
fn py_truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

fn is_null_or_empty_string_or_list(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) => s.is_empty(),
        Some(Value::Array(a)) => a.is_empty(),
        _ => false,
    }
}
fn require_nonempty(record: &Value, fields: &[&str], context: &str) -> Result<()> {
    let missing: Vec<&str> = fields
        .iter()
        .filter(|field| is_null_or_empty_string_or_list(record.get(**field)))
        .copied()
        .collect();
    if !missing.is_empty() {
        bail!("{context}: missing {:?}", missing);
    }
    Ok(())
}

fn python_repr(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "None".into(),
        Some(Value::Bool(true)) => "True".into(),
        Some(Value::Bool(false)) => "False".into(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::String(s)) => format!("'{s}'"),
        other => format!("{}", other.map(|v| v.to_string()).unwrap_or_default()),
    }
}

/// urlparse-based http(s) URL validity: scheme must be http/https, netloc non-empty.
fn url_ok(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if !(scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")) {
        return false;
    }
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    // Strip userinfo; netloc itself must be non-empty.
    let host = authority.rsplit('@').next().unwrap_or("");
    !authority.is_empty() && !host.is_empty()
}

/// Resolve `relative` under `base`, refuse escapes and non-files. Lexical
/// normalization stands in for pathlib.resolve(); symlinks inside a catalog
/// pointing outside are not followed here.
fn resolve_evidence_path(base: &Path, relative: &str, context: &str) -> Result<PathBuf> {
    let joined = base.join(relative);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    bail!("{context}: unavailable local evidence {relative:?}");
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    if !normalized.starts_with(base) || !normalized.is_file() {
        bail!("{context}: unavailable local evidence {relative:?}");
    }
    Ok(normalized)
}

fn validate_file_metadata(path: &Path, record: &Value, context: &str) -> Result<()> {
    let payload = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let expected_bytes = record.get("bytes").and_then(Value::as_u64);
    if Some(payload.len() as u64) != expected_bytes {
        bail!("{context}: byte count differs from the file");
    }
    let expected_sha = record.get("sha256").and_then(Value::as_str);
    if Some(lib::sha256_hex(&payload).as_str()) != expected_sha {
        bail!("{context}: SHA-256 differs from the file");
    }
    Ok(())
}

fn has_suffix(path: &Path, suffixes: &[&str]) -> bool {
    path.extension()
        .map(|ext| {
            let dotted = format!(".{}", ext.to_string_lossy().to_lowercase());
            suffixes.contains(&dotted.as_str())
        })
        .unwrap_or(false)
}

fn evidence_status_of(record: &Value) -> Option<&str> {
    record.get("evidence_status").and_then(Value::as_str)
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn validate_motion(record: &Value, record_path: &str, reference_dir: &Path) -> Result<Vec<String>> {
    let motion = record.get("motion");
    let Some(motion) = motion.and_then(Value::as_array) else {
        bail!("{record_path}: motion must be a list");
    };
    if motion.is_empty() {
        if evidence_status_of(record) == Some("partial") {
            return Ok(Vec::new());
        }
        bail!("{record_path}: complete evidence needs at least one motion asset");
    }

    let mut classes: Vec<String> = Vec::new();
    for (position, item) in motion.iter().enumerate() {
        let context = format!("{record_path}: motion {}", position + 1);
        require_nonempty(
            item,
            &[
                "local_path",
                "source_url",
                "media_kind",
                "bytes",
                "sha256",
                "capture_method",
                "provenance_class",
            ],
            &context,
        )?;
        let media_kind = item.get("media_kind").and_then(Value::as_str).unwrap_or("");
        if !CANONICAL_MOTION_KINDS.contains(&media_kind) {
            bail!(
                "{context}: media kind '{}' is not in the canonical vocabulary {:?}",
                media_kind,
                CANONICAL_MOTION_KINDS
            );
        }
        let provenance_class = item
            .get("provenance_class")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !PROVENANCE_CLASSES.contains(&provenance_class) {
            bail!("{context}: unknown provenance class '{provenance_class}'");
        }
        if provenance_class == "unclassified" {
            bail!("{context}: provenance was never classified");
        }
        if !py_truthy(item.get("measured")) {
            bail!("{context}: asset was never measured; run verify-reference-evidence.py");
        }
        let source_url = item.get("source_url").and_then(Value::as_str).unwrap_or("");
        if !url_ok(source_url) {
            bail!("{context}: invalid source URL");
        }
        let local_path = item.get("local_path").and_then(Value::as_str).unwrap_or("");
        let motion_path = resolve_evidence_path(reference_dir, local_path, &context)?;
        if !has_suffix(&motion_path, MOTION_SUFFIXES) {
            bail!("{context}: unsupported motion format");
        }
        let duration = item.get("duration_seconds").and_then(Value::as_f64);
        let duration_ok = matches!(duration, Some(d) if d >= MIN_MOTION_SECONDS);
        if !duration_ok {
            bail!(
                "{context}: measured duration {} is below the floor",
                python_repr(item.get("duration_seconds"))
            );
        }
        if !has_suffix(&motion_path, &[".cast"]) {
            require_nonempty(item, &["width", "height"], &context)?;
        }
        validate_file_metadata(&motion_path, item, &context)?;
        classes.push(provenance_class.to_string());
    }

    let declared: BTreeSet<String> = record
        .get("motion_provenance")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let measured: BTreeSet<String> = classes.iter().cloned().collect();
    if measured != declared {
        bail!("{record_path}: motion_provenance does not match the motion entries");
    }
    Ok(classes)
}

fn validate_states(record: &Value, record_path: &str, reference_dir: &Path) -> Result<()> {
    let Some(states) = record.get("states").and_then(Value::as_array) else {
        bail!("{record_path}: states must be a list");
    };
    if states.len() < MIN_STATES && evidence_status_of(record) == Some("complete") {
        bail!("{record_path}: complete evidence needs at least {MIN_STATES} local states");
    }
    for (position, item) in states.iter().enumerate() {
        let context = format!("{record_path}: state {}", position + 1);
        require_nonempty(
            item,
            &["local_path", "width", "height", "bytes", "sha256"],
            &context,
        )?;
        let local_path = item.get("local_path").and_then(Value::as_str).unwrap_or("");
        let state_path = resolve_evidence_path(reference_dir, local_path, &context)?;
        if !has_suffix(&state_path, STATE_SUFFIXES) {
            bail!("{context}: unsupported state-image format");
        }
        validate_file_metadata(&state_path, item, &context)?;
    }
    Ok(())
}

fn validate_behaviour(record: &Value, record_path: &str) -> Result<()> {
    let Some(interactions) = record.get("interactions").and_then(Value::as_array) else {
        bail!("{record_path}: interactions must be a list");
    };
    if interactions.len() < MIN_INTERACTIONS && evidence_status_of(record) == Some("complete") {
        bail!(
            "{record_path}: complete evidence needs at least {MIN_INTERACTIONS} observed interactions"
        );
    }
    for (position, item) in interactions.iter().enumerate() {
        require_nonempty(
            item,
            INTERACTION_FIELDS,
            &format!("{record_path}: interaction {}", position + 1),
        )?;
    }

    let journey_value = record.get("journey");
    if py_truthy(journey_value) {
        let journey = journey_value.expect("truthy");
        require_nonempty(journey, JOURNEY_FIELDS, &format!("{record_path}: journey"))?;
        let steps_ok = journey
            .get("steps")
            .and_then(Value::as_array)
            .map(|steps| steps.len() >= MIN_JOURNEY_STEPS)
            .unwrap_or(false);
        if !steps_ok {
            bail!("{record_path}: journey needs at least {MIN_JOURNEY_STEPS} observed steps");
        }
        for (position, step) in journey["steps"].as_array().unwrap().iter().enumerate() {
            require_nonempty(
                step,
                JOURNEY_STEP_FIELDS,
                &format!("{record_path}: journey step {}", position + 1),
            )?;
            let index = step.get("index").and_then(Value::as_i64);
            if index != Some(position as i64 + 1) {
                bail!("{record_path}: journey step order is invalid");
            }
        }
    } else if evidence_status_of(record) == Some("complete") {
        bail!("{record_path}: complete evidence needs a journey");
    }

    match record.get("motion_analysis") {
        Some(analysis) if !analysis.is_null() => {
            let entries: Vec<Value> = match analysis {
                Value::Array(items) => items.clone(),
                other => vec![other.clone()],
            };
            for (position, item) in entries.iter().enumerate() {
                let allowed: std::collections::HashSet<&str> = MOTION_ANALYSIS_FIELDS
                    .iter()
                    .chain(MOTION_ANALYSIS_OPTIONAL.iter())
                    .copied()
                    .collect();
                let obj = match item.as_object() {
                    Some(obj) => obj,
                    None => bail!(
                        "{record_path}: motion analysis {} is malformed",
                        position + 1
                    ),
                };
                let unknown: Vec<String> = obj
                    .keys()
                    .filter(|key| !allowed.contains(key.as_str()))
                    .cloned()
                    .collect();
                if !unknown.is_empty() {
                    bail!(
                        "{record_path}: motion analysis {} has unknown fields {:?}",
                        position + 1,
                        unknown
                    );
                }
                let missing: Vec<&str> = MOTION_ANALYSIS_FIELDS
                    .iter()
                    .filter(|field| !obj.contains_key(**field))
                    .copied()
                    .collect();
                if !missing.is_empty() {
                    bail!(
                        "{record_path}: motion analysis {} omits {:?}",
                        position + 1,
                        missing
                    );
                }
                if let Some(timing) = item.get("timing_class") {
                    if !timing.is_null() {
                        let timing = timing.as_str().unwrap_or("");
                        if !TIMING_CLASSES.contains(&timing) {
                            bail!(
                                "{record_path}: timing class '{timing}' is not one of {TIMING_CLASSES:?}"
                            );
                        }
                    }
                }
            }
        }
        _ => {
            if evidence_status_of(record) == Some("complete") {
                bail!("{record_path}: complete evidence needs motion_analysis");
            }
        }
    }

    let accessibility = record.get("accessibility").unwrap_or(&Value::Null);
    let observations = accessibility.get("observations");
    let unknowns = accessibility.get("unknowns");
    if !observations.map(Value::is_array).unwrap_or(false)
        || !unknowns.map(Value::is_array).unwrap_or(false)
    {
        bail!("{record_path}: accessibility observations and unknowns are required");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Catalog loading
// ---------------------------------------------------------------------------

fn load_full_references(slug: &str, examples: &[Value]) -> Result<Value> {
    let catalog_dir = PathBuf::from(slug);
    let index_path = catalog_dir.join("references.json");
    let index_path_str = index_path.to_string_lossy().to_string();
    let mut index: Value = lib::read_json(index_path.to_str().unwrap())?;

    require_nonempty(
        &index,
        &["schema", "catalog", "reference_count", "references"],
        &index_path_str,
    )?;
    if index.get("schema").and_then(Value::as_str) != Some(INDEX_SCHEMA) {
        bail!(
            "{index_path_str}: expected schema '{INDEX_SCHEMA}', found {}",
            python_repr(index.get("schema"))
        );
    }
    if index.get("catalog").and_then(Value::as_str) != Some(slug) {
        bail!("{index_path_str}: catalog must equal directory name");
    }

    let records: Vec<Value> = index
        .get("references")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if records.len() != examples.len() {
        bail!(
            "{index_path_str}: {} references for {} curated examples",
            records.len(),
            examples.len()
        );
    }
    if index.get("reference_count").and_then(Value::as_u64) != Some(records.len() as u64) {
        bail!("{index_path_str}: reference_count does not match the reference list");
    }

    let mut provenance: BTreeMap<String, usize> = BTreeMap::new();
    let mut statuses: BTreeMap<String, usize> = BTreeMap::new();
    let mut gap_total = 0usize;

    for (position, (entry, example)) in records.iter().zip(examples.iter()).enumerate() {
        let position = position + 1;
        require_nonempty(
            entry,
            &["index", "name", "path", "evidence_status"],
            &index_path_str,
        )?;
        if entry.get("index").and_then(Value::as_u64) != Some(position as u64)
            || entry.get("name") != example.get("name")
        {
            bail!("{index_path_str}: reference {position} does not match sources.json");
        }
        let entry_status = entry
            .get("evidence_status")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !EVIDENCE_STATUSES.contains(&entry_status) {
            bail!("{index_path_str}: reference {position} has status '{entry_status}'");
        }

        let entry_path = entry.get("path").and_then(Value::as_str).unwrap_or("");
        let record_path = resolve_evidence_path(&catalog_dir, entry_path, &index_path_str)?;
        let record_path_str = record_path.to_string_lossy().to_string();
        let record: Value = lib::read_json(record_path.to_str().unwrap())?;
        if record.get("schema").and_then(Value::as_str) != Some(RECORD_SCHEMA) {
            bail!("{record_path_str}: expected schema '{RECORD_SCHEMA}'");
        }
        if record.get("name") != example.get("name")
            || record.get("product_url") != example.get("source_url")
        {
            bail!("{record_path_str}: product identity differs from sources.json");
        }

        let Some(gaps) = record.get("evidence_gaps").and_then(Value::as_array) else {
            bail!("{record_path_str}: evidence_gaps must be a list, empty when nothing is missing");
        };
        let expected = if gaps.is_empty() {
            "complete"
        } else {
            "partial"
        };
        if evidence_status_of(&record) != Some(expected) {
            bail!(
                "{record_path_str}: status {} contradicts {} recorded gaps",
                python_repr(record.get("evidence_status")),
                gaps.len()
            );
        }
        if entry_status != expected
            || entry.get("evidence_gap_count").and_then(Value::as_u64) != Some(gaps.len() as u64)
        {
            bail!("{index_path_str}: reference {position} disagrees with its record");
        }

        let reference_dir = record_path.parent().unwrap_or(Path::new("."));
        for class in validate_motion(&record, &record_path_str, reference_dir)? {
            *provenance.entry(class).or_insert(0) += 1;
        }
        validate_states(&record, &record_path_str, reference_dir)?;
        validate_behaviour(&record, &record_path_str)?;

        *statuses.entry(expected.to_string()).or_insert(0) += 1;
        gap_total += gaps.len();
    }

    let obj = index.as_object_mut().expect("index is an object");
    obj.insert(
        "measured_provenance".into(),
        provenance
            .iter()
            .map(|(k, v)| (k.clone(), json!(v)))
            .collect::<Map<String, Value>>()
            .into(),
    );
    obj.insert("measured_gap_total".into(), json!(gap_total));
    obj.insert(
        "locally_driven_count".into(),
        json!(provenance
            .iter()
            .filter(|(name, _)| LOCAL_PROVENANCE.contains(&name.as_str()))
            .map(|(_, count)| count)
            .sum::<usize>()),
    );
    Ok(index)
}

/// Missing-key list formatted the way Python formats lists of strings.
fn missing_list(fields: &[&str]) -> String {
    let quoted: Vec<String> = fields.iter().map(|f| format!("'{f}'")).collect();
    format!("[{}]", quoted.join(", "))
}

fn load_catalog(slug: &str) -> Result<Value> {
    let source_path = Path::new(slug).join("sources.json");
    let source_path_str = source_path.to_string_lossy().to_string();
    let mut catalog: Value = lib::read_json(source_path.to_str().unwrap())?;

    if catalog.get("schema").and_then(Value::as_str) != Some(CATALOG_SCHEMA) {
        bail!("{source_path_str}: expected schema '{CATALOG_SCHEMA}'");
    }
    if catalog.get("catalog").and_then(Value::as_str) != Some(slug) {
        bail!("{source_path_str}: catalog must equal directory name");
    }

    let examples: Vec<Value> = catalog
        .get("examples")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| anyhow!("{source_path_str}: examples must be a list"))?;

    if examples.is_empty() && catalog.get("status").and_then(Value::as_str) == Some("scaffolded") {
        // A scaffolded catalog is an intentional empty shell: the contract
        // requires records before they are indexed, so an empty type renders
        // with zero counts and its named evidence gaps.
        return Ok(json!({
            "catalog": slug,
            "slug": slug,
            "title": catalog.get("title").cloned().unwrap_or_else(|| json!(slug)),
            "description": catalog.get("description").cloned().unwrap_or_else(|| json!("")),
            "count": 0,
            "image_count": 0,
            "structure_count": 0,
            "complete_record_count": 0,
            "partial_record_count": 0,
            "visual_count": 0,
            "curated_at": catalog.get("curated_at").cloned().unwrap_or_else(|| json!("unknown")),
            "measured_provenance": {},
            "full_reference_catalog": {
                "measured_provenance": {},
                "complete_count": 0,
                "partial_count": 0,
                "locally_driven_count": 0,
                "measured_gap_total": 0,
                "reference_count": 0,
            },
            "source": format!("{slug}/sources.json"),
            "full_reference_source": format!("{slug}/references.json"),
            "scaffolded": true,
            "examples": [],
        }));
    }

    for key in ["count", "visual_count", "structure_count"] {
        if catalog.get(key).and_then(Value::as_u64) != Some(examples.len() as u64) {
            bail!(
                "{source_path_str}: {key} does not match the {} examples",
                examples.len()
            );
        }
    }

    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut urls: BTreeSet<String> = BTreeSet::new();
    let catalog_dir = Path::new(slug);
    for (offset, example) in examples.iter().enumerate() {
        let index = offset + 1;
        let required = [
            "name",
            "source_url",
            "category",
            "selection_note",
            "visual",
            "interface_structure",
        ];
        let missing: Vec<&str> = required
            .iter()
            .filter(|key| !py_truthy(example.get(**key)))
            .copied()
            .collect();
        if !missing.is_empty() {
            bail!(
                "{source_path_str}: example {index} is missing {}",
                missing_list(&missing)
            );
        }
        let source_url = example
            .get("source_url")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !url_ok(source_url) {
            bail!("{source_path_str}: example {index} has an invalid URL");
        }
        let name = example.get("name").and_then(Value::as_str).unwrap_or("");
        let folded_name = name.to_lowercase();
        if !names.insert(folded_name) {
            bail!("{source_path_str}: duplicate example name '{name}'");
        }
        if !urls.insert(source_url.to_string()) {
            bail!("{source_path_str}: duplicate source URL '{source_url}'");
        }

        let visual = example.get("visual").expect("checked above");
        let mut visual_required: Vec<&str> = vec![
            "source_page_url",
            "local_path",
            "capture_kind",
            "captured_at",
            "format",
            "width",
            "height",
            "bytes",
            "sha256",
        ];
        let capture_kind = visual
            .get("capture_kind")
            .and_then(Value::as_str)
            .unwrap_or("");
        if capture_kind == "local-terminal-render" {
            visual_required.push("source_recording_path");
        } else if capture_kind != "local-browser-screenshot" {
            visual_required.push("source_image_url");
        }
        let visual_missing: Vec<&str> = visual_required
            .iter()
            .filter(|key| !py_truthy(visual.get(**key)))
            .copied()
            .collect();
        if !visual_missing.is_empty() {
            bail!(
                "{source_path_str}: example {index} visual is missing {}",
                missing_list(&visual_missing)
            );
        }
        let visual_local = visual
            .get("local_path")
            .and_then(Value::as_str)
            .unwrap_or("");
        let image_path = catalog_dir.join(visual_local);
        let contained = resolve_evidence_path(catalog_dir, visual_local, &source_path_str).is_ok();
        if !contained || !image_path.is_file() {
            bail!("{source_path_str}: example {index} visual path is unavailable");
        }
        let payload = std::fs::read(&image_path)?;
        if Some(payload.len() as u64) != visual.get("bytes").and_then(Value::as_u64) {
            bail!("{source_path_str}: example {index} visual byte count differs");
        }
        if Some(lib::sha256_hex(&payload).as_str()) != visual.get("sha256").and_then(Value::as_str)
        {
            bail!("{source_path_str}: example {index} visual digest differs");
        }

        let structure = example.get("interface_structure").expect("checked above");
        let structure_required = [
            "analysis_kind",
            "image_sha256",
            "orientation",
            "layout_model",
            "panel_summary",
            "regions",
            "detected_separators",
            "visual_density",
            "confidence",
        ];
        // Python tests structure.get(key) in (None, "") here: zero and false pass.
        let structure_missing: Vec<&str> = structure_required
            .iter()
            .filter(|key| {
                matches!(structure.get(**key), None | Some(Value::Null))
                    || structure.get(**key).and_then(Value::as_str) == Some("")
            })
            .copied()
            .collect();
        if !structure_missing.is_empty() {
            bail!(
                "{source_path_str}: example {index} structure is missing {}",
                missing_list(&structure_missing)
            );
        }
        if structure.get("image_sha256") != visual.get("sha256") {
            bail!("{source_path_str}: example {index} structure describes another image");
        }
        let regions_ok = structure
            .get("regions")
            .and_then(Value::as_array)
            .map(|r| !r.is_empty())
            .unwrap_or(false);
        if !regions_ok {
            bail!("{source_path_str}: example {index} has no structural regions");
        }
    }

    catalog
        .as_object_mut()
        .expect("catalog is an object")
        .insert(
            "full_reference_catalog".into(),
            load_full_references(slug, &examples)?,
        );
    Ok(catalog)
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn escape_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

/// Comma-separated "N label" sentence, strongest-provenance first.
fn provenance_sentence(measured_provenance: &Value) -> String {
    let mut pairs: Vec<(String, usize)> = measured_provenance
        .as_object()
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as usize))
                .collect()
        })
        .unwrap_or_default();
    pairs.sort_by(|a, b| b.1.cmp(&a.1)); // stable: ties stay in stored order
    if pairs.is_empty() {
        return "no measured motion".into();
    }
    pairs
        .iter()
        .map(|(name, count)| format!("{count} {}", provenance_label(name)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn full_reference_index<'a>(catalog: &'a Value) -> &'a Value {
    catalog
        .get("full_reference_catalog")
        .expect("attached by loader")
}



// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn discovered_catalogs() -> Vec<String> {
    let mut known: Vec<String> = CATALOGS.iter().map(|s| s.to_string()).collect();
    let mut found: Vec<String> = std::fs::read_dir(".")
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with("-examples") && Path::new(name).is_dir())
        .collect();
    found.sort();
    for name in found {
        if !known.contains(&name) {
            known.push(name);
        }
    }
    known.retain(|slug| Path::new(slug).join("references").is_dir());
    known
}

pub fn run(rest: &[String]) -> Result<()> {
    let mut check = false;
    for arg in rest {
        match arg.as_str() {
            "--check" => check = true,
            "--help" | "-h" => {
                println!("usage: spis generate-example-catalogs [--check]");
                println!("  --check  validate only, write nothing");
                return Ok(());
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    let slugs = discovered_catalogs();
    if slugs.is_empty() {
        bail!("no catalogs with measured references were found");
    }
    let mut catalogs: Vec<Value> = Vec::new();
    for slug in &slugs {
        catalogs.push(load_catalog(slug)?);
    }

    if check {
        for catalog in &catalogs {
            let index = full_reference_index(catalog);
            println!(
                "{}: {} complete, {} partial, {}",
                catalog["catalog"].as_str().unwrap_or("?"),
                index["complete_count"],
                index["partial_count"],
                provenance_sentence(&index["measured_provenance"])
            );
        }
        return Ok(());
    }


    let generated_at = catalogs
        .iter()
        .filter_map(|c| c["curated_at"].as_str())
        .max()
        .expect("curated_at present on every loaded catalog");

    let catalog_entries: Vec<Value> = catalogs
        .iter()
        .map(|catalog| {
            let index = full_reference_index(catalog);
            let slug = catalog["catalog"].as_str().unwrap_or("");
            json!({
                "slug": slug,
                "title": catalog["title"],
                "description": catalog["description"],
                "count": catalog["count"],
                "image_count": catalog["visual_count"],
                "structure_count": catalog["structure_count"],
                "complete_record_count": index["complete_count"],
                "partial_record_count": index["partial_count"],
                "measured_provenance": index["measured_provenance"],
                "source": format!("{slug}/sources.json"),
                "full_reference_source": format!("{slug}/references.json"),
            })
        })
        .collect();

    let index = json!({
        "schema": CATALOG_SCHEMA,
        "generated_at": generated_at,
        "catalog_count": catalogs.len(),
        "example_count": catalogs.iter().map(|c| c["count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "image_count": catalogs.iter().map(|c| c["visual_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "structure_count": catalogs.iter().map(|c| c["structure_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "record_count": catalogs.iter().map(|c| full_reference_index(c)["reference_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "complete_record_count": catalogs.iter().map(|c| full_reference_index(c)["complete_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "partial_record_count": catalogs.iter().map(|c| full_reference_index(c)["partial_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "locally_driven_motion_count": catalogs.iter().map(|c| full_reference_index(c)["locally_driven_count"].as_u64().unwrap_or(0)).sum::<u64>(),
        "catalogs": catalog_entries,
    });
    lib::write_pretty_json("example-catalogs.json", &index)?;
    Ok(())
}
