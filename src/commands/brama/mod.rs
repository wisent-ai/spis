//! Questions to Brama with a fixed set of answers, asked as one typed
//! decision on `POST /v1/decisions`.
//!
//! Spis asks Brama what words and weights used to guess: which discovered
//! pages matter, which image on a page shows the product, what pressing a
//! control does. Each is a `choice` among options the question declares,
//! read from the decision's structure, so the answer is one of them by
//! construction. The gateway is `MODEL_ROUTER_URL`, the decision alias the
//! operator's router alias `MODEL_ROUTER_MODEL` stands for (`best` asks
//! `best-decision-model`; `decision-model` and `best-decision-model` are
//! taken as written), and the bearer the vault role `MODEL_ROUTER_TOKEN_ROLE`
//! names, read through Stado or, on a machine without Stado, from the
//! owner-only file `SPIS_CREDENTIALS_FILE` names; each missing one is
//! refused with the variable and the command that needed it. On a machine
//! without Brama, `SPIS_DECISION_CHAT_MODEL` names a model of the
//! OpenAI-compatible service at `MODEL_ROUTER_URL`, and each question is asked
//! there as one schema-bound chat completion (`direct`).

mod credentials;
mod direct;

use std::io::Read;

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

/// Brama's selector, which the delegated decision alias stands for.
pub(crate) const BEST_ROUTE: &str = "best";
pub(crate) const BEST_DECISION_ALIAS: &str = "best-decision-model";
pub(crate) const DECISION_ALIAS: &str = "decision-model";
/// The Brama route a typed question is posted to, under `/v1/`.
pub(crate) const DECISIONS: &str = "decisions";

/// One environment variable a Brama question reads: what it holds, and
/// what happens when it is unset or blank. `spis docs-site` publishes these
/// rows, so the page states what this module reads.
pub(crate) struct Setting {
    pub(crate) variable: &'static str,
    pub(crate) holds: &'static str,
    pub(crate) unset: &'static str,
}

pub(crate) const ROUTER: Setting = Setting {
    variable: "MODEL_ROUTER_URL",
    holds: "the Brama address",
    unset: "the command that asks Brama is refused, naming the variable and what it holds",
};
pub(crate) const MODEL: Setting = Setting {
    variable: "MODEL_ROUTER_MODEL",
    holds: "a Brama model alias",
    unset: "the command that asks Brama is refused, naming the variable and what it holds",
};
pub(crate) const TOKEN_ROLE: Setting = Setting {
    variable: "MODEL_ROUTER_TOKEN_ROLE",
    holds: "the vault role reference ROLE#FIELD whose value Stado, or SPIS_CREDENTIALS_FILE, gives and Spis sends as the bearer",
    unset: "the question is sent with no bearer; a value that is not ROLE#FIELD, or a secret that cannot be read, is refused",
};
pub(crate) const CREDENTIALS_FILE: Setting = Setting {
    variable: credentials::CREDENTIALS_FILE,
    holds: "an owner-only JSON file of role -> field -> value that answers MODEL_ROUTER_TOKEN_ROLE in Stado's place",
    unset: "the reference is read with `stado credentials get --role` (STADO_BIN names another Stado program); a file other users can read, or one without the role or field, is refused",
};
/// Every setting a typed question reads.
pub(crate) const SETTINGS: &[Setting] = &[
    ROUTER,
    MODEL,
    TOKEN_ROLE,
    CREDENTIALS_FILE,
    direct::CHAT_MODEL,
];

/// The typed answers to `questions` about `state`, keyed as asked: Brama's
/// decision, or the declared chat model's. A refusal, an answer document
/// without `answers`, or a question left unanswered is refused with what the
/// service sent.
pub(crate) fn decide(
    purpose: &str,
    state: &str,
    questions: &Map<String, Value>,
) -> Result<Map<String, Value>> {
    let mut body = match direct::declared()? {
        Some(chat_model) => direct::ask(purpose, &chat_model, state, questions)?,
        None => {
            let model = setting(&MODEL, purpose)?;
            let alias = match model.as_str() {
                BEST_ROUTE | BEST_DECISION_ALIAS => BEST_DECISION_ALIAS,
                DECISION_ALIAS => DECISION_ALIAS,
                other => anyhow::bail!(
                    "{purpose}: {} `{other}` cannot answer a typed question: Brama's POST /v1/{DECISIONS} answers only `{DECISION_ALIAS}` and `{BEST_DECISION_ALIAS}` (`{BEST_ROUTE}` asks the latter); without Brama set {} to a model of the OpenAI-compatible service at {}",
                    MODEL.variable,
                    direct::CHAT_MODEL.variable,
                    ROUTER.variable
                ),
            };
            let payload =
                json!({ "model": alias, "state": state, "questions": questions }).to_string();
            post(purpose, DECISIONS, &payload)?
        }
    };
    let answers = match body.get_mut("answers").map(Value::take) {
        Some(Value::Object(answers)) => answers,
        Some(other) => {
            anyhow::bail!("{purpose}: Brama's decision answers are not an object: {other}")
        }
        None => anyhow::bail!("{purpose}: Brama's decision carries no answers: {body}"),
    };
    if let Some(missing) = questions.keys().find(|key| !answers.contains_key(*key)) {
        anyhow::bail!(
            "{purpose}: Brama's decision answered no `{missing}`: {}",
            Value::Object(answers)
        );
    }
    Ok(answers)
}

/// One `choice` question: `instructions` and the options it may answer,
/// each with what it means.
pub(crate) fn choice(instructions: &str, options: &[(String, String)]) -> Value {
    let criteria: Map<String, Value> = options
        .iter()
        .map(|(option, meaning)| (option.clone(), json!(meaning)))
        .collect();
    json!({ "type": "choice", "instructions": instructions, "criteria": criteria })
}

/// The option a `choice` answer chose and how confident Brama was in it,
/// refused unless it is one of `options`.
pub(crate) fn chosen(
    purpose: &str,
    answer: &Value,
    options: &[(String, String)],
) -> Result<(String, f64)> {
    let option = answer["choice"]
        .as_str()
        .with_context(|| format!("{purpose}: Brama's answer carries no choice: {answer}"))?;
    if !options.iter().any(|(known, _)| known == option) {
        anyhow::bail!("{purpose}: Brama chose `{option}`, which is not one of the options asked");
    }
    let confidence = answer["confidence"]
        .as_f64()
        .with_context(|| format!("{purpose}: Brama's answer carries no confidence: {answer}"))?;
    Ok((option.to_string(), confidence))
}

/// One JSON `payload` to Brama's `/v1/<path>` with the bearer, and the JSON
/// Brama answered; Brama's own refusal is kept with its status and body.
fn post(purpose: &str, path: &str, payload: &str) -> Result<Value> {
    let router = setting(&ROUTER, purpose)?;
    let endpoint = if router.contains("/v1") {
        format!("{}/{path}", router.trim_end_matches('/'))
    } else {
        format!("{}/v1/{path}", router.trim_end_matches('/'))
    };
    // Read before the request so a bad reference is named, never sent anonymously.
    let token = credentials::role_secret(TOKEN_ROLE.variable)
        .with_context(|| format!("{purpose}: the Brama bearer could not be read"))?;
    let mut request = ureq::post(&endpoint).set("Content-Type", "application/json");
    if let Some(token) = &token {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = match request.send_string(payload) {
        Ok(response) => response,
        Err(ureq::Error::Status(status, response)) => {
            let said = response.into_string().with_context(|| {
                format!(
                    "{purpose}: Brama at {endpoint} answered HTTP {status} with an unreadable body"
                )
            })?;
            anyhow::bail!(
                "{purpose}: Brama at {endpoint} answered HTTP {status}: {}",
                said.trim()
            );
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("{purpose}: Brama at {endpoint} could not be reached"));
        }
    };
    let mut body_bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut body_bytes)
        .with_context(|| format!("{purpose}: Brama's answer from {endpoint} could not be read"))?;
    serde_json::from_slice(&body_bytes)
        .with_context(|| format!("{purpose}: Brama answered with a body that is not JSON"))
}

/// The value of `declared`. Unset or blank is refused with what it should
/// hold; a value that is not text is named, not read as unset.
fn setting(declared: &Setting, purpose: &str) -> Result<String> {
    let Setting {
        variable, holds, ..
    } = declared;
    match std::env::var(variable) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        Ok(_) | Err(std::env::VarError::NotPresent) => {
            anyhow::bail!("{purpose}: set {variable} to {holds}")
        }
        Err(error) => anyhow::bail!("{purpose}: {variable} cannot be read: {error}"),
    }
}
