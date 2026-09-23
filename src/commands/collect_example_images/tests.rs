use super::*;

#[test]
fn parses_png_header() {
    let mut data = b"\x89PNG\r\n\x1a\n".to_vec();
    data.extend_from_slice(&[0, 0, 0, 13]); // IHDR length
    data.extend_from_slice(b"IHDR");
    data.extend_from_slice(&1200u32.to_be_bytes());
    data.extend_from_slice(&630u32.to_be_bytes());
    assert_eq!(parse_image_header(&data), Some(("png", 1200, 630)));
}

#[test]
fn parses_jpeg_header() {
    let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    data.extend(b"JFIF\x00");
    data.extend_from_slice(&[0u8; 9]);
    // SOF0 marker
    data.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 0x08]);
    data.extend_from_slice(&800u16.to_be_bytes()); // height
    data.extend_from_slice(&1400u16.to_be_bytes()); // width
    data.push(3);
    assert_eq!(parse_image_header(&data), Some(("jpeg", 1400, 800)));
}

#[test]
fn parses_webp_vp8l_header() {
    let bits_w_minus_1 = 1023u32;
    let bits_h_minus_1 = 700u32 - 1;
    let payload: u32 = 0x2F | bits_w_minus_1 << 8 | bits_h_minus_1 << 22;
    let mut chunk = vec![b'V', b'P', b'8', b'L'];
    chunk.extend_from_slice(&(payload.to_le_bytes().len() as u32 + 5).to_le_bytes());
    chunk.push(0x2F);
    chunk.extend_from_slice(&(bits_w_minus_1 | bits_h_minus_1 << 14).to_le_bytes());
    let mut data = b"RIFF".to_vec();
    data.extend_from_slice(&(chunk.len() as u32 + 4).to_le_bytes());
    data.extend_from_slice(b"WEBP");
    data.extend_from_slice(&chunk);
    assert_eq!(parse_image_header(&data), Some(("webp", 1024, 700)));
}

#[test]
fn parses_gif_header() {
    let mut data = b"GIF89a".to_vec();
    data.extend_from_slice(&500u16.to_le_bytes());
    data.extend_from_slice(&300u16.to_le_bytes());
    assert_eq!(parse_image_header(&data), Some(("gif", 500, 300)));
}

#[test]
fn rejects_unknown_bytes() {
    assert_eq!(parse_image_header(b"<html>not an image</html>"), None);
    assert_eq!(parse_image_header(&[]), None);
}

#[test]
fn joins_urls() {
    let base = "https://example.com/products/app/";
    assert_eq!(
        join_url(base, "https://cdn.example.com/x.png").as_deref(),
        Some("https://cdn.example.com/x.png")
    );
    assert_eq!(
        join_url(base, "//cdn.example.com/y.png").as_deref(),
        Some("https://cdn.example.com/y.png")
    );
    assert_eq!(
        join_url("https://example.com/a/b.html", "/img/z.png").as_deref(),
        Some("https://example.com/img/z.png")
    );
    assert_eq!(
        join_url(base, "../shot.png?v=2").as_deref(),
        Some("https://example.com/products/shot.png?v=2")
    );
    // urljoin passes absolute URIs through; the http(s)/netloc filter in
    // candidate_urls drops them afterwards.
    assert_eq!(
        join_url(base, "data:image/png;base64,xx").as_deref(),
        Some("data:image/png;base64,xx")
    );
}

#[test]
fn cleans_urls() {
    assert_eq!(
        clean_url("https://a.example/x y.png&amp;b=1"),
        "https://a.example/x%20y.png&b=1"
    );
}

#[test]
fn provenance_strips_signed_query() {
    assert_eq!(
        stable_provenance_url(
            "https://private-user-images.githubusercontent.com/abc.png?jwt=x.y.z"
        ),
        "https://private-user-images.githubusercontent.com/abc.png"
    );
    assert_eq!(
        stable_provenance_url("https://example.com/a.png?token=1"),
        "https://example.com/a.png?token=1"
    );
}

#[test]
fn slugs_names() {
    assert_eq!(slugify_name("Acme Dashboard!"), "acme-dashboard");
    assert_eq!(slugify_name("  --Foo__Bar--"), "foo-bar");
    // Mirrors re.sub(r"[^a-z0-9]+", "-", "ünïcode").strip("-").
    assert_eq!(slugify_name("Ünïcode"), "n-code");
}

#[test]
fn scores_prefers_screenshot_words() {
    let good = Candidate {
        url: "https://x/screenshot-dashboard.png".into(),
        hint: String::new(),
        order: 0,
        origin: "img",
    };
    let bad = Candidate {
        url: "https://x/favicon.ico".into(),
        hint: String::new(),
        order: 9,
        origin: "img",
    };
    assert!(preflight_score(&good) > preflight_score(&bad));
}
