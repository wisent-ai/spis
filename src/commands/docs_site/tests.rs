use super::*;

#[test]
fn the_cli_reference_names_the_commands_the_binary_dispatches() {
    let bullets = command_bullets();
    assert_eq!(bullets.len(), super::super::SUBCOMMANDS.len());
    assert!(bullets.iter().any(|line| line.starts_with("`crawl`")));
    assert!(bullets.iter().any(|line| line.starts_with("`docs-corpus`")));
    // The removed Python-era shim's commands must not reappear: the
    // committed brief listed `capture`, `catalogs`, `drift`, `verify` and
    // `sync-readmes`, none of which the binary has.
    for ghost in ["`capture`", "`catalogs`", "`drift`", "`verify`", "`sync-readmes`"] {
        assert!(
            !bullets.iter().any(|line| line.starts_with(ghost)),
            "{ghost} is not a command this binary dispatches"
        );
    }
}

#[test]
fn every_family_is_documented_with_the_preconditions_the_preflight_enforces() {
    let rows = family_rows();
    assert_eq!(rows.len(), 15, "fifteen families, not the documented 13");
    for row in &rows {
        let requires: Vec<&str> = row["requires"]
            .as_array()
            .expect("a requires list")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert!(
            requires.contains(&"cargo --version"),
            "{} does not document the worker program",
            row["family"]
        );
    }
}

#[test]
fn the_documented_bound_is_the_bound_the_code_enforces() {
    let generated = plan("test");
    let text = serde_json::to_string(&generated).expect("a plan");
    assert!(text.contains(&super::super::docs_corpus::MAX_PAGE_RECORDS.to_string()));
    // And the four real sites are named rather than described.
    assert!(text.contains("21-google-cloud-documentation"));
    assert!(text.contains("01-mdn-web-docs"));
}

#[test]
fn the_desktop_surface_is_declared_so_parity_can_be_checked() {
    let declared = brief("test");
    assert_eq!(declared["surfaces"]["desktop"]["declared"], json!(true));
    assert_eq!(declared["surfaces"]["desktop"]["parity_with"], json!("cli"));
}

#[test]
fn the_site_payload_uses_the_canonical_doc_page_model() {
    let payload = docs_payload(&plan("test")).expect("a payload");
    assert_eq!(payload["schema"], json!("wisent.spis-doc-pages.v1"));
    assert_eq!(payload["generatedBy"], json!("spis docs-site"));
    let pages = payload["pages"].as_array().expect("pages");
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0]["slug"], json!("corpus-adoption"));
    assert_eq!(pages[0]["category"], json!("Getting started"));
    assert_eq!(
        pages[0]["sections"][0]["blocks"][0]["type"],
        json!("code")
    );
}
