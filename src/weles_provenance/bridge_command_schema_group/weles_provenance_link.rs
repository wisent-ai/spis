use super::*;

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

    pub(crate) fn retained_member_matches(&self, value: &Value) -> bool {
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

pub(crate) fn verify_document_reference(
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
