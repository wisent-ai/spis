use serde_json::json;
use spis::commands::crawl::executable_word_from_host_receipt;

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
