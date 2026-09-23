use super::*;

/// The submission the bridge retained must be exactly the request this attempt sent.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_submission(
    submission: &weles::WelesSubmission,
    organization_id: &str,
    origin: &str,
    idempotency_key: &str,
    identity: &weles::WelesServiceIdentity,
    binding: &weles::WelesAttemptBinding,
    product_url: &str,
    objective: &str,
    constraints: &[String],
) -> Outcome<()> {
    ensure(
        submission.schema == weles::SUBMISSION_SCHEMA,
        "weles_submission_invalid",
        "the retained submission does not declare the typed submission schema",
    )?;
    ensure(
        submission.organization_id == organization_id
            && submission.origin == origin
            && submission.action == weles::SPIS_WELES_ACTION,
        "weles_submission_invalid",
        "the retained submission task identity differs from the submitted request",
    )?;
    ensure(
        submission.idempotency_key == idempotency_key,
        "weles_submission_invalid",
        "the retained submission carries a different idempotency key",
    )?;
    ensure(
        submission.service_identity == *identity,
        "weles_submission_invalid",
        "the retained submission service identity differs from the runtime directory",
    )?;
    ensure(
        is_sha256_id(&submission.request_digest)
            && submission.request_identity.request_digest == submission.request_digest,
        "weles_submission_invalid",
        "the retained submission request digest is not a bound sha256: identifier",
    )?;
    ensure(
        submission.request_identity.spis_binding == *binding
            && submission.request_document.input.spis_binding == *binding,
        "weles_submission_invalid",
        "the retained submission does not carry the exact signed Spis binding",
    )?;
    ensure(
        submission.request_document.schema == OFFICIAL_REQUEST_SCHEMA
            && submission.request_document.organization_id == organization_id
            && submission.request_document.origin == origin
            && submission.request_document.action == weles::SPIS_WELES_ACTION,
        "weles_submission_invalid",
        "the retained official request is not the canonical current Weles task",
    )?;
    ensure(
        submission.request_document.credential_refs.is_empty()
            && submission.request_document.evidence_policy == "full",
        "weles_submission_invalid",
        "the retained official request is not an anonymous full-evidence request",
    )?;
    ensure(
        submission.request_document.input.product_url == product_url
            && submission.request_document.input.objective == objective
            && submission.request_document.input.constraints == constraints,
        "weles_submission_invalid",
        "the retained official request input differs from the submitted browser task",
    )?;
    ensure(
        is_portable_component(&submission.task_id),
        "weles_task_id_invalid",
        "the Weles task identifier is not a portable recording component",
    )?;
    Ok(())
}
