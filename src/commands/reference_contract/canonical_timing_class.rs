use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The canonical timing class `value` names, in any case and spacing, or
/// nothing when it names none.
pub fn canonical_timing_class(value: Option<&str>) -> Option<String> {
    let class: TimingClass = named(&value?.trim().to_lowercase())?;
    Some(spelling(&class))
}

/// The field names a record object of shape `T` must carry, read from the
/// declared shape itself.
pub fn field_names<T: Default + Serialize>() -> Vec<String> {
    match serde_json::to_value(T::default()) {
        Ok(Value::Object(fields)) => fields.keys().cloned().collect(),
        other => unreachable!("a record shape serializes as an object, not {other:?}"),
    }
}

/// The head of a catalog's `references.json`.
#[derive(Default, Serialize)]
pub struct ReferenceIndex {
    pub schema: Value,
    pub catalog: Value,
    pub reference_count: Value,
}

/// One reference listed in a catalog's `references.json`.
#[derive(Default, Serialize)]
pub struct ReferenceIndexEntry {
    pub index: Value,
    pub name: Value,
    pub path: Value,
    pub evidence_status: Value,
}

/// One motion asset of a record.
#[derive(Default, Serialize)]
pub struct MotionEntry {
    pub local_path: Value,
    pub source_url: Value,
    pub media_kind: Value,
    pub bytes: Value,
    pub sha256: Value,
    pub capture_method: Value,
    pub provenance_class: Value,
}

/// The pixel size of a visual asset; a terminal cast has none.
#[derive(Default, Serialize)]
pub struct Dimensions {
    pub width: Value,
    pub height: Value,
}

/// One state image of a record.
#[derive(Default, Serialize)]
pub struct StateEntry {
    pub local_path: Value,
    pub width: Value,
    pub height: Value,
    pub bytes: Value,
    pub sha256: Value,
}

/// One mapped interaction of a record.
#[derive(Default, Serialize)]
pub struct Interaction {
    pub name: Value,
    pub trigger: Value,
    pub response: Value,
    pub feedback: Value,
    pub cancellation: Value,
    pub failure: Value,
    pub recovery: Value,
    pub evidence: Value,
}

/// A record's onboarding or first-success journey.
#[derive(Default, Serialize)]
pub struct Journey {
    pub actor: Value,
    pub goal: Value,
    pub prerequisites: Value,
    pub steps: Value,
    pub failure_route: Value,
    pub recovery_route: Value,
    pub completion_evidence: Value,
}

/// One ordered step of a journey.
#[derive(Default, Serialize)]
pub struct JourneyStep {
    pub index: Value,
    pub user_action: Value,
    pub system_response: Value,
    pub state: Value,
    pub evidence: Value,
}

/// The analysis of a record's motion. Older records spelled two fields
/// differently; those spellings are read as the canonical field.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MotionAnalysis {
    pub trigger: Value,
    pub start_state: Value,
    pub end_state: Value,
    pub continuity: Value,
    pub timing_class: Value,
    #[serde(alias = "interruption_reversal", alias = "interruption_and_reversal")]
    pub interruption_or_reversal: Value,
    pub feedback: Value,
    #[serde(alias = "reduced_motion_or_nonanimated_equivalent")]
    pub reduced_motion_equivalent: Value,
}

/// What a motion analysis may carry beyond its required fields.
#[derive(Default, Serialize)]
pub struct MotionAnalysisExtras {
    pub source_title: Value,
    pub evidence: Value,
    pub timing_description: Value,
    pub provenance: Value,
}

/// The canonical motion-analysis field an older spelling `key` stands for,
/// or nothing when `key` is no older spelling.
pub fn canonical_analysis_field(key: &str) -> Option<String> {
    let mut probe = serde_json::Map::new();
    probe.insert(key.to_string(), Value::Bool(true));
    let read: MotionAnalysis = serde_json::from_value(Value::Object(probe)).ok()?;
    match serde_json::to_value(read) {
        Ok(Value::Object(fields)) => fields
            .into_iter()
            .find(|(_, value)| value.as_bool() == Some(true))
            .map(|(field, _)| field)
            .filter(|field| field != key),
        _ => None,
    }
}

// The completeness floors below are the reference-record contract:
// https://spis.wisent.com/docs/concept-reference-record
// "A complete record needs at least one measured motion asset of at least
// 0.2 seconds, three state images, eight mapped interactions, five ordered
// journey steps, a complete motion analysis, and at least three
// accessibility observations."

/// A motion asset of fewer frames is a still image: "measured at 0.2 seconds
/// or longer with at least two frames"
/// (https://spis.wisent.com/docs/evidence-contract).
pub const MIN_MOTION_FRAMES: i64 = 2;

/// Every interface family holds "exactly 50 sources and records"
/// (https://spis.wisent.com/docs/concept-catalog).
pub const RECORDS_PER_CATALOG: usize = 50;

#[derive(Clone, Copy, Debug)]
pub struct CompletenessRequirements {
    pub profile: &'static str,
    pub min_motion_seconds: f64,
    pub min_states: usize,
    pub min_journey_steps: usize,
    pub min_interactions: usize,
    pub min_accessibility_observations: usize,
}

/// The completeness profile for a catalog or a path inside it: the family's
/// own, and for a catalog type added later (not a [`Family`]) the
/// interactive-product profile, whose floors every family shares.
pub fn completeness_requirements(path: &std::path::Path) -> CompletenessRequirements {
    let family = path
        .components()
        .find_map(|component| named::<Family>(component.as_os_str().to_str()?));
    let profile = match family {
        Some(family) => family.profile(),
        None => "interactive-product",
    };
    // https://spis.wisent.com/docs/concept-reference-record
    CompletenessRequirements {
        profile,
        // https://spis.wisent.com/docs/concept-reference-record
        min_motion_seconds: 0.2,
        // https://spis.wisent.com/docs/concept-reference-record
        min_states: 3,
        // https://spis.wisent.com/docs/concept-reference-record
        min_journey_steps: 5,
        // https://spis.wisent.com/docs/concept-reference-record
        min_interactions: 8,
        // https://spis.wisent.com/docs/concept-reference-record
        min_accessibility_observations: 3,
    }
}

/// Regenerate rendered catalog metadata through the compiled Rust generator.
///
/// Both mutating subcommands call this after every change so records and indexes
/// cannot drift onto a second implementation path.
pub fn regenerate_index() -> Result<()> {
    super::generate_example_catalogs::run(&[])
}
