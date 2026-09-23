use super::*;

pub fn canonical_timing_class(value: Option<&str>) -> Option<&'static str> {
    let value = value?;
    let normalized = value.trim().to_ascii_lowercase();
    // Return the canonical spelling with original casing semantics preserved by
    // the caller; the vocabulary itself is lowercase.
    if is_timing_class(&normalized) {
        return Some(match normalized.as_str() {
            "instant" => "instant",
            "sub-second" => "sub-second",
            "one-to-three-seconds" => "one-to-three-seconds",
            "multi-second" => "multi-second",
            _ => "continuous",
        });
    }
    TIMING_CLASS_ALIASES
        .iter()
        .find(|(alias, _)| *alias == normalized)
        .map(|(_, canonical)| *canonical)
}

pub const JOURNEY_FIELDS: &[&str] = &[
    "actor",
    "goal",
    "prerequisites",
    "steps",
    "failure_route",
    "recovery_route",
    "completion_evidence",
];

pub const JOURNEY_STEP_FIELDS: &[&str] = &[
    "index",
    "user_action",
    "system_response",
    "state",
    "evidence",
];

pub const RECORD_FIELDS: &[&str] = &[
    "schema",
    "name",
    "product_url",
    "evidence_status",
    "evidence_gaps",
    "upstream_owner",
    "captured_at",
    "motion",
    "motion_provenance",
    "states",
    "interactions",
    "journey",
    "accessibility",
];

pub fn is_evidence_status(value: &str) -> bool {
    matches!(value, "complete" | "partial")
}

#[derive(Clone, Copy, Debug)]
pub struct CompletenessRequirements {
    pub profile: &'static str,
    pub min_motion_seconds: f64,
    pub min_states: usize,
    pub min_journey_steps: usize,
    pub min_interactions: usize,
    pub min_accessibility_observations: usize,
}

/// Return the explicit completeness profile for a catalog or a path inside it.
///
/// The concepts remain meaningful for all current families, so none receives a
/// convenience exception. Profiles are named so a future family whose real
/// surface cannot express a criterion must define a measurable replacement
/// here rather than filling a global field with prose.
pub fn completeness_requirements(path: &std::path::Path) -> CompletenessRequirements {
    let catalog = path.components().find_map(|component| {
        let value = component.as_os_str().to_str()?;
        value.ends_with("-examples").then_some(value)
    }).unwrap_or_default();
    let profile = match catalog {
        "cli-examples" | "tui-examples" => "terminal-product",
        "documentation-site-examples" | "app-store-listing-examples"
        | "pricing-page-examples" | "landing-page-examples" => "document-navigation",
        _ => "interactive-product",
    };
    CompletenessRequirements {
        profile,
        min_motion_seconds: 0.2,
        min_states: 3,
        min_journey_steps: 5,
        min_interactions: 8,
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
