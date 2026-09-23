use super::*;

/// Exercises the lossy-VP8 WebP path against the catalog's committed
/// derivative images (expected dimensions verified via `file`).
#[test]
fn parses_committed_webp_derivatives() {
    let dir = Path::new("web-app-examples/images");
    if !dir.is_dir() {
        return; // fixture not present in this checkout
    }
    let slack = std::fs::read(dir.join("01-slack.webp")).unwrap();
    assert_eq!(parse_image_header(&slack), Some(("webp", 1000, 1000)));
    let discord = std::fs::read(dir.join("03-discord.webp")).unwrap();
    assert_eq!(parse_image_header(&discord), Some(("webp", 1200, 630)));
    for entry in std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("webp") {
            continue;
        }
        let data = std::fs::read(&path).unwrap();
        let (_, w, h) =
            parse_image_header(&data).unwrap_or_else(|| panic!("unparsed: {path:?}"));
        assert!(w > 0 && h > 0, "{path:?}");
    }
}
