use super::*;

/// The runtime product URL, checked: HTTP(S), no credentials, canonical, a real origin, and the
/// surface identity (when there is one) naming exactly this URL and allowing the Spis action.
pub(super) fn checked_product_url(manifest: &super::super::crawl::RuntimeManifest) -> Outcome<(String, url::Url, String)> {
    // `validate_request_and_evidence_manifest` binds every retained URL to the exact
    // product URL of the current record, so the manifest URL must already be canonical.
    let product_url = manifest.runtime_product.product_url.clone();
    let parsed = url::Url::parse(&product_url).map_err(|_| {
        WorkerFailure::new(
            "web_product_url_invalid",
            "the runtime product URL is not a URL",
        )
    })?;
    ensure(
        matches!(parsed.scheme(), "http" | "https")
            && parsed.username().is_empty()
            && parsed.password().is_none(),
        "web_product_url_invalid",
        "the runtime product URL must be HTTP(S) without credentials",
    )?;
    ensure(
        parsed.as_str() == product_url,
        "web_product_url_invalid",
        "the runtime product URL is not in canonical serialized form",
    )?;
    let origin = parsed.origin().ascii_serialization();
    ensure(
        !origin.is_empty() && origin != "null",
        "web_product_url_invalid",
        "the runtime product URL has an opaque origin",
    )?;
    if let Some(surface) = manifest.runtime_product.surface.as_ref() {
        ensure(
            surface.exact_url == product_url && surface.origin == origin,
            "web_surface_identity_mismatch",
            "the runtime surface identity does not name the exact product URL and origin",
        )?;
        ensure(
            surface
                .allowed_actions
                .iter()
                .any(|action| action == weles::SPIS_WELES_ACTION),
            "web_surface_identity_mismatch",
            "the runtime surface identity does not allow the Spis browser action",
        )?;
    }
    Ok((product_url, parsed, origin))
}
