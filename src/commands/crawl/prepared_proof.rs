//! The one check that a prepared-runtime proof binds this runtime, and that
//! a fresh `stado-runtime-readiness verify` observation still attests it.
//! `crawl` (iOS, Android and desktop preflight), `crawl-desktop` and
//! `crawl-mobile` all bind a proof through it, so the three cannot accept
//! different proofs for the same runtime.

use super::{RuntimeExecutionIdentity, RuntimeManifest, RuntimePreparedProof};
use anyhow::{bail, Context, Result};
use serde_json::Value;

/// The proof schema `stado-runtime-readiness` writes.
const PROOF_SCHEMA: &str = "wisent.runtime-preparation-proof.v1";
/// The observation schema `stado-runtime-readiness verify --json` prints.
const OBSERVATION_SCHEMA: &str = "wisent.runtime-readiness-observation.v1";
/// The program that observes a prepared runtime, as the proof names it.
const OBSERVER: &str = "stado-runtime-readiness";
/// The prompt-safety switches a proof and an observation both state true.
const DISABLED: [&str; 3] = [
    "notification_delivery_disabled",
    "permission_prompt_invocation_disabled",
    "notification_prompt_invocation_disabled",
];

/// Whether a readiness `observation` states `field` as a count and that
/// count holds no pending prompt; an absent or non-numeric count is not
/// "none pending".
pub(crate) fn none_pending(observation: &Value, field: &str) -> bool {
    observation.get(field).and_then(Value::as_u64) == Some(0)
}

/// `YYYY-MM-DDTHH:MM:SSZ`, the exact shape `crate::now_iso_utc` emits.
pub(crate) fn is_rfc3339_utc(value: &str) -> bool {
    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ").is_ok()
}

/// A proof checked against the runtime it must bind.
pub(crate) struct BoundProof<'a> {
    pub(crate) proof: &'a RuntimePreparedProof,
    pub(crate) product: &'a str,
    pub(crate) device: &'a str,
    identity: &'a RuntimeExecutionIdentity,
}

/// The manifest's prepared-runtime proof, refused unless it binds `product`
/// on the device, version and executable `identity` names.
pub(crate) fn bound_proof<'a>(
    manifest: &'a RuntimeManifest,
    identity: &'a RuntimeExecutionIdentity,
    product: &'a str,
) -> Result<BoundProof<'a>> {
    let proof = manifest
        .prepared_proof
        .as_ref()
        .context("no independently observed prepared-runtime proof is bound")?;
    // A proof binds one device; an identity that names none has nothing to
    // bind it to, and two absent ids are not the same device.
    let device = identity.device_id.as_deref().context(
        "the runtime identity names no device, so no prepared-runtime proof can be bound to it",
    )?;
    if proof.device_id.as_deref() != Some(device) {
        bail!(
            "the prepared-runtime proof names device {:?}, not this runtime's {device}",
            proof.device_id
        );
    }
    if proof.product_identifier != product {
        bail!(
            "the prepared-runtime proof names product {:?}, not {product:?}",
            proof.product_identifier
        );
    }
    let proof_value = serde_json::to_value(proof)?;
    let switched_off = DISABLED
        .iter()
        .all(|field| proof_value.get(*field).and_then(Value::as_bool) == Some(true));
    if proof.schema != PROOF_SCHEMA
        || proof.observed_by != OBSERVER
        || !proof.evidence_uri.starts_with("stado://")
        || identity.product_version.as_deref() != Some(proof.product_version.as_str())
        || !identity
            .executable_sha256
            .as_deref()
            .is_some_and(|digest| digest.eq_ignore_ascii_case(&proof.executable_sha256))
        || !crate::weles_provenance::is_sha256(&proof.evidence_sha256.to_ascii_lowercase())
        || !proof.installed
        || !proof.first_run_completed
        || proof.pending_permission_prompts != 0
        || proof.pending_notification_prompts != 0
        || !switched_off
        || !manifest.constraints.no_system_permission_prompts
        || !manifest.constraints.no_notifications
        || !is_rfc3339_utc(&proof.observed_at)
    {
        bail!("prepared-runtime proof does not bind the exact product, version and executable with an RFC 3339 UTC observation time, first run completed, zero pending prompts, disabled permission/notification prompt invocation, and disabled notification delivery: {proof_value}");
    }
    Ok(BoundProof {
        proof,
        product,
        device,
        identity,
    })
}

impl BoundProof<'_> {
    /// The arguments of `stado-runtime-readiness` that re-observe this proof.
    pub(crate) fn verify_args(&self) -> [&str; 10] {
        [
            "verify",
            "--json",
            "--product",
            self.product,
            "--device",
            self.device,
            "--evidence-uri",
            &self.proof.evidence_uri,
            "--evidence-sha256",
            &self.proof.evidence_sha256,
        ]
    }

    /// Refuses a fresh `observation` that does not attest this proof's exact
    /// product, device, evidence, version, executable and safe state.
    pub(crate) fn attested(&self, observation: &Value) -> Result<()> {
        let text = |field: &str| observation.get(field).and_then(Value::as_str);
        if text("schema") != Some(OBSERVATION_SCHEMA)
            || observation.get("ready").and_then(Value::as_bool) != Some(true)
            || text("product_identifier") != Some(self.product)
            || text("device_id") != Some(self.device)
            || text("evidence_sha256") != Some(self.proof.evidence_sha256.as_str())
            || !none_pending(observation, "pending_permission_prompts")
            || !none_pending(observation, "pending_notification_prompts")
            || DISABLED
                .iter()
                .any(|field| observation.get(*field).and_then(Value::as_bool) != Some(true))
            || text("product_version") != self.identity.product_version.as_deref()
            || !text("executable_sha256").is_some_and(|digest| {
                self.identity
                    .executable_sha256
                    .as_deref()
                    .is_some_and(|expected| expected.eq_ignore_ascii_case(digest))
            })
        {
            bail!("the fresh runtime-readiness observation does not attest the exact prepared state, version and executable digest: {observation}");
        }
        Ok(())
    }
}
