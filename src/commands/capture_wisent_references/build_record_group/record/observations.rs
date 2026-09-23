use super::*;

/// The record's journey: the steps a person takes through the product, in order.
pub(super) fn journey_steps(f: &Facts) -> Vec<Value> {
    let Facts { measured, steps, name, version_line, version_ok, refusal_line, refusal_status, invalid_cmd, .. } = *f;
    let ev = |kind: &str, extra: &str| f.ev(kind, extra);
    let journey_steps = vec![
        {
            json!({
                "index": 1,
                "user_action": format!("Open the recorded pseudo-terminal at `{}` in an empty scratch directory.", PROMPT.trim()),
                "system_response": "A clean prompt appears; no project, credential, host or queue target is selected.",
                "state": "ready prompt",
                "evidence": format!("media/session.cast at 0–{} s", g(steps["version"]["started_at"].as_f64().unwrap_or(0.0))),
            })
        },
        {
            json!({
                "index": 2,
                "user_action": format!("Run `{}`.", steps["version"]["command"].as_str().unwrap_or_default()),
                "system_response": if version_ok {
                    format!(
                        "{name} prints `{}` and exits {}.",
                        quote(version_line, 90),
                        steps["version"]["exit_status"]
                    )
                } else {
                    format!(
                        "{name} refuses the version flag with status {} and prints `{}`.",
                        steps["version"]["exit_status"],
                        quote(version_line, 90)
                    )
                },
                "state": "version identity",
                "evidence": ev("version", "media/01-version-identity.png"),
            })
        },
        {
            json!({
                "index": 3,
                "user_action": format!("Run `{}`.", steps["help"]["command"].as_str().unwrap_or_default()),
                "system_response": format!("{} lines of the product's own top-level help are printed.", steps["help"]["line_count"]),
                "state": "top-level help surface",
                "evidence": ev("help", "media/02-help-surface.png"),
            })
        },
        {
            json!({
                "index": 4,
                "user_action": format!("Run `{}`.", steps["subcommand-help"]["command"].as_str().unwrap_or_default()),
                "system_response": product.sub_note,
                "state": "subcommand help surface",
                "evidence": ev("subcommand-help", "media/03-subcommand-help.png"),
            })
        },
        {
            json!({
                "index": 5,
                "user_action": format!("Run `{invalid_cmd}`."),
                "system_response": format!("The option is refused: `{}`, status {}.", quote(refusal_line, 90), refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "state": "observed refusal",
                "evidence": ev("invalid-flag", "media/04-refusal.png"),
            })
        },
        {
            json!({
                "index": 6,
                "user_action": format!("Type `{invalid_cmd}` again and press Ctrl-C before Enter."),
                "system_response": if measured["cancel_prompt_restored"].as_bool().unwrap_or(false) {
                    "The unsubmitted line is discarded and the prompt returns; nothing ran."
                } else {
                    "Ctrl-C is accepted and the session continues at the prompt."
                },
                "state": "cancelled pending command",
                "evidence": format!(
                    "media/session.cast at {}–{} s",
                    g(steps["cancellation"]["started_at"].as_f64().unwrap_or(0.0)),
                    g(steps["cancellation"]["ended_at"].as_f64().unwrap_or(0.0))
                ),
            })
        },
        {
            json!({
                "index": 7,
                "user_action": format!("Run `{}` again.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "system_response": format!("Valid help is printed again with status {}: first success is recovered.", steps["recovery-help"]["exit_status"]),
                "state": "recovered first success",
                "evidence": ev("recovery-help", "media/05-recovery.png"),
            })
        },
        {
            json!({
                "index": 8,
                "user_action": format!("Run `{}`.", steps["no-color-help"]["command"].as_str().unwrap_or_default()),
                "system_response": if measured["no_color_text_identical"].as_bool().unwrap_or(false) {
                    "The same help text appears with colour removed, so no state was carried by colour."
                } else {
                    "The help text changes once colour is removed, which is recorded as a difference, not a claim of parity."
                },
                "state": "colour-free help",
                "evidence": ev("no-color-help", ""),
            })
        },
    ];
    journey_steps
}

/// What the recording shows about colour, width and the refusal naming the next action.
pub(super) struct Accessibility<'a> {
    pub(super) observations: Vec<Value>,
    pub(super) colors_help: bool,
    pub(super) identical: bool,
    pub(super) names_next: bool,
    pub(super) fits80: bool,
    pub(super) help_command: &'a str,
    pub(super) nocolor_command: &'a str,
}

pub(super) fn accessibility<'a>(f: &Facts<'a>) -> Accessibility<'a> {
    let Facts { measured, steps, name, refusal_status, invalid_cmd, phrase, .. } = *f;
    let colors_help = measured["colors_help"].as_bool().unwrap_or(false);
    let identical = measured["no_color_text_identical"]
        .as_bool()
        .unwrap_or(false);
    let names_next = measured["refusal_names_next_action"]
        .as_bool()
        .unwrap_or(false);
    let fits80 = measured["help_fits_80"].as_bool().unwrap_or(false);
    let help_command = steps["help"]["command"].as_str().unwrap_or_default();
    let nocolor_command = steps["no-color-help"]["command"]
        .as_str()
        .unwrap_or_default();

    let accessibility_observations = vec![
        format!(
            "Colour: with TERM=xterm-256color on a real pseudo-terminal and NO_COLOR unset, \
             `{help_command}` emitted {} ANSI SGR sequences, so this product {}",
            measured["help_sgr_count"],
            if colors_help {
                "does colour its help output."
            } else {
                "does not colour its help output."
            }
        ),
        format!(
            "NO_COLOR: `{nocolor_command}` printed {} lines with {} SGR sequences; the \
             ANSI-stripped text of the two runs is {}",
            steps["no-color-help"]["line_count"],
            measured["no_color_sgr_count"],
            if identical {
                "byte-identical, so every state the help communicates survives without colour."
            } else {
                "not identical, so the two runs are recorded as different rather than equivalent."
            }
        ),
        format!(
            "Refusal wording: the refusal for `{invalid_cmd}` {}",
            if names_next {
                format!(
                    "names the next action ({}) in the same output.",
                    py_repr(phrase)
                )
            } else {
                "does not name a next action anywhere in its output.".to_string()
            }
        ),
        format!(
            "Terminal width: the widest line of `{help_command}` is {} characters, so the help {}",
            measured["help_max_line_width"],
            if fits80 {
                "fits an 80-column terminal without wrapping."
            } else {
                "does not fit an 80-column terminal and will wrap."
            }
        ),
        "Input: the whole recorded journey is keyboard-only — typed commands, Enter and one \
         Ctrl-C — and every state is emitted as selectable terminal text, not as a drawn widget."
            .to_string(),
        format!(
            "State without colour: success and refusal differ by exit status in the cast \
             ({} against {}), which the shell prints as text.",
            steps["recovery-help"]["exit_status"],
            refusal_status
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".into())
        ),
    ];
    Accessibility { observations: accessibility_observations, colors_help, identical, names_next, fits80, help_command, nocolor_command }
}
