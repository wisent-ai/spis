use super::*;

#[test]
fn compact_stamp_shape() {
    let stamp = compact_stamp();
    // %Y%m%dt%H%M%SZ → 8 digits, 't', 6 digits, 'Z'.
    assert_eq!(stamp.len(), 16);
    assert_eq!(&stamp[8..9], "t");
    assert!(stamp.ends_with('Z'));
    assert!(stamp[..8].bytes().all(|b| b.is_ascii_digit()));
    assert!(stamp[9..15].bytes().all(|b| b.is_ascii_digit()));
}

#[test]
fn finds_batch_ids() {
    assert_eq!(
        find_batch_id("queued\nBatch: batch-1724410001\nok"),
        Some("batch-1724410001".to_string())
    );
    assert_eq!(find_batch_id("no id here"), None);
    assert_eq!(find_batch_id("Batch: notanumber"), None);
    assert_eq!(find_batch_id("Batch: batch-12x"), None);
}
