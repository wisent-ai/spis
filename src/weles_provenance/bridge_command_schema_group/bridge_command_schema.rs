use super::*;

pub const BRIDGE_COMMAND_SCHEMA: &str = "wisent.spis-weles-bridge-command.v1";

pub const BRIDGE_CONFIG_SCHEMA: &str = "wisent.spis-weles-bridge-config.v1";

pub const BRIDGE_TRUST_SCHEMA: &str = "wisent.spis-weles-receipt-trust.v1";

pub const RECEIPT_CHECKPOINT_SCHEMA: &str = "wisent.spis-weles-receipt-checkpoint.v1";

pub const SUBMISSION_SCHEMA: &str = "wisent.spis-weles-submission.v1";

pub const TASK_STATUS_SCHEMA: &str = "wisent.spis-weles-task-status.v1";

pub const CANCELLATION_SCHEMA: &str = "wisent.spis-weles-cancellation.v1";

pub const PROVENANCE_DOCUMENT_SCHEMA: &str = "wisent.spis-weles-provenance.v1";

pub const PROVENANCE_DOCUMENT_REF_SCHEMA: &str =
    "wisent.spis-weles-provenance-document-ref.v1";

pub const PROVENANCE_LINK_SCHEMA: &str = "wisent.spis-provenance-link.v1";

pub const ATTEMPT_BINDING_SCHEMA: &str = "weles.spis-browser-evidence-binding.v1";

pub const ATTEMPT_ENVELOPE_SCHEMA: &str = "wisent.spis-weles-attempt-envelope.v1";

pub const SPIS_WELES_ACTION: &str = "generic_browser_task";

/// The one outcome that means the browser task produced its evidence. Everything that
/// decides whether a record's material counts as CONFIRMED stays bound to this value
/// alone: see `VerifiedProvenanceSet::supports_value`.
pub const SUCCESSFUL_OUTCOME: &str = "completed";

/// Every terminal outcome a Weles receipt can carry.
///
/// This is deliberately WIDER than what the deployed admission service can emit. Its
/// status vocabulary is pinned to `queued`/`running` plus `succeeded`/`failed`/`cancelled`,
/// which it maps to `completed`/`failed`/`cancelled`, so `rejected` is unreachable there
/// today. The list is kept identical to the bridge's `TERMINAL_OUTCOME_BY_STATUS` values
/// (`weles-bridge/spis-weles-bridge.mjs`) on purpose: a receipt the pinned bridge accepts
/// must never be refused here for a reason the bridge does not know, and the producer now
/// refuses to serialize any status outside its pinned vocabulary, so a new one fails
/// loudly at that boundary instead of arriving here unannounced. Read this as the set
/// this repository is willing to verify, NOT as a description of what the service emits.
pub const TERMINAL_OUTCOMES: &[&str] = &["completed", "failed", "cancelled", "rejected"];

pub fn is_terminal_outcome(value: &str) -> bool {
    TERMINAL_OUTCOMES.contains(&value)
}

pub const OFFICIAL_CLIENT_PACKAGE: &str = "@wisent-ai/weles-client";

pub const OFFICIAL_CLIENT_COMMIT: &str =
    "37798a26022a040fbd0a4a4a25c99b5559d95a32";

pub(crate) const BRIDGE_SCRIPT_SHA256: &str = env!("SPIS_BRIDGE_SCRIPT_SHA256");

pub(crate) const MAX_BRIDGE_SCRIPT_BYTES: u64 = 256 * 1024;

pub(crate) const MAX_DOCUMENT_BYTES: u64 = 4 * 1024 * 1024;

pub(crate) const MAX_TRUST_BYTES: u64 = 64 * 1024;

pub(crate) const MAX_RETAINED_EVIDENCE_BYTES: u64 = 8 * 1024 * 1024;

pub(crate) const MAX_BRIDGE_ERROR_BYTES: usize = 64 * 1024;

/// Local re-verification is CPU work over retained bytes.
pub const VERIFY_BRIDGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// `submit`, `get` and `cancel` are real HTTP round trips through the official client.
pub const NETWORK_BRIDGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

pub(crate) const BRIDGE_PATH: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpectedReceiptClaims {
    pub task_id: String,
    pub organization_id: String,
    pub request_digest: String,
    pub result_digest: String,
    pub spis_binding: WelesAttemptBinding,
    pub origin: String,
    pub action: String,
    pub outcome: String,
    pub evidence_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerifiedReceiptClaims {
    pub task_id: String,
    pub organization_id: String,
    pub origin: String,
    pub request_digest: String,
    pub result_digest: String,
    pub spis_binding: WelesAttemptBinding,
    pub action: String,
    pub outcome: String,
    pub evidence_digest: String,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetainedReceipt {
    pub schema: String,
    pub task_id: String,
    pub organization_id: String,
    pub origin: String,
    pub action: String,
    pub outcome: String,
    pub evidence_digest: String,
    pub request_digest: String,
    pub result_digest: String,
    pub spis_binding: WelesAttemptBinding,
    pub key_id: String,
    pub signature: String,
    pub signed_payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetainedArtifact {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WelesOfficialTaskInput {
    pub product_url: String,
    pub objective: String,
    pub constraints: Vec<String>,
    #[serde(rename = "spisBinding")]
    pub spis_binding: WelesAttemptBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesOfficialTaskRequest {
    pub schema: String,
    pub organization_id: String,
    pub origin: String,
    pub action: String,
    pub input: WelesOfficialTaskInput,
    pub credential_refs: Vec<String>,
    pub evidence_policy: String,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WelesEvidenceInventoryEntry {
    pub kind: String,
    pub uri: String,
    pub sha256: String,
    pub bytes: u64,
}

/// The receipt-bound manifest of a SUCCEEDED task: `weles.browser-evidence-manifest.v1`.
/// `deny_unknown_fields` plus every field being required is what pins the shape, so a v2
/// document can never be read as this one.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReceiptBoundEvidenceManifest {
    pub(crate) schema: String,
    pub(crate) task_id: String,
    pub(crate) organization_id: String,
    pub(crate) origin: String,
    pub(crate) action: String,
    pub(crate) outcome: String,
    pub(crate) request_digest: String,
    pub(crate) result_digest: String,
    pub(crate) spis_binding: WelesAttemptBinding,
    pub(crate) requested_url: String,
    pub(crate) effective_url: String,
    pub(crate) final_url: String,
    pub(crate) evidence_inventory: Vec<WelesEvidenceInventoryEntry>,
}

/// The receipt-bound manifest of a FAILED, CANCELLED or REJECTED task:
/// `weles.browser-evidence-manifest.v2`. There is no navigation to sign, so the effective
/// and final URL are ABSENT rather than optional: together with `deny_unknown_fields`,
/// their absence from this struct is what refuses a v1 document here and refuses a v2
/// document that carries them.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NonSuccessEvidenceManifest {
    pub(crate) schema: String,
    pub(crate) task_id: String,
    pub(crate) organization_id: String,
    pub(crate) origin: String,
    pub(crate) action: String,
    pub(crate) outcome: String,
    pub(crate) request_digest: String,
    pub(crate) result_digest: String,
    pub(crate) spis_binding: WelesAttemptBinding,
    pub(crate) requested_url: String,
    pub(crate) evidence_inventory: Vec<WelesEvidenceInventoryEntry>,
}

/// One retained manifest in whichever version its terminal outcome mandates.
pub(crate) enum ReceiptBoundManifest {
    Successful(ReceiptBoundEvidenceManifest),
    NonSuccess(NonSuccessEvidenceManifest),
}

/// The fields every version carries, plus the navigation pair only the successful version
/// signs, so the checks below are written once instead of per version.
pub(crate) struct EvidenceManifestFacts<'a> {
    pub(crate) schema: &'a str,
    pub(crate) expected_schema: &'static str,
    pub(crate) task_id: &'a str,
    pub(crate) organization_id: &'a str,
    pub(crate) origin: &'a str,
    pub(crate) action: &'a str,
    pub(crate) outcome: &'a str,
    pub(crate) request_digest: &'a str,
    pub(crate) result_digest: &'a str,
    pub(crate) spis_binding: &'a WelesAttemptBinding,
    pub(crate) requested_url: &'a str,
    pub(crate) navigation: Option<(&'a str, &'a str)>,
    pub(crate) evidence_inventory: &'a [WelesEvidenceInventoryEntry],
}
