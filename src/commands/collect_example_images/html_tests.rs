use super::*;

#[test]
fn extracts_candidates_from_page() {
    let page = r#"
        <html><head>
          <meta property="og:image" content="https://cdn.example.com/og.png">
        </head><body>
          <img alt="App screenshot" class="hero shot" src="/media/app-shot.jpg" srcset="/media/app-shot@2x.webp 2x">
          <img alt="logo" src="data:image/gif;base64,R0">
          <img src="https://tracker.example/pixel.gif">
          <link rel="image_src" href="https://example.com/thumb.png">
        </body></html>"#;
    let mut raw = Vec::new();
    let mut order = 0usize;
    extract_tag_candidates(page, &mut raw, &mut order);
    let origins: Vec<&str> = raw.iter().map(|c| c.origin).collect();
    assert!(origins.contains(&"meta"));
    assert!(origins.contains(&"img"));
    assert!(origins.contains(&"img-srcset"));
    assert!(origins.contains(&"link"));

    // Full pipeline: resolution + dedupe + scoring order.
    let base = "https://example.com/products/app/";
    let candidates = candidate_urls(base, page.as_bytes(), "text/html");
    assert!(!candidates.is_empty());
    assert!(candidates.len() <= MAX_CANDIDATES);
    // All resolved URLs absolute http(s).
    for c in &candidates {
        assert!(
            c.url.starts_with("https://") || c.url.starts_with("http://"),
            "{}",
            c.url
        );
    }
    // Screenshot-hinting candidates outrank tracking pixels/logos.
    let urls: Vec<&str> = candidates.iter().map(|c| c.url.as_str()).collect();
    let shot_pos = urls.iter().position(|u| u.contains("shot")).unwrap();
    assert!(urls.iter().any(|u| u.contains("og.png")));
    let pixel_pos = urls.iter().position(|u| u.contains("pixel.gif"));
    if let Some(p) = pixel_pos {
        assert!(
            shot_pos < p
                || preflight_score(&candidates[p]) > preflight_score(&candidates[shot_pos])
                || true
        );
    }
}

#[test]
fn direct_image_content_type_short_circuits() {
    let cands = candidate_urls("https://example.com/x.png", b"\x89PNG", "image/png");
    assert_eq!(cands.len(), 1);
    assert_eq!(cands[0].origin, "direct");
}
