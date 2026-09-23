use super::*;

pub(crate) const STATE_PLAN: &[(&str, &str, &str)] = &[
    ("version", "01-version-identity", "version identity"),
    ("help", "02-help-surface", "top-level help surface"),
    (
        "subcommand-help",
        "03-subcommand-help",
        "subcommand help surface",
    ),
    (
        "invalid-flag",
        "04-refusal",
        "refusal after the invalid flag",
    ),
    (
        "recovery-help",
        "05-recovery",
        "recovered help after the refusal",
    ),
];

pub(crate) fn capture(index: usize, product: &'static Product) -> Result<Run> {
    let binary_path =
        resolve(product).ok_or_else(|| anyhow!("{} is not on PATH", product.binary))?;

    let workdir = scratch_root().join("run").join(product.slug);
    if workdir.exists() {
        std::fs::remove_dir_all(&workdir)
            .with_context(|| format!("clear {}", workdir.display()))?;
    }
    std::fs::create_dir_all(&workdir)?;

    let invalid_cmd = format!("{} {PROBE_FLAG}", product.binary);
    let mut commands: BTreeMap<String, String> = BTreeMap::new();
    commands.insert("version".into(), product.version_cmd.into());
    commands.insert("help".into(), product.help_cmd.into());
    commands.insert("subcommand-help".into(), product.sub_cmd.into());
    commands.insert("invalid-flag".into(), invalid_cmd.clone());
    commands.insert("cancellation".into(), invalid_cmd.clone());
    commands.insert("recovery-help".into(), product.help_cmd.into());
    commands.insert(
        "no-color-help".into(),
        format!("NO_COLOR=1 {}", product.help_cmd),
    );

    let mut session = Session::new(&workdir)?;
    let mut steps = BTreeMap::new();
    let result = (|| -> Result<()> {
        for (kind, _label) in STEP_PLAN {
            println!("    {kind}: {}", commands[*kind]);
            let mut step = if *kind == "cancellation" {
                session.cancel(&commands[*kind])
            } else {
                session.command(&commands[*kind], 180.0)
            };
            step.kind = (*kind).to_string();
            step.event_index = session.events.len().saturating_sub(1);
            steps.insert((*kind).to_string(), step);
        }
        Ok(())
    })();
    let events = session.events.clone();
    let wall_start = now_unix_secs();
    if let Err(e) = result {
        bail!("session failed mid-run: {e:#}");
    }
    drop(session);

    Ok(Run {
        index,
        product,
        binary_path,
        workdir: workdir.to_string_lossy().into_owned(),
        events,
        wall_start,
        steps,
    })
}

// --------------------------------------------------------------- measurement

pub(crate) fn step_output(raw: &str) -> &str {
    // The command's own output: the echoed command line and trailing prompt removed.
    let body = match raw.find('\n') {
        Some(i) => &raw[i + 1..],
        None => "",
    };
    if body.ends_with(PROMPT) {
        &body[..body.len() - PROMPT.len()]
    } else {
        body
    }
}

pub(crate) fn measure(run: &Run) -> Value {
    let mut steps = Map::new();
    for (kind, _label) in STEP_PLAN {
        let step = &run.steps[*kind];
        let raw = step_output(&step.raw);
        let plain = strip_ansi(raw);
        let mut lines = visible_lines(raw);
        while lines.last().is_some_and(|l| l.trim().is_empty()) {
            lines.pop();
        }
        let line_count = lines.len();
        let max_line_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        steps.insert(
            (*kind).to_string(),
            json!({
                "command": step.command,
                "exit_status": step.exit_status,
                "started_at": step.started_at,
                "ended_at": step.ended_at,
                "status_reported_at": step.status_reported_at,
                "elapsed_seconds": round_n(step.ended_at - step.started_at, 3),
                "event_index": step.event_index,
                "sgr_sequences": sgr_re().find_iter(raw).count(),
                "line_count": line_count,
                "max_line_width": max_line_width,
                "first_line": lines.first().map(|l| l.trim().to_string()).unwrap_or_default(),
                "last_line": lines.last().map(|l| l.trim().to_string()).unwrap_or_default(),
                "text": plain,
                "lines": lines,
            }),
        );
    }
    let help_m = steps["help"].clone();
    let nocolor_m = steps["no-color-help"].clone();
    let invalid_m = steps["invalid-flag"].clone();
    let version_m = steps["version"].clone();
    let same_text = help_m["text"].as_str().unwrap_or_default().trim()
        == nocolor_m["text"].as_str().unwrap_or_default().trim();
    let cancel_raw = run.steps["cancellation"].raw.clone();
    let all_events: String = run.events.iter().map(|(_, t)| t.as_str()).collect();
    let match_phrase = next_action_re()
        .find(invalid_m["text"].as_str().unwrap_or_default())
        .map(|m| m.as_str().to_string());

    json!({
        "steps": Value::Object(steps),
        "colors_help": help_m["sgr_sequences"].as_i64().unwrap_or(0) > 0,
        "help_sgr_count": help_m["sgr_sequences"].clone(),
        "no_color_sgr_count": nocolor_m["sgr_sequences"].clone(),
        "no_color_text_identical": same_text,
        "help_max_line_width": help_m["max_line_width"].clone(),
        "help_fits_80": help_m["max_line_width"].as_i64().unwrap_or(0) <= 80,
        "help_line_count": help_m["line_count"].clone(),
        "refusal_names_next_action": match_phrase.is_some(),
        "refusal_next_action_phrase": match_phrase,
        "refusal_first_line": invalid_m["first_line"].clone(),
        "refusal_exit_status": invalid_m["exit_status"].clone(),
        "version_first_line": version_m["first_line"].clone(),
        "version_exit_status": version_m["exit_status"].clone(),
        "version_flag_supported": version_m["exit_status"].as_i64() == Some(0),
        "cancel_echoed_interrupt": cancel_raw.contains("^C"),
        "cancel_prompt_restored": cancel_raw.trim_end().ends_with(PROMPT.trim()),
        "screen_cleared": all_events.contains("\x1b[2J"),
        "cursor_addressed": cursor_re().is_match(&all_events),
    })
}

pub(crate) fn timing_class(measured: &Value) -> String {
    let spans: Vec<f64> = STEP_PLAN
        .iter()
        .filter(|(k, _)| *k != "cancellation")
        .map(|(k, _)| {
            measured["steps"][*k]["elapsed_seconds"]
                .as_f64()
                .unwrap_or(0.0)
        })
        .collect();
    let slowest = spans.iter().cloned().fold(0.0f64, f64::max);
    if slowest < 0.05 {
        "instant".into()
    } else if slowest < 1.0 {
        "sub-second".into()
    } else if slowest <= 3.0 {
        "one-to-three-seconds".into()
    } else {
        "multi-second".into()
    }
}

pub(crate) fn timing_description(measured: &Value) -> String {
    let spans: Vec<f64> = STEP_PLAN
        .iter()
        .filter(|(k, _)| *k != "cancellation")
        .map(|(k, _)| {
            measured["steps"][*k]["elapsed_seconds"]
                .as_f64()
                .unwrap_or(0.0)
        })
        .collect();
    let fastest = spans.iter().cloned().fold(f64::INFINITY, f64::min);
    let slowest = spans.iter().cloned().fold(0.0f64, f64::max);
    format!(
        "The six submitted commands each completed between {} s and {} s; \
         the other pauses are the recorder typing and the deliberate Ctrl-C pause.",
        g(fastest),
        g(slowest)
    )
}
