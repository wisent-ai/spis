//! One crawled record's refusal, shared by every crawl surface: the stable
//! code its worker report carries and the sentence that says what happened.

#[derive(Debug)]
pub(crate) struct RecordFailure {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

impl std::fmt::Display for RecordFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for RecordFailure {}

/// The code of the first record refusal in `error`'s chain. An error that
/// carries none is reported under `uncoded`, the code the calling surface
/// names for a record that failed without a named refusal; the error's own
/// sentence travels beside it in the report.
pub(crate) fn record_failure_code(error: &anyhow::Error, uncoded: &'static str) -> &'static str {
    match error
        .chain()
        .find_map(|cause| cause.downcast_ref::<RecordFailure>())
    {
        Some(failure) => failure.code,
        None => uncoded,
    }
}
