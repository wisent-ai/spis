use super::*;

/// What every part of a capture record reads from the run and its measurements, taken once.
pub(super) struct Facts<'a> {
    pub(super) run: &'a Run,
    pub(super) measured: &'a Value,
    pub(super) media: &'a Value,
    pub(super) product: &'static Product,
    pub(super) steps: &'a Value,
    pub(super) cast: &'a Value,
    pub(super) states: &'a Value,
    pub(super) name: &'static str,
    pub(super) binary: &'static str,
    pub(super) version_line: &'a str,
    pub(super) version_ok: bool,
    pub(super) refusal_line: &'a str,
    pub(super) refusal_status: Option<i64>,
    pub(super) recovery_line: &'a str,
    pub(super) invalid_cmd: &'a str,
    pub(super) cancellation_sentence: String,
    pub(super) phrase: &'a str,
    /// The measured flags each sentence is chosen by, read once.
    pub(super) prompt_restored: bool,
    pub(super) echoed_interrupt: bool,
    pub(super) names_next_action: bool,
    pub(super) text_identical: bool,
    pub(super) colors_help: bool,
    pub(super) repainted: bool,
}

impl<'a> Facts<'a> {
    /// Every field read here is written by the measurement this record is
    /// built from; one it lacks is refused naming it, never read as an empty
    /// line, an empty command or a `false` that picks the other sentence.
    pub(super) fn read(run: &'a Run, measured: &'a Value, media: &'a Value) -> Result<Self> {
        let product = run.product;
        let steps = &measured["steps"];
        let cast = &media["cast"];
        let states = &media["states"];
        let name = product.name.as_str();
        let binary = product.binary.as_str();
        let text = |value: &'a Value, field: &str| -> Result<&'a str> {
            value
                .as_str()
                .with_context(|| format!("the measured capture states no {field}"))
        };
        let flag = |field: &str| -> Result<bool> {
            measured[field]
                .as_bool()
                .with_context(|| format!("the measured capture states no {field}"))
        };
        let version_line = text(&steps["version"]["first_line"], "version first line")?;
        let version_ok = flag("version_flag_supported")?;
        let refusal_line = text(&measured["refusal_first_line"], "refusal_first_line")?;
        let refusal_status = measured["refusal_exit_status"].as_i64();
        let recovery_line = text(
            &steps["recovery-help"]["first_line"],
            "recovery-help first line",
        )?;
        let invalid_cmd = text(&steps["invalid-flag"]["command"], "invalid-flag command")?;

        let prompt_restored = flag("cancel_prompt_restored")?;
        let cancellation_sentence = if prompt_restored {
            format!(
                "Ctrl-C on the unsubmitted `{invalid_cmd}` line discarded it and restored the prompt"
            )
        } else {
            format!("Ctrl-C was sent on the unsubmitted `{invalid_cmd}` line and the session continued at the prompt")
        };
        let phrase = measured["refusal_next_action_phrase"]
            .as_str()
            .unwrap_or("");
        Ok(Facts {
            run,
            measured,
            media,
            product,
            steps,
            cast,
            states,
            name,
            binary,
            version_line,
            version_ok,
            refusal_line,
            refusal_status,
            recovery_line,
            invalid_cmd,
            cancellation_sentence,
            phrase,
            prompt_restored,
            echoed_interrupt: flag("cancel_echoed_interrupt")?,
            names_next_action: flag("refusal_names_next_action")?,
            text_identical: flag("no_color_text_identical")?,
            colors_help: flag("colors_help")?,
            repainted: flag("screen_cleared")? || flag("cursor_addressed")?,
        })
    }

    /// Where in the recording a step happened, its exit status, and any extra evidence.
    pub(super) fn ev(&self, kind: &str, extra: &str) -> String {
        let steps = self.steps;
        let s = &steps[kind];
        let mut base = format!(
            "media/session.cast at {}–{} s",
            seconds(&s["started_at"]),
            seconds(&s["ended_at"]),
        );
        if let Some(st) = s["exit_status"].as_i64() {
            base.push_str(&format!("; observed exit {st}"));
        }
        if !extra.is_empty() {
            base.push_str("; ");
            base.push_str(extra);
        }
        base
    }
}

/// An exit status the capture recorded, as the record's sentences say it;
/// one it did not record is "not recorded", never Python's "None".
pub(super) fn status(code: Option<i64>) -> String {
    match code {
        Some(code) => code.to_string(),
        None => "not recorded".to_string(),
    }
}

/// A time in seconds the capture recorded, as the record's sentences say it;
/// one it did not record is "(time not recorded)", never a zero second.
pub(super) fn seconds(value: &Value) -> String {
    match value.as_f64() {
        Some(seconds) => g(seconds),
        None => "(time not recorded)".to_string(),
    }
}
