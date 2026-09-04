use serde_json::json;
use spis::commands::crawl::{
    executable_word_from_host_receipt, failed_host_preflight_record_state,
    host_home_crawl_token_path, host_home_stado_path, host_probe_timeout_report,
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
fn worker_resolves_stado_resources_from_the_placement_home() {
    let home = Path::new("/Users/charles");

    assert_eq!(
        host_home_stado_path(home),
        Path::new("/Users/charles/.stado/bin/stado")
    );
    assert_eq!(
        host_home_crawl_token_path(home),
        Path::new("/Users/charles/.stado/spis-crawls-object-api-token")
    );
}
