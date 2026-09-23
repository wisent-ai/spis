use super::*;

/// The record's interactions: each trigger, response, feedback, cancellation, failure and recovery observed.
pub(super) fn interactions(f: &Facts) -> Vec<Value> {
    let Facts { run, measured, product, steps, name, binary, version_line, version_ok, refusal_line, refusal_status, recovery_line, invalid_cmd, ref cancellation_sentence, phrase, .. } = *f;
    let ev = |kind: &str, extra: &str| f.ev(kind, extra);
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
    interactions
}
