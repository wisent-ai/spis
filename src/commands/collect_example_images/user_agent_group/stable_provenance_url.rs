use super::*;

pub(crate) fn stable_provenance_url(value: &str) -> String {
    const STRIPPED_HOSTS: &[&str] = &[
        "private-user-images.githubusercontent.com",
        "github-production-user-asset-6210df.s3.amazonaws.com",
    ];
    if STRIPPED_HOSTS.contains(&hostname_of(value).as_str()) {
        if let Some((scheme, rest)) = split_scheme(value) {
            let authority_path = rest.split(['?', '#']).next().unwrap_or(rest);
            return format!("{scheme}://{authority_path}");
        }
    }
    value.to_string()
}

// ---------------------------------------------------------------------------
// Selection

pub(crate) fn candidate_urls(page_url: &str, body: &[u8], content_type: &str) -> Vec<Candidate> {
    if content_type.starts_with("image/") {
        return vec![Candidate {
            url: page_url.to_string(),
            hint: "direct image".to_string(),
            order: 0,
            origin: "direct",
        }];
    }
    let text = String::from_utf8_lossy(body)
        .replace("\\/", "/")
        .replace("\\u0026", "&")
        .replace("\\u003d", "=");
    let mut raw: Vec<Candidate> = Vec::new();
    let mut order = 0usize;
    extract_tag_candidates(&text, &mut raw, &mut order);
    extract_css_candidates(&text, &mut raw, &mut order);
    extract_embedded_candidates(&text, &mut raw, &mut order);

    let mut unique: std::collections::HashMap<String, Candidate> = std::collections::HashMap::new();
    let mut insert_order: Vec<String> = Vec::new();
    for candidate in raw {
        let Some(resolved) = join_url(page_url, &candidate.url) else {
            continue;
        };
        let has_netloc = split_scheme(&resolved)
            .map(|(_, rest)| !rest.split('/').next().unwrap_or("").is_empty())
            .unwrap_or(false);
        if !has_netloc {
            continue;
        }
        let clean = clean_url(&resolved);
        if !unique.contains_key(&clean) {
            unique.insert(
                clean.clone(),
                Candidate {
                    url: clean.clone(),
                    hint: candidate.hint.trim().to_string(),
                    order: candidate.order,
                    origin: candidate.origin,
                },
            );
            insert_order.push(clean);
        }
    }
    // Candidates keep the order the page lists them in; which one shows the
    // product is Brama's answer in `select_image`, not a word score here.
    let _ = insert_order;
    let mut result: Vec<Candidate> = unique.into_values().collect();
    result.sort_by_key(|candidate| candidate.order);
    result
}

/// Fetch one candidate and read its container and dimensions from the
/// header; `None` when it is not a raster image this can read.
pub(crate) fn probe_candidate(candidate: &Candidate) -> Option<Probe> {
    let fetched = fetch(
        &candidate.url,
        "image/avif,image/webp,image/png,image/jpeg,image/*",
    )
    .ok()?;
    let (format, width, height) = parse_image_header(&fetched.data)?;
    Some(Probe {
        format,
        width,
        height,
        payload: fetched.data,
        final_url: fetched.final_url,
    })
}

/// The image on `page_url` that shows the product's interface, as Brama
/// chooses it from every readable image on the page with its address, the
/// page's own description of it and its dimensions. Brama answering "none"
/// is refused with that answer, never replaced by a guess.
pub(crate) fn select_image(page_url: &str) -> Result<(Candidate, Probe)> {
    let fetched = fetch(page_url, "text/html,application/xhtml+xml,image/*")?;
    let final_page_url = fetched.final_url.clone();
    let mut probed: Vec<(Candidate, Probe)> =
        candidate_urls(&final_page_url, &fetched.data, &fetched.content_type)
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
    let answer = crate::commands::brama::ask_json(
        "collect-example-images asks Brama which image shows the product",
        "You pick the one image that shows a software product's own interface \
         (a screenshot of its screens, windows or dashboards) from the images on its \
         page. Logos, icons, avatars, badges, illustrations and decorative pictures \
         are not the interface. Return STRICT JSON: {\"index\": number} with the \
         listed index, or {\"index\": null} when no image shows the interface.",
        &format!("Page: {final_page_url}\nImages:\n{}", listing.join("\n")),
    )?;
    let index = answer["index"].as_u64().with_context(|| {
        format!("Brama found no image of the product's interface on {final_page_url}")
    })?;
    let index = usize::try_from(index)
        .ok()
        .filter(|index| *index < probed.len())
        .with_context(|| {
            format!("Brama chose image {index}, which {final_page_url} does not list")
        })?;
    Ok(probed.swap_remove(index))
}

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
    slug.chars().take(60).collect()
}

pub(crate) fn today_utc() -> String {
    crate::now_iso_utc()[..10].to_string()
}

pub(crate) fn store_image(
    catalog_dir: &Path,
    index: usize,
    name: &str,
    page_url: &str,
) -> Result<Value> {
    let (candidate, probe) = select_image(page_url)?;
    // Gap vs. Pillow original: no RGBA flattening onto white, no LANCZOS
    // thumbnail to fit (1400, 1000), no WebP re-encode at quality 82. The
    // ORIGINAL bytes are stored verbatim; dimensions are the source image's.
    let payload = &probe.payload;
    let slug = slugify_name(name);
    let relative_path = format!("images/{index:02}-{slug}.{}", probe.format);
    let destination = catalog_dir.join(&relative_path);
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&destination, payload)?;

    Ok(json!({
        // source_page_url and capture_kind are overwritten by the caller.
        "source_page_url": Value::Null,
        "source_image_url": stable_provenance_url(&probe.final_url),
        "local_path": relative_path,
        "capture_kind": "official-source-image",
        "captured_at": today_utc(),
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
