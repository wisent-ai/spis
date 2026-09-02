//! Fail-closed Weles receipt provenance for retained Spis evidence.
//!
//! The Rust verifier never treats a JSON boolean, a verifier label, a receipt-provided
//! key, or a caller-chosen correlation ID as trust. It re-runs the checked-in Node bridge,
//! which loads the exact pinned official `@wisent-ai/weles-client`, then independently
//! rechecks the returned claims, receipt identity, and retained artifact digest here.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(unix)]
use std::os::unix::process::CommandExt;

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
const BRIDGE_SCRIPT_SHA256: &str = env!("SPIS_BRIDGE_SCRIPT_SHA256");
const MAX_BRIDGE_SCRIPT_BYTES: u64 = 256 * 1024;

const MAX_DOCUMENT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TRUST_BYTES: u64 = 64 * 1024;
const MAX_RETAINED_EVIDENCE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_BRIDGE_ERROR_BYTES: usize = 64 * 1024;
/// Local re-verification is CPU work over retained bytes.
pub const VERIFY_BRIDGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
/// `submit`, `get` and `cancel` are real HTTP round trips through the official client.
pub const NETWORK_BRIDGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
const BRIDGE_PATH: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin";

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
struct ReceiptBoundEvidenceManifest {
    schema: String,
    task_id: String,
    organization_id: String,
    origin: String,
    action: String,
    outcome: String,
    request_digest: String,
    result_digest: String,
    spis_binding: WelesAttemptBinding,
    requested_url: String,
    effective_url: String,
    final_url: String,
    evidence_inventory: Vec<WelesEvidenceInventoryEntry>,
}

/// The receipt-bound manifest of a FAILED, CANCELLED or REJECTED task:
/// `weles.browser-evidence-manifest.v2`. There is no navigation to sign, so the effective
/// and final URL are ABSENT rather than optional: together with `deny_unknown_fields`,
/// their absence from this struct is what refuses a v1 document here and refuses a v2
/// document that carries them.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NonSuccessEvidenceManifest {
    schema: String,
    task_id: String,
    organization_id: String,
    origin: String,
    action: String,
    outcome: String,
    request_digest: String,
    result_digest: String,
    spis_binding: WelesAttemptBinding,
    requested_url: String,
    evidence_inventory: Vec<WelesEvidenceInventoryEntry>,
}

/// One retained manifest in whichever version its terminal outcome mandates.
enum ReceiptBoundManifest {
    Successful(ReceiptBoundEvidenceManifest),
    NonSuccess(NonSuccessEvidenceManifest),
}

/// The fields every version carries, plus the navigation pair only the successful version
/// signs, so the checks below are written once instead of per version.
struct EvidenceManifestFacts<'a> {
    schema: &'a str,
    expected_schema: &'static str,
    task_id: &'a str,
    organization_id: &'a str,
    origin: &'a str,
    action: &'a str,
    outcome: &'a str,
    request_digest: &'a str,
    result_digest: &'a str,
    spis_binding: &'a WelesAttemptBinding,
    requested_url: &'a str,
    navigation: Option<(&'a str, &'a str)>,
    evidence_inventory: &'a [WelesEvidenceInventoryEntry],
}

impl ReceiptBoundManifest {
    /// Parses the retained artifact in exactly the version the signed outcome mandates.
    fn parse(value: &Value, outcome: &str) -> Result<Self, String> {
        if outcome == SUCCESSFUL_OUTCOME {
            serde_json::from_value(value.clone())
                .map(Self::Successful)
                .map_err(|_| {
                    "receipt-bound evidence manifest does not match the typed schema".to_string()
                })
        } else {
            serde_json::from_value(value.clone())
                .map(Self::NonSuccess)
                .map_err(|_| {
                    "receipt-bound non-success evidence manifest does not match the typed schema"
                        .to_string()
                })
        }
    }

    fn facts(&self) -> EvidenceManifestFacts<'_> {
        match self {
            Self::Successful(manifest) => EvidenceManifestFacts {
                schema: &manifest.schema,
                expected_schema: "weles.browser-evidence-manifest.v1",
                task_id: &manifest.task_id,
                organization_id: &manifest.organization_id,
                origin: &manifest.origin,
                action: &manifest.action,
                outcome: &manifest.outcome,
                request_digest: &manifest.request_digest,
                result_digest: &manifest.result_digest,
                spis_binding: &manifest.spis_binding,
                requested_url: &manifest.requested_url,
                navigation: Some((&manifest.effective_url, &manifest.final_url)),
                evidence_inventory: &manifest.evidence_inventory,
            },
            Self::NonSuccess(manifest) => EvidenceManifestFacts {
                schema: &manifest.schema,
                expected_schema: "weles.browser-evidence-manifest.v2",
                task_id: &manifest.task_id,
                organization_id: &manifest.organization_id,
                origin: &manifest.origin,
                action: &manifest.action,
                outcome: &manifest.outcome,
                request_digest: &manifest.request_digest,
                result_digest: &manifest.result_digest,
                spis_binding: &manifest.spis_binding,
                requested_url: &manifest.requested_url,
                navigation: None,
                evidence_inventory: &manifest.evidence_inventory,
            },
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OfficialClientIdentity {
    pub package: String,
    pub commit: String,
    pub key_set_version: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WelesServiceIdentity {
    pub name: String,
    pub generation: u64,
    pub consumer: String,
    pub capability: String,
    pub active_host: String,
    pub endpoint: String,
    pub action: String,
    pub release_id: String,
    pub source_revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesReceiptTrust {
    pub schema: String,
    pub organization_id: String,
    pub allowed_action: String,
    pub receipt_keys: BTreeMap<String, String>,
    pub key_set_version: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesRequestIdentity {
    pub request_digest: String,
    pub spis_binding: WelesAttemptBinding,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesReceiptCheckpoint {
    pub schema: String,
    pub client: OfficialClientIdentity,
    pub receipt: RetainedReceipt,
    pub claims: VerifiedReceiptClaims,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesSubmission {
    pub schema: String,
    pub task_id: String,
    pub organization_id: String,
    pub origin: String,
    pub action: String,
    pub service_identity: WelesServiceIdentity,
    pub idempotency_key: String,
    pub request_digest: String,
    pub request_document: WelesOfficialTaskRequest,
    pub request_identity: WelesRequestIdentity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_checkpoint: Option<WelesReceiptCheckpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesTaskStatus {
    pub schema: String,
    pub task_id: String,
    pub organization_id: String,
    pub origin: String,
    pub action: String,
    pub service_identity: WelesServiceIdentity,
    pub request_identity: WelesRequestIdentity,
    pub result_digest: Option<String>,
    pub status: String,
    pub terminal: bool,
    pub outcome: Option<String>,
    pub result_ref: Option<String>,
    pub artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_checkpoint: Option<WelesReceiptCheckpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesCancellation {
    pub schema: String,
    pub task_id: String,
    pub organization_id: String,
    pub origin: String,
    pub action: String,
    pub service_identity: WelesServiceIdentity,
    pub request_identity: WelesRequestIdentity,
    pub result_digest: Option<String>,
    pub status: String,
    pub terminal: bool,
    pub outcome: Option<String>,
    pub result_ref: Option<String>,
    pub artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_checkpoint: Option<WelesReceiptCheckpoint>,
    pub idempotency_key: String,
}


/// The deterministic document written by `weles-bridge/spis-weles-bridge.mjs`.
///
/// `claims` and `client` are retained audit material, not authority. Rust accepts this
/// document only after the bridge produces a fresh document from `receipt`,
/// `expected_claims`, and `artifact`, and all fields below independently agree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesProvenanceDocument {
    pub schema: String,
    pub id: String,
    pub client: OfficialClientIdentity,
    pub receipt: RetainedReceipt,
    pub claims: VerifiedReceiptClaims,
    pub expected_claims: ExpectedReceiptClaims,
    pub artifact: RetainedArtifact,
}

/// Record-level reference to one bridge-produced verification document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesProvenanceDocumentRef {
    pub schema: String,
    pub path: String,
    pub sha256: String,
}
/// Canonical Spis identity signed inside the Weles receipt and copied into the
/// receipt-bound JSON artifact as `spisBinding`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WelesAttemptBindingService {
    pub name: String,
    pub consumer: String,
    pub capability: String,
    pub directory_generation: u64,
    pub host: String,
    pub endpoint: String,
    pub action: String,
    pub release_id: String,
    pub source_revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WelesAttemptBinding {
    pub schema: String,
    pub run_id: String,
    pub catalog: String,
    pub record: String,
    pub record_key: String,
    pub attempt: u32,
    pub attempt_id: String,
    pub source_revision: String,
    pub source_input_sha256: String,
    pub reference_sha256: String,
    pub artifact_uri: String,
    pub output_uri: String,
    pub service: WelesAttemptBindingService,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WelesAttemptEnvelope {
    pub schema: String,
    pub run_id: String,
    pub catalog: String,
    pub record: String,
    pub record_key: String,
    pub attempt: u32,
    pub attempt_id: String,
    pub stado_job_id: String,
    pub weles_task_id: String,
    pub state: String,
    pub outcome: Option<String>,
    pub service_identity: WelesServiceIdentity,
    pub source_revision: String,
    pub source_input_sha256: String,
    pub reference_sha256: String,
    pub spis_binding: WelesAttemptBinding,
    pub weles_request_document: WelesOfficialTaskRequest,
    pub weles_request_digest: String,
    pub weles_result_digest: Option<String>,
    pub requested_url: String,
    /// `Some` only for a completed attempt: a non-success signs no navigation at all, and
    /// the v2 evidence manifest has no final URL for this field to be compared against.
    pub final_url: Option<String>,
    pub evidence_inventory: Vec<WelesEvidenceInventoryEntry>,
    pub weles_evidence_manifest_uri: String,
    pub weles_evidence_manifest_sha256: Option<String>,
    pub artifact_document_uri: String,
    pub artifact_document_sha256: Option<String>,
    pub observation_document_uri: String,
    pub observation_document_sha256: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProvenanceLinkKind {
    /// The record value names the exact signed artifact file and digest.
    Artifact,
    /// The record value is exactly a JSON value inside the signed artifact.
    Observation,
}

/// A record or observation link to independently verified evidence.
///
/// Observation links require an RFC 6901 JSON pointer and the canonical digest of the
/// pointed-to value. The verifier also compares that value with the record value after
/// recursively removing `provenance` links, so correlation JSON cannot establish trust.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WelesProvenanceLink {
    pub schema: String,
    pub kind: ProvenanceLinkKind,
    pub document_id: String,
    pub artifact_path: String,
    pub artifact_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_pointer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_sha256: Option<String>,
}

#[derive(Debug, Clone)]
struct VerifiedDocument {
    id: String,
    /// The signed terminal outcome this document proves. A document is verified as a
    /// DOCUMENT for every terminal outcome, but only `SUCCESSFUL_OUTCOME` may support a
    /// claim about confirmed source material.
    outcome: String,
    artifact: RetainedArtifact,
    artifact_value: Option<Value>,
}
/// The one public trust document, already read and validated by this process.
///
/// Every bridge operation needs it: the verifier compares persisted claims against it,
/// and any producer has to hand the child the exact bytes it validated rather than
/// letting the child pick its own trust. It is opaque outside this module.
#[derive(Debug, Clone)]
pub struct CanonicalTrust {
    path: PathBuf,
    bytes: Vec<u8>,
    document: WelesReceiptTrust,
}

impl CanonicalTrust {
    /// Reads and validates the checked-in public trust document. This is the gate that
    /// fail-closes every operation while the trust document is unprovisioned.
    pub fn load() -> Result<Self, String> {
        load_canonical_trust()
    }
}


/// Result of verifying every `provenance_documents` reference in one record.
#[derive(Debug, Clone, Default)]
pub struct VerifiedProvenanceSet {
    documents: BTreeMap<String, VerifiedDocument>,
    failures: Vec<String>,
    record_dir: Option<PathBuf>,
}

impl VerifiedProvenanceSet {
    /// Re-run official receipt verification for every referenced document.
    ///
    /// A malformed reference, missing vendored client/public trust/key, unknown key,
    /// claim mismatch, bridge failure, changed artifact, or invalid document remains
    /// in `failures` and never enters the verified set.
    pub fn verify_record(record: &Value, record_dir: &Path) -> Self {
        let mut verified = Self {
            record_dir: Some(record_dir.to_path_buf()),
            ..Self::default()
        };
        let Some(references_value) = record.get("provenance_documents") else {
            return verified;
        };
        let Some(references) = references_value.as_array() else {
            verified
                .failures
                .push("provenance_documents is not an array".to_string());
            return verified;
        };
        if references.is_empty() {
            return verified;
        }
        let trust = match load_canonical_trust() {
            Ok(trust) => trust,
            Err(reason) => {
                verified
                    .failures
                    .push(format!("public receipt trust: {reason}"));
                return verified;
            }
        };
        for (index, reference_value) in references.iter().enumerate() {
            let result =
                verify_document_reference(reference_value, record, record_dir, &trust);
            match result {
                Ok(document) => {
                    if verified.documents.contains_key(&document.id) {
                        verified.failures.push(format!(
                            "provenance document {index} repeats a verified document ID"
                        ));
                    } else {
                        verified.documents.insert(document.id.clone(), document);
                    }
                }
                Err(reason) => verified
                    .failures
                    .push(format!("provenance document {index}: {reason}")),
            }
        }
        verified
    }

    pub fn failures(&self) -> &[String] {
        &self.failures
    }

    /// True only when the value's typed link resolves to freshly verified receipt and
    /// artifact bytes, the link is independently bound to this exact value, AND the
    /// document proves the successful outcome.
    ///
    /// THIS IS THE BOUNDARY. A failure provenance document is verified as a document —
    /// its receipt, claims, manifest, envelope and retained bytes are all re-proved — but
    /// it proves that the browser task did NOT produce evidence, so it can never support a
    /// claim that a record's material is confirmed. Both consumers of this predicate
    /// (`verify_reference_evidence` and `generate_example_catalogs`) classify and admit
    /// material through here and through `provenance_class`, so binding the support to
    /// `SUCCESSFUL_OUTCOME` in this one place keeps a non-success out of every confirmed
    /// claim without weakening its own verification.
    pub fn supports_value(&self, value: &Value) -> bool {
        let Some(link_value) = value.get("provenance") else {
            return false;
        };
        let Ok(link) = serde_json::from_value::<WelesProvenanceLink>(link_value.clone()) else {
            return false;
        };
        if link.schema != PROVENANCE_LINK_SCHEMA
            || !is_sha256_id(&link.document_id)
            || !is_sha256(&link.artifact_sha256)
        {
            return false;
        }
        let Some(document) = self.documents.get(&link.document_id) else {
            return false;
        };
        // Verified, and still not evidence of anything having been captured.
        if document.outcome != SUCCESSFUL_OUTCOME {
            return false;
        }
        if link.artifact_path != document.artifact.path
            || link.artifact_sha256 != document.artifact.sha256
        {
            return false;
        }
        match link.kind {
            ProvenanceLinkKind::Artifact => {
                if link.artifact_pointer.is_some() || link.value_sha256.is_some() {
                    return false;
                }
                value.get("local_path").and_then(Value::as_str)
                    == Some(document.artifact.path.as_str())
                    && value.get("sha256").and_then(Value::as_str)
                        == Some(document.artifact.sha256.as_str())
                    && self.retained_member_matches(value)
            }
            ProvenanceLinkKind::Observation => {
                let (Some(pointer), Some(expected_digest), Some(artifact_value)) = (
                    link.artifact_pointer.as_deref(),
                    link.value_sha256.as_deref(),
                    document.artifact_value.as_ref(),
                ) else {
                    return false;
                };
                if !pointer.starts_with('/') || !is_sha256(expected_digest) {
                    return false;
                }
                let Some(source_value) = artifact_value.pointer(pointer) else {
                    return false;
                };
                let stripped = strip_provenance(value);
                canonical_json_sha256(source_value)
                    .is_ok_and(|digest| digest == expected_digest)
                    && *source_value == stripped
                    && self.retained_member_matches(value)
            }
        }
    }

    fn retained_member_matches(&self, value: &Value) -> bool {
        let local_path = value.get("local_path").and_then(Value::as_str);
        let expected_sha256 = value.get("sha256").and_then(Value::as_str);
        match (local_path, expected_sha256) {
            (None, None) => true,
            (Some(local_path), Some(expected_sha256)) if is_sha256(expected_sha256) => {
                let Some(record_dir) = self.record_dir.as_deref() else {
                    return false;
                };
                let Ok(path) = resolve_retained_file(record_dir, local_path) else {
                    return false;
                };
                let Ok(actual_sha256) = sha256_file(&path) else {
                    return false;
                };
                if actual_sha256 != expected_sha256 {
                    return false;
                }
                match value.get("bytes").and_then(Value::as_u64) {
                    Some(expected_bytes) => fs::metadata(path)
                        .map(|metadata| metadata.len() == expected_bytes)
                        .unwrap_or(false),
                    None => true,
                }
            }
            _ => false,
        }
    }

    pub fn provenance_class(&self, value: &Value) -> &'static str {
        if self.supports_value(value) {
            "weles-signed-browser-evidence"
        } else {
            "unverified-source-media"
        }
    }
}

fn verify_document_reference(
    reference_value: &Value,
    record: &Value,
    record_dir: &Path,
    trust: &CanonicalTrust,
) -> Result<VerifiedDocument, String> {
    let reference: WelesProvenanceDocumentRef =
        serde_json::from_value(reference_value.clone())
            .map_err(|_| "reference does not match the typed schema".to_string())?;
    if reference.schema != PROVENANCE_DOCUMENT_REF_SCHEMA {
        return Err("reference schema is unsupported".to_string());
    }
    if !is_sha256(&reference.sha256) {
        return Err("reference sha256 is not a lowercase SHA-256 digest".to_string());
    }
    let document_path = resolve_retained_file(record_dir, &reference.path)?;
    let bytes = read_limited(&document_path, MAX_DOCUMENT_BYTES)?;
    if sha256_bytes(&bytes) != reference.sha256 {
        return Err("verification document digest does not match its reference".to_string());
    }
    let persisted: WelesProvenanceDocument = serde_json::from_slice(&bytes)
        .map_err(|_| "verification document does not match the typed schema".to_string())?;
    validate_document_shape(&persisted)?;
    validate_document_trust(&persisted, &trust.document)?;
    let fresh = invoke_bridge(&persisted, record_dir, trust)?;
    validate_fresh_document(&persisted, &fresh, record_dir)?;
    let artifact_path = resolve_retained_file(record_dir, &fresh.artifact.path)?;
    let artifact_bytes = read_limited(&artifact_path, MAX_DOCUMENT_BYTES)?;
    if sha256_bytes(&artifact_bytes) != fresh.artifact.sha256 {
        return Err("receipt-bound JSON artifact changed while it was being parsed".to_string());
    }
    let artifact_value: Value = serde_json::from_slice(&artifact_bytes)
        .map_err(|_| "receipt-bound artifact is not the required signed JSON document".to_string())?;
    verify_attempt_binding(record, record_dir, &fresh, &artifact_value, &trust.document)?;
    Ok(VerifiedDocument {
        id: fresh.id,
        // Proved by `validate_document_shape` to be a terminal outcome and to be the
        // identical value in the retained receipt and the expected claims.
        outcome: fresh.expected_claims.outcome.clone(),
        artifact: fresh.artifact,
        artifact_value: Some(artifact_value),
    })
}
fn verify_attempt_binding(
    record: &Value,
    record_dir: &Path,
    document: &WelesProvenanceDocument,
    artifact_value: &Value,
    trust: &WelesReceiptTrust,
) -> Result<(), String> {
    let product_url = record
        .get("product_url")
        .and_then(Value::as_str)
        .ok_or_else(|| "current record has no product_url for receipt origin binding".to_string())?;
    let parsed_product_url = url::Url::parse(product_url)
        .map_err(|_| "current record product_url is not a valid URL".to_string())?;
    if !matches!(parsed_product_url.scheme(), "http" | "https") {
        return Err("current record product_url is not HTTP(S)".to_string());
    }
    if document.expected_claims.origin != parsed_product_url.origin().ascii_serialization() {
        return Err(
            "verified receipt origin does not match the current record product_url origin"
                .to_string(),
        );
    }
    if document.expected_claims.action != SPIS_WELES_ACTION
        || document.expected_claims.action != trust.allowed_action
    {
        return Err("verified receipt action is not the trusted Spis browser action".to_string());
    }

    let record_name = record_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "current record directory has no portable record name".to_string())?;
    let catalog_name = record_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .ok_or_else(|| "current record directory has no portable catalog name".to_string())?;
    let runs = record
        .get("crawl_runs")
        .and_then(Value::as_array)
        .ok_or_else(|| "current record has no imported crawl-run attempts".to_string())?;
    let mut matched: Option<(WelesAttemptEnvelope, &Value)> = None;
    for run in runs {
        let Some(envelope_value) = run.get("weles_attempt_envelope") else {
            continue;
        };
        let envelope: WelesAttemptEnvelope = serde_json::from_value(envelope_value.clone())
            .map_err(|_| {
                "imported Weles attempt envelope does not match the typed schema".to_string()
            })?;
        if envelope.weles_task_id != document.expected_claims.task_id {
            continue;
        }
        if matched.is_some() {
            return Err("current record repeats the receipt task across attempts".to_string());
        }
        matched = Some((envelope, run));
    }
    let Some((envelope, run)) = matched else {
        return Err("verified receipt taskId is not the imported inner Weles task".to_string());
    };
    let outcome = document.expected_claims.outcome.as_str();
    let successful = outcome == SUCCESSFUL_OUTCOME;
    let mut required = vec![
        envelope.run_id.as_str(),
        envelope.catalog.as_str(),
        envelope.record.as_str(),
        envelope.record_key.as_str(),
        envelope.attempt_id.as_str(),
        envelope.stado_job_id.as_str(),
        envelope.weles_task_id.as_str(),
        envelope.source_revision.as_str(),
        envelope.requested_url.as_str(),
        envelope.weles_evidence_manifest_uri.as_str(),
        envelope.artifact_document_uri.as_str(),
        envelope.observation_document_uri.as_str(),
    ];
    // A completed attempt signs a navigation and must name its final URL; a non-success
    // has none to name, and the v2 manifest has no field to compare it against, so the
    // envelope must leave it out rather than fill it in.
    match (successful, envelope.final_url.as_deref()) {
        (true, Some(final_url)) => required.push(final_url),
        (false, None) => {}
        _ => {
            return Err(
                "attempt envelope final URL is present exactly when the outcome is completed"
                    .to_string(),
            )
        }
    }
    if envelope.schema != ATTEMPT_ENVELOPE_SCHEMA
        || envelope.attempt == 0
        || !is_git_revision(&envelope.source_revision)
        || !is_sha256(&envelope.record_key)
        || !is_sha256(&envelope.source_input_sha256)
        || !is_sha256(&envelope.reference_sha256)
        || required.iter().any(|value| value.trim().is_empty())
        || envelope.stado_job_id == envelope.weles_task_id
        // The attempt reports exactly the outcome the receipt signed, under both names.
        || !is_terminal_outcome(&envelope.state)
        || envelope.state != outcome
        || envelope.outcome.as_deref() != Some(outcome)
        || !is_sha256_id(&envelope.weles_request_digest)
        || !envelope
            .weles_result_digest
            .as_deref()
            .is_some_and(is_sha256_id)
        || !envelope
            .weles_evidence_manifest_sha256
            .as_deref()
            .is_some_and(is_sha256)
        || !envelope
            .artifact_document_sha256
            .as_deref()
            .is_some_and(is_sha256)
        || !is_sha256(&envelope.observation_document_sha256)
    {
        return Err(
            "imported Weles attempt envelope is not a typed attempt of the signed outcome"
                .to_string(),
        );
    }
    let weles_evidence_manifest_sha256 = envelope
        .weles_evidence_manifest_sha256
        .as_deref()
        .expect("validated terminal Weles evidence manifest digest");
    let artifact_document_sha256 = envelope
        .artifact_document_sha256
        .as_deref()
        .expect("validated terminal artifact document digest");
    if envelope.catalog != catalog_name
        || envelope.record != record_name
        || run.get("run_id").and_then(Value::as_str) != Some(envelope.run_id.as_str())
        || run.get("stado_job_id").and_then(Value::as_str)
            != Some(envelope.stado_job_id.as_str())
        || run.get("record_key").and_then(Value::as_str) != Some(envelope.record_key.as_str())
        || run.get("attempt").and_then(Value::as_u64) != Some(u64::from(envelope.attempt))
        || run.get("attempt_id").and_then(Value::as_str) != Some(envelope.attempt_id.as_str())
        || run.get("state").and_then(Value::as_str) != Some(envelope.state.as_str())
        || run.get("outcome").and_then(Value::as_str) != envelope.outcome.as_deref()
        || run.get("source_revision").and_then(Value::as_str)
            != Some(envelope.source_revision.as_str())
        || run.get("source_input_sha256").and_then(Value::as_str)
            != Some(envelope.source_input_sha256.as_str())
        || run.get("reference_sha256").and_then(Value::as_str)
            != Some(envelope.reference_sha256.as_str())
        || run.get("weles_evidence_manifest_uri").and_then(Value::as_str)
            != Some(envelope.weles_evidence_manifest_uri.as_str())
        || run.get("weles_evidence_manifest_sha256").and_then(Value::as_str)
            != Some(weles_evidence_manifest_sha256)
        || run.get("artifact_document_uri").and_then(Value::as_str)
            != Some(envelope.artifact_document_uri.as_str())
        || run.get("artifact_document_sha256").and_then(Value::as_str)
            != Some(artifact_document_sha256)
        || run.get("observation_document_uri").and_then(Value::as_str)
            != Some(envelope.observation_document_uri.as_str())
        || run.get("observation_document_sha256").and_then(Value::as_str)
            != Some(envelope.observation_document_sha256.as_str())
    {
        return Err(
            "outer typed crawl run differs from the imported Weles attempt coordinates"
                .to_string(),
        );
    }
    if envelope.service_identity.action != SPIS_WELES_ACTION
        || envelope.service_identity.action != trust.allowed_action
    {
        return Err("attempt service identity action differs from public receipt trust".to_string());
    }
    validate_service_identity(&envelope.service_identity)?;
    validate_attempt_uris(&envelope)?;
    validate_spis_binding(&envelope.spis_binding)?;
    let expected_binding_service = WelesAttemptBindingService {
        name: envelope.service_identity.name.clone(),
        consumer: envelope.service_identity.consumer.clone(),
        capability: envelope.service_identity.capability.clone(),
        directory_generation: envelope.service_identity.generation,
        host: envelope.service_identity.active_host.clone(),
        endpoint: envelope.service_identity.endpoint.clone(),
        action: envelope.service_identity.action.clone(),
        release_id: envelope.service_identity.release_id.clone(),
        source_revision: envelope.service_identity.source_revision.clone(),
    };
    let binding = &envelope.spis_binding;
    if binding.run_id != envelope.run_id
        || binding.catalog != envelope.catalog
        || binding.record != envelope.record
        || binding.record_key != envelope.record_key
        || binding.attempt != envelope.attempt
        || binding.attempt_id != envelope.attempt_id
        || binding.source_revision != envelope.source_revision
        || binding.source_input_sha256 != envelope.source_input_sha256
        || binding.reference_sha256 != envelope.reference_sha256
        || binding.service != expected_binding_service
        || document.expected_claims.request_digest != envelope.weles_request_digest
        || document.expected_claims.result_digest
            != envelope
                .weles_result_digest
                .as_deref()
                .expect("validated terminal result digest")
        || document.expected_claims.spis_binding != *binding
        || document.artifact.sha256 != artifact_document_sha256
        || artifact_document_sha256 != weles_evidence_manifest_sha256
    {
        return Err(
            "verified receipt request/result/binding/artifact differs from the attempt envelope"
                .to_string(),
        );
    }
    // The signed outcome, already proved above to be the receipt's own claim, chooses the
    // manifest version. Nothing else may: a document that does not match the version its
    // outcome mandates is refused, in either direction.
    let manifest =
        ReceiptBoundManifest::parse(artifact_value, &document.expected_claims.outcome)?;
    validate_request_and_evidence_manifest(
        &envelope,
        binding,
        document,
        &parsed_product_url,
        &manifest,
        record_dir,
    )?;
    Ok(())
}


fn validate_attempt_uris(envelope: &WelesAttemptEnvelope) -> Result<(), String> {
    for (label, component) in [
        ("run_id", envelope.run_id.as_str()),
        ("catalog", envelope.catalog.as_str()),
        ("record", envelope.record.as_str()),
        ("attempt_id", envelope.attempt_id.as_str()),
        ("weles_task_id", envelope.weles_task_id.as_str()),
    ] {
        if !is_portable_attempt_component(component) {
            return Err(format!("{label} is not a portable attempt URI component"));
        }
    }
    let base = format!(
        "stado://spis-crawls/{}/{}/{}/{}/attempts/{}/{}",
        envelope.run_id,
        envelope.catalog,
        envelope.record,
        envelope.record_key,
        envelope.attempt,
        envelope.attempt_id,
    );
    let artifact_sha256 = envelope
        .artifact_document_sha256
        .as_deref()
        .expect("validated terminal artifact document digest");
    if envelope.spis_binding.artifact_uri != format!("{base}/artifacts.tar.gz")
        || envelope.spis_binding.output_uri != format!("{base}/worker-output.log")
        || envelope.weles_evidence_manifest_uri
            != format!(
                "stado://weles/recordings/{}/evidence-manifest.json",
                envelope.weles_task_id
            )
        || envelope.artifact_document_uri
            != format!("{base}/weles/artifacts/{artifact_sha256}.json")
        || envelope.observation_document_uri
            != format!(
                "{base}/weles/observations/{}.json",
                envelope.observation_document_sha256
            )
    {
        return Err("Weles attempt URI is not the canonical coordinate reconstruction".to_string());
    }
    Ok(())
}

fn is_portable_attempt_component(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn validate_request_and_evidence_manifest(
    envelope: &WelesAttemptEnvelope,
    binding: &WelesAttemptBinding,
    document: &WelesProvenanceDocument,
    product_url: &url::Url,
    manifest: &ReceiptBoundManifest,
    record_dir: &Path,
) -> Result<(), String> {
    let request = &envelope.weles_request_document;
    if request.schema != "weles.task.current"
        || request.organization_id != document.expected_claims.organization_id
        || request.origin != document.expected_claims.origin
        || request.origin != product_url.origin().ascii_serialization()
        || request.action != document.expected_claims.action
        || request.input.spis_binding != *binding
        || request.input.objective.trim().is_empty()
        || !request.credential_refs.is_empty()
        || request.evidence_policy != "full"
        || request.justification.trim().is_empty()
    {
        return Err("retained official request differs from signed attempt claims".to_string());
    }
    validate_unique_nonempty(&request.input.constraints, "request constraints")?;
    let request_value = serde_json::to_value(request)
        .map_err(|_| "retained official request could not be canonicalized".to_string())?;
    let request_digest = format!("sha256:{}", canonical_json_sha256(&request_value)?);
    if request_digest != envelope.weles_request_digest
        || request_digest != document.expected_claims.request_digest
    {
        return Err("canonical official request digest differs from the signed claim".to_string());
    }

    let requested_url = parse_http_url(&request.input.product_url, "requested product URL")?;
    let envelope_requested_url = parse_http_url(&envelope.requested_url, "attempt requested URL")?;
    // The envelope names a final URL exactly when the attempt completed, which
    // `verify_attempt_binding` already proved against the signed outcome.
    let envelope_final_origin_ok = match envelope.final_url.as_deref() {
        Some(final_url) => {
            parse_http_url(final_url, "attempt final URL")?.origin() == product_url.origin()
        }
        None => true,
    };
    if requested_url != *product_url
        || envelope_requested_url != *product_url
        || request.input.product_url != product_url.as_str()
        || request.input.product_url != envelope.requested_url
        || !envelope_final_origin_ok
    {
        return Err(
            "browser request/final URL differs from the canonical current product URL policy"
                .to_string(),
        );
    }

    validate_evidence_inventory(
        &envelope.evidence_inventory,
        &envelope.weles_task_id,
        record_dir,
        document.expected_claims.outcome == SUCCESSFUL_OUTCOME,
    )?;
    let facts = manifest.facts();
    let manifest_requested_url = parse_http_url(facts.requested_url, "manifest requested URL")?;
    // Only the successful version signs a navigation, and only it can be compared with the
    // envelope's final URL; the non-success version has neither field at all.
    let navigation_matches = match facts.navigation {
        Some((effective_url, final_url)) => {
            let manifest_effective_url = parse_http_url(effective_url, "manifest effective URL")?;
            let manifest_final_url = parse_http_url(final_url, "manifest final URL")?;
            manifest_effective_url.origin() == product_url.origin()
                && manifest_final_url.origin() == product_url.origin()
                && Some(final_url) == envelope.final_url.as_deref()
        }
        None => true,
    };
    if facts.schema != facts.expected_schema
        || facts.task_id != envelope.weles_task_id
        || facts.organization_id != document.expected_claims.organization_id
        || facts.origin != document.expected_claims.origin
        || facts.action != document.expected_claims.action
        || facts.outcome != document.expected_claims.outcome
        || facts.request_digest != document.expected_claims.request_digest
        || facts.result_digest != document.expected_claims.result_digest
        || *facts.spis_binding != *binding
        || manifest_requested_url != *product_url
        || facts.requested_url != envelope.requested_url
        || !navigation_matches
        || facts.evidence_inventory != envelope.evidence_inventory
    {
        return Err(
            "receipt-bound evidence manifest differs from the signed request/result/attempt"
                .to_string(),
        );
    }
    Ok(())
}

fn parse_http_url(value: &str, label: &str) -> Result<url::Url, String> {
    let parsed = url::Url::parse(value).map_err(|_| format!("{label} is invalid"))?;
    if !matches!(parsed.scheme(), "http" | "https")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(format!("{label} is not an HTTP(S) URL without credentials"));
    }
    Ok(parsed)
}

fn validate_unique_nonempty(values: &[String], label: &str) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    if values
        .iter()
        .any(|value| value.trim().is_empty() || !seen.insert(value.as_str()))
    {
        return Err(format!("{label} contain an empty or duplicate entry"));
    }
    Ok(())
}

fn validate_evidence_inventory(
    entries: &[WelesEvidenceInventoryEntry],
    task_id: &str,
    record_dir: &Path,
    require_browser_evidence: bool,
) -> Result<(), String> {
    let prefix = format!("stado://weles/recordings/{task_id}/");
    let screenshot_uri = format!("{prefix}artifacts/browser_evidence_final.png");
    let accessibility_uri =
        format!("{prefix}artifacts/browser_evidence_accessibility_tree.txt");
    let mut kinds = BTreeSet::new();
    let mut uris = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for entry in entries {
        let relative_uri = entry
            .uri
            .strip_prefix(&prefix)
            .ok_or_else(|| {
                "evidence inventory URI is not bound to the exact Weles task".to_string()
            })?;
        // Both receiver layers must judge the same document by the same rule: the bridge
        // requires every component to pass `portableAttemptComponent`, and the service
        // refuses anything outside that alphabet at retention time, so accepting a merely
        // `Component::Normal` name here would leave this layer the weaker of the two.
        if relative_uri.is_empty()
            || relative_uri.contains('\\')
            || !relative_uri.split('/').all(is_portable_attempt_component)
            || !Path::new(relative_uri)
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err("evidence inventory URI is not a canonical immutable path".to_string());
        }
        let kind_matches_uri = match entry.kind.as_str() {
            "screenshot" => entry.uri == screenshot_uri,
            "accessibility_tree" => entry.uri == accessibility_uri,
            kind => kind
                .strip_prefix("artifact:")
                .is_some_and(|path| path == relative_uri),
        };
        total_bytes = total_bytes
            .checked_add(entry.bytes)
            .filter(|total| *total <= MAX_RETAINED_EVIDENCE_BYTES)
            .ok_or_else(|| "retained evidence inventory exceeds the total byte limit".to_string())?;
        if !kind_matches_uri
            || !is_sha256(&entry.sha256)
            || entry.bytes == 0
            || !kinds.insert(entry.kind.as_str())
            || !uris.insert(entry.uri.as_str())
        {
            return Err("evidence inventory contains an invalid or duplicate entry".to_string());
        }
        let retained_path = format!("recordings/{task_id}/{relative_uri}");
        let retained_file = resolve_retained_file(record_dir, &retained_path)?;
        let retained_bytes = read_limited(&retained_file, entry.bytes)?;
        if retained_bytes.len() as u64 != entry.bytes
            || sha256_bytes(&retained_bytes) != entry.sha256
        {
            return Err("retained evidence bytes differ from the signed inventory".to_string());
        }
    }
    // Labelling above is by exact URI and outcome-independent; the DEMAND below applies
    // only to a completed attempt, exactly as the service demands them only from a
    // succeeded task and as the bridge and the worker already scope it. A cancelled task
    // that never captured anything signs an inventory that is legitimately without them.
    if require_browser_evidence
        && (!kinds.contains("screenshot") || !kinds.contains("accessibility_tree"))
    {
        return Err(
            "evidence inventory lacks the required screenshot/accessibility_tree artifacts"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_document_shape(document: &WelesProvenanceDocument) -> Result<(), String> {
    if document.schema != PROVENANCE_DOCUMENT_SCHEMA {
        return Err("verification document schema is unsupported".to_string());
    }
    if document.client.package != OFFICIAL_CLIENT_PACKAGE
        || document.client.commit != OFFICIAL_CLIENT_COMMIT
        || document.client.key_set_version.trim().is_empty()
    {
        return Err("verification document does not name the pinned official client and key set".to_string());
    }
    let receipt = &document.receipt;
    let expected = &document.expected_claims;
    if receipt.schema != "weles.receipt.current"
        || receipt.task_id.trim().is_empty()
        || receipt.organization_id.trim().is_empty()
        || receipt.origin.trim().is_empty()
        || receipt.action.trim().is_empty()
        || !is_terminal_outcome(&receipt.outcome)
        || receipt.evidence_digest.trim().is_empty()
        || receipt.key_id.trim().is_empty()
        || receipt.signature.trim().is_empty()
        || receipt.signed_payload.trim().is_empty()
        || !is_sha256_id(&receipt.request_digest)
        || !is_sha256_id(&receipt.result_digest)
    {
        return Err("retained receipt shape is unsupported".to_string());
    }
    if document.artifact.bytes == 0
        || document.artifact.bytes > MAX_DOCUMENT_BYTES
        || !is_sha256(&document.artifact.sha256)
        || expected.evidence_digest != document.artifact.sha256
    {
        return Err("expected evidenceDigest is not bound to a bounded retained artifact".to_string());
    }
    // Every terminal outcome is verifiable as a document. What the outcome is allowed to
    // support is decided in exactly one other place, `supports_value`, which admits only
    // `SUCCESSFUL_OUTCOME`; a failure proof must be provable without being promotable.
    if !is_terminal_outcome(&expected.outcome)
        || !is_sha256_id(&expected.request_digest)
        || !is_sha256_id(&expected.result_digest)
    {
        return Err(
            "Spis provenance requires a terminal outcome with signed request/result digests"
                .to_string(),
        );
    }
    validate_spis_binding(&expected.spis_binding)?;
    validate_spis_binding(&receipt.spis_binding)?;
    if receipt.task_id != expected.task_id
        || receipt.organization_id != expected.organization_id
        || receipt.origin != expected.origin
        || receipt.action != expected.action
        || receipt.outcome != expected.outcome
        || receipt.evidence_digest != expected.evidence_digest
        || receipt.request_digest != expected.request_digest
        || receipt.result_digest != expected.result_digest
        || receipt.spis_binding != expected.spis_binding
    {
        return Err("retained receipt claim copies differ from caller expectations".to_string());
    }
    Ok(())
}
fn validate_document_trust(
    document: &WelesProvenanceDocument,
    trust: &WelesReceiptTrust,
) -> Result<(), String> {
    if document.expected_claims.organization_id != trust.organization_id
        || document.expected_claims.action != trust.allowed_action
        || document.client.key_set_version != trust.key_set_version
        || !trust.receipt_keys.contains_key(&document.receipt.key_id)
    {
        return Err(
            "verification document differs from the checked-in public receipt trust"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_service_identity(identity: &WelesServiceIdentity) -> Result<(), String> {
    if identity.name != "weles-admission"
        || identity.consumer != "spis"
        || identity.capability != "browser-evidence"
        || identity.active_host.trim().is_empty()
        || identity.action != SPIS_WELES_ACTION
        || !identity.release_id.starts_with("weles-worker@")
        || identity.release_id == "weles-worker@"
        || !is_git_revision(&identity.source_revision)
    {
        return Err("attempt service identity is invalid".to_string());
    }
    validate_api_endpoint(&identity.endpoint, "attempt service identity endpoint")
}
fn validate_spis_binding(binding: &WelesAttemptBinding) -> Result<(), String> {
    let required = [
        binding.run_id.as_str(),
        binding.catalog.as_str(),
        binding.record.as_str(),
        binding.record_key.as_str(),
        binding.attempt_id.as_str(),
        binding.source_revision.as_str(),
        binding.artifact_uri.as_str(),
        binding.output_uri.as_str(),
        binding.service.name.as_str(),
        binding.service.consumer.as_str(),
        binding.service.capability.as_str(),
        binding.service.host.as_str(),
        binding.service.endpoint.as_str(),
        binding.service.release_id.as_str(),
        binding.service.source_revision.as_str(),
        binding.service.action.as_str(),
    ];
    if binding.schema != ATTEMPT_BINDING_SCHEMA
        || binding.attempt == 0
        || required.iter().any(|value| value.trim().is_empty())
        || !is_git_revision(&binding.source_revision)
        || !is_sha256(&binding.record_key)
        || !is_sha256(&binding.source_input_sha256)
        || !is_sha256(&binding.reference_sha256)
        || !is_git_revision(&binding.service.source_revision)
        || !is_portable_attempt_component(&binding.run_id)
        || !is_portable_attempt_component(&binding.catalog)
        || !is_portable_attempt_component(&binding.record)
        || !is_portable_attempt_component(&binding.attempt_id)
        || binding.service.name != "weles-admission"
        || binding.service.consumer != "spis"
        || binding.service.capability != "browser-evidence"
        || binding.service.action != SPIS_WELES_ACTION
        || !binding.service.release_id.starts_with("weles-worker@")
        || binding.service.release_id == "weles-worker@"
    {
        return Err("signed Spis binding is invalid".to_string());
    }
    validate_api_endpoint(&binding.service.endpoint, "signed Spis service endpoint")?;
    validate_attempt_binding_derivation(binding)?;
    let base = format!(
        "stado://spis-crawls/{}/{}/{}/{}/attempts/{}/{}",
        binding.run_id,
        binding.catalog,
        binding.record,
        binding.record_key,
        binding.attempt,
        binding.attempt_id,
    );
    if binding.artifact_uri != format!("{base}/artifacts.tar.gz")
        || binding.output_uri != format!("{base}/worker-output.log")
    {
        return Err("signed Spis artifact/output URIs are not canonical".to_string());
    }
    Ok(())
}

/// Re-derives the runtime record key and attempt identity exactly as the Weles
/// public admission service does, so the Rust layer never accepts a weaker
/// attempt binding than the runtime promises.
fn validate_attempt_binding_derivation(binding: &WelesAttemptBinding) -> Result<(), String> {
    let catalog_key = sha256_bytes(
        format!(
            "{}\0{}\0{}",
            binding.source_revision, binding.run_id, binding.catalog
        )
        .as_bytes(),
    );
    let record_key = sha256_bytes(
        format!(
            "{}\0{}\0{}",
            catalog_key, binding.record, binding.source_input_sha256
        )
        .as_bytes(),
    );
    if binding.record_key != record_key {
        return Err("signed Spis record key is not the runtime derivation".to_string());
    }
    let attempt_fingerprint = sha256_bytes(
        format!(
            "{}\0{}\0{}",
            binding.record_key, binding.attempt, binding.service.host
        )
        .as_bytes(),
    );
    if binding.attempt_id
        != format!(
            "attempt-{}-{}",
            binding.attempt,
            &attempt_fingerprint[..16]
        )
    {
        return Err("signed Spis attempt identity is not the runtime derivation".to_string());
    }
    Ok(())
}
fn validate_api_endpoint(value: &str, label: &str) -> Result<(), String> {
    let endpoint = url::Url::parse(value).map_err(|_| format!("{label} is invalid"))?;
    if !matches!(endpoint.scheme(), "http" | "https")
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.path() != "/api/v1"
        || endpoint.query().is_some()
        || endpoint.fragment().is_some()
        || endpoint.as_str() != value
    {
        return Err(format!("{label} is not the canonical exact /api/v1 base"));
    }
    Ok(())
}



fn load_canonical_trust() -> Result<CanonicalTrust, String> {
    let checked_in = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("weles-bridge")
        .join("weles-receipt-trust.json");
    let metadata = fs::symlink_metadata(&checked_in)
        .map_err(|_| "checked-in public trust document is absent".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("checked-in public trust document is not a regular file".to_string());
    }
    let canonical = fs::canonicalize(&checked_in)
        .map_err(|_| "checked-in public trust document could not be resolved".to_string())?;
    let bytes = read_limited(&canonical, MAX_TRUST_BYTES)?;
    let document: WelesReceiptTrust = serde_json::from_slice(&bytes)
        .map_err(|_| "public trust document does not match the typed schema".to_string())?;
    if document.schema != BRIDGE_TRUST_SCHEMA
        || document.organization_id.trim().is_empty()
        || document.allowed_action != SPIS_WELES_ACTION
        || document.key_set_version.trim().is_empty()
        || document.receipt_keys.is_empty()
        || document
            .receipt_keys
            .iter()
            .any(|(key, value)| key.trim().is_empty() || value.trim().is_empty())
    {
        return Err("public trust document is invalid".to_string());
    }
    Ok(CanonicalTrust {
        path: canonical,
        bytes,
        document,
    })
}


/// A typed bridge failure.
///
/// `code` is the bridge's own machine-readable code whenever the bridge reported for
/// itself, and a Rust-side code (`absent`, `unpinned`, `spawn-failed`, `timeout`,
/// `io-failed`) when it never got that far. `message` is the exact operator-facing text.
#[derive(Debug, Clone)]
pub struct BridgeFailure {
    pub code: String,
    pub message: String,
}

impl BridgeFailure {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

/// One exact invocation of the checked-in bridge.
///
/// The operation is carried by `command.operation`, not by this struct: `submit`, `get`,
/// `cancel` and `verify` differ only in the command document, the output destination,
/// whether a network credential is in play, and the wall-clock budget.
pub struct BridgeInvocation<'a> {
    /// `wisent.spis-weles-bridge-command.v1` document. The bridge runs a strict per
    /// operation key allowlist, so it is passed through exactly as serialized.
    pub command: &'a Value,
    /// The public trust document this process already validated. Its bytes are handed to
    /// the child, which re-checks them against the canonical file, so the child never
    /// gets to choose its own trust.
    pub trust: &'a CanonicalTrust,
    /// Process working directory. `verify` resolves the retained artifact against the
    /// record directory; `submit` resolves a relative `--output` against it.
    pub working_dir: &'a Path,
    /// `Some(path)` persists the document to that file, which `submit` requires for its
    /// request-bound recovery; `None` returns it on bounded stdout, which `get` requires.
    pub output: Option<&'a Path>,
    /// The owner-only protected config carrying the bearer. `None` is the secretless
    /// path: without it the bridge refuses every network operation, and `verify` never
    /// reads a config at all.
    pub config: Option<&'a Path>,
    /// Wall-clock budget for the whole child process.
    pub timeout: std::time::Duration,
}

/// Runs one bridge operation and returns its bounded stdout, which is empty when the
/// document was persisted to `output` instead.
///
/// The script is read, digest-pinned against the build-time embedded source digest, and
/// executed as verified bytes through a data URL, so no on-disk module is loaded by path.
/// The child runs in its own process group with a cleared environment: exactly `PATH`,
/// the canonical trust path, the verified trust bytes, the verified bridge directory and,
/// on the network path, the protected config path. The command document travels on stdin,
/// so no bridge command is ever left on disk.
pub fn run_bridge_command(invocation: &BridgeInvocation<'_>) -> Result<Vec<u8>, BridgeFailure> {
    let absent = |message: &str| BridgeFailure::new("absent", message);
    let script_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("weles-bridge")
        .join("spis-weles-bridge.mjs");
    let bridge_directory = fs::canonicalize(
        script_path
            .parent()
            .ok_or_else(|| absent("checked-in Weles bridge has no resource directory"))?,
    )
    .map_err(|_| absent("checked-in Weles bridge resource directory could not be resolved"))?;
    let script_metadata = fs::symlink_metadata(&script_path)
        .map_err(|_| absent("checked-in Weles bridge is absent"))?;
    if script_metadata.file_type().is_symlink() || !script_metadata.is_file() {
        return Err(absent("checked-in Weles bridge is not a regular non-symlink file"));
    }
    let script = fs::canonicalize(&script_path)
        .map_err(|_| absent("checked-in Weles bridge could not be resolved"))?;
    if script.parent() != Some(bridge_directory.as_path()) {
        return Err(absent(
            "checked-in Weles bridge escaped its canonical resource directory",
        ));
    }
    let script_file =
        fs::File::open(&script).map_err(|_| absent("checked-in Weles bridge could not be opened"))?;
    let script_bytes = read_stream_limited(
        script_file,
        MAX_BRIDGE_SCRIPT_BYTES as usize,
        "checked-in Weles bridge",
    )
    .map_err(|message| BridgeFailure::new("absent", message))?;
    if sha256_bytes(&script_bytes) != BRIDGE_SCRIPT_SHA256 {
        return Err(BridgeFailure::new(
            "unpinned",
            "checked-in Weles bridge differs from the embedded source pin",
        ));
    }
    let bridge_module = format!(
        "await import('data:text/javascript;base64,{}')",
        STANDARD.encode(&script_bytes)
    );
    let input_bytes = serde_json::to_vec(invocation.command)
        .map_err(|_| BridgeFailure::new("io-failed", "could not serialize the bridge command"))?;
    let mut command = Command::new("node");
    command
        .arg("--input-type=module")
        .arg("--eval")
        .arg(bridge_module)
        .arg("--")
        .arg("spis-weles-bridge.mjs")
        .arg("--input")
        .arg("-")
        .arg("--output");
    match invocation.output {
        Some(path) => command.arg(path),
        None => command.arg("-"),
    };
    command
        .current_dir(invocation.working_dir)
        .env_clear()
        .env("PATH", BRIDGE_PATH)
        .env("SPIS_WELES_TRUST_FILE", &invocation.trust.path)
        .env(
            "SPIS_WELES_VERIFIED_TRUST_BASE64",
            STANDARD.encode(&invocation.trust.bytes),
        )
        .env("SPIS_WELES_VERIFIED_BRIDGE_DIR", bridge_directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(config) = invocation.config {
        command.env("SPIS_WELES_CONFIG_FILE", config);
    }
    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    let mut child = command.spawn().map_err(|_| {
        BridgeFailure::new(
            "spawn-failed",
            "could not start Node for the checked-in Weles bridge",
        )
    })?;
    let started = std::time::Instant::now();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| BridgeFailure::new("io-failed", "Weles bridge stdout was unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| BridgeFailure::new("io-failed", "Weles bridge stderr was unavailable"))?;
    let stdout_reader = std::thread::spawn(move || {
        read_stream_limited(stdout, MAX_DOCUMENT_BYTES as usize, "stdout")
    });
    let stderr_reader = std::thread::spawn(move || {
        read_stream_limited(stderr, MAX_BRIDGE_ERROR_BYTES, "stderr")
    });
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| BridgeFailure::new("io-failed", "Weles bridge stdin was unavailable"))?;
    let stdin_writer = std::thread::spawn(move || {
        stdin
            .write_all(&input_bytes)
            .map_err(|_| "could not send the command document to the Weles bridge".to_string())
    });
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < invocation.timeout => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Ok(None) => {
                // The whole group: the official client may itself be waiting on a socket.
                terminate_bridge_process_group(&mut child);
                let _ = stdin_writer.join();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(BridgeFailure::new(
                    "timeout",
                    format!(
                        "official Weles bridge exceeded the {}-second deadline",
                        invocation.timeout.as_secs()
                    ),
                ));
            }
            Err(_) => {
                terminate_bridge_process_group(&mut child);
                let _ = stdin_writer.join();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(BridgeFailure::new(
                    "io-failed",
                    "could not collect the Weles bridge result",
                ));
            }
        }
    };
    let stdin_result = stdin_writer.join();
    let stdout_result = stdout_reader.join();
    let stderr_result = stderr_reader.join();
    if !status.success() {
        let stderr = match stderr_result {
            Ok(Ok(stderr)) => stderr,
            _ => Vec::new(),
        };
        let code = bridge_error_code(&stderr);
        return Err(BridgeFailure {
            message: format!("official Weles bridge failed closed ({code})"),
            code,
        });
    }
    let io_failed = |message: String| BridgeFailure::new("io-failed", message);
    stdin_result
        .map_err(|_| io_failed("official Weles bridge stdin writer failed".to_string()))?
        .map_err(io_failed)?;
    let stdout = stdout_result
        .map_err(|_| io_failed("official Weles bridge stdout reader failed".to_string()))?
        .map_err(io_failed)?;
    stderr_result
        .map_err(|_| io_failed("official Weles bridge stderr reader failed".to_string()))?
        .map_err(io_failed)?;
    Ok(stdout)
}

fn invoke_bridge(
    persisted: &WelesProvenanceDocument,
    record_dir: &Path,
    trust: &CanonicalTrust,
) -> Result<WelesProvenanceDocument, String> {
    let command = serde_json::json!({
        "schema": BRIDGE_COMMAND_SCHEMA,
        "operation": "verify",
        "receipt": persisted.receipt,
        "expectedClaims": persisted.expected_claims,
        "artifact": persisted.artifact,
    });
    let stdout = run_bridge_command(&BridgeInvocation {
        command: &command,
        trust,
        working_dir: record_dir,
        output: None,
        // Re-verification is secretless: it re-reads retained bytes and the public trust
        // document, and must never be able to reach the network.
        config: None,
        timeout: VERIFY_BRIDGE_TIMEOUT,
    })
    .map_err(|failure| failure.message)?;
    serde_json::from_slice(&stdout)
        .map_err(|_| "official Weles bridge returned a malformed verification document".to_string())
}

fn terminate_bridge_process_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    unsafe {
        if libc::killpg(child.id() as libc::pid_t, libc::SIGKILL) == -1 {
            let _ = child.kill();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}

fn validate_fresh_document(
    persisted: &WelesProvenanceDocument,
    fresh: &WelesProvenanceDocument,
    record_dir: &Path,
) -> Result<(), String> {
    validate_document_shape(fresh)?;
    if fresh != persisted {
        return Err(
            "fresh official verification differs from the persisted verification document"
                .to_string(),
        );
    }
    if !claims_match_expected(&fresh.claims, &fresh.expected_claims)
        || fresh.claims.key_id != fresh.receipt.key_id
    {
        return Err("fresh verified claims do not exactly match caller expectations".to_string());
    }
    let artifact_path = resolve_retained_file(record_dir, &fresh.artifact.path)?;
    let metadata = fs::metadata(&artifact_path)
        .map_err(|_| "retained artifact metadata is unavailable".to_string())?;
    if metadata.len() != fresh.artifact.bytes {
        return Err("retained artifact byte count changed".to_string());
    }
    let actual_artifact_digest = sha256_file(&artifact_path)?;
    if actual_artifact_digest != fresh.artifact.sha256
        || fresh.claims.evidence_digest != actual_artifact_digest
    {
        return Err("fresh receipt is not bound to the retained artifact bytes".to_string());
    }
    let expected_id = provenance_id(
        &fresh.receipt,
        &fresh.client.key_set_version,
        &fresh.artifact,
    )?;
    if fresh.id != expected_id || !is_sha256_id(&fresh.id) {
        return Err("verification document ID is not derived from verified receipt material".to_string());
    }
    Ok(())
}

fn claims_match_expected(
    claims: &VerifiedReceiptClaims,
    expected: &ExpectedReceiptClaims,
) -> bool {
    claims.task_id == expected.task_id
        && claims.organization_id == expected.organization_id
        && claims.origin == expected.origin
        && claims.action == expected.action
        && claims.outcome == expected.outcome
        && claims.evidence_digest == expected.evidence_digest
        && claims.request_digest == expected.request_digest
        && claims.result_digest == expected.result_digest
        && claims.spis_binding == expected.spis_binding
}

fn bridge_error_code(stderr: &[u8]) -> String {
    let Ok(value) = serde_json::from_slice::<Value>(stderr) else {
        return "bridge-error".to_string();
    };
    value
        .get("code")
        .and_then(Value::as_str)
        .filter(|code| {
            !code.is_empty()
                && code.len() <= 64
                && code
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
        .unwrap_or("bridge-error")
        .to_string()
}

fn resolve_retained_file(base: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty() || relative.contains('\\') {
        return Err("retained path must be a portable relative path".to_string());
    }
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("retained path escapes the record directory".to_string());
    }
    let canonical_base = fs::canonicalize(base)
        .map_err(|_| "record directory could not be resolved".to_string())?;
    let joined = canonical_base.join(relative_path);
    let link_metadata = fs::symlink_metadata(&joined)
        .map_err(|_| "retained file is absent".to_string())?;
    if link_metadata.file_type().is_symlink() || !link_metadata.is_file() {
        return Err("retained path is not a regular non-symlink file".to_string());
    }
    let canonical_file = fs::canonicalize(&joined)
        .map_err(|_| "retained file could not be resolved".to_string())?;
    if !canonical_file.starts_with(&canonical_base) {
        return Err("retained file resolves outside the record directory".to_string());
    }
    Ok(canonical_file)
}

fn read_stream_limited(
    reader: impl Read,
    limit: usize,
    label: &str,
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| format!("{label} could not be read"))?;
    if bytes.len() > limit {
        return Err(format!("{label} exceeded the size limit"));
    }
    Ok(bytes)
}

fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|_| "retained file could not be opened".to_string())?;
    read_stream_limited(file, limit as usize, "retained JSON document")
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|_| "retained artifact could not be opened".to_string())?;
    let mut hash = Sha256::new();
    std::io::copy(&mut file, &mut DigestWriter(&mut hash))
        .map_err(|_| "retained artifact could not be hashed".to_string())?;
    Ok(hex::encode(hash.finalize()))
}

struct DigestWriter<'a, D>(&'a mut D);

impl<D: Digest> Write for DigestWriter<'_, D> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}


fn is_git_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
fn is_sha256_id(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(is_sha256)
}

fn update_framed(hash: &mut Sha256, label: &str, value: &str) {
    hash.update(label.len().to_string().as_bytes());
    hash.update(b":");
    hash.update(label.as_bytes());
    hash.update(value.len().to_string().as_bytes());
    hash.update(b":");
    hash.update(value.as_bytes());
}

fn provenance_id(
    receipt: &RetainedReceipt,
    key_set_version: &str,
    artifact: &RetainedArtifact,
) -> Result<String, String> {
    let binding_value = serde_json::to_value(&receipt.spis_binding)
        .map_err(|_| "receipt spisBinding could not be canonicalized".to_string())?;
    let binding_json = String::from_utf8(canonical_json_bytes(&binding_value)?)
        .map_err(|_| "canonical receipt spisBinding was not UTF-8".to_string())?;
    let mut hash = Sha256::new();
    for (label, value) in [
        ("receipt.schema", receipt.schema.as_str()),
        ("receipt.keyId", receipt.key_id.as_str()),
        ("receipt.signedPayload", receipt.signed_payload.as_str()),
        ("receipt.signature", receipt.signature.as_str()),
        ("receipt.requestDigest", receipt.request_digest.as_str()),
        ("receipt.resultDigest", receipt.result_digest.as_str()),
        ("receipt.spisBinding", binding_json.as_str()),
        ("keySetVersion", key_set_version),
        ("artifact.path", artifact.path.as_str()),
        ("artifact.sha256", artifact.sha256.as_str()),
    ] {
        update_framed(&mut hash, label, value);
    }
    Ok(format!("sha256:{}", hex::encode(hash.finalize())))
}

fn strip_provenance(value: &Value) -> Value {
    match value {
        Value::Array(entries) => Value::Array(entries.iter().map(strip_provenance).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(key, _)| key.as_str() != "provenance")
                .map(|(key, entry)| (key.clone(), strip_provenance(entry)))
                .collect(),
        ),
        scalar => scalar.clone(),
    }
}

fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, String> {
    const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

    fn write_value(value: &Value, output: &mut String) -> Result<(), String> {
        match value {
            Value::Null => output.push_str("null"),
            Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
            Value::Number(number) => {
                if let Some(value) = number.as_i64() {
                    if value.unsigned_abs() > MAX_SAFE_INTEGER {
                        return Err("JCS input integer exceeds the safe integer range".to_string());
                    }
                    output.push_str(&value.to_string());
                } else if let Some(value) = number.as_u64() {
                    if value > MAX_SAFE_INTEGER {
                        return Err("JCS input integer exceeds the safe integer range".to_string());
                    }
                    output.push_str(&value.to_string());
                } else {
                    // One declared canonicalization, one behavior: `JSON.parse("1.0")`
                    // yields the JS number 1 and the bridge emits `1`, so an integral
                    // double inside the safe-integer range canonicalizes to the same
                    // integer text here. Fractional and out-of-range numbers are
                    // rejected on both sides.
                    let float = number
                        .as_f64()
                        .ok_or_else(|| "JCS input contains an unrepresentable number".to_string())?;
                    if !float.is_finite() || float.fract() != 0.0 {
                        return Err("JCS input contains a fractional number".to_string());
                    }
                    if float.abs() > MAX_SAFE_INTEGER as f64 {
                        return Err("JCS input integer exceeds the safe integer range".to_string());
                    }
                    output.push_str(&(float as i64).to_string());
                }
            }
            Value::String(value) => {
                let serialized = serde_json::to_string(value)
                    .map_err(|_| "JCS string could not be serialized".to_string())?;
                output.push_str(&serialized);
            }
            Value::Array(entries) => {
                output.push('[');
                for (index, entry) in entries.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    write_value(entry, output)?;
                }
                output.push(']');
            }
            Value::Object(object) => {
                let mut entries: Vec<_> = object.iter().collect();
                entries.sort_by(|(left, _), (right, _)| {
                    left.encode_utf16().cmp(right.encode_utf16())
                });
                output.push('{');
                for (index, (key, entry)) in entries.into_iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    let serialized_key = serde_json::to_string(key)
                        .map_err(|_| "JCS object key could not be serialized".to_string())?;
                    output.push_str(&serialized_key);
                    output.push(':');
                    write_value(entry, output)?;
                }
                output.push('}');
            }
        }
        Ok(())
    }

    let mut output = String::new();
    write_value(value, &mut output)?;
    Ok(output.into_bytes())
}

fn canonical_json_sha256(value: &Value) -> Result<String, String> {
    Ok(sha256_bytes(&canonical_json_bytes(value)?))
}
