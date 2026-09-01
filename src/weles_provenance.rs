//! Fail-closed Weles receipt provenance for retained Spis evidence.
//!
//! The Rust verifier never treats a JSON boolean, a verifier label, a receipt-provided
//! key, or a caller-chosen correlation ID as trust. It re-runs the checked-in Node bridge,
//! which loads the exact pinned official `@wisent-ai/weles-client`, then independently
//! rechecks the returned claims, receipt identity, and retained artifact digest here.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

pub const BRIDGE_COMMAND_SCHEMA: &str = "wisent.spis-weles-bridge-command.v1";
pub const BRIDGE_CONFIG_SCHEMA: &str = "wisent.spis-weles-bridge-config.v1";
pub const PROVENANCE_DOCUMENT_SCHEMA: &str = "wisent.spis-weles-provenance.v1";
pub const PROVENANCE_DOCUMENT_REF_SCHEMA: &str =
    "wisent.spis-weles-provenance-document-ref.v1";
pub const PROVENANCE_LINK_SCHEMA: &str = "wisent.spis-provenance-link.v1";
pub const OFFICIAL_CLIENT_PACKAGE: &str = "@wisent-ai/weles-client";
pub const OFFICIAL_CLIENT_COMMIT: &str =
    "37798a26022a040fbd0a4a4a25c99b5559d95a32";

const MAX_DOCUMENT_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpectedReceiptClaims {
    pub task_id: String,
    pub organization_id: String,
    pub origin: String,
    pub action: String,
    pub outcome: String,
    pub evidence_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VerifiedReceiptClaims {
    pub task_id: String,
    pub organization_id: String,
    pub origin: String,
    pub action: String,
    pub outcome: String,
    pub evidence_digest: String,
    pub key_id: String,
    #[serde(flatten)]
    pub additional: Map<String, Value>,
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OfficialClientIdentity {
    pub package: String,
    pub commit: String,
    pub key_set_version: String,
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
    artifact: RetainedArtifact,
    artifact_value: Option<Value>,
}

/// Result of verifying every `provenance_documents` reference in one record.
#[derive(Debug, Clone, Default)]
pub struct VerifiedProvenanceSet {
    documents: BTreeMap<String, VerifiedDocument>,
    failures: Vec<String>,
}

impl VerifiedProvenanceSet {
    /// Re-run official receipt verification for every referenced document.
    ///
    /// A malformed reference, missing vendored client/config/key, unknown key, claim
    /// mismatch, bridge failure, changed artifact, or invalid document remains in
    /// `failures` and never enters the verified set.
    pub fn verify_record(record: &Value, record_dir: &Path) -> Self {
        let mut verified = Self::default();
        let Some(references_value) = record.get("provenance_documents") else {
            return verified;
        };
        let Some(references) = references_value.as_array() else {
            verified
                .failures
                .push("provenance_documents is not an array".to_string());
            return verified;
        };
        for (index, reference_value) in references.iter().enumerate() {
            let result = verify_document_reference(reference_value, record_dir);
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
    /// artifact bytes, and the link is independently bound to this exact value.
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
                canonical_json_sha256(source_value) == expected_digest && *source_value == stripped
            }
        }
    }

    pub fn provenance_class(&self, value: &Value) -> &'static str {
        if self.supports_value(value) {
            "local-browser-recording"
        } else {
            "unverified-source-media"
        }
    }
}

fn verify_document_reference(
    reference_value: &Value,
    record_dir: &Path,
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
    let fresh = invoke_bridge(&persisted, record_dir)?;
    validate_fresh_document(&persisted, &fresh, record_dir)?;
    let artifact_path = resolve_retained_file(record_dir, &fresh.artifact.path)?;
    let artifact_value = if artifact_path.extension().and_then(|value| value.to_str()) == Some("json") {
        let artifact_bytes = read_limited(&artifact_path, MAX_DOCUMENT_BYTES)?;
        serde_json::from_slice(&artifact_bytes).ok()
    } else {
        None
    };
    Ok(VerifiedDocument {
        id: fresh.id,
        artifact: fresh.artifact,
        artifact_value,
    })
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
    if document.receipt.schema != "weles.receipt.current" {
        return Err("retained receipt schema is unsupported".to_string());
    }
    if !is_sha256(&document.artifact.sha256)
        || document.expected_claims.evidence_digest != document.artifact.sha256
    {
        return Err("expected evidenceDigest is not bound to the retained artifact digest".to_string());
    }
    if document.expected_claims.outcome != "completed" {
        return Err("Spis provenance requires the completed terminal outcome".to_string());
    }
    Ok(())
}

fn invoke_bridge(
    persisted: &WelesProvenanceDocument,
    record_dir: &Path,
) -> Result<WelesProvenanceDocument, String> {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("weles-bridge")
        .join("spis-weles-bridge.mjs");
    if !script.is_file() {
        return Err("checked-in Weles bridge is absent".to_string());
    }
    let input = serde_json::json!({
        "schema": BRIDGE_COMMAND_SCHEMA,
        "operation": "verify",
        "receipt": persisted.receipt,
        "expectedClaims": persisted.expected_claims,
        "artifact": persisted.artifact,
    });
    let input_bytes = serde_json::to_vec(&input)
        .map_err(|_| "could not serialize the bridge verification request".to_string())?;
    let mut child = Command::new("node")
        .arg(&script)
        .arg("--input")
        .arg("-")
        .arg("--output")
        .arg("-")
        .current_dir(record_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "could not start Node for the checked-in Weles bridge".to_string())?;
    child
        .stdin
        .take()
        .ok_or_else(|| "Weles bridge stdin was unavailable".to_string())?
        .write_all(&input_bytes)
        .map_err(|_| "could not send the verification document to the Weles bridge".to_string())?;
    let output = child
        .wait_with_output()
        .map_err(|_| "could not collect the Weles bridge result".to_string())?;
    if !output.status.success() {
        let code = bridge_error_code(&output.stderr);
        return Err(format!("official Weles bridge failed closed ({code})"));
    }
    if output.stdout.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err("official Weles bridge output exceeded the size limit".to_string());
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|_| "official Weles bridge returned a malformed verification document".to_string())
}

fn validate_fresh_document(
    persisted: &WelesProvenanceDocument,
    fresh: &WelesProvenanceDocument,
    record_dir: &Path,
) -> Result<(), String> {
    validate_document_shape(fresh)?;
    if fresh.receipt != persisted.receipt
        || fresh.expected_claims != persisted.expected_claims
        || fresh.artifact.path != persisted.artifact.path
        || fresh.artifact.sha256 != persisted.artifact.sha256
        || fresh.artifact.bytes != persisted.artifact.bytes
    {
        return Err("fresh official verification differs from the persisted receipt contract".to_string());
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
    );
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

fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|_| "retained file metadata is unavailable".to_string())?;
    if metadata.len() > limit {
        return Err("retained JSON document exceeded the size limit".to_string());
    }
    fs::read(path).map_err(|_| "retained file could not be read".to_string())
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
) -> String {
    let mut hash = Sha256::new();
    for (label, value) in [
        ("receipt.schema", receipt.schema.as_str()),
        ("receipt.keyId", receipt.key_id.as_str()),
        ("receipt.signedPayload", receipt.signed_payload.as_str()),
        ("receipt.signature", receipt.signature.as_str()),
        ("keySetVersion", key_set_version),
        ("artifact.path", artifact.path.as_str()),
        ("artifact.sha256", artifact.sha256.as_str()),
    ] {
        update_framed(&mut hash, label, value);
    }
    format!("sha256:{}", hex::encode(hash.finalize()))
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

fn canonical_json_sha256(value: &Value) -> String {
    fn ordered(value: &Value) -> Value {
        match value {
            Value::Array(entries) => Value::Array(entries.iter().map(ordered).collect()),
            Value::Object(object) => {
                let ordered_entries: BTreeMap<&String, &Value> = object.iter().collect();
                Value::Object(
                    ordered_entries
                        .into_iter()
                        .map(|(key, entry)| (key.clone(), ordered(entry)))
                        .collect(),
                )
            }
            scalar => scalar.clone(),
        }
    }
    let bytes = serde_json::to_vec(&ordered(value)).unwrap_or_default();
    sha256_bytes(&bytes)
}
