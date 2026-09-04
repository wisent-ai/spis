use serde_json::json;
use spis::commands::crawl::{
    active_worker_stado_programs, executable_word_from_host_receipt,
    failed_host_preflight_record_state, host_home_crawl_token_path, host_probe_timeout_report,
    validate_worker_runtime_files, worker_agent_program_retry_diagnostic,
};
use std::path::Path;

#[test]
fn home_relative_executable_is_expanded_by_the_placement_host() {
    let receipt = json!({
        "target": "charless-mac-mini",
        "argv": ["/opt/homebrew/bin/cargo", "--version"],
        "resolved_executable": "~/.cargo/bin/cargo"
    });

    let word = executable_word_from_host_receipt("charless-mac-mini", &receipt).unwrap();

    assert_eq!(word, "\"$HOME\"/'.cargo/bin/cargo'");
    assert!(!word.contains("/Users/lukaszbartoszcze"));
    assert!(!word.contains("/opt/homebrew/bin/cargo"));
}

#[test]
fn executable_receipt_from_another_host_is_refused() {
    let receipt = json!({
        "target": "lukasz-macbook",
        "argv": ["/opt/homebrew/bin/cargo", "--version"],
        "resolved_executable": "~/.cargo/bin/cargo"
    });

    let error = executable_word_from_host_receipt("charless-mac-mini", &receipt).unwrap_err();

    assert_eq!(
        error.to_string(),
        "host probe receipt target \"lukasz-macbook\" does not match placement host \"charless-mac-mini\""
    );
}

#[test]
fn unanswered_probe_is_retryable_and_keeps_the_record_planned() {
    let check = host_probe_timeout_report(&["hostname", "-f"], 30);
    let report = json!({
        "ready": false,
        "checks": [check]
    });

    assert_eq!(report["checks"][0]["outcome"], "timed_out");
    assert_eq!(
        report["checks"][0]["diagnostic"]["code"],
        "host_probe_timed_out"
    );
    assert_eq!(report["checks"][0]["diagnostic"]["timeout_seconds"], 30);
    assert_eq!(
        report["checks"][0]["diagnostic"]["probe"],
        json!(["hostname", "-f"])
    );
    assert_eq!(failed_host_preflight_record_state(&report), "planned");
    assert_ne!(failed_host_preflight_record_state(&report), "unavailable");
}

#[test]
fn missing_worker_runtime_files_have_distinct_named_failures() {
    let stado = Path::new("/declared/stado");
    let token = Path::new("/Users/charles/.stado/spis-crawls-object-api-token");

    let binary_error = validate_worker_runtime_files(stado, false, token, true).unwrap_err();
    assert_eq!(
        binary_error.to_string(),
        "worker_stado_binary_missing: Stado's declared worker binary is missing at /declared/stado"
    );

    let token_error = validate_worker_runtime_files(stado, true, token, false).unwrap_err();
    assert_eq!(
        token_error.to_string(),
        "worker_crawl_token_missing: the spis-crawls bearer file is missing at /Users/charles/.stado/spis-crawls-object-api-token"
    );
    assert_eq!(
        host_home_crawl_token_path(Path::new("/Users/charles")),
        token
    );
}

#[test]
fn overlapping_agent_declarations_are_unambiguous_when_the_program_matches() {
    let services = json!([
        {
            "host": "charless-mac-mini",
            "state": "active",
            "program": "/Users/charles/.stado/bin/stado",
            "args": ["agent", "--target", "charless-mac-mini"]
        },
        {
            "host": "charless-mac-mini",
            "state": "active",
            "program": "/Users/charles/.stado/bin/stado",
            "args": ["agent", "--target", "charless-mac-mini"]
        }
    ]);

    let programs = active_worker_stado_programs(&services, "charless-mac-mini");

    assert_eq!(
        programs.into_iter().collect::<Vec<_>>(),
        vec!["/Users/charles/.stado/bin/stado"]
    );
}

#[test]
fn unstable_agent_cardinality_is_named_counted_and_retryable() {
    let diagnostic = worker_agent_program_retry_diagnostic(
        "worker_agent_program_unstable: host=charless-mac-mini observed_count=0 observations=6 retryable=true",
    )
    .unwrap();

    assert_eq!(diagnostic["code"], "worker_agent_program_unstable");
    assert_eq!(diagnostic["retryable"], true);
    assert_eq!(diagnostic["observed_count"], 0);
}
