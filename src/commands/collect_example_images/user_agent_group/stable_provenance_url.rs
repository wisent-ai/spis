use super::*;

/// The image address kept as a record's provenance: the address itself, but
/// for hosts that sign each download in its query, whose query and fragment
/// expire and are left out. An address that is no URL is refused naming it.
pub(crate) fn stable_provenance_url(value: &str) -> Result<String> {
    const STRIPPED_HOSTS: &[&str] = &[
        "private-user-images.githubusercontent.com",
        "github-production-user-asset-6210df.s3.amazonaws.com",
    ];
    let mut address =
        url::Url::parse(value).with_context(|| format!("the image address {value:?} is no URL"))?;
    if address
        .host_str()
        .is_some_and(|host| STRIPPED_HOSTS.contains(&host))
    {
        address.set_query(None);
        address.set_fragment(None);
    }
    Ok(address.to_string())
}

// ---------------------------------------------------------------------------
// Selection

/// `content_type` is the media type the server declared, `None` when it sent
/// no Content-Type; only a declared `image/` type makes the page an image.
/// Every candidate is resolved against `page_url` by the URL standard and
/// listed once, in the order the page names it; one that names no web
/// address is not fetchable and is left out, and one that does not resolve
/// is said on stderr with its cause.
pub(crate) fn candidate_urls(
    page_url: &str,
    body: &[u8],
    content_type: Option<&str>,
) -> Result<Vec<Candidate>> {
    if content_type.is_some_and(|media| media.starts_with("image/")) {
        // The page is the image itself: one candidate, at the first position.
        let mut direct = Vec::new();
        direct.push(Candidate {
            url: page_url.to_string(),
            hint: "direct image".to_string(),
            order: direct.len(),
            origin: "direct",
        });
        return Ok(direct);
    }
    let page = url::Url::parse(page_url)
        .with_context(|| format!("the page address {page_url:?} is no URL"))?;
    // A page's bytes are read as UTF-8, each invalid sequence shown as U+FFFD,
    // since HTML's own encoding detection is not this reader's.
    let html = String::from_utf8_lossy(body);
    // Addresses written inside scripts' JSON are unescaped before the text
    // is searched for them.
    let text = html
        .replace("\\/", "/")
        .replace("\\u0026", "&")
        .replace("\\u003d", "=");
    let mut raw: Vec<Candidate> = Vec::new();
    extract_tag_candidates(&html, &mut raw)?;
    extract_css_candidates(&text, &mut raw)?;
    extract_embedded_candidates(&text, &mut raw)?;

    let mut unique: std::collections::HashMap<String, Candidate> = std::collections::HashMap::new();
    for candidate in raw {
        let resolved = match fetchable(&page, &candidate.url) {
            Ok(Some(resolved)) => resolved.to_string(),
            Ok(None) => continue,
            Err(error) => {
                eprintln!("collect-example-images: {error:#}; that candidate is not probed");
                continue;
            }
        };
        unique.entry(resolved.clone()).or_insert(Candidate {
            url: resolved,
            hint: candidate.hint.trim().to_string(),
            order: candidate.order,
            origin: candidate.origin,
        });
    }
    // Candidates keep the order the page lists them in; which one shows the
    // product is Brama's answer in `select_image`, not a word score here.
    let mut result: Vec<Candidate> = unique.into_values().collect();
    result.sort_by_key(|candidate| candidate.order);
    Ok(result)
}

/// Fetch one candidate and read its format and dimensions from the header;
/// `None` when it could not be fetched or its bytes are no image whose header
/// the `image` crate reads, each said on stderr with the cause.
pub(crate) fn probe_candidate(candidate: &Candidate) -> Option<Probe> {
    let fetched = match fetch(
        &candidate.url,
        "image/avif,image/webp,image/png,image/jpeg,image/*",
    ) {
        Ok(fetched) => fetched,
        Err(error) => {
            eprintln!(
                "collect-example-images: candidate {} is not read: {error:#}",
                candidate.url
            );
            return None;
        }
    };
    let (format, width, height) = match image_header(&fetched.data) {
        Ok(header) => header,
        Err(error) => {
            eprintln!(
                "collect-example-images: candidate {} is no readable image: {error:#}",
                candidate.url
            );
            return None;
        }
    };
    Some(Probe {
        format,
        width,
        height,
        payload: fetched.data,
        final_url: fetched.final_url,
    })
}

/// The image on `page_url` that shows the product's interface, as Brama's
/// typed decision chooses it from every readable image on the page with its
/// address, the page's own description of it and its dimensions: a `choice`
/// among the listed images and none of them. Brama choosing none is refused
/// with that answer, never replaced by a guess.
pub(crate) fn select_image(page_url: &str) -> Result<(Candidate, Probe)> {
    let fetched = fetch(page_url, "text/html,application/xhtml+xml,image/*")?;
    let final_page_url = fetched.final_url.clone();
    let mut probed: Vec<(Candidate, Probe)> = candidate_urls(
        &final_page_url,
        &fetched.data,
        fetched.content_type.as_deref(),
    )?
    .into_iter()
    .filter_map(|candidate| probe_candidate(&candidate).map(|probe| (candidate, probe)))
    .collect();
    if probed.is_empty() {
        bail!("{final_page_url} holds no readable raster image");
    }
    let listing: Vec<String> = probed
        .iter()
        .enumerate()
        .map(|(index, (candidate, probe))| {
            format!(
                "{index}: {} | {} | {}x{} {} | found in {}",
                candidate.url,
                candidate.hint,
                probe.width,
                probe.height,
                probe.format,
                candidate.origin
            )
        })
        .collect();
    let purpose = "collect-example-images asks Brama which image shows the product";
    let mut options: Vec<(String, String)> = probed
        .iter()
        .enumerate()
        .map(|(index, (candidate, _))| (format!("{IMAGE}{index}"), candidate.url.clone()))
        .collect();
    options.push((
        NO_INTERFACE.to_string(),
        "no listed image shows the product's interface".to_string(),
    ));
    let mut questions = serde_json::Map::new();
    questions.insert(
        INTERFACE.to_string(),
        crate::commands::brama::choice(
            "The state lists the images on a software product's page. Which one image shows the \
             product's own interface (a screenshot of its screens, windows or dashboards)? Logos, \
             icons, avatars, badges, illustrations and decorative pictures are not the interface.",
            &options,
        ),
    );
    let state = format!("Page: {final_page_url}\nImages:\n{}", listing.join("\n"));
    let answers = crate::commands::brama::decide(purpose, &state, &questions)?;
    let (chosen, _) = crate::commands::brama::chosen(purpose, &answers[INTERFACE], &options)?;
    // Choosing none is Brama's statement that no image shows the interface.
    match options.iter().position(|(option, _)| *option == chosen) {
        Some(index) if index < probed.len() => Ok(probed.swap_remove(index)),
        _ => bail!("Brama found no image of the product's interface on {final_page_url}"),
    }
}

/// The question's key, the prefix of each listed image's option, and the
/// option of no image showing the interface.
const INTERFACE: &str = "interface";
const IMAGE: &str = "image-";
const NO_INTERFACE: &str = "none";

// ---------------------------------------------------------------------------
// Storage

pub(crate) fn slugify_name(name: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for ch in name.to_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            slug.push(ch);
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    // The whole slug: two names sharing their first characters would share a
    // cut one and overwrite each other's stored image.
    slug
}

pub(crate) fn store_image(
    catalog_dir: &Path,
    index: usize,
    name: &str,
    page_url: &str,
) -> Result<Value> {
    let (candidate, probe) = select_image(page_url)?;
    // The original bytes are stored as served; dimensions are the source
    // image's own.
    let payload = &probe.payload;
    let slug = slugify_name(name);
    let relative_path = format!(
        "images/{}.{}",
        crate::commands::reference_contract::ordinal_name(index, &slug),
        probe.format
    );
    let destination = catalog_dir.join(&relative_path);
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&destination, payload)?;

    Ok(json!({
        // source_page_url and capture_kind are overwritten by the caller.
        "source_page_url": Value::Null,
        "source_image_url": stable_provenance_url(&probe.final_url)?,
        "local_path": relative_path,
        "capture_kind": "official-source-image",
        "captured_at": crate::today_utc(),
        "format": probe.format,
        "width": probe.width,
        "height": probe.height,
        "original_width": probe.width,
        "original_height": probe.height,
        "bytes": payload.len(),
        "sha256": crate::sha256_hex(payload),
        "source_hint": if candidate.hint.is_empty() { candidate.origin.to_string() } else { candidate.hint.clone() },
    }))
}

pub(crate) fn write_catalog(source_path: &Path, catalog: &Value) -> Result<()> {
    std::fs::write(source_path, serde_json::to_string_pretty(catalog)? + "\n")?;
    Ok(())
}
