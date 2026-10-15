use super::*;
use crate::commands::reference_contract::Provenance;

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
pub(crate) struct VerifiedDocument {
    pub(crate) id: String,
    /// The signed terminal outcome this document proves. A document is verified as a
    /// DOCUMENT for every terminal outcome, but only `SUCCESSFUL_OUTCOME` may support a
    /// claim about confirmed source material.
    pub(crate) outcome: String,
    pub(crate) artifact: RetainedArtifact,
    pub(crate) artifact_value: Option<Value>,
}

/// The one public trust document, already read and validated by this process.
///
/// Every bridge operation needs it: the verifier compares persisted claims against it,
/// and any producer has to hand the child the exact bytes it validated rather than
/// letting the child pick its own trust. It is opaque outside this module.
#[derive(Debug, Clone)]
pub struct CanonicalTrust {
    pub(crate) path: PathBuf,
    pub(crate) bytes: Vec<u8>,
    pub(crate) document: WelesReceiptTrust,
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
    pub(crate) documents: BTreeMap<String, VerifiedDocument>,
    pub(crate) failures: Vec<String>,
    pub(crate) record_dir: Option<PathBuf>,
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
            let result = verify_document_reference(reference_value, record, record_dir, &trust);
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

    /// `Ok` only when the value's typed link resolves to freshly verified receipt and
    /// artifact bytes, the link is independently bound to this exact value, AND the
    /// document proves the successful outcome; otherwise the one reason it is not.
    ///
    /// THIS IS THE BOUNDARY. A failure provenance document is verified as a document —
    /// its receipt, claims, manifest, envelope and retained bytes are all re-proved — but
    /// it proves that the browser task did NOT produce evidence, so it can never support a
    /// claim that a record's material is confirmed. Both consumers of this check
    /// (`verify_reference_evidence` and `generate_example_catalogs`) classify and admit
    /// material through here and through `provenance_class`, so binding the support to
    /// `SUCCESSFUL_OUTCOME` in this one place keeps a non-success out of every confirmed
    /// claim without weakening its own verification.
    pub fn supports_value(&self, value: &Value) -> Result<(), String> {
        let link_value = value
            .get("provenance")
            .ok_or("it carries no provenance link")?;
        let link = serde_json::from_value::<WelesProvenanceLink>(link_value.clone())
            .map_err(|error| format!("its provenance link is not the typed link: {error}"))?;
        if link.schema != PROVENANCE_LINK_SCHEMA
            || !is_sha256_id(&link.document_id)
            || !is_sha256(&link.artifact_sha256)
        {
            return Err(format!(
                "its provenance link is not a {PROVENANCE_LINK_SCHEMA} link naming a sha256 document and artifact"
            ));
        }
        let document = self.documents.get(&link.document_id).ok_or_else(|| {
            format!(
                "provenance document {} is not among the record's verified documents",
                link.document_id
            )
        })?;
        // Verified, and still not evidence of anything having been captured.
        if document.outcome != SUCCESSFUL_OUTCOME {
            return Err(format!(
                "provenance document {} proves the outcome {}, not {SUCCESSFUL_OUTCOME}",
                document.id, document.outcome
            ));
        }
        if link.artifact_path != document.artifact.path
            || link.artifact_sha256 != document.artifact.sha256
        {
            return Err(format!(
                "its link names another artifact than document {} verified",
                document.id
            ));
        }
        match link.kind {
            ProvenanceLinkKind::Artifact => {
                if link.artifact_pointer.is_some() || link.value_sha256.is_some() {
                    return Err("its artifact link carries an observation pointer or digest".into());
                }
                if value.get("local_path").and_then(Value::as_str)
                    != Some(document.artifact.path.as_str())
                    || value.get("sha256").and_then(Value::as_str)
                        != Some(document.artifact.sha256.as_str())
                {
                    return Err(format!(
                        "its local_path and sha256 are not the artifact document {} verified",
                        document.id
                    ));
                }
            }
            ProvenanceLinkKind::Observation => {
                let (Some(pointer), Some(expected_digest)) = (
                    link.artifact_pointer.as_deref(),
                    link.value_sha256.as_deref(),
                ) else {
                    return Err("its observation link names no artifact pointer or digest".into());
                };
                let artifact_value = document.artifact_value.as_ref().ok_or_else(|| {
                    format!(
                        "verified document {} carries no artifact value to observe",
                        document.id
                    )
                })?;
                if !pointer.starts_with('/') || !is_sha256(expected_digest) {
                    return Err(format!(
                        "observation pointer {pointer} or its digest is malformed"
                    ));
                }
                let source_value = artifact_value.pointer(pointer).ok_or_else(|| {
                    format!(
                        "observation pointer {pointer} names nothing in document {}",
                        document.id
                    )
                })?;
                if canonical_json_sha256(source_value)? != expected_digest {
                    return Err(format!(
                        "the value at {pointer} does not have the linked digest"
                    ));
                }
                if *source_value != strip_provenance(value) {
                    return Err(format!(
                        "the value differs from the observation at {pointer}"
                    ));
                }
            }
        }
        self.retained_member(value)
    }

    /// The provenance a value carries: Weles-signed browser evidence exactly
    /// when this verified set supports it, and otherwise unverified source
    /// media together with the reason `supports_value` gave.
    pub fn provenance(&self, value: &Value) -> (Provenance, Option<String>) {
        match self.supports_value(value) {
            Ok(()) => (Provenance::WelesSignedBrowserEvidence, None),
            Err(reason) => (Provenance::UnverifiedSourceMedia, Some(reason)),
        }
    }

    /// The provenance class as a record spells it.
    pub fn provenance_class(&self, value: &Value) -> &'static str {
        match self.provenance(value).0 {
            Provenance::WelesSignedBrowserEvidence => "weles-signed-browser-evidence",
            Provenance::UnverifiedSourceMedia => "unverified-source-media",
            other => unreachable!(
                "provenance() answers Weles evidence or unverified media, not {other:?}"
            ),
        }
    }
}

pub(crate) fn verify_document_reference(
    reference_value: &Value,
    record: &Value,
    record_dir: &Path,
    trust: &CanonicalTrust,
) -> Result<VerifiedDocument, String> {
    let reference: WelesProvenanceDocumentRef = serde_json::from_value(reference_value.clone())
        .map_err(|error| format!("reference does not match the typed schema: {error}"))?;
    if reference.schema != PROVENANCE_DOCUMENT_REF_SCHEMA {
        return Err("reference schema is unsupported".to_string());
    }
    if !is_sha256(&reference.sha256) {
        return Err("reference sha256 is not a lowercase SHA-256 digest".to_string());
    }
    let document_path = resolve_retained_file(record_dir, &reference.path)?;
    let bytes = read_whole(&document_path)?;
    if sha256_bytes(&bytes) != reference.sha256 {
        return Err("verification document digest does not match its reference".to_string());
    }
    let persisted: WelesProvenanceDocument = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "verification document {} does not match the typed schema: {error}",
            document_path.display()
        )
    })?;
    validate_document_shape(&persisted)?;
    validate_document_trust(&persisted, &trust.document)?;
    let fresh = invoke_bridge(&persisted, record_dir, trust)?;
    validate_fresh_document(&persisted, &fresh, record_dir)?;
    let artifact_path = resolve_retained_file(record_dir, &fresh.artifact.path)?;
    let artifact_bytes = read_limited(&artifact_path, fresh.artifact.bytes)?;
    if sha256_bytes(&artifact_bytes) != fresh.artifact.sha256 {
        return Err("receipt-bound JSON artifact changed while it was being parsed".to_string());
    }
    let artifact_value: Value = serde_json::from_slice(&artifact_bytes).map_err(|error| {
        format!(
            "receipt-bound artifact {} is not the required signed JSON document: {error}",
            artifact_path.display()
        )
    })?;
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
