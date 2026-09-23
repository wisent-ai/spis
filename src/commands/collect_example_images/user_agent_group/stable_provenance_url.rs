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
// Scoring

pub(crate) fn preflight_score(candidate: &Candidate) -> f64 {
    let text = format!("{} {}", candidate.url, candidate.hint).to_lowercase();
    let mut score: f64 = 0.0;
    for (word, weight) in [
        ("screenshot", 35.0),
        ("screen", 18.0),
        ("interface", 24.0),
        ("dashboard", 22.0),
        ("window", 16.0),
        ("workflow", 14.0),
        ("product", 8.0),
        ("hero", 5.0),
        ("app", 4.0),
    ] {
        if text.contains(word) {
            score += weight;
        }
    }
    for (word, weight) in [
        ("logo", -45.0),
        ("icon", -38.0),
        ("avatar", -40.0),
        ("badge", -50.0),
        ("favicon", -60.0),
        ("opengraph", -22.0),
        ("emoji", -45.0),
        ("spinner", -45.0),
    ] {
        if text.contains(word) {
            score += weight;
        }
    }
    if candidate.origin == "meta" {
        score += 9.0;
    }
    if candidate.origin.ends_with("-srcset") {
        score += 7.0;
    }
    score -= candidate.order as f64 * 0.015;
    score
}

pub(crate) fn image_score(candidate: &Candidate, width: u32, height: u32) -> f64 {
    let area = u64::from(width) * u64::from(height);
    let mut score = preflight_score(candidate) + (area.max(1) as f64).log2() * 3.0;
    let ratio = width as f64 / height as f64;
    if (1.15..=2.4).contains(&ratio) {
        score += 14.0;
    } else if (0.45..1.15).contains(&ratio) {
        score += 8.0;
    }
    if width >= 1000 {
        score += 8.0;
    }
    if height >= 600 {
        score += 7.0;
    }
    if width == height {
        score -= 18.0;
    }
    score
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
    let mut result: Vec<Candidate> = unique.into_values().collect();
    result.sort_by(|a, b| {
        preflight_score(b)
            .partial_cmp(&preflight_score(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result.truncate(MAX_CANDIDATES);
    let _ = insert_order;
    result
}

/// Header-side replacement for the Python decode_candidate(): fetch, sniff the
/// container, read dimensions, apply the same size/aspect gates.
pub(crate) fn probe_candidate(candidate: &Candidate) -> Option<Probe> {
    let fetched = fetch(
        &candidate.url,
        MAX_IMAGE_BYTES,
        "image/avif,image/webp,image/png,image/jpeg,image/*",
    )
    .ok()?;
    let (format, width, height) = parse_image_header(&fetched.data)?;
    if width.min(height) < 260 || width.max(height) < 480 {
        return None;
    }
    let wf = f64::from(width);
    let hf = f64::from(height);
    if wf / hf > 5.2 || hf / wf > 3.2 {
        return None;
    }
    Some(Probe {
        format,
        width,
        height,
        payload: fetched.data,
        final_url: fetched.final_url,
    })
}

pub(crate) fn select_image(page_url: &str) -> Result<(Candidate, Probe)> {
    let fetched = fetch(
        page_url,
        MAX_PAGE_BYTES,
        "text/html,application/xhtml+xml,image/*",
    )?;
    let final_page_url = fetched.final_url.clone();
    let candidates = candidate_urls(&final_page_url, &fetched.data, &fetched.content_type);
    let mut best: Option<(f64, Candidate, Probe)> = None;
    // The Python probed candidates on 8 threads; this port probes them
    // sequentially (same selection outcome, slower wall clock).
    for candidate in &candidates {
        let Some(probe) = probe_candidate(candidate) else {
            continue;
        };
        let score = image_score(candidate, probe.width, probe.height);
        if best.as_ref().map(|(s, _, _)| score > *s).unwrap_or(true) {
            best = Some((score, candidate.clone(), probe));
        }
    }
    match best {
        Some((_, candidate, probe)) => Ok((candidate, probe)),
        None => bail!("no qualifying image found"),
    }
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

pub(crate) fn store_image(catalog_dir: &Path, index: usize, name: &str, page_url: &str) -> Result<Value> {
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
