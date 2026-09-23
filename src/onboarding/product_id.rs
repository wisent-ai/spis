use super::*;

pub(crate) const PRODUCT_ID: &str = "spis";

pub(crate) const JOURNEY_ID: &str = "first-use";

pub(crate) const JOURNEY_VERSION: &str = "2026-09-05.1";

pub(crate) const FIRST_SUCCESS_FACT: &str = "corpus_adopted";

pub(crate) const STATE_SCHEMA: &str = "spis.onboarding-state.v1";

pub(crate) const FALLBACK: &str = include_str!("../onboarding_first_use.json");

pub fn run(rest: &[String]) -> Result<()> {
    let mut reset = false;
    for argument in rest {
        match argument.as_str() {
            "--reset" => reset = true,
            "--help" | "-h" => {
                println!("usage: spis onboarding [--reset]");
                println!("  --reset  discard progress and evidence, then restart the walkthrough");
                return Ok(());
            }
            other => bail!("unknown argument: {other} (expected --reset)"),
        }
    }

    let definition = canonical_definition()?;
    let mut state = load_or_start_state(&definition, reset)?;
    if reset {
        println!("Walkthrough progress and first-success evidence reset.");
    }

    if state.get("status").and_then(Value::as_str) == Some("completed") {
        println!("Walkthrough already completed. Run `spis onboarding --reset` to show it again.");
        return Ok(());
    }

    loop {
        let screen_id = state
            .get("current_screen_id")
            .and_then(Value::as_str)
            .context("onboarding state has no current screen")?
            .to_string();
        let screen = screen_by_id(&definition, &screen_id)?;
        render(screen)?;

        if screen.get("screen_kind").and_then(Value::as_str) == Some("first_success") {
            if evidence_satisfied(screen, &state)? {
                state["status"] = Value::String("completed".to_string());
                save_state(&state)?;
                println!("\nWalkthrough completed.");
            } else {
                println!("\nAfter the command succeeds, run `spis onboarding` again.");
            }
            return Ok(());
        }

        wait_for_enter()?;
        let next = next_screen_id(screen)?
            .context("published onboarding screen has no eligible next screen")?;
        screen_by_id(&definition, &next)?;
        state["current_screen_id"] = Value::String(next);
        save_state(&state)?;
    }
}

pub fn record_first_success() -> Result<()> {
    let definition = canonical_definition()?;
    let entry = definition
        .get("entry_screen_id")
        .and_then(Value::as_str)
        .context("canonical onboarding journey has no entry screen")?;
    let path = state_path();
    let mut state = if path.exists() {
        match read_state(&path) {
            Ok(state) if validate_state_identity(&state).is_ok() => state,
            _ => json!({
                "schema": STATE_SCHEMA,
                "product_id": PRODUCT_ID,
                "journey_id": JOURNEY_ID,
                "journey_version": JOURNEY_VERSION,
                "current_screen_id": entry,
                "status": "in_progress",
                "evidence": Map::<String, Value>::new(),
            }),
        }
    } else {
        json!({
            "schema": STATE_SCHEMA,
            "product_id": PRODUCT_ID,
            "journey_id": JOURNEY_ID,
            "journey_version": JOURNEY_VERSION,
            "current_screen_id": entry,
            "status": "in_progress",
            "evidence": Map::<String, Value>::new(),
        })
    };
    let evidence = state
        .get_mut("evidence")
        .and_then(Value::as_object_mut)
        .context("onboarding state has no evidence object")?;
    evidence.insert(FIRST_SUCCESS_FACT.to_string(), Value::Bool(true));
    state["status"] = Value::String("completed".to_string());
    save_state(&state)
}

pub(crate) fn canonical_definition() -> Result<Value> {
    let definition: Value =
        serde_json::from_str(FALLBACK).context("parse canonical onboarding journey")?;
    if definition.get("schema_version").and_then(Value::as_u64) != Some(1)
        || definition.get("product_id").and_then(Value::as_str) != Some(PRODUCT_ID)
        || definition.get("journey_id").and_then(Value::as_str) != Some(JOURNEY_ID)
        || definition.get("journey_version").and_then(Value::as_str) != Some(JOURNEY_VERSION)
        || definition.get("first_success_fact").and_then(Value::as_str)
            != Some(FIRST_SUCCESS_FACT)
    {
        bail!("canonical onboarding journey identity mismatch");
    }

    let entry = definition
        .get("entry_screen_id")
        .and_then(Value::as_str)
        .context("canonical onboarding journey has no entry screen")?;
    let screens = definition
        .get("screens")
        .and_then(Value::as_array)
        .context("canonical onboarding journey has no screens")?;
    let mut ids = HashSet::new();
    for screen in screens {
        let id = screen
            .get("screen_id")
            .and_then(Value::as_str)
            .context("canonical onboarding screen has no id")?;
        if !ids.insert(id) {
            bail!("duplicate canonical onboarding screen id: {id}");
        }
        let presentation = screen
            .get("presentation")
            .and_then(Value::as_object)
            .context("canonical onboarding screen has no presentation")?;
        presentation
            .get("title")
            .and_then(Value::as_str)
            .context("canonical onboarding screen has no presentation title")?;
        presentation
            .get("body")
            .and_then(Value::as_str)
            .context("canonical onboarding screen has no presentation body")?;
    }
    if !ids.contains(entry) {
        bail!("canonical onboarding entry screen does not exist");
    }
    for screen in screens {
        for transition in screen
            .get("transitions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let next = transition
                .get("next_screen_id")
                .and_then(Value::as_str)
                .context("canonical onboarding transition has no target")?;
            if !ids.contains(next) {
                bail!("canonical onboarding transition target does not exist: {next}");
            }
        }
    }
    Ok(definition)
}

pub(crate) fn screen_by_id<'a>(definition: &'a Value, screen_id: &str) -> Result<&'a Value> {
    definition
        .get("screens")
        .and_then(Value::as_array)
        .and_then(|screens| {
            screens
                .iter()
                .find(|screen| screen.get("screen_id").and_then(Value::as_str) == Some(screen_id))
        })
        .with_context(|| format!("published onboarding screen is unavailable: {screen_id}"))
}

pub(crate) fn next_screen_id(screen: &Value) -> Result<Option<String>> {
    screen
        .get("transitions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .max_by_key(|transition| {
            transition
                .get("priority")
                .and_then(Value::as_i64)
                .unwrap_or_default()
        })
        .map(|transition| {
            transition
                .get("next_screen_id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .context("canonical onboarding transition has no target")
        })
        .transpose()
}

pub(crate) fn evidence_satisfied(screen: &Value, state: &Value) -> Result<bool> {
    let rule = screen
        .get("completion_evidence")
        .context("first-success screen has no completion evidence")?;
    if rule.get("kind").and_then(Value::as_str) != Some("fact")
        || rule.get("operator").and_then(Value::as_str) != Some("eq")
    {
        bail!("unsupported canonical onboarding evidence rule");
    }
    let fact = rule
        .get("fact")
        .and_then(Value::as_str)
        .context("canonical onboarding evidence rule has no fact")?;
    let expected = rule
        .get("value")
        .context("canonical onboarding evidence rule has no expected value")?;
    Ok(state.pointer(&format!("/evidence/{fact}")) == Some(expected))
}

pub(crate) fn load_or_start_state(definition: &Value, reset: bool) -> Result<Value> {
    let path = state_path();
    if !reset && path.exists() {
        let state = read_state(&path)?;
        validate_state_identity(&state)?;
        let current = state
            .get("current_screen_id")
            .and_then(Value::as_str)
            .context("stored onboarding state has no current screen")?;
        screen_by_id(definition, current)?;
        return Ok(state);
    }

    let entry = definition
        .get("entry_screen_id")
        .and_then(Value::as_str)
        .context("canonical onboarding journey has no entry screen")?;
    let state = json!({
        "schema": STATE_SCHEMA,
        "product_id": PRODUCT_ID,
        "journey_id": JOURNEY_ID,
        "journey_version": JOURNEY_VERSION,
        "current_screen_id": entry,
        "status": "in_progress",
        "evidence": Map::<String, Value>::new(),
    });
    save_state(&state)?;
    Ok(state)
}

pub(crate) fn read_state(path: &PathBuf) -> Result<Value> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .with_context(|| format!("read onboarding state {}", path.display()))?,
    )
    .with_context(|| {
        format!(
            "parse onboarding state {}; use `spis onboarding --reset`",
            path.display()
        )
    })
}

pub(crate) fn validate_state_identity(state: &Value) -> Result<()> {
    if state.get("schema").and_then(Value::as_str) != Some(STATE_SCHEMA)
        || state.get("product_id").and_then(Value::as_str) != Some(PRODUCT_ID)
        || state.get("journey_id").and_then(Value::as_str) != Some(JOURNEY_ID)
        || state.get("journey_version").and_then(Value::as_str) != Some(JOURNEY_VERSION)
    {
        bail!("stored onboarding state identity mismatch; use `spis onboarding --reset`");
    }
    Ok(())
}
