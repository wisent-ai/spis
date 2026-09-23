use super::*;

// -------------------------------------------------------------- record build

pub(crate) fn build_record(run: &Run, measured: &Value, media: &Value) -> Value {
    let product = run.product;
    let steps = &measured["steps"];
    let cast = &media["cast"];
    let states = &media["states"];
    let name = product.name;
    let binary = product.binary;

    let ev = |kind: &str, extra: &str| -> String {
        let s = &steps[kind];
        let mut base = format!(
            "media/session.cast at {}–{} s",
            g(s["started_at"].as_f64().unwrap_or(0.0)),
            g(s["ended_at"].as_f64().unwrap_or(0.0)),
        );
        if let Some(st) = s["exit_status"].as_i64() {
            base.push_str(&format!("; observed exit {st}"));
        }
        if !extra.is_empty() {
            base.push_str("; ");
            base.push_str(extra);
        }
        base
    };

    let version_line = steps["version"]["first_line"].as_str().unwrap_or_default();
    let version_ok = measured["version_flag_supported"]
        .as_bool()
        .unwrap_or(false);
    let refusal_line = measured["refusal_first_line"].as_str().unwrap_or_default();
    let refusal_status = measured["refusal_exit_status"].as_i64();
    let recovery_line = steps["recovery-help"]["first_line"]
        .as_str()
        .unwrap_or_default();
    let invalid_cmd = steps["invalid-flag"]["command"]
        .as_str()
        .unwrap_or_default();

    let cancellation_sentence = if measured["cancel_prompt_restored"]
        .as_bool()
        .unwrap_or(false)
    {
        format!(
            "Ctrl-C on the unsubmitted `{invalid_cmd}` line discarded it and restored the prompt"
        )
    } else {
        format!("Ctrl-C was sent on the unsubmitted `{invalid_cmd}` line and the session continued at the prompt")
    };
    let phrase = measured["refusal_next_action_phrase"]
        .as_str()
        .unwrap_or("");

    let interactions = vec![
        {
            let s = &steps["version"];
            json!({
                "name": "command entry",
                "trigger": format!("Type `{}` at the `{}` prompt and press Enter.", s["command"].as_str().unwrap_or_default(), PROMPT.trim()),
                "response": format!("{name} starts from {} and writes to the pseudo-terminal.", run.binary_path),
                "feedback": if version_line.is_empty() { "no output on the version form".to_string() } else { quote(version_line, 160) },
                "cancellation": format!("{cancellation_sentence}; nothing was submitted."),
                "failure": format!("`{invalid_cmd}` reaches the same parser and is refused with status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "recovery": format!("Re-enter `{}`.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "evidence": ev("version", "media/01-version-identity.png"),
            })
        },
        {
            let s = &steps["version"];
            let st = s["exit_status"].as_i64();
            json!({
                "name": "version identity",
                "trigger": format!("Ask the installed binary what it is with `{}`.", s["command"].as_str().unwrap_or_default()),
                "response": if version_ok {
                    format!("{name} prints its version identity and exits 0.")
                } else {
                    format!(
                        "{name} has no version flag: it refuses the option with status {} and answers with its usage surface instead.",
                        st.map(|v| v.to_string()).unwrap_or_else(|| "None".into())
                    )
                },
                "feedback": if version_line.is_empty() {
                    format!("The process returns exit status {}.", st.map(|v| v.to_string()).unwrap_or_else(|| "None".into()))
                } else {
                    quote(version_line, 160)
                },
                "cancellation": "The version form returns on its own; Ctrl-C is available at the prompt.",
                "failure": if version_ok {
                    format!("No failure on this path: exit {}.", st.map(|v| v.to_string()).unwrap_or_else(|| "None".into()))
                } else {
                    format!("The version request itself is the failure: exit {}.", st.map(|v| v.to_string()).unwrap_or_else(|| "None".into()))
                },
                "recovery": if version_ok {
                    "None needed.".to_string()
                } else {
                    format!("Read the identity out of `{}` instead.", steps["help"]["command"].as_str().unwrap_or_default())
                },
                "evidence": ev("version", "media/01-version-identity.png"),
            })
        },
        {
            let s = &steps["help"];
            json!({
                "name": "help discovery",
                "trigger": format!("Run `{}`.", s["command"].as_str().unwrap_or_default()),
                "response": format!(
                    "{name} prints {} lines of its own top-level help, widest line {} characters.",
                    s["line_count"], measured["help_max_line_width"]
                ),
                "feedback": match s["first_line"].as_str().unwrap_or_default() {
                    "" => format!("The help process returns exit status {}.", s["exit_status"].as_i64().map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                    fl => quote(fl, 160),
                },
                "cancellation": "The stream is short enough to complete; the prompt stays interruptible.",
                "failure": format!("A misspelled flag on the same surface is refused with status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "recovery": format!("Re-run `{}`.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "evidence": ev("help", "media/02-help-surface.png"),
            })
        },
        {
            let s = &steps["subcommand-help"];
            json!({
                "name": "subcommand surface",
                "trigger": format!("Run `{}`.", s["command"].as_str().unwrap_or_default()),
                "response": product.sub_note,
                "feedback": match s["first_line"].as_str().unwrap_or_default() {
                    "" => format!("The subcommand surface returns exit status {}.", s["exit_status"].as_i64().map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                    fl => quote(fl, 160),
                },
                "cancellation": "Ctrl-C at the prompt abandons the request before submission.",
                "failure": if s["exit_status"].as_i64().unwrap_or(0) != 0 {
                    format!("Observed exit {} on this path.", s["exit_status"])
                } else {
                    "This path returned 0; failure is shown by the invalid flag instead.".to_string()
                },
                "recovery": format!("Return to `{}` for the documented grammar.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "evidence": ev("subcommand-help", "media/03-subcommand-help.png"),
            })
        },
        {
            json!({
                "name": "invalid flag refusal",
                "trigger": format!("Run `{invalid_cmd}`."),
                "response": format!("{name} refuses the unknown option and returns status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "feedback": if refusal_line.is_empty() {
                    format!("The refusal returns exit status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into()))
                } else {
                    quote(refusal_line, 160)
                },
                "cancellation": "The refusal returns immediately; no cancellation was required.",
                "failure": format!("Observed status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "recovery": if measured["refusal_names_next_action"].as_bool().unwrap_or(false) {
                    format!(
                        "The refusal names the next action ({}); running `{}` recovers.",
                        py_repr(phrase),
                        steps["recovery-help"]["command"].as_str().unwrap_or_default()
                    )
                } else {
                    format!("The refusal names no next action; `{}` recovers anyway.", steps["recovery-help"]["command"].as_str().unwrap_or_default())
                },
                "evidence": ev("invalid-flag", "media/04-refusal.png"),
            })
        },
        {
            let s = &steps["version"];
            json!({
                "name": "exit status reporting",
                "trigger": "After each command the recorded shell prints `printf \"exit-status=%s\\n\" \"$?\"`.",
                "response": "The real status of the preceding product invocation appears in the cast as text.",
                "feedback": format!(
                    "exit-status={} after the version form, exit-status={} after the invalid flag.",
                    s["exit_status"], refusal_status.map(|v| json!(v)).unwrap_or(Value::Null)
                ),
                "cancellation": "The status line is a shell builtin write; there is nothing to cancel.",
                "failure": "A missing status line would mean the prompt never returned; every step reported one.",
                "recovery": "Not applicable: the status is evidence, not an action.",
                "evidence": format!(
                    "media/session.cast at {}–{} s and after every other command",
                    g(s["ended_at"].as_f64().unwrap_or(0.0)),
                    g(s["status_reported_at"].as_f64().unwrap_or(0.0)),
                ),
            })
        },
        {
            let s = &steps["cancellation"];
            json!({
                "name": "cancellation",
                "trigger": format!("Type `{invalid_cmd}` and press Ctrl-C instead of Enter."),
                "response": if measured["cancel_prompt_restored"].as_bool().unwrap_or(false) {
                    "The pending line is discarded and the prompt returns; the product never ran."
                } else {
                    "Ctrl-C was accepted and the session continued at the prompt."
                },
                "feedback": if measured["cancel_echoed_interrupt"].as_bool().unwrap_or(false) {
                    "`^C` is echoed in the cast, then a fresh prompt."
                } else {
                    "A fresh prompt follows the interrupt in the cast."
                },
                "cancellation": "Ctrl-C is the observed cancellation mechanism for unsubmitted input.",
                "failure": "The typed command is abandoned on purpose rather than executed.",
                "recovery": format!("Submit `{}` at the restored prompt.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                "evidence": format!(
                    "media/session.cast at {}–{} s",
                    g(s["started_at"].as_f64().unwrap_or(0.0)),
                    g(s["ended_at"].as_f64().unwrap_or(0.0))
                ),
            })
        },
        {
            let s = &steps["recovery-help"];
            json!({
                "name": "recovery",
                "trigger": format!("After the refusal and the cancellation, run `{}` again.", s["command"].as_str().unwrap_or_default()),
                "response": format!("The same installed binary prints valid help and returns status {}.", s["exit_status"]),
                "feedback": if recovery_line.is_empty() {
                    format!("The recovery help returns exit status {}.", s["exit_status"])
                } else {
                    quote(recovery_line, 160)
                },
                "cancellation": "The recovery can itself be interrupted with Ctrl-C at the prompt.",
                "failure": format!("Repeating `{invalid_cmd}` reproduces status {}.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
                "recovery": "The valid help form completes the first-success journey.",
                "evidence": ev("recovery-help", "media/05-recovery.png"),
            })
        },
        {
            let s = &steps["no-color-help"];
            json!({
                "name": "color-free equivalence",
                "trigger": format!("Run `{}`.", s["command"].as_str().unwrap_or_default()),
                "response": format!(
                    "{} ({} lines).",
                    if measured["no_color_text_identical"].as_bool().unwrap_or(false) {
                        "The same help text is printed with the same line count"
                    } else {
                        "The help text differs from the colored run once color is removed"
                    },
                    s["line_count"]
                ),
                "feedback": format!(
                    "{} ANSI colour sequences in the default run, {} with NO_COLOR=1.",
                    measured["help_sgr_count"], measured["no_color_sgr_count"]
                ),
                "cancellation": "Ctrl-C at the prompt applies here as to any other command.",
                "failure": "No failure on this path; it is a measurement of the same success route.",
                "recovery": "Not applicable.",
                "evidence": ev("no-color-help", ""),
            })
        },
    ];

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

    let repainted = measured["screen_cleared"].as_bool().unwrap_or(false)
        || measured["cursor_addressed"].as_bool().unwrap_or(false);
    let duration = cast["duration_seconds"].as_f64().unwrap_or(0.0);

    json!({
        "schema": RECORD_SCHEMA,
        "name": name,
        "product_url": product.product_url,
        "evidence_status": "pending-verification",
        "upstream_owner": "Wisent (wisent-ai)",
        "wisent_product": true,
        "repository": product.repository,
        "captured_at": media["captured_at"].clone(),
        "capture_host": host_facts().host.clone(),
        "installed": {
            "binary": binary,
            "resolved_path": run.binary_path,
            "version_command": steps["version"]["command"].clone(),
            "version_output": quote(version_line, 400),
            "version_exit_status": steps["version"]["exit_status"].clone(),
            "version_flag_supported": version_ok,
        },
        "motion": [{
            "local_path": "media/session.cast",
            "source_url": product.product_url,
            "media_kind": "asciinema-v2-terminal-cast",
            "width": COLS,
            "height": ROWS,
            "duration_seconds": cast["duration_seconds"].clone(),
            "frame_count": cast["frame_count"].clone(),
            "bytes": cast["bytes"].clone(),
            "sha256": cast["sha256"].clone(),
            "capture_method": format!(
                "Real local run of the installed product on this workstation: `{binary}` resolved to \
                 {} and driven through a real pseudo-terminal (PTY) on {}, recorded as an asciinema v2 \
                 terminal cast with the timings of the run. The session issued only read-only commands \
                 — version form, top-level help, one subcommand help surface, one deliberately invalid \
                 flag, Ctrl-C on an unsubmitted line, the recovering help, and the same help with \
                 NO_COLOR=1 — from an empty scratch working directory. No host was contacted, no \
                 credential minted, no vault written, no job submitted and no service restarted.",
                run.binary_path,
                host_sentence()
            ),
            "recording_environment": format!(
                "{SHELL} --norc --noprofile -i on a {COLS}x{ROWS} PTY, TERM=xterm-256color, PAGER=cat, cwd={}",
                run.workdir
            ),
        }],
        "states": states.as_array().map(|v| v.as_slice()).unwrap_or(&[]).iter().map(|state| {
            json!({
                "name": state["label"].clone(),
                "state_name": state["state_name"].clone(),
                "local_path": state["local_path"].clone(),
                "source_motion_path": "media/session.cast",
                "source_relationship": state["source_relationship"].clone(),
                "cast_event_index": state["event_index"].clone(),
                "cast_timestamp_seconds": state["timestamp_seconds"].clone(),
                "width": state["width"].clone(),
                "height": state["height"].clone(),
                "bytes": state["bytes"].clone(),
                "sha256": state["sha256"].clone(),
            })
        }).collect::<Vec<_>>(),
        "interactions": interactions,
        "journey": {
            "actor": "An operator who has just found this Wisent binary on the PATH and wants to know what it is, what it can do, and what it refuses — before pointing it at anything real.",
            "goal": format!(
                "Get the first meaningful {name} result, read its own description of its grammar, see a real \
                 refusal, cancel a pending command safely, and recover — without touching a host, a vault, a \
                 queue or a credential."
            ),
            "prerequisites": [
                format!("{} installed on this workstation at {} (from {})", name, run.binary_path, product.repository),
                format!("An empty scratch working directory ({}) with no project or product state in it", run.workdir),
                format!("A pseudo-terminal at {COLS}x{ROWS} with TERM=xterm-256color, PAGER=cat and NO_COLOR unset"),
                host_sentence().to_string(),
            ],
            "steps": journey_steps,
            "failure_route": [
                format!("Run `{invalid_cmd}`."),
                quote(refusal_line, 160),
                format!("Observe status {} printed by the recorded shell, and the prompt restored.", refusal_status.map(|v| v.to_string()).unwrap_or_else(|| "None".into())),
            ],
            "recovery_route": [
                format!("Run `{}`.", steps["recovery-help"]["command"].as_str().unwrap_or_default()),
                quote(recovery_line, 160),
                format!("Observe status {} and the prompt returned with nothing changed on disk.", steps["recovery-help"]["exit_status"]),
            ],
            "completion_evidence": format!(
                "media/session.cast at {}–{} s plus media/05-recovery.png: the same installed binary answers \
                 again after the refusal and the cancellation. The whole {} s session is local and replayable \
                 with `asciinema play media/session.cast`.",
                g(steps["recovery-help"]["started_at"].as_f64().unwrap_or(0.0)),
                g(steps["recovery-help"]["ended_at"].as_f64().unwrap_or(0.0)),
                g(duration)
            ),
        },
        "motion_analysis": {
            "trigger": "Enter pressed on each typed command in the recorded pseudo-terminal session; the seventh keystroke sequence is Ctrl-C instead of Enter.",
            "start_state": format!(
                "An empty `{}` prompt in {} with no product state, no credential and no target selected.",
                PROMPT.trim(),
                run.workdir
            ),
            "end_state": format!(
                "The prompt restored after `{}`, with the shell's own `exit-status=` line as the last \
                 product-related output.",
                nocolor_command
            ),
            "continuity": if repainted {
                "One append-only text stream: the product repainted the screen at least once, so earlier \
                 states are recoverable only from the cast event list."
            } else {
                "One append-only text stream: no screen clear and no cursor addressing appear anywhere in \
                 the cast, so every state reached stays visible above the next one and the whole journey \
                 can be read as one scroll."
            },
            "timing_class": timing_class(measured),
            "timing_description": timing_description(measured),
            "interruption_or_reversal": format!(
                "Ctrl-C at {} s on the unsubmitted `{invalid_cmd}` line: {}",
                g(steps["cancellation"]["started_at"].as_f64().unwrap_or(0.0)),
                if measured["cancel_echoed_interrupt"].as_bool().unwrap_or(false) {
                    "the shell echoed `^C`, discarded the line and reprinted the prompt, and the product never ran."
                } else {
                    "the line was discarded and the prompt returned, and the product never ran."
                }
            ),
            "feedback": "Completion is signalled twice: the prompt returns, and the recorded shell prints \
                         `exit-status=N` for the command that just ran, so success and refusal are \
                         distinguishable in the text alone.",
            "reduced_motion_equivalent": "There is no animation to reduce. The cast is text appended in order; \
                                          the five PNGs carry the same content statically, and the raw `.cast` \
                                          file can be read as JSON without playback.",
        },
        "accessibility": {
            "measured": true,
            "measurement_method": format!(
                "Measured from this run: SGR sequences counted in the raw PTY bytes, the same help command run \
                 again with NO_COLOR=1 and the two ANSI-stripped texts compared, the refusal text searched for a \
                 named next action, and the widest help line counted against 80 columns."
            ),
            "observations": accessibility_observations,
            "measurements": {
                "help_command": help_command,
                "help_sgr_sequences": measured["help_sgr_count"].clone(),
                "colours_help_output": colors_help,
                "no_color_command": nocolor_command,
                "no_color_sgr_sequences": measured["no_color_sgr_count"].clone(),
                "no_color_text_identical": identical,
                "help_line_count": measured["help_line_count"].clone(),
                "help_max_line_width": measured["help_max_line_width"].clone(),
                "help_fits_80_columns": fits80,
                "refusal_command": invalid_cmd,
                "refusal_exit_status": refusal_status.map(|v| json!(v)).unwrap_or(Value::Null),
                "refusal_names_next_action": names_next,
                "refusal_next_action_phrase": measured["refusal_next_action_phrase"].clone(),
                "cancel_echoed_interrupt": measured["cancel_echoed_interrupt"].clone(),
                "cancel_prompt_restored": measured["cancel_prompt_restored"].clone(),
                "screen_cleared": measured["screen_cleared"].clone(),
                "cursor_addressed": measured["cursor_addressed"].clone(),
            },
            "unknowns": [
                "Screen-reader behaviour was not observed: no screen reader was attached to this PTY.",
                "Colour contrast of any emitted colours was not measured, and no WCAG or terminal-accessibility audit was performed.",
                "Behaviour in a terminal narrower than 80 columns was not observed; only the emitted line widths were measured.",
                "High-contrast themes, non-UTF-8 locales and alternative fonts were not exercised.",
                "Authenticated and target-selected paths were deliberately not run, so nothing here describes the product's accessibility once a host, vault, queue or credential is involved.",
            ],
        },
        "observed_commands": STEP_PLAN.iter().map(|(kind, label)| {
            let s = &steps[*kind];
            json!({
                "step": kind,
                "label": label,
                "command": s["command"].clone(),
                "exit_status": s["exit_status"].clone(),
                "started_at": s["started_at"].clone(),
                "ended_at": s["ended_at"].clone(),
                "line_count": s["line_count"].clone(),
                "max_line_width": s["max_line_width"].clone(),
                "first_line": s["first_line"].clone(),
            })
        }).collect::<Vec<_>>(),
        "evidence_gaps": [],
        "measured_at": Value::Null,
    })
}
