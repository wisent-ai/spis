use super::*;

#[test]
fn strict_json_rejects_duplicate_keys() {
    assert!(strict_json(r#"{"a": 1, "b": 2}"#, "x").is_ok());
    let err = strict_json(r#"{"a": 1, "a": 2}"#, "x");
    assert!(err.is_err(), "{err:?}");
    assert!(strict_json(r#"{"a": [1, {"b": null}]}"#, "x").is_ok());
    assert!(strict_json(r#"{"a": 1,}"#, "x").is_err()); // trailing comma
    assert!(strict_json(r#"{"a": 1} extra"#, "x").is_err()); // trailing junk
    assert_eq!(
        strict_json(r#""é😀""#, "x").unwrap(),
        Value::String("é😀".to_string())
    );
}

#[test]
fn parses_record_selection() {
    assert_eq!(parse_record_selection(None).unwrap(), None);
    let sel = parse_record_selection(Some(&"3,5-7,10".to_string()))
        .unwrap()
        .unwrap();
    assert_eq!(sel, HashSet::from([3, 5, 6, 7, 10]));
    assert!(parse_record_selection(Some(&"".to_string())).is_err());
    assert!(parse_record_selection(Some(&"0".to_string())).is_err());
    assert!(parse_record_selection(Some(&"7-4".to_string())).is_err());
    assert!(parse_record_selection(Some(&"2,".to_string())).is_err());
}

#[test]
fn normalizes_catalog_names() {
    assert_eq!(normalize_catalog("web-app").unwrap(), "web-app-examples");
    assert_eq!(
        normalize_catalog("design-system-examples").unwrap(),
        "design-system-examples"
    );
    assert!(normalize_catalog("").is_err());
    assert!(normalize_catalog("Bad_Catalog").is_err());
    assert!(normalize_catalog("-examples-examples").is_err());
}

#[test]
fn canonicalizes_urls() {
    let v = Value::String("https://example.com/a?b=1#frag".into());
    assert_eq!(
        canonical_url(&v, "r", "f").unwrap(),
        "https://example.com/a?b=1#frag"
    );
    let v = Value::String("https://example.com".into());
    assert_eq!(canonical_url(&v, "r", "f").unwrap(), "https://example.com/");
    assert!(canonical_url(&Value::String("ftp://example.com/".into()), "r", "f").is_err());
    assert!(canonical_url(&Value::String("https://".into()), "r", "f").is_err());
    assert!(canonical_url(&Value::Null, "r", "f").is_err());
}

#[test]
fn slug_checks() {
    assert!(looks_like_slug("01-linear"));
    assert!(looks_like_slug("a"));
    assert!(looks_like_slug("a.b_c-d"));
    assert!(!looks_like_slug(""));
    assert!(!looks_like_slug("-lead"));
    assert!(!looks_like_slug("Has-Caps"));
    assert!(!looks_like_slug(&"a".repeat(82)));
    assert!(looks_like_slug(&"a".repeat(81)));
}

#[test]
fn hex_checks() {
    assert!(is_hex64_lower(&"a".repeat(64)));
    assert!(!is_hex64_lower(&"A".repeat(64)));
    assert!(!is_hex64_lower(&"g".repeat(64)));
    assert!(!is_hex64_lower("abc"));
}

#[test]
fn validates_plan_shape() {
    let reference = Reference {
        catalog: "web-app-examples".into(),
        index: 3,
        name: "Example".into(),
        slug: "03-example".into(),
        path: PathBuf::from("web-app-examples/03-example/reference.json"),
        source_url: "https://example.com/".into(),
    };
    let action = reference.action("batch-1");
    let plan = json!({
        "schema": PLAN_SCHEMA,
        "batch": "batch-1",
        "target": DEFAULT_TARGET,
        "captures": [Value::Object(action.clone())],
    });
    assert!(validate_plan(&plan, DEFAULT_TARGET, &[reference.clone()]).is_ok());

    // Extra key → rejected.
    let mut bad = plan.clone();
    bad.as_object_mut()
        .unwrap()
        .insert("extra".into(), Value::Null);
    assert!(validate_plan(&bad, DEFAULT_TARGET, &[reference.clone()]).is_err());

    // Wrong target → rejected.
    assert!(validate_plan(&plan, "other-host", &[reference.clone()]).is_err());

    // Mutated field → rejected.
    let mut mutated = plan.clone();
    mutated["captures"][0]["source_url"] = Value::String("https://evil.example/".into());
    assert!(validate_plan(&mutated, DEFAULT_TARGET, &[reference]).is_err());
}

#[test]
fn batch_stamp_format() {
    let batch = default_batch();
    // accessibility-YYYYMMDDtHHMMSSz
    assert_eq!(batch.len(), "accessibility-20260823t123456z".len());
    assert!(batch.starts_with("accessibility-"));
    assert!(batch.ends_with('z'));
    let stamp = &batch["accessibility-".len()..];
    assert!(stamp[8..9].as_bytes() == b"t");
    assert!(stamp[..8].bytes().all(|b| b.is_ascii_digit()));
    assert!(stamp[9..15].bytes().all(|b| b.is_ascii_digit()));
}
