use super::*;

mod facts;
mod interactions;
mod observations;

use facts::Facts;
use interactions::interactions;
use observations::{accessibility, journey_steps, Accessibility};

// -------------------------------------------------------------- record build

pub(crate) fn build_record(run: &Run, measured: &Value, media: &Value) -> Value {
    let f = Facts::read(run, measured, media);
    let interactions = interactions(&f);
    let journey_steps = journey_steps(&f);
    let Accessibility { observations: accessibility_observations, colors_help, identical, names_next, fits80, help_command, nocolor_command } = accessibility(&f);
    let Facts { run, measured, media, product, steps, cast, states, name, binary, version_line, version_ok, refusal_line, refusal_status, recovery_line, invalid_cmd, .. } = f;

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
