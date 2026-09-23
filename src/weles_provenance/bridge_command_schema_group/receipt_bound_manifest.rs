use super::*;

impl ReceiptBoundManifest {
    /// Parses the retained artifact in exactly the version the signed outcome mandates.
    pub(crate) fn parse(value: &Value, outcome: &str) -> Result<Self, String> {
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

    pub(crate) fn facts(&self) -> EvidenceManifestFacts<'_> {
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
