use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const PRODUCT_ID: &str = "spis";
const JOURNEY_ID: &str = "first-use";
const JOURNEY_VERSION: &str = "2026-09-03.1";
const FIRST_SUCCESS_FACT: &str = "catalog_validation_reported";
const STATE_SCHEMA: &str = "spis.onboarding-state.v1";
const FALLBACK: &str = include_str!("onboarding_first_use.json");

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
    let path = state_path();
    if !path.exists() {
        return Ok(());
    }

    let mut state = read_state(&path)?;
    validate_state_identity(&state)?;
    if state.get("status").and_then(Value::as_str) == Some("completed") {
        return Ok(());
    }

    let evidence = state
        .get_mut("evidence")
        .and_then(Value::as_object_mut)
        .context("onboarding state has no evidence object; use `spis onboarding --reset`")?;
    evidence.insert(FIRST_SUCCESS_FACT.to_string(), Value::Bool(true));
    save_state(&state)
}

fn canonical_definition() -> Result<Value> {
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

fn screen_by_id<'a>(definition: &'a Value, screen_id: &str) -> Result<&'a Value> {
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

fn next_screen_id(screen: &Value) -> Result<Option<String>> {
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

fn evidence_satisfied(screen: &Value, state: &Value) -> Result<bool> {
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

fn load_or_start_state(definition: &Value, reset: bool) -> Result<Value> {
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

fn read_state(path: &PathBuf) -> Result<Value> {
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

fn validate_state_identity(state: &Value) -> Result<()> {
    if state.get("schema").and_then(Value::as_str) != Some(STATE_SCHEMA)
        || state.get("product_id").and_then(Value::as_str) != Some(PRODUCT_ID)
        || state.get("journey_id").and_then(Value::as_str) != Some(JOURNEY_ID)
        || state.get("journey_version").and_then(Value::as_str) != Some(JOURNEY_VERSION)
    {
        bail!("stored onboarding state identity mismatch; use `spis onboarding --reset`");
    }
    Ok(())
}

fn save_state(state: &Value) -> Result<()> {
    let path = state_path();
    let parent = path
        .parent()
        .context("onboarding state path has no parent")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create onboarding state directory {}", parent.display()))?;

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary = path.with_extension(format!("json.{}.{}.tmp", std::process::id(), nonce));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .with_context(|| format!("create onboarding state {}", temporary.display()))?;
    let body = serde_json::to_vec(state)?;
    file.write_all(&body)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(&temporary, &path)
        .with_context(|| format!("replace onboarding state {}", path.display()))?;
    Ok(())
}

fn render(screen: &Value) -> Result<()> {
    let presentation = screen
        .get("presentation")
        .and_then(Value::as_object)
        .context("canonical onboarding screen has no presentation")?;
    let title = presentation
        .get("title")
        .and_then(Value::as_str)
        .context("canonical onboarding screen has no presentation title")?;
    let body = presentation
        .get("body")
        .and_then(Value::as_str)
        .context("canonical onboarding screen has no presentation body")?;
    println!("\n== {title} ==\n{body}");
    Ok(())
}

fn wait_for_enter() -> Result<()> {
    print!("\nPress Enter to continue.");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(())
}

fn state_path() -> PathBuf {
    if let Some(path) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(path).join("spis/onboarding.json");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local/state/spis/onboarding.json");
    }
    PathBuf::from(".spis/onboarding.json")
}
