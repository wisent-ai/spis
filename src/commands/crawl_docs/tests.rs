use super::*;

fn counts(target_count: usize, outside: u64) -> RetrievalCounts {
    RetrievalCounts {
        target_count,
        retrieved_count: target_count,
        ok_count: target_count,
        text_page_count: target_count,
        diagnostic_count: 0,
        pages_outside_corpus: outside,
    }
}

#[test]
fn a_record_that_fits_and_retrieved_cleanly_is_complete() {
    assert_eq!(retrieval_status(&counts(472, 0)), "retrieval_complete");
}

#[test]
fn a_record_over_the_bound_is_never_complete_however_clean_its_pages_are() {
    // Google Cloud: 216,092 in-scope URLs, 50,000 of them retrieved with
    // no per-page diagnostic at all. Before this it read `complete`.
    let over = counts(MAX_TARGETS, 216_092 - MAX_TARGETS as u64);
    assert_eq!(retrieval_status(&over), "retrieval_over_capacity");
}

#[test]
fn over_capacity_outranks_partial_because_a_retry_cannot_fix_it() {
    let mut over = counts(MAX_TARGETS, 1);
    over.diagnostic_count = 7;
    over.ok_count = MAX_TARGETS - 7;
    assert_eq!(retrieval_status(&over), "retrieval_over_capacity");
}

#[test]
fn a_page_level_failure_without_the_bound_is_still_partial() {
    let mut partial = counts(100, 0);
    partial.ok_count = 99;
    partial.diagnostic_count = 1;
    assert_eq!(retrieval_status(&partial), "retrieval_partial");
}

#[test]
fn an_empty_or_textless_run_keeps_its_own_state() {
    assert_eq!(retrieval_status(&counts(0, 0)), "retrieval_empty");
    let mut textless = counts(10, 5);
    textless.text_page_count = 0;
    // Even over the bound: a run that retrieved no text has a different
    // fault, and naming capacity first would hide it.
    assert_eq!(retrieval_status(&textless), "retrieval_no_text");
}

#[test]
fn the_inventory_digest_is_unchanged_for_a_record_that_fits() {
    // The compatibility guarantee: `corpus_capacity` is absent from the
    // hashed descriptor when nothing was excluded, so every corpus
    // written before the field existed keeps its digest.
    let robots = RobotsSnapshot::default();
    let legacy = lib::sha256_hex(
        &serde_json::to_vec(&json!({
            "targets": Vec::<CrawlTarget>::new(),
            "diagnostics": Vec::<CrawlDiagnostic>::new(),
            "robots": robots,
            "downloaded_bytes": 11u64,
        }))
        .expect("legacy descriptor"),
    );
    let current = inventory_sha256(&[], &[], &robots, 11, None).expect("digest");
    assert_eq!(current, legacy);
    let excluded = inventory_sha256(
        &[],
        &[],
        &robots,
        11,
        Some(CorpusCapacity {
            pages_outside_corpus: 166_092,
            exact: true,
        }),
    )
    .expect("digest");
    assert_ne!(excluded, legacy, "an excluded count must be covered");
}

#[test]
fn the_excluded_count_is_distinct_and_bounded() {
    let mut excluded = std::collections::HashSet::new();
    let mut exact = true;
    note_excluded(&mut excluded, &mut exact, "https://example.com/a");
    note_excluded(&mut excluded, &mut exact, "https://example.com/a");
    note_excluded(&mut excluded, &mut exact, "https://example.com/b");
    assert_eq!(excluded.len(), 2, "the same URL twice is one page");
    assert!(exact);
}
