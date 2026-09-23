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
}

impl<'a> Facts<'a> {
    pub(super) fn read(run: &'a Run, measured: &'a Value, media: &'a Value) -> Self {
        let product = run.product;
        let steps = &measured["steps"];
        let cast = &media["cast"];
        let states = &media["states"];
        let name = product.name;
        let binary = product.binary;
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
        Facts { run, measured, media, product, steps, cast, states, name, binary, version_line, version_ok, refusal_line, refusal_status, recovery_line, invalid_cmd, cancellation_sentence, phrase }
    }

    /// Where in the recording a step happened, its exit status, and any extra evidence.
    pub(super) fn ev(&self, kind: &str, extra: &str) -> String {
        let steps = self.steps;
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
    }
}
